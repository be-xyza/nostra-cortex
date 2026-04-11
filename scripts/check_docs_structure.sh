#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CATALOG_PATH="$ROOT_DIR/docs/_meta/catalog.toml"

if [[ ! -f "$CATALOG_PATH" ]]; then
  echo "FAIL: missing docs catalog at docs/_meta/catalog.toml"
  exit 1
fi

FAILURES=0

while IFS= read -r dir_path; do
  dir_name="$(basename "$dir_path")"
  if [[ "$dir_name" == "_meta" ]]; then
    continue
  fi
  if ! rg -q -- "path = \"docs/$dir_name\"" "$CATALOG_PATH"; then
    echo "FAIL: docs directory '$dir_name' missing from docs/_meta/catalog.toml"
    FAILURES=$((FAILURES + 1))
  fi
done < <(find "$ROOT_DIR/docs" -mindepth 1 -maxdepth 1 -type d | sort)

while IFS= read -r line; do
  rel_path="${line#path = \"}"
  rel_path="${rel_path%\"}"
  if [[ ! -d "$ROOT_DIR/$rel_path" ]]; then
    echo "FAIL: catalog entry points to missing docs path '$rel_path'"
    FAILURES=$((FAILURES + 1))
  fi
done < <(rg '^path = "docs/' "$CATALOG_PATH")

if [[ "$FAILURES" -gt 0 ]]; then
  exit 1
fi

echo "PASS: docs catalog structure checks"
