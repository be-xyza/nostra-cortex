#!/bin/bash
set -euo pipefail

search_cmd() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -n -- "$pattern" "$file"
  else
    grep -nE -- "$pattern" "$file"
  fi
}

has_match() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q -- "$pattern" "$file"
  else
    grep -qE -- "$pattern" "$file"
  fi
}

FILES=(
  "AGENTS.md"
  "nostra/spec.md"
  "research/README.md"
  "docs/reference/README.md"
  "docs/architecture/nostra-cortex-boundary.md"
)

FAILURES=0

echo "=== Nostra/Cortex Terminology Check ==="

for file in "${FILES[@]}"; do
  if [ ! -f "$file" ]; then
    echo "FAIL: missing canonical file $file"
    FAILURES=$((FAILURES + 1))
    continue
  fi

  if has_match "Nostra/Cortex" "$file"; then
    echo "FAIL: ambiguous term 'Nostra/Cortex' found in $file"
    search_cmd "Nostra/Cortex" "$file" || true
    FAILURES=$((FAILURES + 1))
  fi
done

BOUNDARY_DOC="docs/architecture/nostra-cortex-boundary.md"
if [ -f "$BOUNDARY_DOC" ]; then
  if ! has_match "Nostra = platform authority \(what exists\)" "$BOUNDARY_DOC"; then
    echo "FAIL: canonical Nostra definition missing"
    FAILURES=$((FAILURES + 1))
  fi
  if ! has_match "Cortex = execution runtime \(how work runs\)" "$BOUNDARY_DOC"; then
    echo "FAIL: canonical Cortex definition missing"
    FAILURES=$((FAILURES + 1))
  fi
  if ! has_match "Nostra Cortex = product umbrella" "$BOUNDARY_DOC"; then
    echo "FAIL: canonical umbrella definition missing"
    FAILURES=$((FAILURES + 1))
  fi
fi

echo ""
if [ "$FAILURES" -eq 0 ]; then
  echo "✅ Terminology checks passed"
  exit 0
else
  echo "❌ $FAILURES terminology check(s) failed"
  exit 1
fi
