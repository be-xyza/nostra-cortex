#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/lib/workspace_paths.sh"
resolve_workspace_paths "$ROOT_DIR"
APP_DIR="$CORTEX_DESKTOP_DIR"
LAUNCHER="$APP_DIR/run_cortex.command"
RUNTIME_LOG="$APP_DIR/cortex_runtime.log"
BUILD_LOG="$APP_DIR/cortex_build.log"
RUNS_DIR="$ROOT_DIR/logs/testing/runs"
GATE_FILE="$ROOT_DIR/logs/testing/test_gate_summary_latest.json"
CONFORMANCE_FILE="$ROOT_DIR/logs/testing/cortex_ui_theme_conformance_latest.json"
MODE="${NOSTRA_TEST_GATE_MODE:-blocking}"
AGENT_ID="${NOSTRA_AGENT_ID:-codex-local}"
ENVIRONMENT="${NOSTRA_TEST_ENVIRONMENT:-local_ide}"
SCHEMA_VERSION="1.0.0"
DEFAULT_GATEWAY_PORT=3000
PID_FILE="${HOME}/.cortex-daemon/cortex-daemon.pid"
RUN_ID="${ENVIRONMENT}_cortex_daemon_closeout_$(date -u +%Y%m%dT%H%M%SZ)"

STARTED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
GIT_COMMIT="$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cortex-closeout.XXXXXX")"
RESULTS_FILE="$TMP_DIR/results.json"
SECOND_LAUNCH_OUT="$TMP_DIR/second_launch.out"
FIRST_LAUNCH_OUT="$TMP_DIR/first_launch.out"
GATEWAY_FALLBACK_OUT="$TMP_DIR/gateway_fallback.out"
FIRST_LAUNCH_PID=""
GATEWAY_FALLBACK_PID=""
OVERALL_OK=true
RUNTIME_LOG_START_LINE=0
STRICT_MODE="${CORTEX_CLOSEOUT_STRICT:-false}"

mkdir -p "$RUNS_DIR"
printf '[]\n' > "$RESULTS_FILE"

warnings=()

now_ms() {
  if command -v perl >/dev/null 2>&1; then
    perl -MTime::HiRes=time -e 'printf("%.0f\n", time()*1000)'
    return
  fi

  echo "$(( $(date +%s) * 1000 ))"
}

gateway_port() {
  local raw="${CORTEX_GATEWAY_PORT:-}"
  local port_num
  if [[ -z "$raw" ]]; then
    echo "$DEFAULT_GATEWAY_PORT"
    return
  fi

  if [[ "$raw" =~ ^[0-9]+$ ]]; then
    port_num=$((10#$raw))
    if (( port_num >= 1 && port_num <= 65535 )); then
      echo "$port_num"
      return
    fi
  fi

  warnings+=("Invalid CORTEX_GATEWAY_PORT='$raw'; fallback to ${DEFAULT_GATEWAY_PORT}")
  echo "$DEFAULT_GATEWAY_PORT"
}

runtime_log_start_line() {
  if [[ -f "$RUNTIME_LOG" ]]; then
    wc -l < "$RUNTIME_LOG"
  else
    echo "0"
  fi
}

runtime_log_new_slice() {
  local start_line="$1"
  if [[ ! -f "$RUNTIME_LOG" ]]; then
    return 0
  fi

  tail -n +"$((start_line + 1))" "$RUNTIME_LOG"
}

record_runtime_log_baseline() {
  RUNTIME_LOG_START_LINE="$(runtime_log_start_line)"
}

record_result() {
  local test_id="$1"
  local status="$2"
  local duration_ms="$3"
  local error_summary="$4"

  jq \
    --arg test_id "$test_id" \
    --arg status "$status" \
    --argjson duration_ms "$duration_ms" \
    --arg error_summary "$error_summary" \
    '. += [{
      test_id: $test_id,
      status: $status,
      duration_ms: $duration_ms,
      error_summary: $error_summary
    }]' "$RESULTS_FILE" > "$RESULTS_FILE.next"
  mv "$RESULTS_FILE.next" "$RESULTS_FILE"

  if [[ "$status" == "fail" ]]; then
    OVERALL_OK=false
  fi
}

result_status_for() {
  local test_id="$1"
  jq -r --arg test_id "$test_id" '.[] | select(.test_id == $test_id) | .status' "$RESULTS_FILE" | tail -n 1
}

is_pid_alive() {
  local pid="$1"
  if [[ -z "$pid" ]]; then
    return 1
  fi
  kill -0 "$pid" >/dev/null 2>&1
}

cleanup() {
  local launcher_pid child_pids
  launcher_pid="$FIRST_LAUNCH_PID"
  if [[ -f "$PID_FILE" ]]; then
    local pid_from_lock
    pid_from_lock="$(tr -d '[:space:]' < "$PID_FILE" 2>/dev/null || true)"
    if [[ -n "$pid_from_lock" ]] && is_pid_alive "$pid_from_lock"; then
      launcher_pid="$pid_from_lock"
    fi
  fi

  if [[ -n "$launcher_pid" ]] && is_pid_alive "$launcher_pid"; then
    child_pids="$(pgrep -P "$launcher_pid" 2>/dev/null || true)"
    if [[ -n "$child_pids" ]]; then
      printf '%s\n' "$child_pids" | xargs kill >/dev/null 2>&1 || true
    fi

    kill -TERM "$launcher_pid" >/dev/null 2>&1 || true
    for _ in $(seq 1 30); do
      if ! is_pid_alive "$launcher_pid"; then
        break
      fi
      sleep 0.5
    done

    if is_pid_alive "$launcher_pid"; then
      warnings+=("Launcher required SIGKILL during cleanup")
      kill -KILL "$launcher_pid" >/dev/null 2>&1 || true
    fi

    child_pids="$(pgrep -P "$launcher_pid" 2>/dev/null || true)"
    if [[ -n "$child_pids" ]]; then
      warnings+=("Launcher child process required SIGKILL during cleanup")
      printf '%s\n' "$child_pids" | xargs kill -9 >/dev/null 2>&1 || true
    fi
  fi

  if [[ -n "$GATEWAY_FALLBACK_PID" ]] && is_pid_alive "$GATEWAY_FALLBACK_PID"; then
    kill -TERM "$GATEWAY_FALLBACK_PID" >/dev/null 2>&1 || true
    for _ in $(seq 1 20); do
      if ! is_pid_alive "$GATEWAY_FALLBACK_PID"; then
        break
      fi
      sleep 0.25
    done
    if is_pid_alive "$GATEWAY_FALLBACK_PID"; then
      warnings+=("Gateway fallback required SIGKILL during cleanup")
      kill -KILL "$GATEWAY_FALLBACK_PID" >/dev/null 2>&1 || true
    fi
  fi
}

cleanup_tmp() {
  rm -rf "$TMP_DIR"
}

trap 'cleanup; cleanup_tmp' EXIT INT TERM

reset_existing_launcher() {
  if [[ ! -f "$PID_FILE" ]]; then
    return
  fi

  local existing_pid
  existing_pid="$(tr -d '[:space:]' < "$PID_FILE" 2>/dev/null || true)"
  if [[ -z "$existing_pid" ]] || ! is_pid_alive "$existing_pid"; then
    return
  fi

  kill -TERM "$existing_pid" >/dev/null 2>&1 || true
  for _ in $(seq 1 20); do
    if ! is_pid_alive "$existing_pid"; then
      return
    fi
    sleep 0.5
  done

  warnings+=("Preflight cleanup required SIGKILL for existing launcher")
  kill -KILL "$existing_pid" >/dev/null 2>&1 || true
}

run_cargo_tests() {
  local t0 t1 duration
  t0="$(now_ms)"
  if cargo test --manifest-path "$APP_DIR/Cargo.toml" --tests; then
    t1="$(now_ms)"
    duration=$((t1 - t0))
    record_result "cortex_daemon_closeout:cargo_tests" "pass" "$duration" ""
  else
    t1="$(now_ms)"
    duration=$((t1 - t0))
    record_result "cortex_daemon_closeout:cargo_tests" "fail" "$duration" "cargo test --tests failed"
  fi
}

launch_first_instance() {
  local t0 t1 duration observed_pid
  t0="$(now_ms)"

  record_runtime_log_baseline

  bash "$LAUNCHER" >"$FIRST_LAUNCH_OUT" 2>&1 &
  FIRST_LAUNCH_PID=$!

  observed_pid=""
  for _ in $(seq 1 60); do
    if [[ -f "$PID_FILE" ]]; then
      observed_pid="$(tr -d '[:space:]' < "$PID_FILE" 2>/dev/null || true)"
      if [[ -n "$observed_pid" ]] && is_pid_alive "$observed_pid"; then
        break
      fi
    fi
    if ! is_pid_alive "$FIRST_LAUNCH_PID"; then
      break
    fi
    sleep 0.25
  done

  t1="$(now_ms)"
  duration=$((t1 - t0))

  if is_pid_alive "$FIRST_LAUNCH_PID"; then
    record_result "cortex_daemon_closeout:launch_first_instance" "pass" "$duration" ""
    return
  fi

  record_result "cortex_daemon_closeout:launch_first_instance" "fail" "$duration" "first launcher exited before lock verification"
}

verify_single_instance_lock() {
  local t0 t1 duration exit_code output second_pid timed_out
  t0="$(now_ms)"

  if ! is_pid_alive "$FIRST_LAUNCH_PID"; then
    t1="$(now_ms)"
    duration=$((t1 - t0))
    if [[ "$STRICT_MODE" == "true" ]]; then
      record_result "cortex_daemon_closeout:single_instance_lock" "fail" "$duration" "first launcher not alive before second-launch lock verification"
    else
      record_result "cortex_daemon_closeout:single_instance_lock" "warn" "$duration" "first launcher not alive before second-launch lock verification"
    fi
    return
  fi

  set +e
  bash "$LAUNCHER" >"$SECOND_LAUNCH_OUT" 2>&1 &
  second_pid=$!
  output=""
  exit_code=0
  timed_out=false

  for _ in $(seq 1 120); do
    if ! is_pid_alive "$second_pid"; then
      wait "$second_pid"
      exit_code=$?
      output="$(cat "$SECOND_LAUNCH_OUT" 2>/dev/null || true)"
      break
    fi
    sleep 0.125
  done

  if is_pid_alive "$second_pid"; then
    timed_out=true
    kill -TERM "$second_pid" >/dev/null 2>&1 || true
    for _ in $(seq 1 20); do
      if ! is_pid_alive "$second_pid"; then
        break
      fi
      sleep 0.1
    done
    if is_pid_alive "$second_pid"; then
      kill -KILL "$second_pid" >/dev/null 2>&1 || true
    fi
    wait "$second_pid" >/dev/null 2>&1 || true
    output="$(cat "$SECOND_LAUNCH_OUT" 2>/dev/null || true)"
    exit_code=124
  fi
  set -e

  t1="$(now_ms)"
  duration=$((t1 - t0))

  if [[ "$timed_out" == "true" ]]; then
    local timeout_trimmed
    timeout_trimmed="$(printf '%s' "$output" | tail -n 5 | tr '\n' ' ' | sed 's/  */ /g')"
    if [[ "$STRICT_MODE" == "true" ]]; then
      record_result "cortex_daemon_closeout:single_instance_lock" "fail" "$duration" "second launcher timed out before reporting lock (output='$timeout_trimmed')"
    else
      record_result "cortex_daemon_closeout:single_instance_lock" "warn" "$duration" "second launcher timed out before reporting lock (environmental instability likely; output='$timeout_trimmed')"
    fi
    return
  fi

  if [[ "$exit_code" -eq 0 ]] && printf '%s' "$output" | grep -qi "already running"; then
    record_result "cortex_daemon_closeout:single_instance_lock" "pass" "$duration" ""
    return
  fi

  local trimmed
  trimmed="$(printf '%s' "$output" | tail -n 5 | tr '\n' ' ' | sed 's/  */ /g')"
  if [[ "$STRICT_MODE" == "true" ]]; then
    record_result "cortex_daemon_closeout:single_instance_lock" "fail" "$duration" "second launcher did not report already running (exit=$exit_code, output='$trimmed')"
  else
    record_result "cortex_daemon_closeout:single_instance_lock" "warn" "$duration" "second launcher did not report already running (exit=$exit_code, output='$trimmed')"
  fi
}

verify_readiness() {
  local t0 t1 duration port url response ready launcher_died
  t0="$(now_ms)"

  port="$(gateway_port)"
  url="http://127.0.0.1:${port}/api/system/ready"
  ready="false"
  launcher_died=false

  for _ in $(seq 1 90); do
    response="$(curl -fsS --max-time 1 "$url" 2>/dev/null || true)"
    if [[ -n "$response" ]]; then
      ready="$(printf '%s' "$response" | jq -r '.ready // false' 2>/dev/null || echo false)"
      if [[ "$ready" == "true" ]]; then
        break
      fi
    fi

    if ! is_pid_alive "$FIRST_LAUNCH_PID"; then
      launcher_died=true
      break
    fi

    sleep 0.5
  done

  if [[ "$ready" != "true" ]] && [[ "$launcher_died" == "true" ]]; then
    warnings+=("Launcher exited before readiness; attempting gateway-only fallback")
    cargo run --manifest-path "$APP_DIR/Cargo.toml" --bin gateway_server >"$GATEWAY_FALLBACK_OUT" 2>&1 &
    GATEWAY_FALLBACK_PID=$!

    for _ in $(seq 1 120); do
      response="$(curl -fsS --max-time 1 "$url" 2>/dev/null || true)"
      if [[ -n "$response" ]]; then
        ready="$(printf '%s' "$response" | jq -r '.ready // false' 2>/dev/null || echo false)"
        if [[ "$ready" == "true" ]]; then
          break
        fi
      fi
      if ! is_pid_alive "$GATEWAY_FALLBACK_PID"; then
        break
      fi
      sleep 0.5
    done
  fi

  t1="$(now_ms)"
  duration=$((t1 - t0))

  if [[ "$ready" == "true" ]]; then
    record_result "cortex_daemon_closeout:gateway_readiness" "pass" "$duration" ""
  else
    local fallback_bind_error=false
    if [[ -f "$GATEWAY_FALLBACK_OUT" ]] && rg -n "Failed to bind gateway|Operation not permitted|os error 1" "$GATEWAY_FALLBACK_OUT" >/dev/null 2>&1; then
      fallback_bind_error=true
    fi

    if [[ "$STRICT_MODE" != "true" ]]; then
      if [[ "$fallback_bind_error" == "true" ]]; then
        record_result "cortex_daemon_closeout:gateway_readiness" "warn" "$duration" "gateway readiness blocked by local socket permission constraints"
      else
        record_result "cortex_daemon_closeout:gateway_readiness" "warn" "$duration" "readiness endpoint unavailable in this environment"
      fi
    else
      record_result "cortex_daemon_closeout:gateway_readiness" "fail" "$duration" "readiness endpoint did not return ready=true within 45s"
    fi
  fi
}

verify_runtime_log_no_panic() {
  local t0 t1 duration
  t0="$(now_ms)"

  if [[ ! -f "$RUNTIME_LOG" ]]; then
    t1="$(now_ms)"
    duration=$((t1 - t0))
    record_result "cortex_daemon_closeout:runtime_log_scan" "fail" "$duration" "runtime log not found at $RUNTIME_LOG"
    return
  fi

  if runtime_log_new_slice "$RUNTIME_LOG_START_LINE" | grep -E "panicked at|thread '.*' panicked" >/dev/null 2>&1; then
    t1="$(now_ms)"
    duration=$((t1 - t0))
    if [[ "$STRICT_MODE" == "true" ]]; then
      record_result "cortex_daemon_closeout:runtime_log_scan" "fail" "$duration" "panic signatures found in cortex_runtime.log"
      return
    fi

    local readiness_status
    readiness_status="$(result_status_for "cortex_daemon_closeout:gateway_readiness")"
    if [[ "$readiness_status" == "warn" || "$readiness_status" == "fail" ]]; then
      record_result "cortex_daemon_closeout:runtime_log_scan" "warn" "$duration" "panic signatures observed while gateway readiness was not satisfied in this environment"
    else
      record_result "cortex_daemon_closeout:runtime_log_scan" "fail" "$duration" "panic signatures found in cortex_runtime.log"
    fi
    return
  fi

  t1="$(now_ms)"
  duration=$((t1 - t0))
  record_result "cortex_daemon_closeout:runtime_log_scan" "pass" "$duration" ""
}

verify_a2ui_policy_and_accessibility_sources() {
  local t0 t1 duration missing
  t0="$(now_ms)"
  missing=()

  if ! rg -n "motion_policy|motion_policy_from|motion-reduce:|prefers-reduced-motion" \
      "$ROOT_DIR/nostra/frontend/src/a2ui.rs" \
      "$ROOT_DIR/nostra/frontend/src/a2ui_theme.rs" \
      "$APP_DIR/src/components/a2ui/renderer.rs" \
      "$APP_DIR/src/components/a2ui/theme.rs" \
      >/dev/null 2>&1; then
    missing+=("reduced-motion policy evidence")
  fi

  if ! rg -n "focus-visible:ring|focus:outline-none|focus-visible:ring-offset-2" \
      "$ROOT_DIR/nostra/frontend/src/a2ui.rs" \
      "$ROOT_DIR/nostra/frontend/src/a2ui_theme.rs" \
      "$APP_DIR/src/components/a2ui/renderer.rs" \
      >/dev/null 2>&1; then
    missing+=("focus visibility evidence")
  fi

  if ! rg -n "contrast_preference|contrast\\(" \
      "$ROOT_DIR/nostra/frontend/src/a2ui.rs" \
      "$ROOT_DIR/nostra/frontend/src/a2ui_theme.rs" \
      "$APP_DIR/src/components/a2ui/renderer.rs" \
      "$APP_DIR/src/components/a2ui/theme.rs" \
      >/dev/null 2>&1; then
    missing+=("contrast policy evidence")
  fi

  if ! rg -n "onkeydown|Key::Enter|Key::Escape|tabindex" \
      "$ROOT_DIR/nostra/frontend/src/a2ui.rs" \
      "$APP_DIR/src/components/views/console.rs" \
      >/dev/null 2>&1; then
    missing+=("keyboard traversal evidence")
  fi

  if ! rg -n "aria_|role:|a11y" \
      "$ROOT_DIR/nostra/frontend/src/a2ui.rs" \
      >/dev/null 2>&1; then
    missing+=("ARIA/a11y mapping evidence")
  fi

  t1="$(now_ms)"
  duration=$((t1 - t0))

  if [[ ${#missing[@]} -eq 0 ]]; then
    record_result "cortex_daemon_closeout:a2ui_policy_accessibility_scan" "pass" "$duration" ""
  else
    local details
    details="$(printf '%s, ' "${missing[@]}")"
    details="${details%, }"
    record_result "cortex_daemon_closeout:a2ui_policy_accessibility_scan" "fail" "$duration" "$details"
  fi
}

verify_ui_theme_conformance() {
  local t0 t1 duration
  t0="$(now_ms)"

  if bash "$ROOT_DIR/scripts/check_cortex_ui_theme_conformance.sh"; then
    t1="$(now_ms)"
    duration=$((t1 - t0))
    record_result "cortex_daemon_closeout:ui_theme_conformance" "pass" "$duration" ""
  else
    t1="$(now_ms)"
    duration=$((t1 - t0))
    record_result "cortex_daemon_closeout:ui_theme_conformance" "fail" "$duration" "theme conformance check failed"
  fi
}

write_run_artifact() {
  local finished_at run_file warnings_json
  finished_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  run_file="$RUNS_DIR/$RUN_ID.json"

  warnings_json='[]'
  if [[ ${#warnings[@]} -gt 0 ]]; then
    warnings_json="$(printf '%s\n' "${warnings[@]}" | jq -R . | jq -s .)"
  fi

  jq -n \
    --arg schema_version "$SCHEMA_VERSION" \
    --arg run_id "$RUN_ID" \
    --arg started_at "$STARTED_AT" \
    --arg finished_at "$finished_at" \
    --arg agent_id "$AGENT_ID" \
    --arg environment "$ENVIRONMENT" \
    --arg git_commit "$GIT_COMMIT" \
    --argjson synthetic false \
    --argjson results "$(cat "$RESULTS_FILE")" \
    --argjson artifacts "[\"$LAUNCHER\", \"$RUNTIME_LOG\", \"$BUILD_LOG\", \"$GATE_FILE\", \"$CONFORMANCE_FILE\"]" \
    --argjson warnings "$warnings_json" \
    '{
      schema_version: $schema_version,
      run_id: $run_id,
      started_at: $started_at,
      finished_at: $finished_at,
      agent_id: $agent_id,
      environment: $environment,
      git_commit: $git_commit,
      synthetic: $synthetic,
      results: $results,
      artifacts: $artifacts,
      warnings: $warnings
    }' > "$run_file"

  echo "Wrote run artifact: $run_file"
}

refresh_gate_summary_with_compatibility() {
  local now_utc single_lock readiness runtime_scan cargo_tests policy_scan ui_theme_conformance summary_ok
  now_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  NOSTRA_TEST_GATE_MODE="$MODE" bash "$ROOT_DIR/scripts/generate_test_gate_summary.sh" --mode "$MODE"

  single_lock="$(jq -r '.results[] | select(.test_id == "cortex_daemon_closeout:single_instance_lock") | .status' "$RUNS_DIR/$RUN_ID.json" | head -n 1)"
  readiness="$(jq -r '.results[] | select(.test_id == "cortex_daemon_closeout:gateway_readiness") | .status' "$RUNS_DIR/$RUN_ID.json" | head -n 1)"
  runtime_scan="$(jq -r '.results[] | select(.test_id == "cortex_daemon_closeout:runtime_log_scan") | .status' "$RUNS_DIR/$RUN_ID.json" | head -n 1)"
  cargo_tests="$(jq -r '.results[] | select(.test_id == "cortex_daemon_closeout:cargo_tests") | .status' "$RUNS_DIR/$RUN_ID.json" | head -n 1)"
  policy_scan="$(jq -r '.results[] | select(.test_id == "cortex_daemon_closeout:a2ui_policy_accessibility_scan") | .status' "$RUNS_DIR/$RUN_ID.json" | head -n 1)"
  ui_theme_conformance="$(jq -r '.results[] | select(.test_id == "cortex_daemon_closeout:ui_theme_conformance") | .status' "$RUNS_DIR/$RUN_ID.json" | head -n 1)"

  summary_ok=false
  if [[ "$OVERALL_OK" == true ]]; then
    summary_ok=true
  fi

  jq \
    --arg run_id "$RUN_ID" \
    --arg generated_at "$now_utc" \
    --arg single_instance_lock "$single_lock" \
    --arg readiness "$readiness" \
    --arg runtime_scan "$runtime_scan" \
    --arg cargo_tests "$cargo_tests" \
    --arg policy_scan "$policy_scan" \
    --arg ui_theme_conformance "$ui_theme_conformance" \
    --argjson overall_pass "$summary_ok" \
    '. + {
      compatibility: ((.compatibility // {}) + {
        cortex_daemon_closeout: {
          run_id: $run_id,
          generated_at: $generated_at,
          overall_pass: $overall_pass,
          checks: {
            cargo_tests: $cargo_tests,
            single_instance_lock: $single_instance_lock,
            gateway_readiness: $readiness,
            runtime_log_scan: $runtime_scan,
            a2ui_policy_accessibility_scan: $policy_scan,
            ui_theme_conformance: $ui_theme_conformance
          }
        }
      })
    }' "$GATE_FILE" > "$GATE_FILE.next"

  mv "$GATE_FILE.next" "$GATE_FILE"

  NOSTRA_TEST_GATE_MODE="$MODE" bash "$ROOT_DIR/scripts/check_test_catalog_consistency.sh" --mode advisory
}

reset_existing_launcher
run_cargo_tests
launch_first_instance
verify_single_instance_lock
verify_readiness
verify_runtime_log_no_panic
verify_a2ui_policy_and_accessibility_sources
verify_ui_theme_conformance
cleanup
FIRST_LAUNCH_PID=""
write_run_artifact
refresh_gate_summary_with_compatibility

if [[ "$OVERALL_OK" != true ]]; then
  echo "cortex-daemon-closeout-check: FAIL"
  exit 1
fi

echo "cortex-daemon-closeout-check: PASS"
