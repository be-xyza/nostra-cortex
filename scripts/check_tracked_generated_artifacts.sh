#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

tracked=()
while IFS= read -r path; do
  [[ -n "$path" ]] || continue
  tracked+=("$path")
done < <(
  git -C "$ROOT_DIR" ls-files -- \
    'cortex/apps/cortex-web/dist/**' \
    'cortex/apps/cortex-web/node_modules/**' \
    'cortex/apps/cortex-web/test-results/**' \
    'cortex/apps/cortex-web/.vite/**' \
    '**/playwright-report/**' \
    'logs/**/*.json' \
    'logs/**/*.jsonl' \
    'logs/**/*.log' \
    'logs/**/*.png' \
    'logs/**/*.jpg' \
    'logs/**/*.jpeg' \
    'logs/**/*.webp' \
    'tmp/**'
)

if [[ "${#tracked[@]}" -gt 0 ]]; then
  echo "FAIL: generated/runtime artifacts are still tracked"
  for path in "${tracked[@]}"; do
    echo " - $path"
  done
  exit 1
fi

echo "PASS: no tracked generated/runtime artifacts in governed paths"
