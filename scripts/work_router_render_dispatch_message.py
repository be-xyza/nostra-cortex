#!/usr/bin/env python3
"""Render a human-facing dispatch message from a WorkRouter bundle."""

from __future__ import annotations

import argparse
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
    / "DispatchRequestV1.schema.json"
)


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def extract_request(payload: object) -> dict:
    if not isinstance(payload, dict):
        raise ValueError("dispatch message input must be a JSON object")
    request = payload.get("dispatchRequest", payload)
    if not isinstance(request, dict):
        raise ValueError("input missing dispatchRequest object")
    jsonschema.validate(request, load_json(SCHEMA_PATH))
    return request


def render_text(request: dict) -> str:
    prompt = request["prompt"]
    lines = [
        f"[{request['requestedLevel']}] {prompt['title']}",
        "",
        prompt["summary"],
        "",
        f"Task: {request['taskRef']}",
        f"Risk: {request['riskLevel']}",
        f"Authority ceiling: {request['authorityCeiling']}",
        f"Transport: {request['transport']['kind']}:{request['transport']['channelRef']}",
        "",
        "Reply with one decision:",
        ", ".join(request["allowedDecisions"]),
        "",
        "Forbidden in this request:",
        ", ".join(request["forbiddenActions"]),
    ]
    if "detailsRef" in prompt:
        lines.insert(4, f"Details: {prompt['detailsRef']}")
    return "\n".join(lines)


def render_json(request: dict) -> str:
    message = {
        "requestId": request["requestId"],
        "transport": request["transport"],
        "title": request["prompt"]["title"],
        "summary": request["prompt"]["summary"],
        "taskRef": request["taskRef"],
        "requestedLevel": request["requestedLevel"],
        "authorityCeiling": request["authorityCeiling"],
        "riskLevel": request["riskLevel"],
        "allowedDecisions": request["allowedDecisions"],
        "forbiddenActions": request["forbiddenActions"],
    }
    return json.dumps(message, indent=2)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", help="Path to WorkRouter bundle or DispatchRequestV1 JSON")
    parser.add_argument("--format", choices=["text", "json"], default="text")
    args = parser.parse_args(argv[1:])

    bundle_path = Path(args.bundle)
    if not bundle_path.is_absolute():
        bundle_path = ROOT / bundle_path
    request = extract_request(load_json(bundle_path))
    rendered = render_json(request) if args.format == "json" else render_text(request)
    sys.stdout.write(rendered)
    if not rendered.endswith("\n"):
        sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
