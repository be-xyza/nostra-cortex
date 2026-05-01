#!/usr/bin/env python3
"""Guarded Telegram DispatchTransportAdapterV1 for WorkRouter outbox envelopes."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import urllib.parse
import urllib.request
from datetime import UTC, datetime
from pathlib import Path

import jsonschema

from work_router_paths import work_router_log_root


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "research" / "132-eudaemon-alpha-initiative"
ENVELOPE_SCHEMA = BASE / "schemas" / "DispatchTransportEnvelopeV1.schema.json"
DECISION_SCHEMA = BASE / "schemas" / "DispatchDecisionV1.schema.json"
ADAPTER_SCHEMA = BASE / "schemas" / "DispatchTransportAdapterV1.schema.json"
ADAPTER_FIXTURE = BASE / "examples" / "dispatch_transport_adapter_telegram_guarded.v1.json"
DEFAULT_OUTBOX = work_router_log_root() / "outbox"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def validate_adapter() -> None:
    jsonschema.validate(load_json(ADAPTER_FIXTURE), load_json(ADAPTER_SCHEMA))


def load_envelope(path: Path) -> dict:
    envelope = load_json(path)
    jsonschema.validate(envelope, load_json(ENVELOPE_SCHEMA))
    if envelope["transport"]["kind"] != "telegram":
        raise ValueError(f"{path}: Telegram adapter cannot send transport kind {envelope['transport']['kind']}")
    return envelope


def envelope_path(outbox: Path, run_id: str) -> Path:
    return outbox / f"{run_id}.json"


def send_telegram_message(token: str, chat_id: str, body: str, timeout: int) -> dict:
    url = f"https://api.telegram.org/bot{token}/sendMessage"
    data = urllib.parse.urlencode(
        {
            "chat_id": chat_id,
            "text": body,
            "disable_web_page_preview": "true",
        }
    ).encode("utf-8")
    request = urllib.request.Request(url, data=data, method="POST")
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.loads(response.read().decode("utf-8"))


def validate_decision(decision_path: Path, envelope: dict) -> dict:
    decision = load_json(decision_path)
    jsonschema.validate(decision, load_json(DECISION_SCHEMA))
    if decision["requestId"] != envelope["requestId"]:
        raise ValueError(
            f"{decision_path}: decision requestId {decision['requestId']} does not match envelope requestId {envelope['requestId']}"
        )
    return decision


def apply_decision(run_id: str, decision_path: Path) -> str:
    command = [
        "bash",
        str(ROOT / "scripts" / "work_router_apply_run_decision.sh"),
        "--run-id",
        run_id,
        "--decision",
        str(decision_path),
    ]
    completed = subprocess.run(command, cwd=ROOT, check=True, text=True, capture_output=True)
    return completed.stdout.strip()


def write_adapter_receipt(outbox: Path, envelope: dict, *, status: str, message_ref: str | None, dry_run: bool) -> Path:
    receipt = {
        "schemaVersion": "1.0.0",
        "receiptId": f"telegram-adapter-receipt:{envelope['runId']}",
        "envelopeId": envelope["envelopeId"],
        "runId": envelope["runId"],
        "requestId": envelope["requestId"],
        "transport": envelope["transport"],
        "status": status,
        "messageRef": message_ref or f"telegram:dry-run:{envelope['runId']}",
        "dryRun": dry_run,
        "recordedAt": utc_now(),
    }
    target = outbox / f"{envelope['runId']}.telegram-receipt.json"
    target.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    return target


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--outbox", default=str(DEFAULT_OUTBOX))
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--dry-run", action="store_true", help="Do not call Telegram API; default safe mode unless --live is set")
    parser.add_argument("--live", action="store_true", help="Send via Telegram API using environment credentials")
    parser.add_argument("--decision", help="Optional DispatchDecisionV1 file to apply after sending")
    parser.add_argument("--timeout", type=int, default=15)
    args = parser.parse_args(argv[1:])

    validate_adapter()
    if args.live and args.dry_run:
        raise ValueError("choose either --dry-run or --live, not both")
    live = args.live

    outbox = Path(args.outbox)
    if not outbox.is_absolute():
        outbox = ROOT / outbox
    path = envelope_path(outbox, args.run_id)
    if not path.exists():
        raise FileNotFoundError(path)
    envelope = load_envelope(path)

    message_ref = None
    if live:
        token = os.environ.get("WORK_ROUTER_TELEGRAM_BOT_TOKEN")
        chat_id = os.environ.get("WORK_ROUTER_TELEGRAM_CHAT_ID") or envelope["transport"]["channelRef"]
        if not token:
            raise ValueError("WORK_ROUTER_TELEGRAM_BOT_TOKEN is required for --live")
        response = send_telegram_message(token, chat_id, envelope["message"]["body"], args.timeout)
        if not response.get("ok"):
            raise RuntimeError(f"Telegram send failed: {response}")
        message = response.get("result", {})
        message_ref = f"telegram:{chat_id}:{message.get('message_id', 'unknown')}"
    else:
        sys.stdout.write(envelope["message"]["body"])
        if not envelope["message"]["body"].endswith("\n"):
            sys.stdout.write("\n")

    receipt_path = write_adapter_receipt(outbox, envelope, status="sent", message_ref=message_ref, dry_run=not live)
    applied_run = None
    if args.decision:
        decision_path = Path(args.decision)
        if not decision_path.is_absolute():
            decision_path = ROOT / decision_path
        validate_decision(decision_path, envelope)
        applied_run = apply_decision(envelope["runId"], decision_path)

    json.dump(
        {
            "sent": True,
            "dryRun": not live,
            "runId": envelope["runId"],
            "envelope": str(path),
            "adapterReceipt": str(receipt_path),
            "appliedRun": applied_run,
        },
        sys.stdout,
        indent=2,
    )
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
