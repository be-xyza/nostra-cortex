#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
INDEX_PATH="$ROOT_DIR/shared/standards/README.md"
python3 "$ROOT_DIR/scripts/validate_standards_registry.py" --root "$ROOT_DIR"

python3 - "$ROOT_DIR" "$INDEX_PATH" <<'PY'
from __future__ import annotations

import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
index_path = Path(sys.argv[2])

required_docs = {
    "ACCESSIBILITY.md": root / "shared" / "standards" / "ACCESSIBILITY.md",
    "LOCAL_FIRST.md": root / "shared" / "standards" / "LOCAL_FIRST.md",
    "ROLE_SEMANTICS.md": root / "shared" / "standards" / "ROLE_SEMANTICS.md",
    "TECHNOLOGY_NEUTRALITY.md": root / "shared" / "standards" / "TECHNOLOGY_NEUTRALITY.md",
}

required_root_contracts = {
    "agent_failure_modes.toml": root / "shared" / "standards" / "agent_failure_modes.toml",
    "agent_preflight_contract.toml": root / "shared" / "standards" / "agent_preflight_contract.toml",
    "alignment_contracts.toml": root / "shared" / "standards" / "alignment_contracts.toml",
    "antigravity_rule_policy.toml": root / "shared" / "standards" / "antigravity_rule_policy.toml",
    "dynamic_source_contract.toml": root / "shared" / "standards" / "dynamic_source_contract.toml",
}

required_root_artifacts = {
    "alignment_contract_exceptions.json": root / "shared" / "standards" / "alignment_contract_exceptions.json",
    "antigravity_rule_dispositions.json": root / "shared" / "standards" / "antigravity_rule_dispositions.json",
    "dynamic_source_exceptions.json": root / "shared" / "standards" / "dynamic_source_exceptions.json",
}

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
link_targets = set(re.findall(r"\[[^\]]+\]\(([^)]+)\)", index_text))

failures: list[str] = []
for name, file_path in required_docs.items():
    if not file_path.exists():
        failures.append(f"missing core standards doc: {file_path}")
        continue
    relative = f"./{name}"
    if relative not in link_targets:
        failures.append(f"root index does not reference core standards doc: {relative}")

for name, file_path in required_root_contracts.items():
    if not file_path.exists():
        failures.append(f"missing root contract file: {file_path}")
        continue
    relative = f"./{name}"
    if relative not in link_targets:
        failures.append(f"root index does not reference root contract file: {relative}")

for name, file_path in required_root_artifacts.items():
    if not file_path.exists():
        failures.append(f"missing root governed artifact: {file_path}")
        continue
    relative = f"./{name}"
    if relative not in link_targets:
        failures.append(f"root index does not reference root governed artifact: {relative}")

for group, readme_path in required_groups.items():
    if not readme_path.exists():
        failures.append(f"missing group README: {readme_path}")
        continue
    relative = f"./{group}/README.md"
    if relative not in link_targets:
        failures.append(f"root index does not reference group README: {relative}")

required_refs = [
    "./standards_registry.toml",
    "./schema_guided_extraction_context.json",
    "./knowledge_graphs/schema_registry.toml",
    "./branding/brand_policy.schema.json",
    "./branding/brand_policy_v1.json",
    "./branding/brand_visual_state_cases_v1.json",
]
for ref in required_refs:
    if ref not in link_targets:
        failures.append(f"root index missing required reference: {ref}")

if "./knowledge_graphs/README.md" not in link_targets:
    failures.append("root index must reference the knowledge_graphs README")

if failures:
    print("FAIL: standards discovery index coverage failed")
    for failure in failures:
        print(f" - {failure}")
    raise SystemExit(1)

print(
    "PASS: standards discovery index covers "
    f"{len(required_docs)} core docs, "
    f"{len(required_root_contracts)} root contracts, "
    f"{len(required_root_artifacts)} root artifacts, "
    f"{len(required_groups)} groups"
)
PY
