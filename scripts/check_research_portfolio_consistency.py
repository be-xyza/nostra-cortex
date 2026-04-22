#!/usr/bin/env python3
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
STATUS = ROOT / "research" / "RESEARCH_INITIATIVES_STATUS.md"


def main() -> int:
    raw = STATUS.read_text(encoding="utf-8")
    if any(marker in raw for marker in ("<<<<<<<", "=======", ">>>>>>>")):
        print("FAIL: unresolved merge markers in RESEARCH_INITIATIVES_STATUS.md")
        return 1

    ids: set[int] = set()
    dirs: set[str] = set()
    bad: list[str] = []
    warnings: list[str] = []

    row_re = re.compile(r"^\|\s*(\d{3})\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|")
    for line in raw.splitlines():
        m = row_re.match(line)
        if not m:
            continue
        iid_str, directory, _status = (v.strip() for v in m.groups())
        iid = int(iid_str)

        if iid in ids:
            bad.append(f"duplicate id {iid_str}")
        ids.add(iid)

        if directory in dirs:
            bad.append(f"duplicate directory index {directory}")
        dirs.add(directory)

        if not (ROOT / "research" / directory).exists() and "placeholder" not in _status.lower():
            # If it explicitly says "placeholder" in status, we tolerate missing directory natively, otherwise fail.
            bad.append(f"missing directory research/{directory}, but indexed in table without 'placeholder' status.")

    # 1. Check for gaps in sequence
    if ids:
        max_id = max(ids)
        for i in range(max_id + 1):
            if i not in ids:
                bad.append(f"gap in sequence: ID {i:03d} is missing from the index")

    # 2. Check for physical directories not in index
    for p in (ROOT / "research").iterdir():
        if p.is_symlink():
            # Compatibility shims are allowed for renamed initiatives as long as the canonical
            # directory is indexed and the symlink simply preserves older references.
            continue
        if p.is_dir() and re.match(r"^\d{3}-", p.name):
            if p.name not in dirs:
                bad.append(f"unindexed physical directory found: {p.name}")

    if bad:
        print("FAIL: portfolio consistency violations")
        for issue in bad:
            print(f" - {issue}")
        return 1

    print(f"PASS: {len(ids)} indexed initiatives with unique ids/directories, no gaps, and no unindexed folders.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
