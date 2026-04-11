#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CATALOG_PATH="$ROOT_DIR/docs/_meta/catalog.toml"

if [[ ! -f "$CATALOG_PATH" ]]; then
  echo "FAIL: missing docs catalog at docs/_meta/catalog.toml"
  exit 1
fi

FAILURES=0

catalog_has_path() {
  local docs_path="$1"
  if command -v rg >/dev/null 2>&1; then
    rg -q -- "path = \"$docs_path\"" "$CATALOG_PATH"
  else
    grep -Fq -- "path = \"$docs_path\"" "$CATALOG_PATH"
  fi
}

catalog_paths() {
  if command -v rg >/dev/null 2>&1; then
    rg '^path = "docs/' "$CATALOG_PATH"
  else
    grep '^path = "docs/' "$CATALOG_PATH"
  fi
}

while IFS= read -r dir_path; do
  dir_name="$(basename "$dir_path")"
  if [[ "$dir_name" == "_meta" ]]; then
    continue
  fi
  if ! catalog_has_path "docs/$dir_name"; then
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
done < <(catalog_paths)

if [[ "$FAILURES" -gt 0 ]]; then
  exit 1
fi

echo "PASS: docs catalog structure checks"
