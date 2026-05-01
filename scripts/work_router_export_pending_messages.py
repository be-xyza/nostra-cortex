#!/usr/bin/env python3
"""Export pending WorkRouter dispatch messages into a local transport outbox."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import UTC, datetime
from pathlib import Path

import jsonschema

from work_router_paths import work_router_log_root


ROOT = Path(__file__).resolve().parents[1]
LOG_ROOT = work_router_log_root()
RUNS_DIR = LOG_ROOT / "runs"
DEFAULT_OUTBOX = LOG_ROOT / "outbox"
SCHEMA_PATH = (
    ROOT
    / "research"
    / "132-eudaemon-alpha-initiative"
    / "schemas"
    / "DispatchTransportEnvelopeV1.schema.json"
)


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def pending_runs() -> list[tuple[Path, dict]]:
    runs = []
    for path in sorted(RUNS_DIR.glob("*/run.json")):
        try:
            payload = load_json(path)
        except (OSError, json.JSONDecodeError):
            continue
        if payload.get("status") == "pending_decision":
            runs.append((path, payload))
    return runs


def build_envelope(run_path: Path, run: dict, created_at: str) -> dict:
    router_bundle_path = Path(run["artifactRefs"]["routerBundle"])
    router_bundle = load_json(router_bundle_path)
    request = router_bundle["dispatchRequest"]
    message_path = Path(run["artifactRefs"]["message"])
    message = message_path.read_text(encoding="utf-8")
    run_id = run["runId"]
    return {
        "schemaVersion": "1.0.0",
        "envelopeId": f"dispatch-envelope:{run_id}",
        "runId": run_id,
        "requestId": request["requestId"],
        "transport": {
            "kind": request["transport"]["kind"],
            "channelRef": request["transport"]["channelRef"],
        },
        "message": {
            "format": "text",
            "body": message,
        },
        "artifactRefs": {
            "run": str(run_path),
            "dispatchRequest": str(router_bundle_path),
            "message": str(message_path),
            "receipt": run["artifactRefs"]["receipt"],
        },
        "createdAt": created_at,
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--outbox", default=str(DEFAULT_OUTBOX), help="Output directory for transport envelopes")
    parser.add_argument("--run-id", help="Export only one pending run id")
    parser.add_argument("--created-at", default=utc_now())
    args = parser.parse_args(argv[1:])

    outbox = Path(args.outbox)
    if not outbox.is_absolute():
        outbox = ROOT / outbox
    outbox.mkdir(parents=True, exist_ok=True)
    schema = load_json(SCHEMA_PATH)

    exported = []
    for run_path, run in pending_runs():
        if args.run_id and run.get("runId") != args.run_id:
            continue
        envelope = build_envelope(run_path, run, args.created_at)
        jsonschema.validate(envelope, schema)
        target = outbox / f"{run['runId']}.json"
        target.write_text(json.dumps(envelope, indent=2) + "\n", encoding="utf-8")
        exported.append(str(target))

    json.dump({"exported": exported, "count": len(exported)}, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
