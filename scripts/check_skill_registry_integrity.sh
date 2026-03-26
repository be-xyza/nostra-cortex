#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

echo "== skills registry validation"
bash "$ROOT_DIR/scripts/run_repo_python.sh" "$ROOT_DIR/scripts/sync_skills_registry.py" --mode validate

echo "== skills registry drift check"
bash "$ROOT_DIR/scripts/run_repo_python.sh" "$ROOT_DIR/scripts/sync_skills_registry.py" --mode check
