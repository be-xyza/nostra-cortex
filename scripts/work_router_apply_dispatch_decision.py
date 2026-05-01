#!/usr/bin/env python3
"""Apply a DispatchDecisionV1 to a dry-run WorkRouter dispatch bundle."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import UTC, datetime
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "research" / "132-eudaemon-alpha-initiative"
SCHEMA_DIR = BASE / "schemas"

SCHEMAS = {
    "workRouterDecision": SCHEMA_DIR / "WorkRouterDecisionV1.schema.json",
    "dispatchRequest": SCHEMA_DIR / "DispatchRequestV1.schema.json",
    "dispatchDecision": SCHEMA_DIR / "DispatchDecisionV1.schema.json",
    "codeChangeRequest": SCHEMA_DIR / "CodeChangeRequestV1.schema.json",
}

LEVEL_ORDER = {"D0": 0, "D1": 1, "D2": 2, "D3": 3, "D4": 4, "D5": 5}


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def validate_record(name: str, record: object) -> None:
    jsonschema.validate(record, load_json(SCHEMAS[name]))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def code_change_from_decision(bundle: dict, decision: dict, created_at: str) -> dict | None:
    router = bundle["workRouterDecision"]
    request = bundle["dispatchRequest"]

    if decision["decision"] != "approve":
        return None

    route = router["recommendedRoute"]
    approved_level = decision["approvedLevel"]
    if route != "patch_prep" or approved_level != "D1":
        return None

    task_ref = request["taskRef"]
    task_slug = task_ref.split(":", 1)[-1]
    return {
        "schemaVersion": "1.0.0",
        "codeChangeRequestId": f"code-change-request:{task_slug}",
        "dispatchDecisionRef": decision["decisionId"],
        "taskRef": task_ref,
        "mode": "patch_prep",
        "authorityLevel": "D1",
        "riskLevel": request["riskLevel"],
        "scope": {
            "allowedPaths": ["research/132-eudaemon-alpha-initiative/"],
            "notes": "Decision-approved D1 patch-prep only; source mutation remains forbidden.",
        },
        "requiredChecks": [
            "bash scripts/check_dynamic_config_contract.sh",
            "bash scripts/check_novel_task_intake.sh",
        ],
        "forbiddenActions": [
            "repo_mutation",
            "runtime_mutation",
            "commit",
            "push",
            "deploy",
            "graph_mutation",
        ],
        "expectedOutputs": ["handoff", "risk_note", "review_prompt"],
        "createdAt": created_at,
    }


def apply_decision(bundle: dict, decision: dict, created_at: str) -> dict:
    router = bundle.get("workRouterDecision")
    request = bundle.get("dispatchRequest")
    require(isinstance(router, dict), "bundle missing workRouterDecision")
    require(isinstance(request, dict), "bundle missing dispatchRequest")
    validate_record("workRouterDecision", router)
    validate_record("dispatchRequest", request)
    validate_record("dispatchDecision", decision)

    require(
        decision["requestId"] == request["requestId"],
        "dispatch decision requestId does not match dispatch request",
    )
    require(
        LEVEL_ORDER[decision["approvedLevel"]] <= LEVEL_ORDER[request["authorityCeiling"]],
        f"dispatch decision approvedLevel {decision['approvedLevel']} exceeds authority ceiling {request['authorityCeiling']}",
    )
    require(
        decision["decision"] in request["allowedDecisions"],
        "dispatch decision is not allowed by dispatch request",
    )

    output = {
        "workRouterDecision": router,
        "dispatchRequest": request,
        "dispatchDecision": decision,
    }
    code_change = code_change_from_decision(bundle, decision, created_at)
    if code_change is not None:
        validate_record("codeChangeRequest", code_change)
        output["codeChangeRequest"] = code_change

    return output


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", help="Path to dry-run WorkRouter dispatch bundle JSON")
    parser.add_argument("decision", help="Path to DispatchDecisionV1 JSON")
    parser.add_argument("--created-at", default=utc_now())
    args = parser.parse_args(argv[1:])

    bundle_path = Path(args.bundle)
    decision_path = Path(args.decision)
    if not bundle_path.is_absolute():
        bundle_path = ROOT / bundle_path
    if not decision_path.is_absolute():
        decision_path = ROOT / decision_path

    bundle = load_json(bundle_path)
    decision = load_json(decision_path)
    require(isinstance(bundle, dict), f"{bundle_path}: bundle must be an object")
    require(isinstance(decision, dict), f"{decision_path}: decision must be an object")

    output = apply_decision(bundle, decision, args.created_at)
    json.dump(output, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
