#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_RS="$ROOT_DIR/nostra/apps/cortex-desktop/src/gateway/server.rs"
FIXTURE_ROOT="$ROOT_DIR/nostra/apps/cortex-desktop/tests/fixtures/gateway_baseline"
INVENTORY_TSV="$FIXTURE_ROOT/endpoint_inventory.tsv"
INVENTORY_JSON="$FIXTURE_ROOT/endpoint_inventory.json"
EXEMPTIONS_JSON="$FIXTURE_ROOT/approved_exemptions.json"
PARITY_CASES_DIR="$FIXTURE_ROOT/parity_cases"

if [[ ! -f "$SERVER_RS" ]]; then
  echo "FAIL: missing gateway server source at $SERVER_RS"
  exit 1
fi

ROOT_DIR="$ROOT_DIR" python3 - <<'PY'
import json
import re
import sys
import os
from pathlib import Path

root = Path(os.environ["ROOT_DIR"])
server = root / "nostra/apps/cortex-desktop/src/gateway/server.rs"
fixture_root = root / "nostra/apps/cortex-desktop/tests/fixtures/gateway_baseline"
inventory_tsv = fixture_root / "endpoint_inventory.tsv"
inventory_json = fixture_root / "endpoint_inventory.json"
exemptions_json = fixture_root / "approved_exemptions.json"
parity_cases_dir = fixture_root / "parity_cases"

raw = server.read_text()
pattern = re.compile(r'\.route\(\s*"([^"]+)"\s*,\s*(get|post|put|delete|patch)\(', re.IGNORECASE | re.MULTILINE)
routes = {(m.group(2).upper(), m.group(1).strip()) for m in pattern.finditer(raw)}

if not routes:
    print("FAIL: no routes discovered from gateway/server.rs")
    sys.exit(1)


def load_tsv(path: Path):
    entries = set()
    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        method, endpoint = line.split("\t", 1)
        entries.add((method.strip().upper(), endpoint.strip()))
    return entries


def load_json_inventory(path: Path):
    parsed = json.loads(path.read_text())
    return {(item["method"].upper(), item["path"]) for item in parsed.get("endpoints", [])}


def load_exemptions(path: Path):
    parsed = json.loads(path.read_text())
    return {(item["method"].upper(), item["path"]) for item in parsed.get("exemptions", [])}


if not inventory_tsv.exists():
    print(f"FAIL: missing inventory TSV: {inventory_tsv}")
    sys.exit(1)
if not inventory_json.exists():
    print(f"FAIL: missing inventory JSON: {inventory_json}")
    sys.exit(1)
if not exemptions_json.exists():
    print(f"FAIL: missing exemptions JSON: {exemptions_json}")
    sys.exit(1)
if not parity_cases_dir.exists():
    print(f"FAIL: missing parity cases directory: {parity_cases_dir}")
    sys.exit(1)

in_tsv = load_tsv(inventory_tsv)
in_json = load_json_inventory(inventory_json)
exemptions = load_exemptions(exemptions_json)
case_count = len(list(parity_cases_dir.glob("*.json")))

ok = True

if routes != in_tsv:
    print("FAIL: route inventory mismatch between gateway/server.rs and endpoint_inventory.tsv")
    print(f"  routes={len(routes)} tsv={len(in_tsv)}")
    missing = sorted(routes - in_tsv)
    extra = sorted(in_tsv - routes)
    if missing:
        print("  missing from TSV:")
        for method, path in missing[:20]:
            print(f"    {method}\t{path}")
    if extra:
        print("  extra in TSV:")
        for method, path in extra[:20]:
            print(f"    {method}\t{path}")
    ok = False

if in_tsv != in_json:
    print("FAIL: endpoint_inventory.json does not mirror endpoint_inventory.tsv")
    print(f"  tsv={len(in_tsv)} json={len(in_json)}")
    ok = False

if len(exemptions) != 0:
    print("FAIL: approved_exemptions_count must be 0 by default")
    print(f"  exemptions={len(exemptions)}")
    ok = False

if len(in_tsv) != case_count + len(exemptions):
    print("FAIL: inventory_count != fixture_count + approved_exemptions_count")
    print(f"  inventory={len(in_tsv)} fixtures={case_count} exemptions={len(exemptions)}")
    ok = False

if ok:
    print("PASS: gateway parity inventory is synchronized")
    print(f"  inventory={len(in_tsv)} fixtures={case_count} exemptions={len(exemptions)}")
    sys.exit(0)

sys.exit(1)
PY
