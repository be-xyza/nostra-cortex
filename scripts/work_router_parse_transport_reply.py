#!/usr/bin/env python3
"""Parse a transport reply into DispatchDecisionV1 for a WorkRouter envelope."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import UTC, datetime
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "research" / "132-eudaemon-alpha-initiative"
ENVELOPE_SCHEMA = BASE / "schemas" / "DispatchTransportEnvelopeV1.schema.json"
DECISION_SCHEMA = BASE / "schemas" / "DispatchDecisionV1.schema.json"
ALIASES_PATH = BASE / "dispatch_aliases.v1.json"

DECISIONS = {"approve", "reject", "revise", "escalate", "pause"}


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def normalize_reply(reply: str) -> str:
    normalized = reply.strip().lower()
    aliases = load_json(ALIASES_PATH)["decisionAliases"]
    return aliases.get(normalized, normalized)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("envelope", help="Path to DispatchTransportEnvelopeV1 JSON")
    parser.add_argument("--reply", required=True, help="Transport reply text, e.g. approve/reject/revise/escalate/pause")
    parser.add_argument("--decider-kind", default="operator", choices=["human", "steward", "operator"])
    parser.add_argument("--decider-id", default="user")
    parser.add_argument("--decided-at", default=utc_now())
    parser.add_argument("--rationale", default="Decision parsed from transport reply.")
    parser.add_argument("--output", help="Optional output DispatchDecisionV1 path")
    args = parser.parse_args(argv[1:])

    envelope_path = Path(args.envelope)
    if not envelope_path.is_absolute():
        envelope_path = ROOT / envelope_path
    envelope = load_json(envelope_path)
    jsonschema.validate(envelope, load_json(ENVELOPE_SCHEMA))

    decision_value = normalize_reply(args.reply)
    if decision_value not in DECISIONS:
        raise ValueError(
            f"unsupported dispatch reply '{args.reply}'. Expected one of: {', '.join(sorted(DECISIONS))}"
        )

    approved_level = "D1" if decision_value == "approve" else "D0"
    decision = {
        "schemaVersion": "1.0.0",
        "decisionId": f"dispatch-decision:{envelope['runId']}",
        "requestId": envelope["requestId"],
        "decision": decision_value,
        "approvedLevel": approved_level,
        "decider": {
            "kind": args.decider_kind,
            "id": args.decider_id,
        },
        "rationale": args.rationale,
        "conditions": ["transport_reply", "requires_matching_request"],
        "decidedAt": args.decided_at,
    }
    jsonschema.validate(decision, load_json(DECISION_SCHEMA))

    rendered = json.dumps(decision, indent=2) + "\n"
    if args.output:
        output_path = Path(args.output)
        if not output_path.is_absolute():
            output_path = ROOT / output_path
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
