#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CORTEX_ROOT="${REPO_ROOT}/cortex"

if [[ -n "${NOSTRA_EUDAEMON_ENV_FILE:-}" && -f "${NOSTRA_EUDAEMON_ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${NOSTRA_EUDAEMON_ENV_FILE}"
  set +a
fi

export NOSTRA_AUTHZ_DEV_MODE="${NOSTRA_AUTHZ_DEV_MODE:-0}"
export NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER="${NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER:-0}"
export NOSTRA_AGENT_IDENTITY_ENFORCEMENT="${NOSTRA_AGENT_IDENTITY_ENFORCEMENT:-1}"
export CORTEX_GATEWAY_PORT="${CORTEX_GATEWAY_PORT:-3000}"
export CORTEX_GATEWAY_LEGACY_DISPATCH_MODE="${CORTEX_GATEWAY_LEGACY_DISPATCH_MODE:-http_loopback}"

cd "${CORTEX_ROOT}"
cargo build --release -p cortex-gateway
exec "${CORTEX_ROOT}/target/release/cortex-gateway"
