#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
RUNTIME_DIR="${REPO_ROOT}/tmp/heap_mode_stack"
mkdir -p "${RUNTIME_DIR}"

GATEWAY_URL="${CORTEX_GATEWAY_URL:-http://127.0.0.1:3000}"
WEB_URL="${CORTEX_WEB_URL:-http://127.0.0.1:4173}"
WORKSPACE_ID="${CORTEX_HEAP_DEFAULT_WORKSPACE_ID:-01ARZ3NDEKTSV4RRFFQ69G5FAV}"

GW_PID_FILE="${RUNTIME_DIR}/gateway.pid"
WEB_PID_FILE="${RUNTIME_DIR}/web.pid"
GW_LOG="${RUNTIME_DIR}/gateway.log"
WEB_LOG="${RUNTIME_DIR}/web.log"

usage() {
  cat <<USAGE
Usage: $(basename "$0") <up|down|restart|status|seed|desktop>

Commands:
  up       Start gateway + web (heap parity enabled)
  down     Stop gateway + web started by this script
  restart  down then up
  status   Show process + endpoint health
  seed     Emit demo heap blocks to the gateway
  desktop  Run desktop with heap-mode adapter enabled (foreground)

Environment:
  CORTEX_GATEWAY_URL                 Default: http://127.0.0.1:3000
  CORTEX_WEB_URL                     Default: http://127.0.0.1:4173
  CORTEX_HEAP_DEFAULT_WORKSPACE_ID   Default: 01ARZ3NDEKTSV4RRFFQ69G5FAV
  NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL
  NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED  Default: false
USAGE
}

is_running() {
  local pid_file="$1"
  if [[ ! -f "${pid_file}" ]]; then
    return 1
  fi
  local pid
  pid="$(cat "${pid_file}" 2>/dev/null || true)"
  if [[ -z "${pid}" ]]; then
    return 1
  fi
  kill -0 "${pid}" 2>/dev/null
}

wait_http() {
  local name="$1"
  local url="$2"
  local timeout_secs="${3:-60}"
  local waited=0
  until curl -fsS "${url}" >/dev/null 2>&1; do
    sleep 1
    waited=$((waited + 1))
    if (( waited >= timeout_secs )); then
      echo "error: ${name} did not become healthy within ${timeout_secs}s (${url})" >&2
      return 1
    fi
  done
}

start_gateway() {
  if is_running "${GW_PID_FILE}"; then
    echo "[heap_stack] gateway already running pid=$(cat "${GW_PID_FILE}")"
    return 0
  fi
  echo "[heap_stack] starting gateway -> ${GW_LOG}"
  nohup env CORTEX_GATEWAY_LEGACY_DISPATCH_MODE="${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE:-http_loopback}" \
    NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL:-}" \
    NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED:-false}" \
    NOSTRA_AGENT_ID="${NOSTRA_AGENT_ID:-agent:cortex-heap-stack-local}" \
    "${REPO_ROOT}/run_cortex_gateway" dev >"${GW_LOG}" 2>&1 &
  echo "$!" >"${GW_PID_FILE}"
  wait_http "gateway" "${GATEWAY_URL}/api/system/ready" 120
}

start_web() {
  if is_running "${WEB_PID_FILE}"; then
    echo "[heap_stack] web already running pid=$(cat "${WEB_PID_FILE}")"
    return 0
  fi
  echo "[heap_stack] starting web -> ${WEB_LOG}"
  nohup env \
    VITE_CORTEX_GATEWAY_URL="${GATEWAY_URL}" \
    VITE_HEAP_PARITY_ENABLED=true \
    "${REPO_ROOT}/run_cortex_web" dev >"${WEB_LOG}" 2>&1 &
  echo "$!" >"${WEB_PID_FILE}"
  wait_http "web" "${WEB_URL}" 120
}

stop_by_pid_file() {
  local label="$1"
  local pid_file="$2"
  if ! is_running "${pid_file}"; then
    rm -f "${pid_file}"
    echo "[heap_stack] ${label} not running"
    return 0
  fi
  local pid
  pid="$(cat "${pid_file}")"
  echo "[heap_stack] stopping ${label} pid=${pid}"
  kill "${pid}" 2>/dev/null || true
  sleep 1
  if kill -0 "${pid}" 2>/dev/null; then
    kill -9 "${pid}" 2>/dev/null || true
  fi
  rm -f "${pid_file}"
}

status() {
  local gw_state="stopped"
  local web_state="stopped"
  if is_running "${GW_PID_FILE}"; then
    gw_state="running pid=$(cat "${GW_PID_FILE}")"
  fi
  if is_running "${WEB_PID_FILE}"; then
    web_state="running pid=$(cat "${WEB_PID_FILE}")"
  fi

  echo "[heap_stack] gateway=${gw_state}"
  echo "[heap_stack] web=${web_state}"
  if curl -fsS "${GATEWAY_URL}/api/system/ready" >/dev/null 2>&1; then
    echo "[heap_stack] gateway_health=ok url=${GATEWAY_URL}/api/system/ready"
  else
    echo "[heap_stack] gateway_health=down url=${GATEWAY_URL}/api/system/ready"
  fi
  if curl -fsS "${WEB_URL}" >/dev/null 2>&1; then
    echo "[heap_stack] web_health=ok url=${WEB_URL}"
  else
    echo "[heap_stack] web_health=down url=${WEB_URL}"
  fi
  echo "[heap_stack] workspace_id=${WORKSPACE_ID}"
  echo "[heap_stack] execution_sink_url=${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL:-<disabled>}"
  echo "[heap_stack] execution_sink_fail_closed=${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED:-false}"
  echo "[heap_stack] logs: ${GW_LOG} | ${WEB_LOG}"
}

seed() {
  CORTEX_GATEWAY_URL="${GATEWAY_URL}" \
    CORTEX_HEAP_DEFAULT_WORKSPACE_ID="${WORKSPACE_ID}" \
    "${REPO_ROOT}/scripts/seed_heap_mode_demo.sh"
}

desktop() {
  echo "[heap_stack] launching desktop with CORTEX_HEAP_MODE=1"
  CORTEX_HEAP_MODE=1 \
    CORTEX_HEAP_DEFAULT_WORKSPACE_ID="${WORKSPACE_ID}" \
    "${REPO_ROOT}/run_cortex_desktop" dev
}

cmd="${1:-status}"
case "${cmd}" in
  up)
    start_gateway
    start_web
    echo "[heap_stack] stack is up"
    echo "[heap_stack] web: ${WEB_URL}"
    echo "[heap_stack] gateway: ${GATEWAY_URL}"
    echo "[heap_stack] next: $(basename "$0") seed"
    ;;
  down)
    stop_by_pid_file "web" "${WEB_PID_FILE}"
    stop_by_pid_file "gateway" "${GW_PID_FILE}"
    ;;
  restart)
    stop_by_pid_file "web" "${WEB_PID_FILE}"
    stop_by_pid_file "gateway" "${GW_PID_FILE}"
    start_gateway
    start_web
    echo "[heap_stack] stack restarted"
    ;;
  status)
    status
    ;;
  seed)
    seed
    ;;
  desktop)
    desktop
    ;;
  *)
    usage
    exit 1
    ;;
esac
