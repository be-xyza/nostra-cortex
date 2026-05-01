#!/usr/bin/env python3
"""Validate an unknown route review and print the proposed alias mapping."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = (
    ROOT
    / "research"
    / "132-eudaemon-alpha-initiative"
    / "schemas"
    / "UnknownDispatchRouteReviewV1.schema.json"
)


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("Usage: scripts/work_router_apply_unknown_review.py <review.json>", file=sys.stderr)
        return 1
    path = Path(argv[1])
    if not path.is_absolute():
        path = ROOT / path
    review = load_json(path)
    jsonschema.validate(review, load_json(SCHEMA_PATH))

    output = {
        "reviewId": review["reviewId"],
        "status": review["status"],
        "recommendedClassification": review["recommendedClassification"],
        "proposedMapping": review.get("proposedMapping"),
        "note": "Review validated. Alias registry updates remain a separate governed edit.",
    }
    json.dump(output, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
