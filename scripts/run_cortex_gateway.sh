#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CORTEX_ROOT="${REPO_ROOT}/cortex"

MODE="${1:-dev}"
TARGET_DIR="${REPO_ROOT}/.cache/cargo-target/root-workspace"
DOCLING_VENV_PYTHON="${REPO_ROOT}/.cache/docling-venv/bin/python"
DOCLING_UPSTREAM_SCRIPT="${REPO_ROOT}/scripts/docling_upstream_adapter.py"

configure_parser_adapters() {
  local profile="$1"
  local target_subdir="debug"
  if [[ "$profile" == "release" ]]; then
    target_subdir="release"
    cargo build \
      --manifest-path "${REPO_ROOT}/nostra/extraction/Cargo.toml" \
      --release \
      --bin nostra-docling-adapter \
      --bin nostra-liteparse-adapter \
      --bin nostra-markitdown-adapter
  else
    cargo build \
      --manifest-path "${REPO_ROOT}/nostra/extraction/Cargo.toml" \
      --bin nostra-docling-adapter \
      --bin nostra-liteparse-adapter \
      --bin nostra-markitdown-adapter
  fi

  export NOSTRA_DOCLING_ADAPTER_EXECUTABLE="${NOSTRA_DOCLING_ADAPTER_EXECUTABLE:-${TARGET_DIR}/${target_subdir}/nostra-docling-adapter}"
  export NOSTRA_LITEPARSE_ADAPTER_EXECUTABLE="${NOSTRA_LITEPARSE_ADAPTER_EXECUTABLE:-${TARGET_DIR}/${target_subdir}/nostra-liteparse-adapter}"
  export NOSTRA_MARKITDOWN_ADAPTER_EXECUTABLE="${NOSTRA_MARKITDOWN_ADAPTER_EXECUTABLE:-${TARGET_DIR}/${target_subdir}/nostra-markitdown-adapter}"

  if [[ -z "${NOSTRA_DOCLING_UPSTREAM_COMMAND_JSON:-}" ]] && [[ -x "${DOCLING_VENV_PYTHON}" ]] && [[ -f "${DOCLING_UPSTREAM_SCRIPT}" ]]; then
    export NOSTRA_DOCLING_UPSTREAM_COMMAND_JSON="[\"${DOCLING_VENV_PYTHON}\",\"${DOCLING_UPSTREAM_SCRIPT}\"]"
  fi
}

configure_gateway_env() {
  export NOSTRA_WORKSPACE_ROOT="${NOSTRA_WORKSPACE_ROOT:-$REPO_ROOT}"
  export CORTEX_PROVIDER_STATE_PATH="${CORTEX_PROVIDER_STATE_PATH:-$REPO_ROOT/_system/providers.json}"
  export CORTEX_GATEWAY_LEGACY_DISPATCH_MODE="${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE:-http_loopback}"
  export NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL:-}"
  export NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED:-false}"
  export NOSTRA_AGENT_ID="${NOSTRA_AGENT_ID:-agent:cortex-gateway-local}"
  BUILD_ID="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
  BUILD_TIME="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  export GIT_SHA="${GIT_SHA:-$BUILD_ID}"
  export BUILD_TIME_UTC="${BUILD_TIME_UTC:-$BUILD_TIME}"
}

log_gateway_startup() {
  local sink_state="disabled"
  local docling_upstream_state="disabled"
  if [[ -n "${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL}" ]]; then
    sink_state="enabled"
  fi
  if [[ -n "${NOSTRA_DOCLING_UPSTREAM_COMMAND_JSON:-}" ]]; then
    docling_upstream_state="enabled"
  fi
  echo "[run_cortex_gateway] build_id=${GIT_SHA} build_time_utc=${BUILD_TIME_UTC} mode=${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE} agent_id=${NOSTRA_AGENT_ID} execution_sink=${sink_state} fail_closed=${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED} docling_adapter=${NOSTRA_DOCLING_ADAPTER_EXECUTABLE} docling_upstream=${docling_upstream_state}"
}

if [[ "$MODE" == "dev" ]]; then
  cd "$CORTEX_ROOT"
  configure_gateway_env
  # Local dev convenience: enable authz dev mode so browser clients can request roles via query/header.
  # Production/CI should explicitly disable these env vars.
  export NOSTRA_AUTHZ_DEV_MODE="${NOSTRA_AUTHZ_DEV_MODE:-true}"
  export NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER="${NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER:-true}"
  configure_parser_adapters "dev"
  log_gateway_startup
  exec cargo run -p cortex-gateway
elif [[ "$MODE" == "release" ]]; then
  cd "$CORTEX_ROOT"
  configure_gateway_env
  configure_parser_adapters "release"
  log_gateway_startup
  cargo build --release -p cortex-gateway
  exec "$CORTEX_ROOT/target/release/cortex-gateway"
else
  echo "Usage: $(basename "$0") [dev|release]" >&2
  exit 1
fi
