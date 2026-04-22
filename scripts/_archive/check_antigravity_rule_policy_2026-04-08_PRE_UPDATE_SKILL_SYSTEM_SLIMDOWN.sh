#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

bash "$ROOT_DIR/scripts/run_repo_python.sh" "$ROOT_DIR/scripts/check_antigravity_rule_policy.py" "$@"
