#!/usr/bin/env python3
"""Create a DispatchTransportReceiptV1 for a dispatch request."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import UTC, datetime
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "research" / "132-eudaemon-alpha-initiative" / "schemas"
REQUEST_SCHEMA = SCHEMA_DIR / "DispatchRequestV1.schema.json"
RECEIPT_SCHEMA = SCHEMA_DIR / "DispatchTransportReceiptV1.schema.json"


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def extract_request(payload: object) -> dict:
    if not isinstance(payload, dict):
        raise ValueError("receipt input must be a JSON object")
    request = payload.get("dispatchRequest", payload)
    if not isinstance(request, dict):
        raise ValueError("input missing dispatchRequest object")
    jsonschema.validate(request, load_json(REQUEST_SCHEMA))
    return request


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", help="Path to WorkRouter bundle or DispatchRequestV1 JSON")
    parser.add_argument("--status", default="sent", choices=["queued", "sent", "delivered", "read", "replied", "failed"])
    parser.add_argument("--message-ref", required=True)
    parser.add_argument("--decision-id")
    parser.add_argument("--recorded-at", default=utc_now())
    parser.add_argument("--error-summary")
    args = parser.parse_args(argv[1:])

    bundle_path = Path(args.bundle)
    if not bundle_path.is_absolute():
        bundle_path = ROOT / bundle_path
    request = extract_request(load_json(bundle_path))

    receipt = {
        "schemaVersion": "1.0.0",
        "receiptId": f"dispatch-receipt:{request['requestId']}",
        "requestId": request["requestId"],
        "transport": {
            "kind": request["transport"]["kind"],
            "messageRef": args.message_ref,
        },
        "status": args.status,
        "recordedAt": args.recorded_at,
    }
    if args.decision_id:
        receipt["decisionId"] = args.decision_id
    if args.error_summary:
        receipt["errorSummary"] = args.error_summary

    jsonschema.validate(receipt, load_json(RECEIPT_SCHEMA))
    json.dump(receipt, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
