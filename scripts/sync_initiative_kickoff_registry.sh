#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
exec python3 "$ROOT_DIR/scripts/sync_initiative_kickoff_registry.py" "$@"
