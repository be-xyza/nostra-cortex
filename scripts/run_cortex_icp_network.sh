#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ICP_PROJECT_ROOT="${REPO_ROOT}/ops/hetzner/icp/eudaemon-alpha"
ICP_NETWORK_NAME="${CORTEX_ICP_NETWORK_NAME:-local}"
ICP_GATEWAY_PORT="${CORTEX_ICP_GATEWAY_PORT:-4943}"
ICP_CLI_BIN="${ICP_CLI_BIN:-}"

if [[ -n "${NOSTRA_EUDAEMON_ENV_FILE:-}" && -f "${NOSTRA_EUDAEMON_ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${NOSTRA_EUDAEMON_ENV_FILE}"
  set +a
fi

ICP_NETWORK_NAME="${CORTEX_ICP_NETWORK_NAME:-${ICP_NETWORK_NAME}}"
ICP_GATEWAY_PORT="${CORTEX_ICP_GATEWAY_PORT:-${ICP_GATEWAY_PORT}}"

if [[ -z "${ICP_CLI_BIN}" ]]; then
  if command -v icp >/dev/null 2>&1; then
    ICP_CLI_BIN="icp"
  elif command -v icp-cli >/dev/null 2>&1; then
    ICP_CLI_BIN="icp-cli"
  fi
fi

if [[ -z "${ICP_CLI_BIN}" ]]; then
  echo "icp-cli is required on PATH" >&2
  exit 1
fi

STATUS_ARGS=(network status --project-root-override "${ICP_PROJECT_ROOT}" "${ICP_NETWORK_NAME}")
START_ARGS=(network start --project-root-override "${ICP_PROJECT_ROOT}" -d "${ICP_NETWORK_NAME}")

if "${ICP_CLI_BIN}" "${STATUS_ARGS[@]}" >/dev/null 2>&1; then
  echo "Managed ICP network '${ICP_NETWORK_NAME}' is already running."
  exit 0
fi

if command -v lsof >/dev/null 2>&1 && lsof -nP -iTCP:${ICP_GATEWAY_PORT} -sTCP:LISTEN >/dev/null 2>&1; then
  echo "Port ${ICP_GATEWAY_PORT} is already occupied, but '${ICP_CLI_BIN} network status' did not recognize the repo-managed network." >&2
  echo "A healthy unmanaged local replica (for example pocket-ic) may already be listening there." >&2
  echo "Manual remediation required before starting the canonical icp-cli lane:" >&2
  lsof -nP -iTCP:${ICP_GATEWAY_PORT} -sTCP:LISTEN >&2
  exit 1
fi

"${ICP_CLI_BIN}" "${START_ARGS[@]}"
"${ICP_CLI_BIN}" "${STATUS_ARGS[@]}"
