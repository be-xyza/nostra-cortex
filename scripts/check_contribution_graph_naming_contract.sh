#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SCAN_PATHS=(
  "$ROOT_DIR/cortex/apps/cortex-eudaemon/src"
  "$ROOT_DIR/cortex/apps/cortex-worker/src"
  "$ROOT_DIR/cortex/apps/cortex-web/src"
  "$ROOT_DIR/nostra/extraction/src"
  "$ROOT_DIR/nostra/A2UI/specification/v0_8/json/standard_catalog_definition.json"
  "$ROOT_DIR/shared/standards/siq"
  "$ROOT_DIR/shared/standards/knowledge_graphs"
  "$ROOT_DIR/scripts/siq_tools.py"
  "$ROOT_DIR/scripts/check_siq_artifact_consistency.sh"
  "$ROOT_DIR/research/000-contribution-graph"
  "$ROOT_DIR/docs/architecture"
)

PATTERN='initiative-graph|initiative_graph|InitiativeGraph|InitiativeNode|InitiativeEdge|initiativeId|initiative_id'

if rg -n --hidden -S "$PATTERN" "${SCAN_PATHS[@]}" \
  -g '!**/_archive/**' \
  -g '!**/node_modules/**' \
  -g '!**/dist/**' \
  -g '!**/contribution-graph-naming.md' \
  -g '!**/check_contribution_graph_naming_contract.sh'; then
  echo "FAIL: contribution graph naming contract violation(s) detected"
  exit 1
fi

echo "PASS: contribution graph naming contract"
