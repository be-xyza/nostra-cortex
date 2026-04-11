#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

README_PATH = ROOT / "docs" / "reference" / "README.md"
AGENTS_PATH = ROOT / "AGENTS.md"

README_REQUIRED = [
    "Canonical root: `research/reference/`",
    "Default authority mode is `recommendation_only`.",
    "Sensitive actions (`rename`, `merge`, `archive`, `delete`, root moves, scope changes) require steward escalation",
    "Decide placement (`research/reference/repos`, `research/reference/topics/<topic>`, or `research/reference/inbox`).",
    "`links_to_nostra_cortex` (canonical key; do not use `possible_links_to_nostra_cortex`)",
    "`initiative_refs` (list of research initiative IDs)",
]

AGENTS_REQUIRED = [
    "## Reference Intake Protocol (Agents)",
    "Command contract: `reference intake`",
    "Command contract: `knowledge intake`",
    "`links_to_nostra_cortex`",
    "`initiative_refs`",
]


def check_strings(path: Path, required: list[str], failures: list[str]) -> None:
    if not path.exists():
        failures.append(f"missing required file {path.relative_to(ROOT)}")
        return
    text = path.read_text(encoding="utf-8")
    for item in required:
        if item not in text:
            failures.append(f"{path.relative_to(ROOT)} missing required text: {item}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--strict", action="store_true")
    _ = parser.parse_args()

    failures: list[str] = []
    check_strings(README_PATH, README_REQUIRED, failures)
    check_strings(AGENTS_PATH, AGENTS_REQUIRED, failures)

    if failures:
        print("FAIL: reference constitutional contract checks failed")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print("PASS: reference constitutional contract checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
