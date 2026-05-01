#!/usr/bin/env python3
"""Dry-run Hermes Gateway message ingestion for WorkRouter dispatch envelopes."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTBOX = ROOT / "logs" / "work_router" / "outbox"
PROCESSED_DIR = ROOT / "logs" / "work_router" / "hermes_gateway_processed"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def command_json(text: str, created_at: str) -> dict:
    completed = subprocess.run(
        [
            "python3",
            str(ROOT / "scripts" / "work_router_command.py"),
            "--text",
            text,
            "--created-at",
            created_at,
            "--json",
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    return json.loads(completed.stdout)


def parse_reply(envelope: Path, text: str, decider_id: str, output: Path, created_at: str) -> None:
    subprocess.run(
        [
            "python3",
            str(ROOT / "scripts" / "work_router_parse_transport_reply.py"),
            str(envelope),
            "--reply",
            text,
            "--decider-kind",
            "operator",
            "--decider-id",
            decider_id,
            "--decided-at",
            created_at,
            "--output",
            str(output),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )


def apply_run_decision(run_id: str, decision: Path) -> str:
    completed = subprocess.run(
        [
            "bash",
            str(ROOT / "scripts" / "work_router_apply_run_decision.sh"),
            "--run-id",
            run_id,
            "--decision",
            str(decision),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    return completed.stdout.strip()


def pending_envelopes(outbox: Path, request_id: str, conversation_id: str) -> list[Path]:
    matches = []
    for path in sorted(outbox.glob("*.json")):
        try:
            envelope = load_json(path)
        except (OSError, json.JSONDecodeError):
            continue
        transport = envelope.get("transport", {})
        if transport.get("kind") != "hermes_gateway":
            continue
        if envelope.get("requestId") != request_id:
            continue
        channel = str(transport.get("channelRef", ""))
        if channel == conversation_id:
            matches.append(path)
    return matches


def processed_path(message_id: str) -> Path:
    return PROCESSED_DIR / f"{message_id}.json"


def mark_processed(message_id: str, payload: dict) -> None:
    PROCESSED_DIR.mkdir(parents=True, exist_ok=True)
    processed_path(message_id).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def handle_message(message: dict, outbox: Path, created_at: str) -> dict:
    message_id = message["messageId"]
    if processed_path(message_id).exists():
        return {"messageId": message_id, "status": "skipped_duplicate"}

    if message.get("gateway") != "hermes_gateway":
        result = {"messageId": message_id, "status": "rejected", "reason": "gateway must be hermes_gateway"}
        mark_processed(message_id, result)
        return result

    text = str(message.get("text", "")).strip()
    request_id = str(message.get("requestId", "")).strip()
    conversation_id = str(message.get("conversationId", "")).strip()
    sender_id = str((message.get("sender") or {}).get("id", "unknown"))

    if not text:
        result = {"messageId": message_id, "status": "ignored_empty"}
        mark_processed(message_id, result)
        return result
    if not request_id:
        result = {"messageId": message_id, "status": "needs_review", "reason": "missing requestId"}
        mark_processed(message_id, result)
        return result

    command = command_json(text, created_at)
    kind = command["command"]["kind"]
    if kind in {"pending", "status", "latest", "show", "help"}:
        result = {
            "messageId": message_id,
            "status": "d0_response",
            "body": command["response"]["body"],
        }
        mark_processed(message_id, result)
        return result

    envelopes = pending_envelopes(outbox, request_id, conversation_id)
    if kind == "decision_alias":
        if len(envelopes) != 1:
            result = {
                "messageId": message_id,
                "status": "needs_review",
                "reason": f"expected exactly one pending Hermes Gateway envelope for request {request_id}, found {len(envelopes)}",
            }
            mark_processed(message_id, result)
            return result
        envelope = envelopes[0]
        decision_path = outbox / f"{envelope.stem}.hermes-gateway-decision.json"
        parse_reply(envelope, command["command"]["args"]["decision"], sender_id, decision_path, created_at)
        applied_run = apply_run_decision(envelope.stem, decision_path)
        result = {
            "messageId": message_id,
            "status": "decision_applied",
            "decision": command["command"]["args"]["decision"],
            "requestId": request_id,
            "runId": envelope.stem,
            "appliedRun": applied_run,
        }
        mark_processed(message_id, result)
        return result

    result = {
        "messageId": message_id,
        "status": "unknown_recorded",
        "requestId": request_id,
        "unknownRef": command["response"].get("unknownRef"),
    }
    mark_processed(message_id, result)
    return result


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("message", help="Hermes Gateway message fixture")
    parser.add_argument("--outbox", default=str(DEFAULT_OUTBOX))
    parser.add_argument("--created-at", default="2026-05-01T01:00:00Z")
    args = parser.parse_args(argv[1:])

    message_path = Path(args.message)
    if not message_path.is_absolute():
        message_path = ROOT / message_path
    outbox = Path(args.outbox)
    if not outbox.is_absolute():
        outbox = ROOT / outbox

    result = handle_message(load_json(message_path), outbox, args.created_at)
    json.dump(result, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
