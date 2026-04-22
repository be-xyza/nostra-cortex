#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

python3 - "$ROOT_DIR" <<'PY'
from __future__ import annotations

import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
agents = root / "AGENTS.md"
if not agents.exists():
    print("FAIL: missing AGENTS.md")
    raise SystemExit(1)

text = agents.read_text(encoding="utf-8")
checks = {
    "siq": [
        "scripts/run_siq_checks.sh",
        "scripts/check_siq_artifact_consistency.sh",
    ],
    "test_catalog": [
        "scripts/test_catalog_refresh.sh",
        "scripts/check_test_catalog_consistency.sh",
    ],
}

missing: list[str] = []
for key, paths in checks.items():
    section_present = re.search(rf"##\s+{re.escape('SIQ Contract' if key == 'siq' else 'Test Catalog Contract')}", text) is not None
    if not section_present:
        missing.append(f"AGENTS.md missing section for {key}")
    for rel in paths:
        if rel not in text:
            missing.append(f"AGENTS.md missing contract reference '{rel}'")
        if not (root / rel).exists():
            missing.append(f"missing contract script '{rel}'")

if missing:
    print("FAIL: contract surface integrity check failed")
    for row in missing:
        print(f" - {row}")
    raise SystemExit(1)

print("PASS: SIQ/test-catalog contract surface references are live")
PY
