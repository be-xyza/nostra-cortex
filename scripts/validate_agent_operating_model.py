#!/usr/bin/env python3
"""Validate Initiative 132 agent harness/profile/promotion contracts."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import jsonschema


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "research" / "132-eudaemon-alpha-initiative"
SCHEMA_DIR = BASE / "schemas"
EXAMPLE_DIR = BASE / "examples"

LEVEL_ORDER = {"D0": 0, "D1": 1, "D2": 2, "D3": 3, "D4": 4, "D5": 5}
MUTATION_FORBIDDEN_ACTIONS = {
    "source_mutation",
    "runtime_mutation",
    "commit",
    "push",
    "pull_request",
    "deploy",
    "canister_call",
    "graph_mutation",
    "authority_escalation",
}


def load_json(path: Path) -> object:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def schema(name: str) -> object:
    return load_json(SCHEMA_DIR / name)


def validate_harness_registry(path: Path) -> dict:
    payload = load_json(path)
    jsonschema.validate(payload, schema("AgentHarnessRegistryV1.schema.json"))
    seen: set[str] = set()
    for harness in payload["harnesses"]:
        harness_id = harness["harnessId"]
        if harness_id in seen:
            raise ValueError(f"{path}: duplicate harnessId {harness_id}")
        seen.add(harness_id)
        if harness["status"] == "active" and not harness["evaluationEvidenceRefs"]:
            raise ValueError(f"{path}: active harness {harness_id} requires evaluationEvidenceRefs")
    return payload


def validate_profiles(path: Path, harness_ids: set[str]) -> dict:
    payload = load_json(path)
    jsonschema.validate(payload, schema("AgentProfileV1.schema.json"))
    seen: set[str] = set()
    for profile in payload["profiles"]:
        agent_id = profile["agentId"]
        if agent_id in seen:
            raise ValueError(f"{path}: duplicate agentId {agent_id}")
        seen.add(agent_id)
        if profile["harnessId"] not in harness_ids:
            raise ValueError(f"{path}: profile {agent_id} references unknown harnessId {profile['harnessId']}")

        authority = profile["authority"]
        max_level = authority["maxDispatchLevel"]
        forbidden = set(profile["forbiddenActions"])

        if LEVEL_ORDER[max_level] <= LEVEL_ORDER["D1"]:
            if authority["sourceMutationAllowed"] or authority["runtimeMutationAllowed"]:
                raise ValueError(f"{path}: D0-D1 profile {agent_id} cannot allow source or runtime mutation")
            missing = MUTATION_FORBIDDEN_ACTIONS.difference(forbidden)
            if missing:
                raise ValueError(
                    f"{path}: D0-D1 profile {agent_id} must forbid mutation collapse actions: {sorted(missing)}"
                )

        if "provider_topology_exposure" not in forbidden:
            raise ValueError(f"{path}: profile {agent_id} must forbid provider_topology_exposure")

        boundary = profile["runtimeBoundary"]
        boundary_status = boundary["status"]
        if profile["lifecycleState"] in {"active", "pilot"} and boundary_status not in {"checked", "blocked"}:
            raise ValueError(f"{path}: active/pilot profile {agent_id} requires checked or blocked runtimeBoundary")
        if boundary_status == "checked" and "boundaryRef" not in boundary:
            raise ValueError(f"{path}: checked profile {agent_id} requires runtimeBoundary.boundaryRef")

    return payload


def validate_promotion_gates(path: Path, profile_ids: set[str]) -> dict:
    payload = load_json(path)
    jsonschema.validate(payload, schema("AgentPromotionGateV1.schema.json"))
    seen: set[str] = set()
    for gate in payload["gates"]:
        gate_id = gate["gateId"]
        if gate_id in seen:
            raise ValueError(f"{path}: duplicate gateId {gate_id}")
        seen.add(gate_id)
        if gate["agentId"] not in profile_ids:
            raise ValueError(f"{path}: gate {gate_id} references unknown agentId {gate['agentId']}")
        if gate["targetState"] == "active" and not gate["evidenceRefs"]:
            raise ValueError(f"{path}: active promotion gate {gate_id} requires evidenceRefs")
    return payload


def validate_run_evidence(path: Path, profiles: dict[str, dict]) -> dict:
    payload = load_json(path)
    if not isinstance(payload, dict):
        raise ValueError(f"{path}: run evidence fixture must be a JSON object")
    if payload.get("schemaVersion") != "1.0.0":
        raise ValueError(f"{path}: run evidence registry must use schemaVersion 1.0.0")
    evidence_items = payload.get("evidence")
    if not isinstance(evidence_items, list) or not evidence_items:
        raise ValueError(f"{path}: run evidence registry must include evidence entries")

    evidence_schema = schema("AgentRunEvidenceV1.schema.json")
    seen: set[str] = set()
    for evidence in evidence_items:
        jsonschema.validate(evidence, evidence_schema)
        evidence_id = evidence["evidenceId"]
        if evidence_id in seen:
            raise ValueError(f"{path}: duplicate evidenceId {evidence_id}")
        seen.add(evidence_id)

        agent_id = evidence["agentId"]
        if agent_id not in profiles:
            raise ValueError(f"{path}: evidence {evidence_id} references unknown agentId {agent_id}")
        profile = profiles[agent_id]
        if evidence["harnessId"] != profile["harnessId"]:
            raise ValueError(f"{path}: evidence {evidence_id} harnessId does not match profile")

        evidence_authority = evidence["authority"]
        profile_authority = profile["authority"]
        if LEVEL_ORDER[evidence_authority["maxDispatchLevel"]] > LEVEL_ORDER[profile_authority["maxDispatchLevel"]]:
            raise ValueError(f"{path}: evidence {evidence_id} exceeds profile authority ceiling")
        if evidence_authority["sourceMutationAllowed"] or evidence_authority["runtimeMutationAllowed"]:
            raise ValueError(f"{path}: evidence {evidence_id} cannot allow mutation in v1")

        confirmed = set(evidence["forbiddenActionsConfirmed"])
        missing = MUTATION_FORBIDDEN_ACTIONS.difference(confirmed)
        if missing:
            raise ValueError(
                f"{path}: evidence {evidence_id} must confirm mutation collapse actions: {sorted(missing)}"
            )

    return payload


def validate_activation_roadmap(path: Path, harness_ids: set[str]) -> dict:
    payload = load_json(path)
    jsonschema.validate(payload, schema("AgentActivationRoadmapV1.schema.json"))
    seen: set[str] = set()
    for entry in payload["entries"]:
        agent_id = entry["candidateAgentId"]
        if agent_id in seen:
            raise ValueError(f"{path}: duplicate candidateAgentId {agent_id}")
        seen.add(agent_id)
        if entry["proposedHarnessId"] not in harness_ids:
            raise ValueError(f"{path}: roadmap entry {agent_id} references unknown harness")

        forbidden = set(entry["forbiddenBeforeActivation"])
        missing = MUTATION_FORBIDDEN_ACTIONS.difference(forbidden)
        if missing:
            raise ValueError(
                f"{path}: roadmap entry {agent_id} must forbid mutation collapse before activation: {sorted(missing)}"
            )

        level = entry["authorityCeiling"]
        stage = entry["activationStage"]
        if LEVEL_ORDER[level] >= LEVEL_ORDER["D2"] and stage != "future_governed_lane":
            raise ValueError(f"{path}: roadmap entry {agent_id} has D2+ authority outside future_governed_lane")
        if LEVEL_ORDER[level] >= LEVEL_ORDER["D2"] and entry["targetLifecycleState"] != "proposed":
            raise ValueError(f"{path}: roadmap entry {agent_id} D2+ lane must remain proposed")
    return payload


def main(argv: list[str]) -> int:
    harness_path = Path(argv[1]) if len(argv) > 1 else EXAMPLE_DIR / "agent_harness_registry.v1.json"
    profiles_path = Path(argv[2]) if len(argv) > 2 else EXAMPLE_DIR / "agent_profiles.v1.json"
    gates_path = Path(argv[3]) if len(argv) > 3 else EXAMPLE_DIR / "agent_promotion_gates.v1.json"
    evidence_path = Path(argv[4]) if len(argv) > 4 else EXAMPLE_DIR / "agent_run_evidence.v1.json"
    roadmap_path = Path(argv[5]) if len(argv) > 5 else EXAMPLE_DIR / "agent_activation_roadmap.v1.json"
    harness_path = harness_path if harness_path.is_absolute() else ROOT / harness_path
    profiles_path = profiles_path if profiles_path.is_absolute() else ROOT / profiles_path
    gates_path = gates_path if gates_path.is_absolute() else ROOT / gates_path
    evidence_path = evidence_path if evidence_path.is_absolute() else ROOT / evidence_path
    roadmap_path = roadmap_path if roadmap_path.is_absolute() else ROOT / roadmap_path

    harnesses = validate_harness_registry(harness_path)
    harness_ids = {harness["harnessId"] for harness in harnesses["harnesses"]}
    profiles = validate_profiles(
        profiles_path,
        harness_ids,
    )
    validate_promotion_gates(gates_path, {profile["agentId"] for profile in profiles["profiles"]})
    validate_run_evidence(evidence_path, {profile["agentId"]: profile for profile in profiles["profiles"]})
    validate_activation_roadmap(roadmap_path, harness_ids)
    print("PASS: agent operating model contracts")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
