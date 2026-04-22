#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(os.environ.get("NOSTRA_WORKSPACE_ROOT", Path(__file__).resolve().parents[1]))
STATUS_FILE = ROOT / "research" / "RESEARCH_INITIATIVES_STATUS.md"
SIQ_DIR = ROOT / "logs" / "siq"
SIQ_RUNS_DIR = SIQ_DIR / "runs"
COVERAGE_PATH = SIQ_DIR / "siq_coverage_latest.json"
DEPENDENCY_PATH = SIQ_DIR / "siq_dependency_closure_latest.json"
SUMMARY_PATH = SIQ_DIR / "siq_gate_summary_latest.json"
GRAPH_PATH = SIQ_DIR / "graph_projection_latest.json"
WAIVERS_PATH = SIQ_DIR / "waivers_latest.json"

REQUIRED_EDGE_TYPES = [
    "contribution_has_rule",
    "rule_has_run",
    "run_emits_violation",
    "violation_backed_by_evidence",
    "contribution_has_waiver",
]

RULES = [
    {
        "id": "siq_governance_execution_contract",
        "severity": "P0",
        "owner": "Systems Steward",
        "source_standard": "research/121-cortex-memory-fs/INTEGRITY_DEPENDENCIES.md",
    },
    {
        "id": "siq_host_parity_contract",
        "severity": "P0",
        "owner": "Systems Steward",
        "source_standard": "research/123-cortex-web-architecture/SPATIAL_PLANE_DESKTOP_PARITY_SPEC.md",
    },
    {
        "id": "siq_graph_projection_contract",
        "severity": "P1",
        "owner": "Research Steward",
        "source_standard": "shared/standards/siq/siq_graph_projection.schema.json",
    },
]

INTEGRITY_SET = ["097", "099", "101", "103", "105", "118", "121", "123"]

CONTRIBUTION_META: dict[str, dict[str, Any]] = {
    "097": {
        "directory": "097-nostra-cortex-alignment",
        "owner": "Systems Steward",
        "rules": [
            "siq_governance_execution_contract",
            "siq_graph_projection_contract",
        ],
        "dependencies_in": [],
        "dependencies_out": ["118", "121", "123"],
        "evidence": [
            {
                "path": "research/097-nostra-cortex-alignment/IMPLEMENTATION_PLAN.md",
                "must_contain": ["Governance Gates", "Contract & Type Alignment"],
            },
            {
                "path": "research/097-nostra-cortex-alignment/DECISIONS.md",
                "must_contain": ["Cross-Initiative"],
            },
        ],
    },
    "099": {
        "directory": "099-upstream-dependency-cleanup",
        "owner": "Systems Steward",
        "rules": [
            "siq_governance_execution_contract",
            "siq_graph_projection_contract",
        ],
        "dependencies_in": [],
        "dependencies_out": ["121"],
        "evidence": [
            {
                "path": "research/099-upstream-dependency-cleanup/DECISIONS.md",
                "must_contain": ["dependency"],
            }
        ],
    },
    "101": {
        "directory": "101-upstream-warning-remediation",
        "owner": "Systems Steward",
        "rules": [
            "siq_governance_execution_contract",
            "siq_graph_projection_contract",
        ],
        "dependencies_in": [],
        "dependencies_out": ["121"],
        "evidence": [
            {
                "path": "research/101-upstream-warning-remediation/DECISIONS.md",
                "must_contain": ["warning"],
            }
        ],
    },
    "103": {
        "directory": "103-agent-client-protocol-alignment",
        "owner": "Systems Steward",
        "rules": [
            "siq_governance_execution_contract",
            "siq_graph_projection_contract",
        ],
        "dependencies_in": [],
        "dependencies_out": ["121"],
        "evidence": [
            {
                "path": "research/103-agent-client-protocol-alignment/PLAN.md",
                "must_contain": ["protocol"],
            },
            {
                "path": "research/103-agent-client-protocol-alignment/DECISIONS.md",
                "must_contain": ["pilot"],
            },
        ],
    },
    "105": {
        "directory": "105-cortex-test-catalog",
        "owner": "Systems Steward",
        "rules": [
            "siq_governance_execution_contract",
            "siq_graph_projection_contract",
        ],
        "dependencies_in": [],
        "dependencies_out": ["121"],
        "evidence": [
            {
                "path": "research/105-cortex-test-catalog/DECISIONS.md",
                "must_contain": ["schema"],
            },
            {
                "path": "research/105-cortex-test-catalog/VERIFY.md",
                "must_contain": ["overall_verdict"],
            },
        ],
    },
    "118": {
        "directory": "118-cortex-runtime-extraction",
        "owner": "Systems Steward",
        "rules": [
            "siq_governance_execution_contract",
            "siq_host_parity_contract",
            "siq_graph_projection_contract",
        ],
        "dependencies_in": ["097"],
        "dependencies_out": ["121", "123"],
        "evidence": [
            {
                "path": "research/118-cortex-runtime-extraction/PLAN.md",
                "must_contain": ["runtime"],
            },
            {
                "path": "research/118-cortex-runtime-extraction/DECISIONS.md",
                "must_contain": ["governance", "host plurality"],
            },
            {
                "path": "research/118-cortex-runtime-extraction/PHASE_5_OPERATIONAL_CLOSURE_EVIDENCE_2026-02-16.md",
                "must_contain": ["operationally-closed", "governance"],
            },
        ],
    },
    "121": {
        "directory": "121-cortex-memory-fs",
        "owner": "Systems Steward",
        "rules": [
            "siq_governance_execution_contract",
            "siq_host_parity_contract",
            "siq_graph_projection_contract",
        ],
        "dependencies_in": ["097", "099", "101", "103", "105", "118", "123"],
        "dependencies_out": [],
        "evidence": [
            {
                "path": "research/121-cortex-memory-fs/PLAN.md",
                "must_contain": ["Memory FS", "118", "103"],
            },
            {
                "path": "research/121-cortex-memory-fs/REQUIREMENTS.md",
                "must_contain": ["cortex.branch", "cortex.merge"],
            },
            {
                "path": "research/121-cortex-memory-fs/INTEGRITY_DEPENDENCIES.md",
                "must_contain": ["blocked", "siq_host_parity_contract", "siq_governance_execution_contract"],
            },
        ],
    },
    "123": {
        "directory": "123-cortex-web-architecture",
        "owner": "Systems Steward",
        "rules": [
            "siq_governance_execution_contract",
            "siq_host_parity_contract",
            "siq_graph_projection_contract",
        ],
        "dependencies_in": ["118"],
        "dependencies_out": ["121"],
        "evidence": [
            {
                "path": "research/123-cortex-web-architecture/PLAN.md",
                "must_contain": ["host", "parity"],
            },
            {
                "path": "research/123-cortex-web-architecture/DECISIONS.md",
                "must_contain": ["Dual-Host Runtime Contract", "cortex:a2ui:event"],
            },
            {
                "path": "research/123-cortex-web-architecture/VERIFY.md",
                "must_contain": ["check_cortex_dual_host_parity.sh", "PASS"],
            },
            {
                "path": "research/123-cortex-web-architecture/SPATIAL_PLANE_DESKTOP_PARITY_SPEC.md",
                "must_contain": ["cortex:a2ui:event", "run_id"],
            },
        ],
    },
}


@dataclass
class RuleOutcome:
    status: str
    failures: list[dict[str, Any]]
    notes: list[str]


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def parse_status_index() -> dict[str, str]:
    if not STATUS_FILE.exists():
        return {}
    status_map: dict[str, str] = {}
    row_re = re.compile(r"^\|\s*([0-9]{3})\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|")
    for line in STATUS_FILE.read_text(encoding="utf-8").splitlines():
        match = row_re.match(line)
        if not match:
            continue
        contribution_id = match.group(1).strip()
        status = match.group(3).strip().lower()
        status_map[contribution_id] = status
    return status_map


def git_short_commit() -> str:
    try:
        out = subprocess.check_output(
            ["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"],
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
        return out or "unknown"
    except Exception:
        return "unknown"


def normalize_path(path: str) -> str:
    return path.replace("\\", "/")


def file_contains(path: Path, needles: list[str]) -> tuple[bool, str]:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        return False, str(exc)
    lower_text = text.lower()
    for needle in needles:
        if needle.lower() not in lower_text:
            return False, f"missing token '{needle}'"
    return True, ""


def build_coverage(generated_at: str) -> dict[str, Any]:
    statuses = parse_status_index()
    contributions: list[dict[str, Any]] = []
    for contribution_id in INTEGRITY_SET:
        meta = CONTRIBUTION_META[contribution_id]
        closure_state = "closed" if statuses.get(contribution_id) in {"completed", "archived"} else "open"
        evidence = []
        for item in meta["evidence"]:
            evidence.append(
                {
                    "path": normalize_path(item["path"]),
                    "must_contain": item["must_contain"],
                    "exists": (ROOT / item["path"]).exists(),
                }
            )
        contributions.append(
            {
                "contribution_id": contribution_id,
                "directory": meta["directory"],
                "status": statuses.get(contribution_id, "unknown"),
                "owner": meta["owner"],
                "rules": meta["rules"],
                "dependency_in": sorted(meta["dependencies_in"]),
                "dependency_out": sorted(meta["dependencies_out"]),
                "closure_state": closure_state,
                "evidence": evidence,
            }
        )

    contributions.sort(key=lambda entry: entry["contribution_id"])
    return {
        "schema_version": "1.0.0",
        "generated_at": generated_at,
        "integrity_set": INTEGRITY_SET,
        "contributions": contributions,
    }


def build_dependency_closure(coverage: dict[str, Any], generated_at: str) -> dict[str, Any]:
    known_ids = {entry["contribution_id"] for entry in coverage["contributions"]}
    rows: list[dict[str, Any]] = []
    for entry in coverage["contributions"]:
        missing = [dep for dep in entry["dependency_in"] if dep not in known_ids]
        rows.append(
            {
                "contribution_id": entry["contribution_id"],
                "required_dependencies": entry["dependency_in"],
                "satisfied_dependencies": [dep for dep in entry["dependency_in"] if dep in known_ids],
                "missing_dependencies": missing,
                "closure_state": "blocked" if missing else "ready",
            }
        )
    overall = "blocked" if any(row["missing_dependencies"] for row in rows) else "ready"
    rows.sort(key=lambda row: row["contribution_id"])
    return {
        "schema_version": "1.0.0",
        "generated_at": generated_at,
        "integrity_set": INTEGRITY_SET,
        "overall_closure_state": overall,
        "rows": rows,
    }


def check_governance_execution_contract(coverage: dict[str, Any]) -> RuleOutcome:
    failures: list[dict[str, Any]] = []
    notes: list[str] = []
    coverage_ids = {entry["contribution_id"] for entry in coverage["contributions"]}

    for required in INTEGRITY_SET:
        if required not in coverage_ids:
            failures.append(
                {
                    "class": "missing_gate_evidence",
                    "message": f"contribution {required} missing from SIQ coverage artifact",
                    "contribution_id": required,
                    "evidence_path": None,
                }
            )

    for entry in coverage["contributions"]:
        if "siq_governance_execution_contract" not in entry["rules"]:
            continue
        if not entry["evidence"]:
            failures.append(
                {
                    "class": "missing_gate_evidence",
                    "message": "contribution has no governance evidence paths",
                    "contribution_id": entry["contribution_id"],
                    "evidence_path": None,
                }
            )
            continue

        for evidence in entry["evidence"]:
            full_path = ROOT / evidence["path"]
            if not full_path.exists():
                failures.append(
                    {
                        "class": "missing_gate_evidence",
                        "message": "required governance evidence file is missing",
                        "contribution_id": entry["contribution_id"],
                        "evidence_path": evidence["path"],
                    }
                )
                continue

            ok, reason = file_contains(full_path, evidence["must_contain"])
            if not ok:
                failures.append(
                    {
                        "class": "governance_contract_mismatch",
                        "message": f"evidence content mismatch: {reason}",
                        "contribution_id": entry["contribution_id"],
                        "evidence_path": evidence["path"],
                    }
                )

    for item in load_waivers_raw():
        owner = str(item.get("owner", "")).strip()
        expires_at = str(item.get("expires_at", "")).strip()
        contribution_id = str(item.get("contribution_id", "")).strip() or None
        if not owner or not expires_at:
            failures.append(
                {
                    "class": "governance_contract_mismatch",
                    "message": "waiver is missing required owner or expires_at",
                    "contribution_id": contribution_id,
                    "evidence_path": "logs/siq/waivers_latest.json",
                }
            )

    if not failures:
        notes.append("governance evidence coverage and contract tokens are present")
    return RuleOutcome("pass" if not failures else "fail", failures, notes)


def check_host_parity_contract(coverage: dict[str, Any], dependency: dict[str, Any]) -> RuleOutcome:
    failures: list[dict[str, Any]] = []
    notes: list[str] = []

    rows = {row["contribution_id"]: row for row in dependency.get("rows", [])}
    dep_121 = rows.get("121")
    if dep_121 is None:
        failures.append(
            {
                "class": "parity_contract_drift",
                "message": "dependency closure row for contribution 121 is missing",
                "contribution_id": "121",
                "evidence_path": None,
            }
        )
    elif dep_121["closure_state"] != "ready":
        failures.append(
            {
                "class": "parity_contract_drift",
                "message": "contribution 121 dependency closure is not ready",
                "contribution_id": "121",
                "evidence_path": None,
            }
        )

    required_tokens = [
        (
            "research/123-cortex-web-architecture/VERIFY.md",
            ["check_cortex_dual_host_parity.sh", "PASS"],
            "123",
        ),
        (
            "research/123-cortex-web-architecture/SPATIAL_PLANE_DESKTOP_PARITY_SPEC.md",
            ["cortex:a2ui:event", "run_id"],
            "123",
        ),
        (
            "research/118-cortex-runtime-extraction/DECISIONS.md",
            ["host plurality", "governance"],
            "118",
        ),
    ]

    for rel_path, needles, contribution_id in required_tokens:
        full_path = ROOT / rel_path
        if not full_path.exists():
            failures.append(
                {
                    "class": "missing_gate_evidence",
                    "message": "required host parity evidence file is missing",
                    "contribution_id": contribution_id,
                    "evidence_path": rel_path,
                }
            )
            continue
        ok, reason = file_contains(full_path, needles)
        if not ok:
            failures.append(
                {
                    "class": "parity_contract_drift",
                    "message": f"host parity evidence mismatch: {reason}",
                    "contribution_id": contribution_id,
                    "evidence_path": rel_path,
                }
            )

    if not failures:
        notes.append("host parity evidence linkage and event lock contract are aligned")
    return RuleOutcome("pass" if not failures else "fail", failures, notes)


def load_waivers_raw() -> list[dict[str, Any]]:
    if not WAIVERS_PATH.exists():
        return []
    try:
        data = json.loads(WAIVERS_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return []
    if not isinstance(data, list):
        return []
    return [item for item in data if isinstance(item, dict)]


def load_waivers() -> list[dict[str, Any]]:
    waivers: list[dict[str, Any]] = []
    for item in load_waivers_raw():
        contribution_id = str(item.get("contribution_id", "")).strip()
        owner = str(item.get("owner", "")).strip()
        expires_at = str(item.get("expires_at", "")).strip()
        if not owner or not expires_at:
            continue

        waiver_id = str(item.get("waiver_id", "")).strip()
        if not waiver_id:
            waiver_id = f"waiver:{contribution_id}:{owner}" if contribution_id else f"waiver:{owner}"
        waivers.append(
            {
                "waiver_id": waiver_id,
                "contribution_id": contribution_id,
                "owner": owner,
                "expires_at": expires_at,
            }
        )
    waivers.sort(key=lambda item: item["waiver_id"])
    return waivers


def build_graph_projection(
    coverage: dict[str, Any],
    summary_failures: list[dict[str, Any]],
    run_id: str,
    generated_at: str,
    waivers: list[dict[str, Any]],
) -> dict[str, Any]:
    contributions = []
    rules = []
    gate_runs = []
    violations = []
    evidence_nodes = []
    waiver_nodes = []
    edges: list[dict[str, str]] = []

    rule_set: set[str] = set()
    evidence_set: set[str] = set()

    for entry in coverage["contributions"]:
        contribution_node_id = f"contribution:{entry['contribution_id']}"
        contributions.append(
            {
                "id": contribution_node_id,
                "kind": "Contribution",
                "contribution_id": entry["contribution_id"],
                "status": entry["status"],
                "directory": entry["directory"],
                "closure_state": entry["closure_state"],
            }
        )
        for rule_id in entry["rules"]:
            rule_set.add(rule_id)
            rule_node_id = f"rule:{rule_id}"
            edges.append(
                {
                    "type": "contribution_has_rule",
                    "from": contribution_node_id,
                    "to": rule_node_id,
                }
            )
        for evidence in entry["evidence"]:
            evidence_id = f"evidence:{evidence['path']}"
            evidence_set.add(evidence_id)
            evidence_nodes.append(
                {
                    "id": evidence_id,
                    "kind": "Evidence",
                    "path": evidence["path"],
                    "exists": evidence["exists"],
                }
            )

    gate_run_node = {
        "id": f"run:{run_id}",
        "kind": "GateRun",
        "run_id": run_id,
        "generated_at": generated_at,
    }
    gate_runs.append(gate_run_node)

    for rule_id in sorted(rule_set):
        rules.append(
            {
                "id": f"rule:{rule_id}",
                "kind": "Rule",
                "rule_id": rule_id,
            }
        )
        edges.append(
            {
                "type": "rule_has_run",
                "from": f"rule:{rule_id}",
                "to": gate_run_node["id"],
            }
        )

    for index, failure in enumerate(summary_failures):
        failure_id = f"violation:{index:03d}:{failure['rule_id']}:{failure['class']}"
        violations.append(
            {
                "id": failure_id,
                "kind": "Violation",
                "rule_id": failure["rule_id"],
                "class": failure["class"],
                "contribution_id": failure.get("contribution_id"),
                "message": failure["message"],
            }
        )
        edges.append(
            {
                "type": "run_emits_violation",
                "from": gate_run_node["id"],
                "to": failure_id,
            }
        )
        evidence_path = failure.get("evidence_path")
        if evidence_path:
            evidence_id = f"evidence:{evidence_path}"
            if evidence_id not in evidence_set:
                evidence_set.add(evidence_id)
                evidence_nodes.append(
                    {
                        "id": evidence_id,
                        "kind": "Evidence",
                        "path": evidence_path,
                        "exists": (ROOT / evidence_path).exists(),
                    }
                )
            edges.append(
                {
                    "type": "violation_backed_by_evidence",
                    "from": failure_id,
                    "to": evidence_id,
                }
            )

    for waiver in waivers:
        waiver_node_id = f"waiver:{waiver['waiver_id']}"
        waiver_nodes.append(
            {
                "id": waiver_node_id,
                "kind": "Waiver",
                "waiver_id": waiver["waiver_id"],
                "contribution_id": waiver["contribution_id"],
                "owner": waiver["owner"],
                "expires_at": waiver["expires_at"],
            }
        )
        if waiver["contribution_id"]:
            edges.append(
                {
                    "type": "contribution_has_waiver",
                    "from": f"contribution:{waiver['contribution_id']}",
                    "to": waiver_node_id,
                }
            )

    def edge_id(edge: dict[str, str]) -> str:
        return f"{edge['type']}:{edge['from']}->{edge['to']}"

    dedup_edges: dict[str, dict[str, str]] = {}
    for edge in edges:
        dedup_edges[edge_id(edge)] = edge

    normalized_edges = []
    for key in sorted(dedup_edges):
        edge = dedup_edges[key]
        normalized_edges.append(
            {
                "edge_id": key,
                "type": edge["type"],
                "from": edge["from"],
                "to": edge["to"],
            }
        )

    entities = {
        "contributions": sorted(contributions, key=lambda item: item["id"]),
        "rules": sorted(rules, key=lambda item: item["id"]),
        "gate_runs": sorted(gate_runs, key=lambda item: item["id"]),
        "violations": sorted(violations, key=lambda item: item["id"]),
        "evidence": sorted(evidence_nodes, key=lambda item: item["id"]),
        "waivers": sorted(waiver_nodes, key=lambda item: item["id"]),
    }

    fingerprint_source = {
        "edge_types": REQUIRED_EDGE_TYPES,
        "entities": entities,
        "edges": normalized_edges,
    }
    canonical = json.dumps(fingerprint_source, sort_keys=True, separators=(",", ":"))
    fingerprint = hashlib.sha256(canonical.encode("utf-8")).hexdigest()

    return {
        "schema_version": "1.0.0",
        "generated_at": generated_at,
        "run_id": run_id,
        "graph_fingerprint": fingerprint,
        "integrity_set": INTEGRITY_SET,
        "edge_types": REQUIRED_EDGE_TYPES,
        "entities": entities,
        "edges": normalized_edges,
    }


def check_graph_projection_contract(
    projection: dict[str, Any],
    coverage: dict[str, Any],
    summary_failures: list[dict[str, Any]],
    run_id: str,
) -> RuleOutcome:
    failures: list[dict[str, Any]] = []
    notes: list[str] = []

    required_keys = {
        "schema_version",
        "generated_at",
        "run_id",
        "graph_fingerprint",
        "integrity_set",
        "edge_types",
        "entities",
        "edges",
    }
    missing = sorted(required_keys - set(projection.keys()))
    if missing:
        failures.append(
            {
                "class": "governance_contract_mismatch",
                "message": f"graph projection is missing required keys: {', '.join(missing)}",
                "contribution_id": None,
                "evidence_path": "logs/siq/graph_projection_latest.json",
            }
        )

    edge_types = projection.get("edge_types", [])
    if sorted(edge_types) != sorted(REQUIRED_EDGE_TYPES):
        failures.append(
            {
                "class": "governance_contract_mismatch",
                "message": "graph projection edge_types do not match SIQ contract",
                "contribution_id": None,
                "evidence_path": "logs/siq/graph_projection_latest.json",
            }
        )

    all_node_ids: set[str] = set()
    for bucket in ("contributions", "rules", "gate_runs", "violations", "evidence", "waivers"):
        for node in projection.get("entities", {}).get(bucket, []):
            node_id = node.get("id")
            if isinstance(node_id, str):
                all_node_ids.add(node_id)

    violation_evidence_links: set[str] = set()
    for edge in projection.get("edges", []):
        edge_from = edge.get("from")
        edge_to = edge.get("to")
        edge_type = edge.get("type")
        if edge_from not in all_node_ids or edge_to not in all_node_ids:
            failures.append(
                {
                    "class": "governance_contract_mismatch",
                    "message": "edge references unknown node",
                    "contribution_id": None,
                    "evidence_path": "logs/siq/graph_projection_latest.json",
                }
            )
            continue
        if edge_type == "violation_backed_by_evidence":
            violation_evidence_links.add(edge_from)

    for node in projection.get("entities", {}).get("violations", []):
        node_id = node.get("id")
        if isinstance(node_id, str) and node_id not in violation_evidence_links:
            failures.append(
                {
                    "class": "governance_contract_mismatch",
                    "message": "violation node missing evidence linkage",
                    "contribution_id": node.get("contribution_id"),
                    "evidence_path": "logs/siq/graph_projection_latest.json",
                }
            )

    deterministic_a = build_graph_projection(
        coverage=coverage,
        summary_failures=summary_failures,
        run_id=run_id,
        generated_at="2000-01-01T00:00:00Z",
        waivers=[],
    )
    deterministic_b = build_graph_projection(
        coverage=coverage,
        summary_failures=summary_failures,
        run_id=run_id,
        generated_at="2000-01-01T00:00:00Z",
        waivers=[],
    )
    if deterministic_a["graph_fingerprint"] != deterministic_b["graph_fingerprint"]:
        failures.append(
            {
                "class": "parity_contract_drift",
                "message": "graph projection deterministic fingerprint mismatch",
                "contribution_id": None,
                "evidence_path": "logs/siq/graph_projection_latest.json",
            }
        )

    if not failures:
        notes.append("graph projection contract and deterministic fingerprint checks passed")
    return RuleOutcome("pass" if not failures else "fail", failures, notes)


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def summarize_failures(results: list[dict[str, Any]]) -> list[dict[str, Any]]:
    failures: list[dict[str, Any]] = []
    for result in results:
        if result["status"] != "fail":
            continue
        for failure in result["failures"]:
            failures.append(
                {
                    "rule_id": result["id"],
                    "severity": result["severity"],
                    "class": failure["class"],
                    "message": failure["message"],
                    "contribution_id": failure.get("contribution_id"),
                    "evidence_path": failure.get("evidence_path"),
                }
            )
    return failures


def run_refresh(mode: str) -> int:
    generated_at = utc_now()
    run_id = f"siq_{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')}"

    coverage = build_coverage(generated_at)
    dependency = build_dependency_closure(coverage, generated_at)

    governance_outcome = check_governance_execution_contract(coverage)
    host_parity_outcome = check_host_parity_contract(coverage, dependency)

    preliminary_results = [
        {
            **RULES[0],
            "status": governance_outcome.status,
            "failures": governance_outcome.failures,
            "notes": governance_outcome.notes,
        },
        {
            **RULES[1],
            "status": host_parity_outcome.status,
            "failures": host_parity_outcome.failures,
            "notes": host_parity_outcome.notes,
        },
    ]

    preliminary_failures = summarize_failures(preliminary_results)
    projection = build_graph_projection(
        coverage=coverage,
        summary_failures=preliminary_failures,
        run_id=run_id,
        generated_at=generated_at,
        waivers=load_waivers(),
    )
    graph_outcome = check_graph_projection_contract(
        projection=projection,
        coverage=coverage,
        summary_failures=preliminary_failures,
        run_id=run_id,
    )

    results = preliminary_results + [
        {
            **RULES[2],
            "status": graph_outcome.status,
            "failures": graph_outcome.failures,
            "notes": graph_outcome.notes,
        }
    ]

    failures = summarize_failures(results)
    counts = {
        "pass": sum(1 for result in results if result["status"] == "pass"),
        "fail": sum(1 for result in results if result["status"] == "fail"),
    }

    p0_failed = any(item["severity"] == "P0" for item in failures)
    p1_failed = any(item["severity"] == "P1" for item in failures)

    if mode == "observe":
        required_gates_pass = True
    elif mode == "softgate":
        required_gates_pass = not p0_failed
    else:
        required_gates_pass = not (p0_failed or p1_failed)

    overall_verdict = "ready" if required_gates_pass else "not-ready"

    run_payload = {
        "schema_version": "1.0.0",
        "run_id": run_id,
        "generated_at": generated_at,
        "mode": mode,
        "policy_path": "shared/standards/alignment_contracts.toml",
        "policy_version": 1,
        "overall_verdict": overall_verdict,
        "required_gates_pass": required_gates_pass,
        "counts": counts,
        "failures": failures,
        "results": results,
        "git_commit": git_short_commit(),
    }

    summary_payload = {
        "schema_version": "1.0.0",
        "generated_at": generated_at,
        "mode": mode,
        "latest_run_id": run_id,
        "overall_verdict": overall_verdict,
        "required_gates_pass": required_gates_pass,
        "counts": counts,
        "failures": failures,
    }

    write_json(COVERAGE_PATH, coverage)
    write_json(DEPENDENCY_PATH, dependency)
    write_json(GRAPH_PATH, projection)
    write_json(SIQ_RUNS_DIR / f"{run_id}.json", run_payload)
    write_json(SUMMARY_PATH, summary_payload)

    print(f"SIQ refresh complete (mode={mode})")
    print(f"run_id={run_id}")
    print(f"overall_verdict={overall_verdict}")
    print(f"counts: pass={counts['pass']} fail={counts['fail']}")
    if failures:
        for failure in failures:
            print(
                f" - {failure['rule_id']} [{failure['class']}]: {failure['message']}"
                + (f" ({failure['evidence_path']})" if failure.get("evidence_path") else "")
            )

    return 0 if required_gates_pass else 1


def run_single_check(check_id: str) -> int:
    generated_at = utc_now()
    coverage = build_coverage(generated_at)
    dependency = build_dependency_closure(coverage, generated_at)

    if check_id == "governance":
        outcome = check_governance_execution_contract(coverage)
        rule = RULES[0]
    elif check_id == "host-parity":
        outcome = check_host_parity_contract(coverage, dependency)
        rule = RULES[1]
    elif check_id == "graph-projection":
        run_id = "siq_single_check"
        preliminary_failures = []
        projection = build_graph_projection(
            coverage=coverage,
            summary_failures=preliminary_failures,
            run_id=run_id,
            generated_at=generated_at,
            waivers=load_waivers(),
        )
        outcome = check_graph_projection_contract(
            projection=projection,
            coverage=coverage,
            summary_failures=preliminary_failures,
            run_id=run_id,
        )
        rule = RULES[2]
    else:
        raise ValueError(f"unknown check id: {check_id}")

    if outcome.status == "pass":
        print(f"PASS: {rule['id']}")
        for note in outcome.notes:
            print(f" - {note}")
        return 0

    print(f"FAIL: {rule['id']}")
    for failure in outcome.failures:
        suffix = f" ({failure['evidence_path']})" if failure.get("evidence_path") else ""
        print(f" - [{failure['class']}] {failure['message']}{suffix}")
    return 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="SIQ tooling")
    subparsers = parser.add_subparsers(dest="command", required=True)

    refresh = subparsers.add_parser("refresh", help="Generate SIQ artifacts and evaluate gates")
    refresh.add_argument("--mode", choices=["observe", "softgate", "hardgate"], default="observe")

    subparsers.add_parser("check-governance", help="Run governance execution contract check")
    subparsers.add_parser("check-host-parity", help="Run host parity contract check")
    subparsers.add_parser("check-graph-projection", help="Run graph projection contract check")

    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.command == "refresh":
        return run_refresh(args.mode)
    if args.command == "check-governance":
        return run_single_check("governance")
    if args.command == "check-host-parity":
        return run_single_check("host-parity")
    if args.command == "check-graph-projection":
        return run_single_check("graph-projection")
    print("unsupported command", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
