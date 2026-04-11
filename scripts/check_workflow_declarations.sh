#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

echo "== workflows registry validation"
bash "$ROOT_DIR/scripts/run_repo_python.sh" "$ROOT_DIR/scripts/sync_workflows_registry.py" --mode validate

echo "== workflows registry drift check"
bash "$ROOT_DIR/scripts/run_repo_python.sh" "$ROOT_DIR/scripts/sync_workflows_registry.py" --mode check

echo "== workflow validation tests"
bash "$ROOT_DIR/scripts/run_repo_python.sh" -m unittest "$ROOT_DIR/tests/test_quick_validate_registry_asset.py"
