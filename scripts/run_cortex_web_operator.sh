#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export VITE_II_OPERATOR_AUTH_ENABLED="${VITE_II_OPERATOR_AUTH_ENABLED:-true}"
export VITE_II_PROVIDER_URL="${VITE_II_PROVIDER_URL:-https://id.ai/authorize}"
export VITE_CORTEX_GATEWAY_URL="${VITE_CORTEX_GATEWAY_URL:-http://127.0.0.1:3000}"

echo "[run_cortex_web_operator] Internet Identity operator sign-in enabled for trusted local web host"
echo "[run_cortex_web_operator] provider=${VITE_II_PROVIDER_URL} gateway=${VITE_CORTEX_GATEWAY_URL}"

exec "${SCRIPT_DIR}/run_cortex_web.sh" dev
