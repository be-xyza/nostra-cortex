#!/usr/bin/env python3
"""Validate WorkRouterV1 dispatch fixtures for Initiative 132."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "research" / "132-eudaemon-alpha-initiative"
SCHEMA_DIR = BASE / "schemas"
DEFAULT_FIXTURE_DIR = BASE / "examples"

SCHEMAS = {
    "workRouterDecision": "WorkRouterDecisionV1.schema.json",
    "dispatchRequest": "DispatchRequestV1.schema.json",
    "dispatchDecision": "DispatchDecisionV1.schema.json",
    "codeChangeRequest": "CodeChangeRequestV1.schema.json",
    "transportReceipt": "DispatchTransportReceiptV1.schema.json",
}

LEVEL_ORDER = {"D0": 0, "D1": 1, "D2": 2, "D3": 3, "D4": 4, "D5": 5}
HIGH_RISK = {"high", "structural"}
SENSITIVE_ACTIONS = {"commit", "push", "deploy", "runtime_mutation", "canister_call", "graph_mutation"}


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def load_schemas() -> dict[str, object]:
    return {key: load_json(SCHEMA_DIR / filename) for key, filename in SCHEMAS.items()}


def require_level_within_ceiling(level: str, ceiling: str, path: Path, label: str) -> None:
    if LEVEL_ORDER[level] > LEVEL_ORDER[ceiling]:
        raise ValueError(f"{path}: {label} {level} exceeds authority ceiling {ceiling}")


def validate_cross_record(payload: dict, path: Path) -> None:
    router = payload.get("workRouterDecision")
    request = payload.get("dispatchRequest")
    decision = payload.get("dispatchDecision")
    code_change = payload.get("codeChangeRequest")
    receipt = payload.get("transportReceipt")

    if isinstance(router, dict):
        require_level_within_ceiling(
            router["requiredDispatchLevel"],
            router["authorityCeiling"],
            path,
            "router requiredDispatchLevel",
        )

    if isinstance(request, dict):
        require_level_within_ceiling(
            request["requestedLevel"],
            request["authorityCeiling"],
            path,
            "dispatch requestedLevel",
        )

    if isinstance(router, dict) and isinstance(request, dict):
        if router["taskRef"] != request["taskRef"]:
            raise ValueError(f"{path}: router taskRef does not match dispatch request taskRef")
        if router["authorityCeiling"] != request["authorityCeiling"]:
            raise ValueError(f"{path}: router authority ceiling does not match dispatch request")
        if router["requiredDispatchLevel"] != request["requestedLevel"]:
            raise ValueError(f"{path}: router required level does not match dispatch requested level")

    if isinstance(request, dict) and isinstance(decision, dict):
        if decision["requestId"] != request["requestId"]:
            raise ValueError(f"{path}: decision requestId does not match dispatch request")
        require_level_within_ceiling(
            decision["approvedLevel"],
            request["authorityCeiling"],
            path,
            "decision approvedLevel",
        )

    if isinstance(decision, dict) and isinstance(code_change, dict):
        if code_change["dispatchDecisionRef"] != decision["decisionId"]:
            raise ValueError(f"{path}: code change request does not reference dispatch decision")
        if code_change["authorityLevel"] != decision["approvedLevel"]:
            raise ValueError(f"{path}: code change authority level must equal approved level")

    if isinstance(request, dict) and isinstance(receipt, dict):
        if receipt["requestId"] != request["requestId"]:
            raise ValueError(f"{path}: receipt requestId does not match dispatch request")

    if isinstance(decision, dict) and isinstance(receipt, dict) and "decisionId" in receipt:
        if receipt["decisionId"] != decision["decisionId"]:
            raise ValueError(f"{path}: receipt decisionId does not match dispatch decision")

    risk = None
    route = None
    if isinstance(router, dict):
        risk = router["classification"]["riskLevel"]
        route = router["recommendedRoute"]
    elif isinstance(request, dict):
        risk = request["riskLevel"]

    if risk in HIGH_RISK and route == "code_change_candidate":
        raise ValueError(f"{path}: high or structural risk cannot route to code_change_candidate")

    if isinstance(code_change, dict):
        code_level = code_change["authorityLevel"]
        mode = code_change["mode"]
        forbidden = set(code_change.get("forbiddenActions", []))

        if code_level == "D1" and mode != "patch_prep":
            raise ValueError(f"{path}: D1 code changes must be patch_prep only")
        if code_level in {"D2", "D3", "D4", "D5"} and risk in HIGH_RISK:
            raise ValueError(f"{path}: D2+ code change is not allowed for high or structural risk")
        if code_level in {"D3", "D4", "D5"} and not SENSITIVE_ACTIONS.intersection(forbidden):
            raise ValueError(f"{path}: D3+ requests must explicitly forbid sensitive collapse actions")

    if isinstance(request, dict):
        # Transport kind may be telegram, but no authority field may be transport-specific.
        serialized = json.dumps(request, sort_keys=True).lower()
        forbidden_authority_words = ["telegramauthority", "telegram_level", "telegramlevel"]
        if any(word in serialized for word in forbidden_authority_words):
            raise ValueError(f"{path}: transport-specific authority naming is forbidden")


def validate_file(schemas: dict[str, object], path: Path) -> None:
    payload = load_json(path)
    if not isinstance(payload, dict):
        raise ValueError(f"{path}: fixture must be a JSON object")

    for key, schema in schemas.items():
        if key in payload:
            jsonschema.validate(payload[key], schema)

    validate_cross_record(payload, path)


def main(argv: list[str]) -> int:
    schemas = load_schemas()
    paths = [Path(arg) for arg in argv[1:]]
    if not paths:
        paths = sorted(DEFAULT_FIXTURE_DIR.glob("work_router_dispatch_*.json"))

    if not paths:
        print("No WorkRouter dispatch fixtures found.", file=sys.stderr)
        return 1

    for path in paths:
        resolved = path if path.is_absolute() else ROOT / path
        validate_file(schemas, resolved)
        print(f"PASS: {resolved}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
