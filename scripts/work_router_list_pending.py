#!/usr/bin/env python3
"""List local WorkRouter runs waiting for a dispatch decision."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNS_DIR = ROOT / "logs" / "work_router" / "runs"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    pending = []
    for path in sorted(RUNS_DIR.glob("*/run.json")):
        try:
            payload = load_json(path)
        except (OSError, json.JSONDecodeError):
            continue
        if payload.get("status") == "pending_decision":
            pending.append(
                {
                    "runId": payload.get("runId"),
                    "taskRef": payload.get("summary", {}).get("taskRef"),
                    "route": payload.get("summary", {}).get("route"),
                    "riskLevel": payload.get("summary", {}).get("riskLevel"),
                    "maxLevel": payload.get("authority", {}).get("maxLevel"),
                    "message": payload.get("artifactRefs", {}).get("message"),
                    "run": str(path),
                }
            )
    json.dump({"pending": pending, "count": len(pending)}, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
