#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STATUS_PATH = ROOT / "research" / "RESEARCH_INITIATIVES_STATUS.md"

NON_ACTIVE_STATUSES = {"archived", "superseded", "completed"}


def parse_index() -> list[tuple[str, str, str]]:
    rows: list[tuple[str, str, str]] = []
    row_re = re.compile(r"^\|\s*(\d{3})\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|")
    for line in STATUS_PATH.read_text(encoding="utf-8").splitlines():
        match = row_re.match(line)
        if not match:
            continue
        rows.append((match.group(1).strip(), match.group(2).strip(), match.group(3).strip().lower()))
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description="Detect template residue in active initiative folders")
    parser.add_argument("--strict", action="store_true", help="Fail when residue is detected")
    args = parser.parse_args()

    residue: list[str] = []
    for iid, directory, status in parse_index():
        if status in NON_ACTIVE_STATUSES:
            continue
        folder = ROOT / "research" / directory
        if not folder.exists() or not folder.is_dir():
            continue
        for candidate in sorted(folder.glob("*.template.md")):
            residue.append(str(candidate.relative_to(ROOT)))

    if residue:
        level = "FAIL" if args.strict else "WARN"
        print(f"{level}: template residue found in active initiatives")
        for path in residue:
            print(f" - {path}")
        return 1 if args.strict else 0

    print("PASS: no active initiative template residue detected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
