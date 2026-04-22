#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CORTEX_ROOT="${REPO_ROOT}/cortex"

MODE="${1:-dev}"

if [[ "$MODE" == "dev" ]]; then
  cd "$CORTEX_ROOT"
  # Local dev convenience: enable authz dev mode so browser clients can request roles via query/header.
  # Production/CI should explicitly disable these env vars.
  export NOSTRA_AUTHZ_DEV_MODE="${NOSTRA_AUTHZ_DEV_MODE:-true}"
  export NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER="${NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER:-true}"
  export CORTEX_GATEWAY_LEGACY_DISPATCH_MODE="${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE:-http_loopback}"
  export NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL:-}"
  export NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED:-false}"
  export NOSTRA_AGENT_ID="${NOSTRA_AGENT_ID:-agent:cortex-gateway-local}"
  BUILD_ID="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
  BUILD_TIME="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  export GIT_SHA="${GIT_SHA:-$BUILD_ID}"
  export BUILD_TIME_UTC="${BUILD_TIME_UTC:-$BUILD_TIME}"
  SINK_STATE="disabled"
  if [[ -n "${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL}" ]]; then
    SINK_STATE="enabled"
  fi
  echo "[run_cortex_gateway] build_id=${GIT_SHA} build_time_utc=${BUILD_TIME_UTC} mode=${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE} agent_id=${NOSTRA_AGENT_ID} execution_sink=${SINK_STATE} fail_closed=${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED}"
  exec cargo run -p cortex-gateway
elif [[ "$MODE" == "release" ]]; then
  cd "$CORTEX_ROOT"
  export CORTEX_GATEWAY_LEGACY_DISPATCH_MODE="${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE:-http_loopback}"
  export NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL:-}"
  export NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED:-false}"
  export NOSTRA_AGENT_ID="${NOSTRA_AGENT_ID:-agent:cortex-gateway-local}"
  BUILD_ID="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
  BUILD_TIME="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  export GIT_SHA="${GIT_SHA:-$BUILD_ID}"
  export BUILD_TIME_UTC="${BUILD_TIME_UTC:-$BUILD_TIME}"
  SINK_STATE="disabled"
  if [[ -n "${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL}" ]]; then
    SINK_STATE="enabled"
  fi
  echo "[run_cortex_gateway] build_id=${GIT_SHA} build_time_utc=${BUILD_TIME_UTC} mode=${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE} agent_id=${NOSTRA_AGENT_ID} execution_sink=${SINK_STATE} fail_closed=${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED}"
  cargo build --release -p cortex-gateway
  exec "$CORTEX_ROOT/target/release/cortex-gateway"
else
  echo "Usage: $(basename "$0") [dev|release]" >&2
  exit 1
fi
