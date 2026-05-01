#!/usr/bin/env python3
"""Create draft route reviews for unknown WorkRouter command text."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import UTC, datetime
from pathlib import Path

import jsonschema

from work_router_paths import work_router_log_root


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "research" / "132-eudaemon-alpha-initiative"
LOG_ROOT = work_router_log_root()
UNKNOWN_DIR = LOG_ROOT / "unknown"
REVIEW_DIR = LOG_ROOT / "unknown_reviews"
SCHEMA_PATH = BASE / "schemas" / "UnknownDispatchRouteReviewV1.schema.json"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def classify(normalized: str) -> tuple[str, dict | None, str]:
    if normalized in {"apprve", "aprove", "approv", "apprve please", "approve please"}:
        return "decision_alias", {"alias": normalized, "canonical": "approve"}, "Looks like an approval typo."
    if normalized in {"whats pending", "what is pending", "pending?"}:
        return "d0_command", {"alias": normalized, "canonical": "pending"}, "Looks like a pending queue query."
    if normalized.startswith("new task") or normalized.startswith("create task"):
        return "task_intake_candidate", None, "Looks like a task intake phrase; keep separate from dispatch approval."
    return "ignore_or_reject", None, "No safe route inferred."


def review_for_unknown(path: Path, created_at: str) -> dict:
    unknown = load_json(path)
    normalized = unknown["normalizedText"]
    classification, mapping, notes = classify(normalized)
    review = {
        "schemaVersion": "1.0.0",
        "reviewId": f"unknown-route-review:{path.stem}",
        "unknownRef": str(path),
        "rawText": unknown["rawText"],
        "normalizedText": normalized,
        "recommendedClassification": classification,
        "status": "draft",
        "reviewer": {
            "kind": "agent",
            "id": "work-router-review-unknowns",
        },
        "createdAt": created_at,
        "notes": notes,
    }
    if mapping:
        review["proposedMapping"] = mapping
    jsonschema.validate(review, load_json(SCHEMA_PATH))
    return review


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--unknown", help="Specific unknown JSON file")
    parser.add_argument("--out-dir", default=str(REVIEW_DIR))
    parser.add_argument("--created-at", default=utc_now())
    args = parser.parse_args(argv[1:])

    paths = [Path(args.unknown)] if args.unknown else sorted(UNKNOWN_DIR.glob("*.json"))
    out_dir = Path(args.out_dir)
    if not out_dir.is_absolute():
        out_dir = ROOT / out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    reviews = []
    for path in paths:
        if not path.is_absolute():
            path = ROOT / path
        review = review_for_unknown(path, args.created_at)
        target = out_dir / f"{Path(path).stem}.review.json"
        target.write_text(json.dumps(review, indent=2) + "\n", encoding="utf-8")
        reviews.append(str(target))

    json.dump({"reviews": reviews, "count": len(reviews)}, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
