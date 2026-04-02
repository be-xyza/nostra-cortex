#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ICP_PROJECT_ROOT="${REPO_ROOT}/ops/hetzner/icp/eudaemon-alpha"
ICP_NETWORK_NAME="${CORTEX_ICP_NETWORK_NAME:-local}"
ICP_CLI_BIN="${ICP_CLI_BIN:-}"

if [[ -n "${NOSTRA_EUDAEMON_ENV_FILE:-}" && -f "${NOSTRA_EUDAEMON_ENV_FILE}" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "${NOSTRA_EUDAEMON_ENV_FILE}"
  set +a
fi

ICP_NETWORK_NAME="${CORTEX_ICP_NETWORK_NAME:-${ICP_NETWORK_NAME}}"

if [[ -z "${ICP_CLI_BIN}" ]]; then
  if command -v icp >/dev/null 2>&1; then
    ICP_CLI_BIN="icp"
  elif command -v icp-cli >/dev/null 2>&1; then
    ICP_CLI_BIN="icp-cli"
  fi
fi

if [[ -z "${ICP_CLI_BIN}" ]]; then
  echo "icp-cli is not installed; nothing to stop."
  exit 0
fi

if ! "${ICP_CLI_BIN}" network status --project-root-override "${ICP_PROJECT_ROOT}" "${ICP_NETWORK_NAME}" >/dev/null 2>&1; then
  echo "Managed ICP network '${ICP_NETWORK_NAME}' is already stopped."
  exit 0
fi

"${ICP_CLI_BIN}" network stop --project-root-override "${ICP_PROJECT_ROOT}" "${ICP_NETWORK_NAME}"
