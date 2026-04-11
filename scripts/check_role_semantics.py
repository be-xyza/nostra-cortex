#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGETS = [
    ROOT / "AGENTS.md",
    ROOT / "nostra" / "spec.md",
    ROOT / "research" / "README.md",
    ROOT / "docs" / "reference" / "README.md",
]
TARGETS.extend(sorted((ROOT / "docs" / "architecture").glob("*.md")))
EXCLUDES = {
    ROOT / "docs" / "architecture" / "semantic-primitives-audit.md",
    ROOT / "docs" / "architecture" / "contribution-graph-naming.md",
}
BANNED_PATTERNS = {
    "mayor": re.compile(r"\bMayor\b"),
    "governor": re.compile(r"\bGovernor\b"),
    "president": re.compile(r"\bPresident\b"),
    "ruler": re.compile(r"\bRuler\b"),
}


def main() -> int:
    failures: list[str] = []
    observed = 0

    for path in TARGETS:
        if not path.exists() or path in EXCLUDES:
            continue
        text = path.read_text(encoding="utf-8")
        observed += 1
        for label, pattern in BANNED_PATTERNS.items():
            if pattern.search(text):
                failures.append(f"{path.relative_to(ROOT)}: banned civic-role term '{label}' present")

    if failures:
        print("FAIL: role semantics checks failed")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print(f"PASS: role semantics checks ({observed} docs scanned)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
