#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CONTRACT_PATH="$ROOT_DIR/shared/standards/agent_preflight_contract.toml"

if [[ ! -f "$CONTRACT_PATH" ]]; then
  echo "FAIL: missing preflight contract $CONTRACT_PATH" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/run_repo_python.sh" "$ROOT_DIR/scripts/check_agent_preflight_contract.py" "$CONTRACT_PATH"

for cmd in \
  "$ROOT_DIR/scripts/check_repo_python_runtime.sh" \
  "$ROOT_DIR/scripts/check_clean_worktree.sh" \
  "$ROOT_DIR/scripts/check_tracked_generated_artifacts.sh" \
  "$ROOT_DIR/scripts/check_dynamic_config_contract.sh" \
  "$ROOT_DIR/scripts/check_skill_registry_integrity.sh" \
  "$ROOT_DIR/scripts/check_skill_policy.sh" \
  "$ROOT_DIR/scripts/check_workflow_declarations.sh" \
  "$ROOT_DIR/scripts/check_alignment_contract_targets.sh"
do
  if [[ ! -f "$cmd" ]]; then
    echo "FAIL: required command target missing: $cmd" >&2
    exit 1
  fi
done

echo "PASS: agent preflight contract checks"
