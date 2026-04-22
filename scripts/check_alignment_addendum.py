#!/usr/bin/env python3
from __future__ import annotations

import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
STRICT_INITIATIVE_SET = {"097", "099", "101", "103", "105", "118", "121", "123", "124", "125"}


def read_frontmatter(path: pathlib.Path) -> dict[str, str]:
    raw = path.read_text(encoding="utf-8")
    if not raw.startswith("---\n"):
        return {}
    end = raw.find("\n---", 4)
    if end == -1:
        return {}
    frontmatter = raw[4:end]
    out: dict[str, str] = {}
    for key in ("id", "status", "portfolio_role"):
        match = re.search(rf"(?m)^\s*{re.escape(key)}\s*:\s*(.+?)\s*$", frontmatter)
        out[key] = match.group(1).strip().strip('"\'') if match else ""
    return out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []
    for plan in sorted((ROOT / "research").glob("[0-9][0-9][0-9]-*/PLAN.md")):
        frontmatter = read_frontmatter(plan)
        status = frontmatter.get("status", "")
        if status != "active":
            continue
        initiative_id = frontmatter.get("id", "")
        enforce_strict = initiative_id in STRICT_INITIATIVE_SET
        if args.strict and not enforce_strict:
            continue
        body = plan.read_text(encoding="utf-8")
        if "Alignment Addendum" in body:
            continue
        failures.append(f"missing 'Alignment Addendum' section: {plan.relative_to(ROOT)}")

    if failures:
        print("FAIL: alignment addendum coverage gaps")
        for issue in failures:
            print(f" - {issue}")
        return 1 if args.strict else 0

    print("PASS: alignment addendum check")
    return 0


if __name__ == "__main__":
    sys.exit(main())
