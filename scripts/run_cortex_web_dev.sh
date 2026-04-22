#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="${NOSTRA_WORKSPACE_ROOT:-$(cd "${SCRIPT_DIR}/.." && pwd)}"
APP_DIR="${WORKSPACE_ROOT}/cortex/apps/cortex-web"
HOST="${CORTEX_WEB_HOST:-127.0.0.1}"
PORT="${CORTEX_WEB_PORT:-4173}"
GRAPH_V2="${VITE_CAPABILITY_GRAPH_V2_ENABLED:-true}"
HEAP_PARITY="${VITE_HEAP_PARITY_ENABLED:-true}"
GATEWAY_URL="${CORTEX_WEB_GATEWAY_URL:-${VITE_CORTEX_GATEWAY_URL:-http://127.0.0.1:3000}}"

if [[ ! -d "${APP_DIR}" ]]; then
  echo "cortex-web app directory not found: ${APP_DIR}" >&2
  exit 1
fi

cd "${APP_DIR}"
export VITE_CAPABILITY_GRAPH_V2_ENABLED="${GRAPH_V2}"
export VITE_HEAP_PARITY_ENABLED="${HEAP_PARITY}"
export VITE_CORTEX_GATEWAY_URL="${GATEWAY_URL}"
if [[ "${VITE_CORTEX_GATEWAY_URL}" != "http://127.0.0.1:3000" ]]; then
  echo "[run_cortex_web_dev] warning: noncanonical gateway override detected (${VITE_CORTEX_GATEWAY_URL}). Canonical local gateway is http://127.0.0.1:3000." >&2
fi
echo "[run_cortex_web_dev] host=${HOST} port=${PORT} gateway=${VITE_CORTEX_GATEWAY_URL}"
exec npm run dev -- --host "${HOST}" --port "${PORT}"
