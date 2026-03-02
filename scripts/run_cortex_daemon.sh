#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CORTEX_ROOT="${REPO_ROOT}/cortex"
APP_ROOT="${CORTEX_ROOT}/apps/cortex-eudaemon"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [mode]

Modes:
  dev         Run with cargo in dev profile (default)
  release     Build release binary and run once

Examples:
  $(basename "$0")
  $(basename "$0") release
USAGE
}

MODE="${1:-dev}"

if [[ "$MODE" == "-h" || "$MODE" == "--help" ]]; then
  usage
  exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found in PATH" >&2
  exit 1
fi

if [[ ! -d "$CORTEX_ROOT" ]]; then
  echo "error: cortex workspace not found at $CORTEX_ROOT" >&2
  exit 1
fi

case "$MODE" in
  dev)
    echo "[run_cortex_daemon] starting dev mode"
    cd "$CORTEX_ROOT"
    export CORTEX_GATEWAY_LEGACY_DISPATCH_MODE="${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE:-http_loopback}"
    export NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL:-}"
    export NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED:-false}"
    export NOSTRA_AGENT_ID="${NOSTRA_AGENT_ID:-agent:cortex-eudaemon-local}"
    BUILD_ID="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
    BUILD_TIME="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    export GIT_SHA="${GIT_SHA:-$BUILD_ID}"
    export BUILD_TIME_UTC="${BUILD_TIME_UTC:-$BUILD_TIME}"
    SINK_STATE="disabled"
    if [[ -n "${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL}" ]]; then
      SINK_STATE="enabled"
    fi
    echo "[run_cortex_daemon] build_id=${GIT_SHA} build_time_utc=${BUILD_TIME_UTC} mode=${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE} agent_id=${NOSTRA_AGENT_ID} execution_sink=${SINK_STATE} fail_closed=${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED}"
    exec cargo run -p cortex-eudaemon --bin cortex_eudaemon
    ;;
  release)
    echo "[run_cortex_daemon] building release binary"
    cd "$CORTEX_ROOT"
    export CORTEX_GATEWAY_LEGACY_DISPATCH_MODE="${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE:-http_loopback}"
    export NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL:-}"
    export NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED="${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED:-false}"
    export NOSTRA_AGENT_ID="${NOSTRA_AGENT_ID:-agent:cortex-eudaemon-local}"
    BUILD_ID="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
    BUILD_TIME="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    export GIT_SHA="${GIT_SHA:-$BUILD_ID}"
    export BUILD_TIME_UTC="${BUILD_TIME_UTC:-$BUILD_TIME}"
    SINK_STATE="disabled"
    if [[ -n "${NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL}" ]]; then
      SINK_STATE="enabled"
    fi
    echo "[run_cortex_daemon] build_id=${GIT_SHA} build_time_utc=${BUILD_TIME_UTC} mode=${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE} agent_id=${NOSTRA_AGENT_ID} execution_sink=${SINK_STATE} fail_closed=${NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED}"
    cargo build --release -p cortex-eudaemon --bin cortex_eudaemon

    BIN_PATH=""
    TARGET_DIR="$(cargo metadata --format-version=1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p' | head -n1)"
    if [[ -n "$TARGET_DIR" && -x "$TARGET_DIR/release/cortex_eudaemon" ]]; then
      BIN_PATH="$TARGET_DIR/release/cortex_eudaemon"
    elif [[ -x "$CORTEX_ROOT/target/release/cortex_eudaemon" ]]; then
      BIN_PATH="$CORTEX_ROOT/target/release/cortex_eudaemon"
    fi

    if [[ -z "$BIN_PATH" ]]; then
      echo "error: release binary not found after build" >&2
      exit 1
    fi

    echo "[run_cortex_daemon] launching $BIN_PATH"
    exec "$BIN_PATH"
    ;;
  *)
    echo "error: unknown mode '$MODE'" >&2
    usage
    exit 1
    ;;
esac
