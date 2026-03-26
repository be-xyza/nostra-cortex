#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LIB_DIR = ROOT / "scripts" / "lib"
if str(LIB_DIR) not in sys.path:
    sys.path.insert(0, str(LIB_DIR))

from toml_loader import tomllib


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_agent_preflight_contract.py <contract_path>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    contract = tomllib.loads(path.read_text(encoding="utf-8"))

    preflight = contract.get("preflight", {})
    steps = preflight.get("required_steps", [])
    commands = preflight.get("required_commands", [])
    evidence = preflight.get("required_evidence_sections", [])

    missing: list[str] = []
    if not steps:
        missing.append("required_steps")
    if not commands:
        missing.append("required_commands")
    if not evidence:
        missing.append("required_evidence_sections")

    if missing:
        print("FAIL: preflight contract missing required sections: " + ", ".join(missing))
        return 1

    print("PASS: preflight contract has required sections")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
