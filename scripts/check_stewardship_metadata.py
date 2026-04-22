#!/usr/bin/env python3
from __future__ import annotations

import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]


def parse_frontmatter(text: str) -> str:
    if not text.startswith("---\n"):
        return ""
    end = text.find("\n---", 4)
    if end == -1:
        return ""
    return text[4:end]


def find_scalar(frontmatter: str, key: str) -> str:
    m = re.search(rf"(?m)^\s*{re.escape(key)}\s*:\s*(.+?)\s*$", frontmatter)
    return m.group(1).strip().strip('"\'') if m else ""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []
    for plan in sorted((ROOT / "research").glob("[0-9][0-9][0-9]-*/PLAN.md")):
        raw = plan.read_text(encoding="utf-8")
        fm = parse_frontmatter(raw)
        if not fm:
            if args.strict:
                failures.append(f"missing frontmatter: {plan.relative_to(ROOT)}")
            continue

        status = find_scalar(fm, "status")
        role = find_scalar(fm, "portfolio_role")
        needs_strict = status in {"active", "completed"} or role == "anchor"
        if not needs_strict:
            continue

        if "stewardship:" not in fm:
            failures.append(f"missing stewardship block: {plan.relative_to(ROOT)}")
            continue

        for field in ("layer", "primary_steward", "domain"):
            if not re.search(rf"(?m)^\s+{field}\s*:\s*.+$", fm):
                failures.append(f"missing stewardship.{field}: {plan.relative_to(ROOT)}")

    if failures:
        print("FAIL: stewardship metadata violations")
        for issue in failures:
            print(f" - {issue}")
        return 1

    print("PASS: stewardship metadata contract satisfied")
    return 0


if __name__ == "__main__":
    sys.exit(main())
