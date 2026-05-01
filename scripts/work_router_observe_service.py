#!/usr/bin/env python3
"""Observe-only WorkRouter service loop for D0-D1 bootstrap."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import UTC, datetime
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTBOX = ROOT / "logs" / "work_router" / "outbox"
DEFAULT_HEARTBEAT = ROOT / "logs" / "work_router" / "service" / "heartbeat.json"
DEFAULT_EVIDENCE = ROOT / "logs" / "work_router" / "agent_run_evidence" / "workrouter-observe-loop-latest.json"
AGENT_EVIDENCE_SCHEMA = (
    ROOT
    / "research"
    / "132-eudaemon-alpha-initiative"
    / "schemas"
    / "AgentRunEvidenceV1.schema.json"
)


def utc_now() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require_env(name: str, expected: str) -> None:
    actual = os.environ.get(name)
    if actual != expected:
        raise RuntimeError(f"{name} must be {expected!r}, got {actual!r}")


def validate_env() -> None:
    require_env("WORK_ROUTER_MAX_DISPATCH_LEVEL", "D1")
    require_env("WORK_ROUTER_SOURCE_MUTATION_ALLOWED", "0")
    require_env("WORK_ROUTER_RUNTIME_MUTATION_ALLOWED", "0")
    require_env("WORK_ROUTER_REQUIRE_REQUEST_ID", "1")
    require_env("WORK_ROUTER_UNKNOWN_REPLIES_REVIEW_ONLY", "1")
    require_env("WORK_ROUTER_TRANSPORTS_ENABLED", "cli")
    require_env("WORK_ROUTER_LIVE_TRANSPORT_ENABLED", "0")
    mode = os.environ.get("WORK_ROUTER_MODE", "observe")
    if mode != "observe":
        raise RuntimeError(f"WORK_ROUTER_MODE must be 'observe', got {mode!r}")


def run_json(command: list[str]) -> dict:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    return json.loads(completed.stdout)


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def observe_once(outbox: Path, heartbeat: Path, evidence: Path, observed_at: str) -> dict:
    validate_env()
    pending = run_json(["python3", str(ROOT / "scripts" / "work_router_list_pending.py")])
    exported = run_json(
        [
            "python3",
            str(ROOT / "scripts" / "work_router_export_pending_messages.py"),
            "--outbox",
            str(outbox),
            "--created-at",
            observed_at,
        ]
    )
    heartbeat_payload = {
        "schemaVersion": "1.0.0",
        "service": "cortex-workrouter",
        "mode": "observe",
        "maxDispatchLevel": "D1",
        "mutationAllowed": False,
        "liveTransportEnabled": False,
        "pendingCount": pending["count"],
        "exportedCount": exported["count"],
        "outbox": str(outbox),
        "observedAt": observed_at,
    }
    write_json(heartbeat, heartbeat_payload)

    evidence_payload = {
        "schemaVersion": "1.0.0",
        "evidenceId": f"agent-run:workrouter:observe-loop:{observed_at.replace(':', '').replace('-', '')}",
        "agentId": "workrouter",
        "harnessId": "native-workrouter",
        "profileRef": "agent-profile-registry:init132:v1#workrouter",
        "runKind": "observe",
        "authority": {
            "maxDispatchLevel": "D1",
            "sourceMutationAllowed": False,
            "runtimeMutationAllowed": False,
        },
        "status": "recorded",
        "startedAt": observed_at,
        "finishedAt": observed_at,
        "inputRefs": ["logs/work_router/runs"],
        "outputRefs": [str(heartbeat), str(outbox)],
        "forbiddenActionsConfirmed": [
            "source_mutation",
            "runtime_mutation",
            "commit",
            "push",
            "pull_request",
            "deploy",
            "canister_call",
            "graph_mutation",
            "provider_topology_exposure",
            "authority_escalation",
        ],
        "summary": "Observe-only WorkRouter service loop exported pending dispatch envelopes and wrote heartbeat evidence without source or runtime mutation.",
    }
    jsonschema.validate(evidence_payload, load_json(AGENT_EVIDENCE_SCHEMA))
    write_json(evidence, evidence_payload)

    return {
        "status": "observed",
        "pending": pending,
        "exported": exported,
        "heartbeat": str(heartbeat),
        "evidence": str(evidence),
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--once", action="store_true", help="Run one observe cycle and exit")
    parser.add_argument("--interval-seconds", type=int, default=300)
    parser.add_argument("--outbox", default=str(DEFAULT_OUTBOX))
    parser.add_argument("--heartbeat", default=str(DEFAULT_HEARTBEAT))
    parser.add_argument("--evidence", default=str(DEFAULT_EVIDENCE))
    parser.add_argument("--observed-at", default=utc_now())
    args = parser.parse_args(argv[1:])

    outbox = Path(args.outbox)
    heartbeat = Path(args.heartbeat)
    evidence = Path(args.evidence)
    if not outbox.is_absolute():
        outbox = ROOT / outbox
    if not heartbeat.is_absolute():
        heartbeat = ROOT / heartbeat
    if not evidence.is_absolute():
        evidence = ROOT / evidence

    if args.once:
        json.dump(observe_once(outbox, heartbeat, evidence, args.observed_at), sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0

    while True:
        result = observe_once(outbox, heartbeat, evidence, utc_now())
        print(json.dumps(result, sort_keys=True), flush=True)
        time.sleep(args.interval_seconds)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
