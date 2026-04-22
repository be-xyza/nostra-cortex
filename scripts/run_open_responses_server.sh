#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

IMAGE="${OPEN_RESPONSES_SERVER_IMAGE:-ghcr.io/teabranch/open-responses-server:latest}"

HOST="${CORTEX_PROVIDER_RUNTIME_HOST:-127.0.0.1}"
PORT="${CORTEX_PROVIDER_RUNTIME_PORT:-8080}"
API_KEY="${CORTEX_PROVIDER_RUNTIME_API_KEY:-${OPENAI_API_KEY:-}}"

INTERNAL_BASE_URL="${OPENAI_BASE_URL_INTERNAL:-http://host.docker.internal:11434}"
ALLOW_SIDECAR_MCP="${CORTEX_PROVIDER_RUNTIME_ALLOW_SIDECAR_MCP:-false}"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [--] [docker-run-args...]

Starts open-responses-server as a local sidecar.

Environment (defaults shown):
  OPEN_RESPONSES_SERVER_IMAGE=ghcr.io/teabranch/open-responses-server:latest
  CORTEX_PROVIDER_RUNTIME_HOST=127.0.0.1
  CORTEX_PROVIDER_RUNTIME_PORT=8080
  CORTEX_PROVIDER_RUNTIME_API_KEY=\${OPENAI_API_KEY:-}
  OPENAI_BASE_URL_INTERNAL=http://host.docker.internal:11434

Notes:
  - MCP is intentionally NOT configured for v1 (sidecar must not execute tools).
  - Safety: passing MCP_SERVERS_CONFIG_PATH is blocked unless CORTEX_PROVIDER_RUNTIME_ALLOW_SIDECAR_MCP=true.
  - Healthcheck: curl http://<host>:<port>/health
  - OpenAPI: curl http://<host>:<port>/openapi.json
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "error: docker not found in PATH" >&2
  exit 1
fi

python3 - "$HOST" "$PORT" <<'PY'
import socket, sys

host = sys.argv[1]
port = int(sys.argv[2])

sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
try:
    sock.bind((host, port))
except OSError as exc:
    print(f"error: port {host}:{port} unavailable: {exc}", file=sys.stderr)
    raise SystemExit(1)
finally:
    sock.close()
PY

BASE_URL="http://${HOST}:${PORT}"
echo "[run_open_responses_server] image=${IMAGE} base_url=${BASE_URL} internal_base_url=${INTERNAL_BASE_URL}"
echo "[run_open_responses_server] healthcheck: curl ${BASE_URL}/health"
echo "[run_open_responses_server] openapi: curl ${BASE_URL}/openapi.json"

for arg in "$@"; do
  if [[ "$arg" == *"MCP_SERVERS_CONFIG_PATH"* ]]; then
    if [[ "${ALLOW_SIDECAR_MCP}" != "true" ]]; then
      echo "error: refusing to start sidecar with MCP enabled (MCP_SERVERS_CONFIG_PATH detected in args)" >&2
      echo "note: v1 requires Cortex-owned tool loop; set CORTEX_PROVIDER_RUNTIME_ALLOW_SIDECAR_MCP=true to override intentionally" >&2
      exit 2
    fi
    echo "warn: sidecar MCP enabled by args (CORTEX_PROVIDER_RUNTIME_ALLOW_SIDECAR_MCP=true)" >&2
    break
  fi
done

EXTRA_DOCKER_ARGS=()
if [[ "$(uname -s)" == "Linux" ]] && [[ "${INTERNAL_BASE_URL}" == *"host.docker.internal"* ]]; then
  EXTRA_DOCKER_ARGS+=(--add-host=host.docker.internal:host-gateway)
fi

exec docker run --rm \
  "${EXTRA_DOCKER_ARGS[@]}" \
  -p "${HOST}:${PORT}:8080" \
  -e "API_ADAPTER_HOST=0.0.0.0" \
  -e "API_ADAPTER_PORT=8080" \
  -e "OPENAI_BASE_URL=${BASE_URL}" \
  -e "OPENAI_BASE_URL_INTERNAL=${INTERNAL_BASE_URL}" \
  -e "OPENAI_API_KEY=${API_KEY}" \
  "$IMAGE" \
  "$@"
