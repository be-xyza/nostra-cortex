#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

if [[ ! -f "$ROOT_DIR/shared/standards/dynamic_source_contract.toml" ]]; then
  echo "FAIL: missing dynamic source contract at shared/standards/dynamic_source_contract.toml" >&2
  exit 1
fi

echo "== dynamic config contract: governance bypass scan"
bash "$ROOT_DIR/scripts/run_repo_python.sh" "$ROOT_DIR/scripts/check_governance_bypass_hardcodes.py"

echo "== dynamic config contract: workspace root hardcode scan"
bash "$ROOT_DIR/scripts/check_no_hardcoded_workspace_paths.sh"
