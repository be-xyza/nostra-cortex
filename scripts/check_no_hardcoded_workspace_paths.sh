#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
source "$ROOT_DIR/scripts/lib/workspace_paths.sh"
resolve_workspace_paths "$ROOT_DIR"

scan_targets=("$ROOT_DIR/scripts")
if [[ -f "$CORTEX_DESKTOP_DIR/src/gateway/server.rs" ]]; then
  scan_targets+=("$CORTEX_DESKTOP_DIR/src/gateway/server.rs")
fi

hits=()
while IFS= read -r line; do
  hits+=("$line")
done < <(rg -n "/Users/xaoj/ICP" \
  "${scan_targets[@]}" \
  -g '!**/_archive/**' \
  -g '!**/__pycache__/**' \
  -g '!check_no_hardcoded_workspace_paths.sh' \
  -g '!**/*.log' \
  -S)

if [[ ${#hits[@]} -gt 0 ]]; then
  echo "FAIL: hardcoded workspace root paths detected"
  for hit in "${hits[@]}"; do
    echo " - $hit"
  done
  exit 1
fi

echo "PASS: no hardcoded workspace root paths in active gateway/scripts"
