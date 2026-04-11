#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MATCH_FILE="/tmp/reference_path_hygiene_matches.txt"

declare -a PATTERNS=(
  "research/references/"
  "docs/references/"
  "research/reference/repositories/"
  "possible_links_to_nostra_cortex"
)

declare -a TARGETS=(
  "$ROOT_DIR/AGENTS.md"
  "$ROOT_DIR/docs/reference/README.md"
  "$ROOT_DIR/research/reference/index.md"
  "$ROOT_DIR/research/reference/index.toml"
  "$ROOT_DIR/research/reference/analysis"
)

FAILURES=0

for pattern in "${PATTERNS[@]}"; do
  if rg -n --glob '!**/_archive/**' -- "$pattern" "${TARGETS[@]}" >"$MATCH_FILE" 2>/dev/null; then
    if [[ "$pattern" == "possible_links_to_nostra_cortex" ]]; then
      grep -v 'docs/reference/README.md:.*do not use `possible_links_to_nostra_cortex`' "$MATCH_FILE" >"${MATCH_FILE}.filtered" || true
      mv "${MATCH_FILE}.filtered" "$MATCH_FILE"
    fi
    if [[ ! -s "$MATCH_FILE" ]]; then
      continue
    fi
    echo "FAIL: legacy reference path pattern detected: $pattern"
    cat "$MATCH_FILE"
    FAILURES=$((FAILURES + 1))
  fi
done

rm -f "$MATCH_FILE"

if [[ "$FAILURES" -gt 0 ]]; then
  exit 1
fi

echo "PASS: reference path hygiene checks"
