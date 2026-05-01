#!/usr/bin/env python3
"""Create a transport-neutral WorkRouter dispatch bundle from NovelTaskIntakeV1."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import UTC, datetime
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "research" / "132-eudaemon-alpha-initiative"
NOVEL_SCHEMA = BASE / "schemas" / "NovelTaskIntakeV1.schema.json"
DISPATCH_SCHEMAS = {
    "workRouterDecision": BASE / "schemas" / "WorkRouterDecisionV1.schema.json",
    "dispatchRequest": BASE / "schemas" / "DispatchRequestV1.schema.json",
}

LEVEL_BY_AUTHORITY = {
    "l0": "D0",
    "l1": "D1",
    "l2": "D2",
    "l3": "D3",
    "l4": "D4",
}

LEVEL_ORDER = {"D0": 0, "D1": 1, "D2": 2, "D3": 3, "D4": 4, "D5": 5}
HIGH_RISK = {"high", "structural"}


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def dispatch_level_for_intake(intake: dict) -> str:
    primitive = intake["recommendedPrimitive"]
    risk = intake["riskLevel"]
    ceiling = LEVEL_BY_AUTHORITY[intake["authorityCeiling"]]

    if primitive == "stop":
        requested = "D0"
    elif primitive in {"hermes_advisory", "heap", "closeout", "workflow", "chronicle"}:
        requested = "D1"
    elif primitive in {"proposal", "steward_review"}:
        requested = "D1"
    else:
        requested = "D0"

    if risk in HIGH_RISK:
        requested = "D1"

    if LEVEL_ORDER[requested] > LEVEL_ORDER[ceiling]:
        return ceiling
    return requested


def route_for_intake(intake: dict) -> tuple[str, str]:
    primitive = intake["recommendedPrimitive"]
    risk = intake["riskLevel"]
    hermes_profile = intake["hermesProfile"]

    if primitive == "stop":
        return "blocked", "blocked"
    if risk in HIGH_RISK:
        return "steward_gate", "governance"
    if primitive == "hermes_advisory":
        return "hermes_advisory", "advisory"
    if hermes_profile == "hermescortexdev":
        return "patch_prep", "patch_prep"
    if primitive in {"heap", "closeout", "chronicle"}:
        return "status_digest", "status"
    if primitive == "workflow":
        return "patch_prep", "workflow"
    if primitive in {"proposal", "steward_review"}:
        return "steward_gate", "governance"
    return "blocked", "blocked"


def forbidden_actions_for_level(level: str) -> list[str]:
    forbidden = ["runtime_mutation", "commit", "push", "deploy", "graph_mutation"]
    if level in {"D0", "D1"}:
        forbidden.insert(0, "repo_mutation")
    return forbidden


def required_checks(intake: dict) -> list[str]:
    checks = ["bash scripts/check_dynamic_config_contract.sh"]
    if intake["hermesProfile"] in {"hermes132", "hermescortexdev"}:
        checks.append("bash scripts/check_novel_task_intake.sh")
    return checks


def build_bundle(intake: dict, *, transport: str, channel_ref: str, created_at: str) -> dict:
    task_id = intake["taskId"]
    requested_level = dispatch_level_for_intake(intake)
    authority_ceiling = LEVEL_BY_AUTHORITY[intake["authorityCeiling"]]
    route, task_shape = route_for_intake(intake)

    if requested_level in {"D2", "D3", "D4", "D5"}:
        # This dry-run slice intentionally proves D0-D1 only.
        requested_level = "D1"
        authority_ceiling = min(authority_ceiling, "D1", key=lambda level: LEVEL_ORDER[level])

    router_id = f"router-decision:{task_id}"
    request_id = f"dispatch-request:{task_id}"

    router = {
        "schemaVersion": "1.0.0",
        "routerDecisionId": router_id,
        "taskRef": f"novel-task:{task_id}",
        "classification": {
            "taskShape": task_shape,
            "riskLevel": intake["riskLevel"],
            "uncertainty": "medium" if intake.get("uncertainties") else "low",
        },
        "recommendedRoute": route,
        "authorityCeiling": authority_ceiling,
        "requiredDispatchLevel": requested_level,
        "rationale": (
            "Dry-run route generated from NovelTaskIntakeV1; dispatch is transport-neutral "
            "and no repo or runtime mutation is authorized."
        ),
        "nextRequestRef": request_id,
        "createdAt": created_at,
    }

    request = {
        "schemaVersion": "1.0.0",
        "requestId": request_id,
        "taskRef": f"novel-task:{task_id}",
        "transport": {
            "kind": transport,
            "channelRef": channel_ref,
            "adapterRef": f"dispatch_transport_{transport}",
        },
        "requestedLevel": requested_level,
        "authorityCeiling": authority_ceiling,
        "riskLevel": intake["riskLevel"],
        "prompt": {
            "title": f"Approve {route.replace('_', ' ')}",
            "summary": intake["desiredOutcome"],
            "detailsRef": f"novel-task:{task_id}",
        },
        "allowedDecisions": ["approve", "reject", "revise", "escalate", "pause"],
        "forbiddenActions": forbidden_actions_for_level(requested_level),
        "createdAt": created_at,
        "idempotencyKey": request_id,
    }

    bundle = {
        "workRouterDecision": router,
        "dispatchRequest": request,
    }

    return bundle


def validate_bundle(bundle: dict) -> None:
    for key, path in DISPATCH_SCHEMAS.items():
        if key in bundle:
            jsonschema.validate(bundle[key], load_json(path))


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("intake", help="Path to NovelTaskIntakeV1 JSON")
    parser.add_argument("--transport", default="telegram", choices=["hermes_gateway", "telegram", "cortex_web", "cli", "matrix", "email", "webhook", "other"])
    parser.add_argument("--channel-ref", default="operator-private")
    parser.add_argument("--created-at", default=utc_now())
    args = parser.parse_args(argv[1:])

    intake_path = Path(args.intake)
    if not intake_path.is_absolute():
        intake_path = ROOT / intake_path

    intake = load_json(intake_path)
    if not isinstance(intake, dict):
        raise ValueError(f"{intake_path}: intake must be an object")
    jsonschema.validate(intake, load_json(NOVEL_SCHEMA))

    bundle = build_bundle(
        intake,
        transport=args.transport,
        channel_ref=args.channel_ref,
        created_at=args.created_at,
    )
    validate_bundle(bundle)
    json.dump(bundle, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
