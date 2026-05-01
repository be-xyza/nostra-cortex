#!/usr/bin/env python3
"""Parse and answer D0 WorkRouter chat commands without mutation."""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import UTC, datetime
from pathlib import Path

import jsonschema

from work_router_paths import work_router_log_root


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "research" / "132-eudaemon-alpha-initiative"
LOG_ROOT = work_router_log_root()
RUNS_DIR = LOG_ROOT / "runs"
LATEST = LOG_ROOT / "latest.json"
UNKNOWN_DIR = LOG_ROOT / "unknown"
COMMAND_SCHEMA = BASE / "schemas" / "DispatchCommandV1.schema.json"
RESPONSE_SCHEMA = BASE / "schemas" / "DispatchCommandResponseV1.schema.json"
ALIASES_PATH = BASE / "dispatch_aliases.v1.json"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def slug(text: str) -> str:
    value = re.sub(r"[^A-Za-z0-9._:-]+", "-", text.strip())[:48].strip("-")
    return value or "empty"


def normalize(text: str) -> str:
    return re.sub(r"\s+", " ", text.strip().lower())


def load_aliases() -> tuple[dict[str, str], dict[str, str]]:
    payload = load_json(ALIASES_PATH)
    return payload["decisionAliases"], payload["d0CommandAliases"]


def pending_runs() -> list[dict]:
    runs = []
    for path in sorted(RUNS_DIR.glob("*/run.json")):
        try:
            payload = load_json(path)
        except (OSError, json.JSONDecodeError):
            continue
        if payload.get("status") == "pending_decision":
            runs.append(payload)
    return runs


def safe_run_summary(run: dict) -> str:
    summary = run.get("summary", {})
    authority = run.get("authority", {})
    return (
        f"{run.get('runId')} | {run.get('status')} | {summary.get('route')} | "
        f"{summary.get('riskLevel')} | max {authority.get('maxLevel')} | {summary.get('taskRef')}"
    )


def parse_command(text: str, created_at: str) -> dict:
    raw = text
    value = normalize(text)
    command_id = f"dispatch-command:{created_at}:{slug(value)}"
    decision_aliases, d0_aliases = load_aliases()

    if value in d0_aliases:
        kind = d0_aliases[value]
        args = {}
    elif value.startswith("show "):
        kind = "show"
        args = {"runId": value.split(" ", 1)[1].strip()}
    elif value in decision_aliases:
        kind = "decision_alias"
        args = {"decision": decision_aliases[value]}
    else:
        kind = "unknown"
        args = {}

    command = {
        "schemaVersion": "1.0.0",
        "commandId": command_id,
        "rawText": raw,
        "kind": kind,
        "args": args,
        "createdAt": created_at,
    }
    jsonschema.validate(command, load_json(COMMAND_SCHEMA))
    return command


def record_unknown(command: dict, created_at: str) -> str:
    UNKNOWN_DIR.mkdir(parents=True, exist_ok=True)
    target = UNKNOWN_DIR / f"{created_at.replace(':', '').replace('-', '')}-{slug(command['rawText'])}.json"
    payload = {
        "schemaVersion": "1.0.0",
        "commandId": command["commandId"],
        "rawText": command["rawText"],
        "normalizedText": normalize(command["rawText"]),
        "status": "needs_routing_review",
        "createdAt": created_at,
    }
    target.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return str(target)


def respond(command: dict, created_at: str) -> dict:
    kind = command["kind"]
    body = ""
    status = "ok"
    unknown_ref = None

    if kind == "help":
        body = "Commands: pending, status, latest, show <run_id>. Decisions: approve/reject/revise/escalate/pause only when tied to a pending request."
    elif kind == "pending":
        runs = pending_runs()
        body = "No pending runs." if not runs else "\n".join(safe_run_summary(run) for run in runs)
    elif kind == "status":
        runs = list(RUNS_DIR.glob("*/run.json"))
        pending = pending_runs()
        body = f"WorkRouter local runs: {len(runs)} total, {len(pending)} pending."
    elif kind == "latest":
        if LATEST.exists():
            body = safe_run_summary(load_json(LATEST))
        else:
            body = "No latest WorkRouter run found."
    elif kind == "show":
        run_id = command.get("args", {}).get("runId", "")
        path = RUNS_DIR / run_id / "run.json"
        if path.exists():
            body = safe_run_summary(load_json(path))
        else:
            status = "error"
            body = f"Run not found: {run_id}"
    elif kind == "decision_alias":
        decision = command["args"]["decision"]
        status = "needs_review"
        body = f"Decision alias parsed as `{decision}`. A matching dispatch envelope is required before this can apply."
    else:
        status = "needs_review"
        unknown_ref = record_unknown(command, created_at)
        body = "Unknown command recorded for routing review. Try: help, pending, status, latest, show <run_id>."

    response = {
        "schemaVersion": "1.0.0",
        "commandId": command["commandId"],
        "status": status,
        "body": body,
        "mutationAllowed": False,
        "createdAt": created_at,
    }
    if unknown_ref:
        response["unknownRef"] = unknown_ref
    jsonschema.validate(response, load_json(RESPONSE_SCHEMA))
    return response


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--text", required=True)
    parser.add_argument("--created-at", default=utc_now())
    parser.add_argument("--json", action="store_true", help="Emit full command/response JSON")
    args = parser.parse_args(argv[1:])

    command = parse_command(args.text, args.created_at)
    response = respond(command, args.created_at)
    if args.json:
        json.dump({"command": command, "response": response}, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        sys.stdout.write(response["body"])
        if not response["body"].endswith("\n"):
            sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
