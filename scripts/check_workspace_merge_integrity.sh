#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
STRICT_LEGACY_PATHS=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict-legacy-paths)
      STRICT_LEGACY_PATHS=true
      shift
      ;;
    *)
      echo "FAIL: unknown arg '$1'" >&2
      exit 2
      ;;
  esac
done

unresolved=()
while IFS= read -r path; do
  unresolved+=("$path")
done < <(git -C "$ROOT_DIR" ls-files -u | awk '{print $4}' | sort -u)
fail=0
warn=0
if [[ ${#unresolved[@]} -gt 0 ]]; then
  echo "FAIL: unresolved merge index entries (${#unresolved[@]})"
  for path in "${unresolved[@]}"; do
    echo " - $path"
  done
  fail=1
fi

for component in cortex-domain cortex-runtime cortex-ic-adapter; do
  canonical="$ROOT_DIR/cortex/libraries/$component"
  duplicate="$ROOT_DIR/nostra/libraries/$component"
  if [[ -d "$canonical" && -e "$duplicate" ]]; then
    if [[ "$STRICT_LEGACY_PATHS" == true ]]; then
      echo "FAIL: duplicate cortex library outside canonical path: nostra/libraries/$component"
      fail=1
    else
      echo "WARN: transitional duplicate cortex library still present: nostra/libraries/$component"
      warn=1
    fi
  fi
done

if git -C "$ROOT_DIR" ls-files --error-unmatch 'nostra/apps/cortex-desktop/*' >/dev/null 2>&1; then
  if [[ "$STRICT_LEGACY_PATHS" == true ]]; then
    echo "FAIL: legacy tracked app paths still present under nostra/apps/cortex-desktop"
    fail=1
  else
    echo "WARN: transitional legacy app paths still present under nostra/apps/cortex-desktop"
    warn=1
  fi
fi

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi

if [[ "$warn" -ne 0 ]]; then
  echo "PASS: workspace merge integrity check (with transitional legacy path warnings)"
  exit 0
fi

echo "PASS: workspace merge integrity check"
