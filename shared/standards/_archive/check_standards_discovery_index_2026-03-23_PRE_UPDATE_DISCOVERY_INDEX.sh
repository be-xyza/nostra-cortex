#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
INDEX_PATH="$ROOT_DIR/shared/standards/README.md"

python3 - "$ROOT_DIR" "$INDEX_PATH" <<'PY'
from __future__ import annotations

import sys
from pathlib import Path

root = Path(sys.argv[1])
index_path = Path(sys.argv[2])

required_groups = {
    "branding": root / "shared" / "standards" / "branding" / "README.md",
    "heap": root / "shared" / "standards" / "heap" / "README.md",
    "knowledge_graphs": root / "shared" / "standards" / "knowledge_graphs" / "README.md",
    "siq": root / "shared" / "standards" / "siq" / "README.md",
    "testing": root / "shared" / "standards" / "testing" / "README.md",
}

if not index_path.exists():
    print(f"FAIL: missing standards discovery index: {index_path}")
    raise SystemExit(1)

index_text = index_path.read_text(encoding="utf-8")

failures: list[str] = []
for group, readme_path in required_groups.items():
    if not readme_path.exists():
        failures.append(f"missing group README: {readme_path}")
        continue
    relative = f"./{group}/README.md"
    if relative not in index_text:
        failures.append(f"root index does not reference group README: {relative}")
    if group not in index_text:
        failures.append(f"root index does not name group: {group}")

required_refs = [
    "schema_registry.toml",
    "brand_policy.schema.json",
    "brand_policy_v1.json",
    "brand_visual_state_cases_v1.json",
]
for ref in required_refs:
    if ref not in index_text:
        failures.append(f"root index missing required reference: {ref}")

if "./knowledge_graphs/README.md" not in index_text:
    failures.append("root index must reference the knowledge_graphs README")

if failures:
    print("FAIL: standards discovery index coverage failed")
    for failure in failures:
        print(f" - {failure}")
    raise SystemExit(1)

print(f"PASS: standards discovery index covers {len(required_groups)} groups")
PY
