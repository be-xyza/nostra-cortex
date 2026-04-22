#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

compare_file() {
  local left="$1"
  local right="$2"
  local label="$3"
  if [[ ! -e "$left" && ! -e "$right" ]]; then
    echo "[SKIP] DID pair absent: $label"
    return 0
  fi
  if [[ ! -e "$left" || ! -e "$right" ]]; then
    echo "[FAIL] DID sync pair incomplete: $label" >&2
    echo "  left:  $left" >&2
    echo "  right: $right" >&2
    return 1
  fi
  if ! diff -u "$left" "$right" >/dev/null; then
    echo "[FAIL] DID drift detected: $label" >&2
    echo "  left:  $left" >&2
    echo "  right: $right" >&2
    return 1
  fi
  echo "[OK] $label"
}

compare_normalized_workflow_decl() {
  local hyphen_file="$1"
  local underscore_file="$2"
  if [[ ! -e "$hyphen_file" && ! -e "$underscore_file" ]]; then
    echo "[SKIP] Workflow hyphen/underscore pair absent: $(basename "$hyphen_file")"
    return 0
  fi
  if [[ ! -e "$hyphen_file" || ! -e "$underscore_file" ]]; then
    echo "[FAIL] Workflow declaration pair incomplete between hyphen/underscore trees" >&2
    echo "  hyphen:     $hyphen_file" >&2
    echo "  underscore: $underscore_file" >&2
    return 1
  fi
  if ! diff -u \
    <(sed 's/workflow_engine\.did/workflow-engine.did/g' "$underscore_file") \
    "$hyphen_file" >/dev/null; then
    echo "[FAIL] Workflow declaration drift detected between hyphen/underscore trees" >&2
    echo "  hyphen:     $hyphen_file" >&2
    echo "  underscore: $underscore_file" >&2
    return 1
  fi
  echo "[OK] Workflow hyphen/underscore parity: $(basename "$hyphen_file")"
}

compare_file \
  "$ROOT_DIR/nostra/backend/workflow_engine/workflow_engine.did" \
  "$ROOT_DIR/nostra/src/declarations/workflow_engine/workflow_engine.did" \
  "workflow_engine.did -> declarations/workflow_engine"

compare_file \
  "$ROOT_DIR/nostra/backend/governance/governance.did" \
  "$ROOT_DIR/nostra/src/declarations/governance/governance.did" \
  "governance.did -> declarations/governance"

compare_file \
  "$ROOT_DIR/nostra/backend/nostra_backend.did" \
  "$ROOT_DIR/nostra/src/declarations/nostra_backend/nostra_backend.did" \
  "nostra_backend.did -> declarations/nostra_backend"

compare_normalized_workflow_decl \
  "$ROOT_DIR/nostra/src/declarations/workflow-engine/workflow-engine.did" \
  "$ROOT_DIR/nostra/src/declarations/workflow_engine/workflow_engine.did"

compare_normalized_workflow_decl \
  "$ROOT_DIR/nostra/src/declarations/workflow-engine/workflow-engine.did.js" \
  "$ROOT_DIR/nostra/src/declarations/workflow_engine/workflow_engine.did.js"

compare_normalized_workflow_decl \
  "$ROOT_DIR/nostra/src/declarations/workflow-engine/workflow-engine.did.d.ts" \
  "$ROOT_DIR/nostra/src/declarations/workflow_engine/workflow_engine.did.d.ts"

echo "DID declaration sync check passed."
