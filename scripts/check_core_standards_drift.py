#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMMANDS = [
    ["bash", "scripts/check_no_hardcoded_canister_ids.sh"],
    ["bash", "scripts/check_no_hardcoded_workspace_paths.sh"],
    ["bash", "scripts/check_dynamic_config_contract.sh"],
]


def main() -> int:
    failures: list[str] = []
    for command in COMMANDS:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
        if result.returncode != 0:
            failures.append(f"{' '.join(command)} failed\n{result.stdout}{result.stderr}")

    if failures:
        print("FAIL: core standards drift checks failed")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print("PASS: core standards drift checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
