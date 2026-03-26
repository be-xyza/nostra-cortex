#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CONTRACT_PATH="$ROOT_DIR/shared/standards/alignment_contracts.toml"

bash "$ROOT_DIR/scripts/run_repo_python.sh" "$ROOT_DIR/scripts/check_alignment_contract_targets.py" "$ROOT_DIR" "$CONTRACT_PATH"
