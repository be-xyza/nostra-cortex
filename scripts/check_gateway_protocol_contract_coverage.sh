#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/lib/workspace_paths.sh"
resolve_workspace_paths "$ROOT_DIR"
GATEWAY_APP_DIR="$CORTEX_EUDAEMON_DIR"
INVENTORY_TSV="$GATEWAY_APP_DIR/tests/fixtures/gateway_baseline/endpoint_inventory.tsv"
CONTRACT_JSON="$ROOT_DIR/research/118-cortex-runtime-extraction/GATEWAY_PROTOCOL_CONTRACT_2026-02-16.json"

if [[ ! -f "$INVENTORY_TSV" ]]; then
  echo "FAIL: missing inventory TSV: $INVENTORY_TSV"
  exit 1
fi

if [[ ! -f "$CONTRACT_JSON" ]]; then
  echo "FAIL: missing contract JSON: $CONTRACT_JSON"
  exit 1
fi

ROOT_DIR="$ROOT_DIR" GATEWAY_APP_DIR="$GATEWAY_APP_DIR" python3 - <<'PY'
import json
import os
import sys
from pathlib import Path

root = Path(os.environ["ROOT_DIR"])
gateway_app_dir = Path(os.environ["GATEWAY_APP_DIR"])
inventory_tsv = gateway_app_dir / "tests/fixtures/gateway_baseline/endpoint_inventory.tsv"
contract_json = root / "research/118-cortex-runtime-extraction/GATEWAY_PROTOCOL_CONTRACT_2026-02-16.json"

required_fields = {
    "method",
    "path_template",
    "request_schema",
    "response_schema",
    "error_normalization",
    "event_emissions",
    "transaction_boundary",
    "idempotency_semantics",
}

inventory = []
for line in inventory_tsv.read_text().splitlines():
    line = line.strip()
    if not line:
        continue
    method, path = line.split("\t", 1)
    inventory.append((method.strip().upper(), path.strip()))

contract = json.loads(contract_json.read_text())
entries = contract.get("endpoints", [])

if not isinstance(entries, list):
    print("FAIL: contract JSON field 'endpoints' must be an array")
    sys.exit(1)

ok = True
contract_pairs = []
for idx, entry in enumerate(entries):
    if not isinstance(entry, dict):
        print(f"FAIL: contract endpoint at index {idx} is not an object")
        ok = False
        continue

    missing = [field for field in required_fields if field not in entry]
    if missing:
        print(f"FAIL: contract endpoint {idx} missing required fields: {', '.join(sorted(missing))}")
        ok = False

    method = str(entry.get("method", "")).upper().strip()
    path = str(entry.get("path_template", "")).strip()

    if not method or not path:
        print(f"FAIL: contract endpoint {idx} has empty method/path_template")
        ok = False
        continue

    contract_pairs.append((method, path))

inventory_set = set(inventory)
contract_set = set(contract_pairs)

if len(contract_pairs) != len(contract_set):
    print("FAIL: duplicate method+path_template entries found in contract JSON")
    ok = False

if len(inventory) != len(entries):
    print(
        "FAIL: inventory_count != contract_entries_count "
        f"(inventory={len(inventory)} contract={len(entries)})"
    )
    ok = False

missing = sorted(inventory_set - contract_set)
extra = sorted(contract_set - inventory_set)

if missing:
    print("FAIL: missing method+path entries in contract JSON:")
    for method, path in missing[:30]:
        print(f"  {method}\t{path}")
    if len(missing) > 30:
        print(f"  ... and {len(missing) - 30} more")
    ok = False

if extra:
    print("FAIL: extra method+path entries in contract JSON not present in inventory:")
    for method, path in extra[:30]:
        print(f"  {method}\t{path}")
    if len(extra) > 30:
        print(f"  ... and {len(extra) - 30} more")
    ok = False

if ok:
    print("PASS: gateway protocol contract covers full inventory")
    print(f"  inventory={len(inventory)} contract={len(entries)}")
    sys.exit(0)

sys.exit(1)
PY
