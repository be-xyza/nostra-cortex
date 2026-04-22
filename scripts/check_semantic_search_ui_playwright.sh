#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/lib/workspace_paths.sh"
resolve_workspace_paths "$ROOT_DIR"
FRONTEND_DIR="$ROOT_DIR/nostra/frontend"
GATEWAY_MANIFEST="$CORTEX_DESKTOP_MANIFEST"
OUT="$ROOT_DIR/logs/knowledge/ui_playwright_matrix_latest.json"
REPORT="$ROOT_DIR/logs/knowledge/ui_playwright_report_latest.json"
SERVER_LOG="$ROOT_DIR/logs/knowledge/ui_playwright_server_latest.log"
GATEWAY_LOG="$ROOT_DIR/logs/knowledge/ui_playwright_gateway_latest.log"
PLAYWRIGHT_STDERR="$ROOT_DIR/logs/knowledge/ui_playwright_stderr_latest.log"
port_in_use() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
  else
    ss -ltn "( sport = :$port )" 2>/dev/null | tail -n +2 | grep -q .
  fi
}

pick_free_port() {
  local candidate="$1"
  while port_in_use "$candidate"; do
    candidate=$((candidate + 1))
  done
  printf '%s' "$candidate"
}

PORT="$(pick_free_port "${NOSTRA_UI_TEST_PORT:-3011}")"
GATEWAY_PORT="$(pick_free_port "${CORTEX_GATEWAY_PORT:-3001}")"
if [[ "$PORT" == "$GATEWAY_PORT" ]]; then
  PORT="$(pick_free_port "$((GATEWAY_PORT + 1))")"
fi
BASE_URL="http://127.0.0.1:${PORT}"
SPEC_PATH="tests/e2e/semantic-search-ui.spec.ts"

mkdir -p "$(dirname "$OUT")"

if ! command -v dx >/dev/null 2>&1; then
  echo "Missing required command: dx" >&2
  exit 1
fi

if ! command -v npx >/dev/null 2>&1; then
  echo "Missing required command: npx" >&2
  exit 1
fi

if [[ ! -f "$ROOT_DIR/$SPEC_PATH" ]]; then
  echo "Missing Playwright spec: $ROOT_DIR/$SPEC_PATH" >&2
  exit 1
fi

SERVER_PID=""
GATEWAY_PID=""

cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$GATEWAY_PID" ]] && kill -0 "$GATEWAY_PID" >/dev/null 2>&1; then
    kill "$GATEWAY_PID" >/dev/null 2>&1 || true
    wait "$GATEWAY_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

echo "== Starting Cortex gateway for semantic UI Playwright checks"
(
  cd "$ROOT_DIR"
  CORTEX_GATEWAY_PORT="$GATEWAY_PORT" cargo run \
    --quiet \
    --manifest-path "$GATEWAY_MANIFEST" \
    --bin gateway_server
) >"$GATEWAY_LOG" 2>&1 &
GATEWAY_PID=$!

gateway_ready=false
for _ in $(seq 1 120); do
  if curl -fsS "http://127.0.0.1:${GATEWAY_PORT}/api/health" >/dev/null 2>&1; then
    gateway_ready=true
    break
  fi
  if ! kill -0 "$GATEWAY_PID" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if [[ "$gateway_ready" != "true" ]]; then
  echo "Gateway did not become ready on port ${GATEWAY_PORT}" >&2
  tail -n 80 "$GATEWAY_LOG" >&2 || true
  exit 1
fi
if ! kill -0 "$GATEWAY_PID" >/dev/null 2>&1; then
  echo "Gateway process exited unexpectedly before frontend startup" >&2
  tail -n 80 "$GATEWAY_LOG" >&2 || true
  exit 1
fi

echo "== Starting frontend server for semantic UI Playwright checks"
(
  cd "$FRONTEND_DIR"
  TERM=xterm-256color COLORTERM=truecolor dx serve \
    --platform web \
    --open false \
    --watch false \
    --hot-reload false \
    --addr 127.0.0.1 \
    --port "$PORT"
) >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

server_ready=false
for _ in $(seq 1 120); do
  if curl -fsS "$BASE_URL" >/dev/null 2>&1; then
    server_ready=true
    break
  fi
  if ! kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

build_ready=false
if [[ "$server_ready" == "true" ]]; then
  # `dx serve` does not consistently emit a "Build completed successfully" marker.
  # Once the HTTP endpoint is live, pause briefly for the initial bundle and continue.
  sleep 5
  if kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    build_ready=true
  fi
fi
if [[ "$server_ready" == "true" ]] && ! kill -0 "$SERVER_PID" >/dev/null 2>&1; then
  server_ready=false
fi

playwright_exit=1
expected_count=0
unexpected_count=0
report_valid=false
error_summary=""

if [[ "$gateway_ready" == "true" && "$server_ready" == "true" && "$build_ready" == "true" ]]; then
  set +e
  (
    cd "$ROOT_DIR"
    BASE_URL="$BASE_URL" npx playwright test "$SPEC_PATH" \
      --config "$ROOT_DIR/playwright.config.ts" \
      --project=chromium \
      --reporter=json
  ) >"$REPORT" 2>"$PLAYWRIGHT_STDERR"
  playwright_exit=$?
  set -e

  if jq -e '.' "$REPORT" >/dev/null 2>&1; then
    report_valid=true
    expected_count="$(jq -r '.stats.expected // 0' "$REPORT")"
    unexpected_count="$(jq -r '.stats.unexpected // 0' "$REPORT")"
  else
    error_summary="invalid playwright json report"
  fi
else
  if [[ "$gateway_ready" != "true" ]]; then
    error_summary="gateway did not become ready on 127.0.0.1:${GATEWAY_PORT}"
  elif [[ "$server_ready" != "true" ]]; then
    error_summary="frontend server did not become ready on ${BASE_URL}"
  else
    error_summary="frontend server became reachable but process exited before tests"
  fi
fi

if [[ -z "$error_summary" && "$playwright_exit" -ne 0 ]]; then
  if [[ -f "$PLAYWRIGHT_STDERR" ]]; then
    error_summary="$(tail -n 20 "$PLAYWRIGHT_STDERR" | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g' | cut -c1-320)"
  else
    error_summary="playwright exited with non-zero status"
  fi
fi

if [[ -z "$error_summary" && "$report_valid" != "true" ]]; then
  error_summary="playwright report was not valid JSON"
fi

ready=false
if [[ "$server_ready" == "true" \
   && "$gateway_ready" == "true" \
   && "$playwright_exit" -eq 0 \
   && "$report_valid" == "true" \
   && "$unexpected_count" -eq 0 \
   && "$expected_count" -ge 3 ]]; then
  ready=true
fi

jq -n \
  --arg generated_at "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
  --arg base_url "$BASE_URL" \
  --arg spec_path "$SPEC_PATH" \
  --arg server_log "$SERVER_LOG" \
  --arg report "$REPORT" \
  --arg gateway_log "$GATEWAY_LOG" \
  --arg stderr_log "$PLAYWRIGHT_STDERR" \
  --argjson gateway_ready "$gateway_ready" \
  --argjson server_ready "$server_ready" \
  --argjson build_ready "$build_ready" \
  --argjson report_valid "$report_valid" \
  --argjson playwright_exit_code "$playwright_exit" \
  --argjson expected_count "$expected_count" \
  --argjson unexpected_count "$unexpected_count" \
  --arg error_summary "$error_summary" \
  --argjson ready "$ready" \
  '{
    generated_at: $generated_at,
    checks: {
      server_ready: $server_ready,
      gateway_ready: $gateway_ready,
      build_ready: $build_ready,
      report_valid: $report_valid,
      playwright_exit_code: $playwright_exit_code,
      tests_expected: $expected_count,
      tests_unexpected: $unexpected_count,
      main_nav_live_search_and_ask: ($expected_count >= 1 and $unexpected_count == 0),
      ideation_surface_live_controls: ($expected_count >= 2 and $unexpected_count == 0),
      projects_surface_live_controls: ($expected_count >= 3 and $unexpected_count == 0)
    },
    spec_path: $spec_path,
    base_url: $base_url,
    artifacts: {
      playwright_report: $report,
      playwright_stderr: $stderr_log,
      frontend_server_log: $server_log,
      gateway_server_log: $gateway_log
    },
    error_summary: (if $error_summary == "" then null else $error_summary end),
    ready: $ready
  }' > "$OUT"

echo "Wrote UI Playwright matrix: $OUT"
cat "$OUT"

if [[ "$ready" != "true" ]]; then
  echo "Semantic search Playwright checks failed." >&2
  exit 1
fi
