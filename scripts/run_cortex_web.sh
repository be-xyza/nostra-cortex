#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WEB_ROOT="${REPO_ROOT}/cortex/apps/cortex-web"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [mode]

Modes:
  dev       Install deps (if needed) and run Vite dev server (default)
  build     Install deps (if needed) and run production build
  preview   Build then run Vite preview server
USAGE
}

MODE="${1:-dev}"
if [[ "$MODE" == "-h" || "$MODE" == "--help" ]]; then
  usage
  exit 0
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "error: npm not found in PATH" >&2
  exit 1
fi

if [[ ! -d "$WEB_ROOT" ]]; then
  echo "error: web host not found at $WEB_ROOT" >&2
  exit 1
fi

cd "$WEB_ROOT"

if [[ ! -d node_modules ]]; then
  echo "[run_cortex_web] installing dependencies"
  npm install --no-audit --no-fund
fi

BUILD_ID="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
BUILD_TIME="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
export VITE_CORTEX_BUILD_ID="${VITE_CORTEX_BUILD_ID:-$BUILD_ID}"
export VITE_CORTEX_BUILD_TIME_UTC="${VITE_CORTEX_BUILD_TIME_UTC:-$BUILD_TIME}"
export VITE_CORTEX_GATEWAY_URL="${VITE_CORTEX_GATEWAY_URL:-http://127.0.0.1:3000}"
if [[ "${VITE_CORTEX_GATEWAY_URL}" != "http://127.0.0.1:3000" ]]; then
  echo "[run_cortex_web] warning: noncanonical gateway override detected (${VITE_CORTEX_GATEWAY_URL}). Canonical local gateway is http://127.0.0.1:3000." >&2
fi
echo "[run_cortex_web] build_id=${VITE_CORTEX_BUILD_ID} build_time_utc=${VITE_CORTEX_BUILD_TIME_UTC} gateway=${VITE_CORTEX_GATEWAY_URL}"

case "$MODE" in
  dev)
    exec "${SCRIPT_DIR}/run_cortex_web_dev.sh"
    ;;
  build)
    exec npm run build
    ;;
  preview)
    npm run build
    exec npm run preview -- --host 127.0.0.1 --port 4173
    ;;
  *)
    echo "error: unknown mode '$MODE'" >&2
    usage
    exit 1
    ;;
esac
