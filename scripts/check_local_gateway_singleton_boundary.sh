#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/lib/workspace_paths.sh"
resolve_workspace_paths "$ROOT_DIR"

violations="$(
  rg -n "get_gateway\(" \
    "$CORTEX_DESKTOP_DIR/src/gateway/runtime_host.rs" \
    "$CORTEX_DESKTOP_DIR/src/gateway/server.rs" \
    "$CORTEX_DESKTOP_DIR/src/services/resilience_service.rs" \
    "$CORTEX_DESKTOP_DIR/src/services/agent_service.rs" \
    || true
)"

if [[ -n "$violations" ]]; then
  echo "FAIL: local-gateway singleton boundary violated for phase-5 target flows"
  echo "$violations"
  exit 1
fi

echo "PASS: phase-5 target flows do not call local_gateway::get_gateway directly"
