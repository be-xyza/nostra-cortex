#!/usr/bin/env bash
set -euo pipefail

workspace_root="${NOSTRA_WORKSPACE_ROOT:-$(git rev-parse --show-toplevel)}"
gateway_base="${CORTEX_ACP_BASE_URL:-${CORTEX_DESKTOP_ACP_BASE_URL:-http://127.0.0.1:4943}}"
plan_script="${workspace_root}/scripts/plan_a2ui_terminal_payload.sh"
acp_cwd="${A2UI_TERMINAL_ACP_CWD:-${workspace_root}/_bmad}"
run_gateway_script="${workspace_root}/scripts/run_cortex_acp_gateway.sh"
auto_start="${CORTEX_ACP_AUTO_START:-0}"
gateway_port="${CORTEX_GATEWAY_PORT:-${gateway_base##*:}}"
gateway_log="${TMPDIR:-/tmp}/a2ui-terminal-acp-gateway-${gateway_port}.log"
gateway_pid=""

if [[ ! -x "${plan_script}" ]]; then
  echo "plan wrapper not found or not executable at ${plan_script}" >&2
  exit 1
fi

mkdir -p "${acp_cwd}"

cleanup() {
  if [[ -n "${gateway_pid}" ]]; then
    kill "${gateway_pid}" >/dev/null 2>&1 || true
    wait "${gateway_pid}" >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT

health_check() {
  curl -sS "${gateway_base}/api/health" >/dev/null
}

ensure_gateway() {
  if health_check 2>/dev/null; then
    return 0
  fi

  if [[ "${auto_start}" != "1" ]]; then
    echo "gateway not reachable at ${gateway_base}; set CORTEX_ACP_AUTO_START=1 to launch a compatible host" >&2
    exit 1
  fi

  if [[ ! -x "${run_gateway_script}" ]]; then
    echo "gateway runner not found or not executable at ${run_gateway_script}" >&2
    exit 1
  fi

  env \
    NOSTRA_WORKSPACE_ROOT="${workspace_root}" \
    CORTEX_GATEWAY_PORT="${gateway_port}" \
    bash "${run_gateway_script}" >"${gateway_log}" 2>&1 &
  gateway_pid="$!"

  for _ in $(seq 1 80); do
    if health_check 2>/dev/null; then
      return 0
    fi
    sleep 0.25
  done

  echo "gateway did not become healthy at ${gateway_base}" >&2
  if [[ -f "${gateway_log}" ]]; then
    cat "${gateway_log}" >&2
  fi
  exit 1
}

build_command() {
  local fixture="$1"
  printf "NOSTRA_WORKSPACE_ROOT=%q %q --fixture=%q" \
    "${workspace_root}" \
    "${plan_script}" \
    "${fixture}"
}

create_terminal() {
  local fixture="$1"
  local shell_command
  shell_command="$(build_command "${fixture}")"
  curl -sS \
    -X POST \
    -H 'content-type: application/json' \
    "${gateway_base}/api/acp/terminal/create" \
    -d "$(jq -n \
      --arg session_id "a2ui-terminal-smoke" \
      --arg command "/bin/bash" \
      --arg cwd "${acp_cwd}" \
      --arg shell_command "${shell_command}" \
      '{
        sessionId: $session_id,
        command: $command,
        args: ["-lc", $shell_command],
        cwd: $cwd,
        outputByteLimit: 65536
      }')"
}

wait_terminal() {
  local terminal_id="$1"
  curl -sS \
    -X POST \
    -H 'content-type: application/json' \
    "${gateway_base}/api/acp/terminal/wait_for_exit" \
    -d "{\"terminalId\":\"${terminal_id}\",\"timeoutMs\":20000}"
}

read_output() {
  local terminal_id="$1"
  curl -sS \
    -X POST \
    -H 'content-type: application/json' \
    "${gateway_base}/api/acp/terminal/output" \
    -d "{\"terminalId\":\"${terminal_id}\",\"limit\":65536}"
}

decode_output_payload() {
  jq '(.output | fromjson)'
}

run_case() {
  local fixture="$1"
  local create_json
  create_json="$(create_terminal "${fixture}")"
  local terminal_id
  terminal_id="$(printf '%s' "${create_json}" | jq -r '.terminalId // .terminal_id // empty')"
  if [[ -z "${terminal_id}" ]]; then
    echo "failed to create terminal for fixture ${fixture}" >&2
    echo "${create_json}" >&2
    exit 1
  fi

  wait_terminal "${terminal_id}" >/dev/null
  read_output "${terminal_id}" | decode_output_payload
}

ensure_gateway

echo "== repo-heap-note"
run_case "repo-heap-note" | jq '{mode, title, summaryLines, terminalDocumentValidation}'
echo "== repo-workflow-trace"
run_case "repo-workflow-trace" | jq '{mode, title, handoff, terminalDocumentValidation}'
