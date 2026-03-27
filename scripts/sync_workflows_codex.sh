#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
ide="${0##*_}"
ide="${ide%.sh}"
bash "$ROOT_DIR/scripts/run_repo_python.sh" "$ROOT_DIR/scripts/sync_workflows_registry.py" --ide "$ide" "$@"
