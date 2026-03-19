#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACT_JSON="$ROOT_DIR/research/118-cortex-runtime-extraction/GATEWAY_PROTOCOL_CONTRACT_2026-02-16.json"

if [[ ! -f "$CONTRACT_JSON" ]]; then
  echo "FAIL: missing contract JSON: $CONTRACT_JSON"
  exit 1
fi

ROOT_DIR="$ROOT_DIR" python3 - <<'PY'
import json
import os
import re
import sys
from pathlib import Path

contract_path = Path(os.environ["ROOT_DIR"]) / "research/118-cortex-runtime-extraction/GATEWAY_PROTOCOL_CONTRACT_2026-02-16.json"
contract = json.loads(contract_path.read_text())
entries = contract.get("endpoints", [])

if not isinstance(entries, list):
    print("FAIL: contract endpoints must be a list")
    sys.exit(1)

allowed_boundaries = {
    "read_only",
    "single_request_mutation",
    "multi_step_best_effort",
    "host_managed",
    "streaming_session",
}
allowed_idempotency = {
    "not_applicable",
    "optional_header",
    "recommended_header",
    "required_header",
}
allowed_methods = {"GET", "POST", "PUT"}
param_re = re.compile(r"^[a-z][a-z0-9_]*$")

ok = True
seen = set()
for idx, entry in enumerate(entries):
    if not isinstance(entry, dict):
        print(f"FAIL: endpoint[{idx}] must be an object")
        ok = False
        continue

    method = str(entry.get("method", "")).upper().strip()
    path = str(entry.get("path_template", "")).strip()
    boundary = str(entry.get("transaction_boundary", "")).strip()
    idem_mode = str(entry.get("idempotency_semantics", {}).get("mode", "")).strip()

    pair = (method, path)
    if pair in seen:
        print(f"FAIL: duplicate endpoint contract: {method} {path}")
        ok = False
    seen.add(pair)

    if method not in allowed_methods:
        print(f"FAIL: endpoint[{idx}] has unsupported method: {method}")
        ok = False

    if not path.startswith("/") or "?" in path:
        print(f"FAIL: endpoint[{idx}] has invalid path_template: {path}")
        ok = False

    if boundary not in allowed_boundaries:
        print(f"FAIL: endpoint[{idx}] has invalid transaction_boundary: {boundary}")
        ok = False

    if idem_mode not in allowed_idempotency:
        print(f"FAIL: endpoint[{idx}] has invalid idempotency_semantics.mode: {idem_mode}")
        ok = False

    path_params = [segment[1:] for segment in path.split("/") if segment.startswith(":")]
    if len(path_params) != len(set(path_params)):
        print(f"FAIL: endpoint[{idx}] path_template has duplicate params: {path}")
        ok = False
    for param in path_params:
        if not param_re.match(param):
            print(f"FAIL: endpoint[{idx}] has invalid path param '{param}' in {path}")
            ok = False

if ok:
    print("PASS: gateway protocol contract descriptors are strict-valid")
    print(f"  descriptors={len(entries)}")
    sys.exit(0)

sys.exit(1)
PY
