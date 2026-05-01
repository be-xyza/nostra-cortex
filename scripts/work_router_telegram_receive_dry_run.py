#!/usr/bin/env python3
"""Dry-run Telegram update ingestion for WorkRouter dispatch envelopes."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTBOX = ROOT / "logs" / "work_router" / "outbox"
PROCESSED_DIR = ROOT / "logs" / "work_router" / "telegram_processed"


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


def pending_envelopes(outbox: Path, chat_id: str) -> list[Path]:
    matches = []
    for path in sorted(outbox.glob("*.json")):
        try:
            envelope = load_json(path)
        except (OSError, json.JSONDecodeError):
            continue
        transport = envelope.get("transport", {})
        if transport.get("kind") != "telegram":
            continue
        channel = str(transport.get("channelRef", ""))
        if channel in {chat_id, "operator-private"}:
            matches.append(path)
    return matches


def processed_path(update_id: int) -> Path:
    return PROCESSED_DIR / f"{update_id}.json"


def mark_processed(update_id: int, payload: dict) -> None:
    PROCESSED_DIR.mkdir(parents=True, exist_ok=True)
    processed_path(update_id).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def handle_update(update: dict, outbox: Path, created_at: str) -> dict:
    update_id = update["update_id"]
    if processed_path(update_id).exists():
        return {"updateId": update_id, "status": "skipped_duplicate"}

    message = update.get("message") or {}
    text = str(message.get("text", "")).strip()
    chat_id = str((message.get("chat") or {}).get("id", ""))
    sender_id = str((message.get("from") or {}).get("id", chat_id or "unknown"))
    if not text:
        result = {"updateId": update_id, "status": "ignored_empty"}
        mark_processed(update_id, result)
        return result

    command = command_json(text, created_at)
    kind = command["command"]["kind"]
    if kind in {"pending", "status", "latest", "show", "help"}:
        result = {
            "updateId": update_id,
            "status": "d0_response",
            "body": command["response"]["body"],
        }
        mark_processed(update_id, result)
        return result

    envelopes = pending_envelopes(outbox, chat_id)
    if kind == "decision_alias":
        if len(envelopes) != 1:
            result = {
                "updateId": update_id,
                "status": "needs_review",
                "reason": f"expected exactly one pending envelope for chat {chat_id}, found {len(envelopes)}",
            }
            mark_processed(update_id, result)
            return result
        envelope = envelopes[0]
        decision_path = outbox / f"{envelope.stem}.telegram-decision.json"
        parse_reply(envelope, command["command"]["args"]["decision"], sender_id, decision_path, created_at)
        applied_run = apply_run_decision(envelope.stem, decision_path)
        result = {
            "updateId": update_id,
            "status": "decision_applied",
            "decision": command["command"]["args"]["decision"],
            "runId": envelope.stem,
            "appliedRun": applied_run,
        }
        mark_processed(update_id, result)
        return result

    result = {
        "updateId": update_id,
        "status": "unknown_recorded",
        "unknownRef": command["response"].get("unknownRef"),
    }
    mark_processed(update_id, result)
    return result


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("updates", help="Telegram getUpdates JSON fixture")
    parser.add_argument("--outbox", default=str(DEFAULT_OUTBOX))
    parser.add_argument("--created-at", default="2026-04-30T12:48:00Z")
    args = parser.parse_args(argv[1:])

    updates_path = Path(args.updates)
    if not updates_path.is_absolute():
        updates_path = ROOT / updates_path
    outbox = Path(args.outbox)
    if not outbox.is_absolute():
        outbox = ROOT / outbox

    payload = load_json(updates_path)
    results = [handle_update(update, outbox, args.created_at) for update in payload.get("result", [])]
    json.dump({"results": results, "count": len(results)}, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
