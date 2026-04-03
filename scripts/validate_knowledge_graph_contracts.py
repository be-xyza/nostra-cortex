#!/usr/bin/env python3
from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any

try:
    import tomllib
except Exception as exc:  # pragma: no cover
    raise SystemExit(f"FAIL: tomllib unavailable: {exc}")

try:
    import jsonschema
except Exception as exc:  # pragma: no cover
    raise SystemExit(f"FAIL: jsonschema unavailable: {exc}")


SCHEMA_VERSION = "1.0.0"
VALID_SCOPE_VALUES = {"system", "actor", "agent"}
QUERY_CASES = {
    "actor": (
        "shared/standards/knowledge_graphs/examples/triple_query/research_space_query_request_v1.json",
        "shared/standards/knowledge_graphs/examples/triple_query/research_space_query_response_v1.json",
    ),
    "system": (
        "shared/standards/knowledge_graphs/examples/triple_query/system_scope_query_request_v1.json",
        "shared/standards/knowledge_graphs/examples/triple_query/system_scope_query_response_v1.json",
    ),
    "agent": (
        "shared/standards/knowledge_graphs/examples/triple_query/agent_scope_query_request_v1.json",
        "shared/standards/knowledge_graphs/examples/triple_query/agent_scope_query_response_v1.json",
    ),
    "any": (
        "shared/standards/knowledge_graphs/examples/triple_query/any_scope_query_request_v1.json",
        "shared/standards/knowledge_graphs/examples/triple_query/any_scope_query_response_v1.json",
    ),
    "zero_result": (
        "shared/standards/knowledge_graphs/examples/triple_query/zero_result_query_request_v1.json",
        "shared/standards/knowledge_graphs/examples/triple_query/zero_result_query_response_v1.json",
    ),
    "provenance_disabled": (
        "shared/standards/knowledge_graphs/examples/triple_query/provenance_disabled_query_request_v1.json",
        "shared/standards/knowledge_graphs/examples/triple_query/provenance_disabled_query_response_v1.json",
    ),
    "scope_isolation": (
        "shared/standards/knowledge_graphs/examples/triple_query/scope_isolation_query_request_v1.json",
        "shared/standards/knowledge_graphs/examples/triple_query/scope_isolation_query_response_v1.json",
    ),
    "multi_hop_planning": (
        "shared/standards/knowledge_graphs/examples/triple_query/multi_hop_planning_query_request_v1.json",
        "shared/standards/knowledge_graphs/examples/triple_query/multi_hop_planning_query_response_v1.json",
    ),
}
POSITIVE_ONTOLOGY_EXAMPLES = [
    "shared/ontology/examples/research_space_ontology_v1.json",
    "shared/ontology/examples/operations_space_ontology_v1.json",
    "shared/ontology/examples/adversarial_extension_ontology_v1.json",
]
NEGATIVE_ONTOLOGY_EXAMPLES = {
    "shared/ontology/examples/invalid_core_redefinition_ontology_v1.json": "core_term_redefinition",
    "shared/ontology/examples/invalid_relation_endpoint_ontology_v1.json": "invalid_relation_endpoint",
    "shared/ontology/examples/invalid_provenance_extension_ontology_v1.json": "invalid_provenance_extension",
}
NEGATIVE_BUNDLE_EXAMPLES = {
    "shared/standards/knowledge_graphs/examples/negative/missing_ref_bundle_v1.json": "missing_required_reference",
    "shared/standards/knowledge_graphs/examples/negative/incompatible_ontology_version_bundle_v1.json": "incompatible_ontology_version",
    "shared/standards/knowledge_graphs/examples/negative/incompatible_bundle_version_bundle_v1.json": "incompatible_bundle_version",
    "shared/standards/knowledge_graphs/examples/negative/non_portable_export_bundle_v1.json": "non_portable_export_ref",
}
POSITIVE_BUNDLE_EXAMPLES = [
    ("shared/standards/knowledge_graphs/examples/research_space_knowledge_bundle_v1.json", False),
    ("shared/standards/knowledge_graphs/examples/operations_space_knowledge_bundle_v1.json", False),
    ("shared/standards/knowledge_graphs/examples/research_space_knowledge_bundle_export_v1.json", True),
    ("shared/standards/knowledge_graphs/examples/operations_space_graph_only_bundle_v1.json", True),
]
ROUNDTRIP_SOURCE = (
    "shared/standards/knowledge_graphs/examples/roundtrip/research_space_export_bundle_roundtrip_source_v1.json"
)
ROUNDTRIP_NORMALIZED = (
    "shared/standards/knowledge_graphs/examples/roundtrip/research_space_export_bundle_roundtrip_normalized_v1.json"
)
REQUIRED_COMPARATORS = {
    "trustgraph",
    "json_ld_1_1",
    "shacl_core",
    "owlish",
    "horned_owl",
    "sparql_1_1",
}
REQUIRED_SHACL_CHECKS = {
    "relation_endpoint_targets",
    "core_term_redefinition",
    "core_property_requiredness",
    "closed_provenance_scope",
}
REQUIRED_SPARQL_CONCEPTS = {
    "triple_pattern_filtering",
    "named_graph_scoping",
    "deterministic_ordering",
    "sparql_update_semantics",
}
TOPOLOGY_SCHEMA = "shared/standards/knowledge_graphs/explore_topology_view.schema.json"
TOPOLOGY_EXAMPLES = [
    "shared/standards/knowledge_graphs/examples/explore/research_space_topology_view_v1.json",
    "shared/standards/knowledge_graphs/examples/explore/research_agent_topology_view_v1.json",
]
FREEZE_REPORT = "shared/ontology/freeze_readiness_report.json"
EXPECTED_CORE_PROPERTIES = {
    "label": {
        "applies_to": {"Space", "Contribution", "Capability", "Relation"},
        "value_type": "Text",
        "cardinality": "single",
    },
    "description": {
        "applies_to": {"Space", "Contribution", "Capability", "Relation"},
        "value_type": "Text",
        "cardinality": "optional",
    },
    "resource_ref": {
        "applies_to": {"Space", "Contribution", "Capability", "Relation"},
        "value_type": "ResourceRef",
        "cardinality": "single",
    },
    "space_id": {
        "applies_to": {"Space", "Contribution"},
        "value_type": "Text",
        "cardinality": "single",
    },
    "authority_mode": {
        "applies_to": {"Contribution", "Relation"},
        "value_type": "Text",
        "cardinality": "single",
    },
    "provenance_scope": {
        "applies_to": {"Contribution", "Relation"},
        "value_type": "ProvenanceScope",
        "cardinality": "single",
    },
    "scope_ref": {
        "applies_to": {"Contribution", "Relation"},
        "value_type": "ResourceRef",
        "cardinality": "optional",
    },
}


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def schema_path(root: Path, relative: str) -> Path:
    return root / relative


def canonical_scope_rank(scope: str) -> int:
    return {"system": 0, "actor": 1, "agent": 2}.get(scope, 99)


def canonical_query_sort_key(triple: dict[str, Any]) -> tuple[Any, ...]:
    return (
        canonical_scope_rank(triple["graph_scope"]),
        triple["subject"],
        triple["predicate"],
        triple["object"],
        canonical_scope_rank(triple["provenance_scope"]),
        triple["source_ref"],
    )


def validate_schema(instance: dict[str, Any], schema: dict[str, Any], name: str) -> list[str]:
    try:
        jsonschema.validate(instance=instance, schema=schema)
    except jsonschema.ValidationError as exc:
        return [f"{name}: {exc.message}"]
    return []


def build_registry_report(root: Path) -> dict[str, Any]:
    registry_path = root / "shared/standards/knowledge_graphs/schema_registry.toml"
    registry = tomllib.loads(load_text(registry_path))
    schemas = registry.get("schemas", [])
    artifacts = registry.get("artifacts", [])
    failures: list[str] = []
    observed_ids: set[str] = set()
    observed_paths: set[str] = set()
    required_registry_keys = {
        "id",
        "title",
        "group",
        "path",
        "purpose",
        "owner",
        "aliases",
        "known_consumers",
        "status",
    }
    valid_statuses = {"active", "draft", "legacy"}

    def validate_entries(entries: list[dict[str, Any]], entry_type: str) -> None:
        for entry in entries:
            missing_keys = sorted(required_registry_keys - entry.keys())
            if missing_keys:
                failures.append(
                    f"{entry_type} {entry.get('id', '<unknown>')}: missing keys {', '.join(missing_keys)}"
                )
                continue

            entry_id = str(entry["id"]).strip()
            entry_path = str(entry["path"]).strip()
            if not entry_id:
                failures.append(f"{entry_type}: empty id")
            elif entry_id in observed_ids:
                failures.append(f"duplicate registry id: {entry_id}")
            else:
                observed_ids.add(entry_id)

            if not entry_path:
                failures.append(f"{entry_id}: empty path")
            else:
                relative_path = Path(entry_path)
                if relative_path.is_absolute():
                    failures.append(f"{entry_id}: path must be relative, got {entry_path}")
                observed_paths.add(entry_path)
                if not (root / relative_path).exists():
                    failures.append(f"{entry_id}: missing path on disk: {entry_path}")
                if entry_type == "schema" and not entry_path.endswith(".schema.json"):
                    failures.append(f"{entry_id}: schema entry must end with .schema.json")
                if entry_type == "artifact" and entry_path.endswith(".schema.json"):
                    failures.append(f"{entry_id}: artifact entry must not point at a schema")

            aliases = entry.get("aliases")
            if not isinstance(aliases, list) or not aliases:
                failures.append(f"{entry_id}: aliases must be a non-empty list")
            consumers = entry.get("known_consumers")
            if not isinstance(consumers, list):
                failures.append(f"{entry_id}: known_consumers must be a list")

            status = str(entry["status"]).strip()
            if status not in valid_statuses:
                failures.append(f"{entry_id}: invalid status {status}")

    validate_entries(schemas, "schema")
    validate_entries(artifacts, "artifact")

    schema_files = {
        str(path.relative_to(root)).replace("\\", "/")
        for path in (root / "shared/standards").rglob("*.schema.json")
        if "_archive" not in path.parts
    }
    artifact_files = {
        str(path.relative_to(root)).replace("\\", "/")
        for path in (root / "shared/standards").rglob("*.json")
        if "_archive" not in path.parts and not path.name.endswith(".schema.json")
    }
    missing_from_registry = sorted((schema_files | artifact_files) - observed_paths)
    extra_registry_paths = sorted(observed_paths - (schema_files | artifact_files))
    if missing_from_registry:
        failures.append("registry is missing files:")
        failures.extend(f" - {path}" for path in missing_from_registry)
    if extra_registry_paths:
        failures.append("registry references paths that do not exist:")
        failures.extend(f" - {path}" for path in extra_registry_paths)

    return {
        "schema_count": len(schemas),
        "artifact_count": len(artifacts),
        "discovered_files": len(schema_files) + len(artifact_files),
        "failures": failures,
    }


def validate_scope_rule(scope: dict[str, Any]) -> list[str]:
    named = scope["named_graph_scope"]
    scope_ref = scope.get("scope_ref")
    if named in {"actor", "agent"} and not scope_ref:
        return [f"scope_ref is required for {named} scope"]
    if named == "system" and scope_ref:
        return ["scope_ref is forbidden for system scope"]
    if named == "any" and scope_ref:
        return ["scope_ref must be omitted for any scope"]
    return []


def validate_query_fixture_matrix(root: Path) -> dict[str, Any]:
    request_schema = load_json(root / "shared/standards/knowledge_graphs/triple_query_request.schema.json")
    response_schema = load_json(root / "shared/standards/knowledge_graphs/triple_query_response.schema.json")
    failures: list[str] = []
    covered_cases: set[str] = set()

    for case_name, (request_path_str, response_path_str) in QUERY_CASES.items():
        request_path = root / request_path_str
        response_path = root / response_path_str
        request = load_json(request_path)
        response = load_json(response_path)
        failures.extend(validate_schema(request, request_schema, request_path_str))
        failures.extend(validate_schema(response, response_schema, response_path_str))
        failures.extend(f"{request_path_str}: {msg}" for msg in validate_scope_rule(request["scope"]))
        failures.extend(f"{response_path_str}: {msg}" for msg in validate_scope_rule(response["scope"]))

        if request["query_id"] != response["query_id"]:
            failures.append(f"{case_name}: request/response query_id mismatch")
        if request["ordering_strategy"] != "canonical" or response["ordering_strategy"] != "canonical":
            failures.append(f"{case_name}: ordering_strategy must stay canonical")
        if request["scope"] != response["scope"]:
            failures.append(f"{case_name}: request and response scope must match")
        if response["result_count"] != len(response["triples"]):
            failures.append(f"{case_name}: result_count must equal number of triples")

        expected_ordinals = list(range(len(response["triples"])))
        actual_ordinals = [triple["ordinal"] for triple in response["triples"]]
        if actual_ordinals != expected_ordinals:
            failures.append(f"{case_name}: ordinals must be sequential from zero")

        if response["triples"] != sorted(response["triples"], key=canonical_query_sort_key):
            failures.append(f"{case_name}: triples are not in canonical order")

        named_scope = request["scope"]["named_graph_scope"]
        if named_scope in VALID_SCOPE_VALUES:
            if any(item["graph_scope"] != named_scope for item in response["triples"]):
                failures.append(f"{case_name}: response leaked triples outside requested scope")

        if not request["filters"]["include_provenance"]:
            if any("confidence" in item for item in response["triples"]):
                failures.append(
                    f"{case_name}: confidence should be omitted when include_provenance=false"
                )

        if any(item["provenance_scope"] not in VALID_SCOPE_VALUES for item in response["triples"]):
            failures.append(f"{case_name}: invalid provenance_scope in response")

        covered_cases.add(case_name)

    missing_cases = sorted(set(QUERY_CASES) - covered_cases)
    return {
        "covered_cases": covered_cases,
        "missing_cases": missing_cases,
        "failures": failures,
    }


def normalize_bundle(bundle: dict[str, Any]) -> dict[str, Any]:
    normalized = copy.deepcopy(bundle)
    policy = normalized["retrieval_policy"]
    policy["authority_weights"] = sorted(
        policy["authority_weights"], key=lambda item: canonical_scope_rank(item["scope"])
    )
    policy["allowed_predicates"] = sorted(policy["allowed_predicates"])
    policy["named_graph_scopes"] = sorted(
        policy["named_graph_scopes"], key=canonical_scope_rank
    )
    return normalized


def is_portable_ref(value: str) -> bool:
    disallowed_prefixes = ("/", "file://", "http://127.0.0.1", "http://localhost")
    if value.startswith(disallowed_prefixes):
        return False
    if value.startswith("nostra://") and not value.endswith("/latest"):
        return True
    return False


def validate_bundle_semantics(bundle: dict[str, Any], export_grade: bool) -> list[str]:
    failures: list[str] = []
    required_ref_fields = [
        "ontology_ref",
        "graph_snapshot_ref",
        "provenance_root",
    ]
    for field in required_ref_fields:
        if not bundle.get(field):
            failures.append("missing_required_reference")

    policy = bundle["retrieval_policy"]
    retrieval_mode = policy.get("retrieval_mode")
    if retrieval_mode not in {"graph_only", "hybrid_graph_embedding"}:
        failures.append("invalid_retrieval_mode")
    if retrieval_mode == "hybrid_graph_embedding" and not bundle.get("embeddings_manifest_ref"):
        failures.append("embeddings_manifest_required")
    if retrieval_mode == "graph_only" and bundle.get("embeddings_manifest_ref") is None:
        pass

    compatibility = bundle["compatibility"]
    if compatibility["ontology_version"] != SCHEMA_VERSION:
        failures.append("incompatible_ontology_version")
    if compatibility["bundle_version"] != SCHEMA_VERSION:
        failures.append("incompatible_bundle_version")

    if export_grade:
        refs = [
            bundle["ontology_ref"],
            bundle["graph_snapshot_ref"],
            bundle["provenance_root"],
        ]
        if bundle.get("embeddings_manifest_ref"):
            refs.append(bundle["embeddings_manifest_ref"])
        if not all(is_portable_ref(value) for value in refs):
            failures.append("non_portable_export_ref")
    return failures


def validate_negative_bundle_examples(root: Path) -> list[dict[str, str]]:
    bundle_schema = load_json(root / "shared/standards/knowledge_graphs/knowledge_bundle.schema.json")
    observed: list[dict[str, str]] = []
    for path_str, expected_reason in NEGATIVE_BUNDLE_EXAMPLES.items():
        path = root / path_str
        bundle = load_json(path)
        schema_failures = validate_schema(bundle, bundle_schema, path_str)
        semantic_failures = validate_bundle_semantics(bundle, export_grade=True)
        combined = schema_failures + semantic_failures
        if expected_reason == "missing_required_reference" and not combined:
            raise ValueError(f"{path_str} unexpectedly passed")
        if expected_reason != "missing_required_reference" and expected_reason not in combined:
            raise ValueError(f"{path_str} did not fail with {expected_reason}")
        observed.append({"fixture": path_str, "reason": expected_reason})
    return observed


def validate_bundle_roundtrip(
    root: Path, source_path: Path, normalized_path: Path
) -> dict[str, Any]:
    source_bundle = load_json(source_path)
    expected_normalized = load_json(normalized_path)
    normalized = normalize_bundle(source_bundle)
    normalized_match = normalized == expected_normalized
    idempotent = normalize_bundle(expected_normalized) == expected_normalized
    return {
        "source": str(source_path.relative_to(root)).replace("\\", "/"),
        "normalized": str(normalized_path.relative_to(root)).replace("\\", "/"),
        "normalized_match": normalized_match,
        "idempotent": idempotent,
    }


def compare_core_term(definition: dict[str, Any], core_definition: dict[str, Any]) -> bool:
    keys = {"id", "description", "source", "target", "applies_to", "value_type", "cardinality"}
    filtered = {key: definition.get(key) for key in keys if key in definition}
    filtered_core = {key: core_definition.get(key) for key in keys if key in core_definition}
    return filtered == filtered_core


def ontology_failure_reason(messages: list[str]) -> str:
    joined = " ".join(messages)
    if "redefines core term" in joined:
        return "core_term_redefinition"
    if "invalid relation endpoint" in joined:
        return "invalid_relation_endpoint"
    if "closed core set" in joined:
        return "invalid_provenance_extension"
    return "semantic_validation_failed"


def validate_ontology_semantics(ontology: dict[str, Any], core_ontology: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    class_index = {item["id"]: item for item in ontology["classes"]}
    relation_index = {item["id"]: item for item in ontology["relations"]}
    property_index = {item["id"]: item for item in ontology["properties"]}
    core_class_index = {item["id"]: item for item in core_ontology["classes"]}
    core_relation_index = {item["id"]: item for item in core_ontology["relations"]}
    core_property_index = {item["id"]: item for item in core_ontology["properties"]}

    for relation in ontology["relations"]:
        if relation["source"] not in class_index or relation["target"] not in class_index:
            failures.append(
                f"invalid relation endpoint: {relation['id']} references undefined source/target"
            )

    for prop in ontology["properties"]:
        missing_targets = [name for name in prop["applies_to"] if name not in class_index]
        if missing_targets:
            failures.append(
                f"property {prop['id']} references undefined classes: {', '.join(missing_targets)}"
            )

    for item in ontology["classes"]:
        for parent in item.get("extends", []):
            if parent not in class_index:
                failures.append(f"class {item['id']} extends undefined class {parent}")
            if parent == "ProvenanceScope" and item["id"] != "ProvenanceScope":
                failures.append(
                    f"class {item['id']} attempts to extend ProvenanceScope, which is a closed core set"
                )

    for core_id, expected in EXPECTED_CORE_PROPERTIES.items():
        actual = property_index.get(core_id)
        if actual is None:
            failures.append(f"missing required core property semantics: {core_id}")
            continue
        if set(actual["applies_to"]) != expected["applies_to"]:
            failures.append(f"missing required core property semantics: {core_id}")
        if actual["value_type"] != expected["value_type"]:
            failures.append(f"missing required core property semantics: {core_id}")
        if actual.get("cardinality") != expected["cardinality"]:
            failures.append(f"missing required core property semantics: {core_id}")

    for item_id, core_item in core_class_index.items():
        if item_id in class_index and not compare_core_term(class_index[item_id], core_item):
            failures.append(f"{item_id} redefines core term semantics")
    for item_id, core_item in core_relation_index.items():
        if item_id in relation_index and not compare_core_term(relation_index[item_id], core_item):
            failures.append(f"{item_id} redefines core term semantics")
    for item_id, core_item in core_property_index.items():
        if item_id in property_index and not compare_core_term(property_index[item_id], core_item):
            failures.append(f"{item_id} redefines core term semantics")

    return failures


def load_reference_alignment_matrix(root: Path) -> dict[str, Any]:
    matrix = load_json(root / "shared/ontology/reference_alignment_matrix.json")
    comparators = [entry["id"] for entry in matrix["comparators"]]
    missing = sorted(REQUIRED_COMPARATORS - set(comparators))
    if missing:
        raise ValueError(f"reference alignment matrix missing comparators: {', '.join(missing)}")
    for entry in matrix["comparators"]:
        if entry.get("freeze_outcome") != "pass":
            raise ValueError(
                f"reference alignment comparator missing pass outcome: {entry['id']}"
            )
        evidence_refs = entry.get("evidence_refs")
        if not isinstance(evidence_refs, list) or not evidence_refs:
            raise ValueError(
                f"reference alignment comparator missing evidence_refs: {entry['id']}"
            )
    return {"comparators": comparators, "entries": matrix["comparators"]}


def validate_shacl_checklist(root: Path) -> list[str]:
    checklist = load_json(root / "shared/ontology/shacl_core_validation_checklist.json")
    observed = {item["id"] for item in checklist["items"]}
    missing = sorted(REQUIRED_SHACL_CHECKS - observed)
    failures: list[str] = []
    if missing:
        failures.append(f"missing SHACL-style coverage items: {', '.join(missing)}")
    for item in checklist["items"]:
        if item.get("status") != "covered":
            failures.append(f"SHACL-style checklist item not covered: {item['id']}")
        if not item.get("covered_by"):
            failures.append(f"SHACL-style checklist item missing covered_by: {item['id']}")
    return failures


def validate_sparql_matrix(root: Path) -> list[str]:
    matrix = load_json(root / "shared/standards/knowledge_graphs/sparql_query_facade_matrix.json")
    rows = {item["concept"]: item for item in matrix["comparisons"]}
    missing = sorted(REQUIRED_SPARQL_CONCEPTS - set(rows))
    failures: list[str] = []
    if missing:
        failures.append(f"missing SPARQL comparator concepts: {', '.join(missing)}")
    update_row = rows.get("sparql_update_semantics")
    if update_row and update_row.get("support") != "out_of_scope":
        failures.append("sparql_update_semantics must remain out_of_scope")
    return failures


def validate_jsonld_projection(root: Path, core_ontology: dict[str, Any]) -> list[str]:
    projection = load_json(root / "shared/ontology/core_ontology_v1.experimental.jsonld")
    failures: list[str] = []
    projected_classes = {entry["id"] for entry in projection["classes"]}
    expected_classes = {f"nostra:{item['id']}" for item in core_ontology["classes"]}
    if projected_classes != expected_classes:
        failures.append("JSON-LD projection classes drift from canonical ontology")
    projected_relations = {entry["id"] for entry in projection["relations"]}
    expected_relations = {f"nostra:{item['id']}" for item in core_ontology["relations"]}
    if projected_relations != expected_relations:
        failures.append("JSON-LD projection relations drift from canonical ontology")
    projected_properties = {entry["id"] for entry in projection["properties"]}
    expected_properties = {f"nostra:{item['id']}" for item in core_ontology["properties"]}
    if projected_properties != expected_properties:
        failures.append("JSON-LD projection properties drift from canonical ontology")
    return failures


def validate_topology_view(root: Path) -> list[str]:
    schema = load_json(root / TOPOLOGY_SCHEMA)
    failures: list[str] = []
    for example_path in TOPOLOGY_EXAMPLES:
        example = load_json(root / example_path)
        failures.extend(validate_schema(example, schema, example_path))
        if example.get("ordering_strategy") != "canonical":
            failures.append(f"{example_path}: ordering_strategy must stay canonical")
        node_ids = [item["node_id"] for item in example.get("nodes", [])]
        if node_ids != sorted(node_ids):
            failures.append(f"{example_path}: nodes must be sorted canonically by node_id")
        edge_ids = [item["edge_id"] for item in example.get("edges", [])]
        if edge_ids != sorted(edge_ids):
            failures.append(f"{example_path}: edges must be sorted canonically by edge_id")
    return failures


def validate_freeze_report(root: Path) -> list[str]:
    report = load_json(root / FREEZE_REPORT)
    failures: list[str] = []
    decision = report.get("decision")
    if decision not in {"freeze", "revise"}:
        failures.append("freeze readiness report decision must be freeze or revise")
    if report.get("recommendation") not in {"ratify_v1", "revise_before_freeze"}:
        failures.append("freeze readiness report recommendation is invalid")
    comparators = {item["id"]: item for item in report.get("comparators", [])}
    missing = sorted(REQUIRED_COMPARATORS - set(comparators))
    if missing:
        failures.append(
            f"freeze readiness report missing comparator outcomes: {', '.join(missing)}"
        )
    if decision == "freeze":
        if report.get("semantic_exceptions"):
            failures.append("freeze readiness report must not carry semantic exceptions")
        for comparator in comparators.values():
            if comparator.get("outcome") != "pass":
                failures.append(
                    f"freeze readiness report comparator must pass for freeze: {comparator['id']}"
                )
        for gate_name, gate_status in report.get("validation_gates", {}).items():
            if gate_status != "pass":
                failures.append(f"freeze readiness gate must pass: {gate_name}")
    return failures


def build_validation_report(root: Path) -> dict[str, Any]:
    core_ontology = load_json(root / "shared/ontology/core_ontology_v1.json")
    ontology_report = {
        "positive_examples": list(POSITIVE_ONTOLOGY_EXAMPLES),
        "negative_examples": [
            {"fixture": path, "reason": reason}
            for path, reason in NEGATIVE_ONTOLOGY_EXAMPLES.items()
        ],
    }
    query_report = validate_query_fixture_matrix(root)
    return {
        "ontology": ontology_report,
        "query": query_report,
        "reference_alignment": load_reference_alignment_matrix(root),
        "freeze_report": load_json(root / FREEZE_REPORT),
    }


def validate_all(root: Path) -> list[str]:
    failures: list[str] = []
    registry_report = build_registry_report(root)
    failures.extend(registry_report["failures"])

    ontology_schema = load_json(root / "shared/standards/knowledge_graphs/ontology_manifest.schema.json")
    bundle_schema = load_json(root / "shared/standards/knowledge_graphs/knowledge_bundle.schema.json")
    request_schema = load_json(root / "shared/standards/knowledge_graphs/triple_query_request.schema.json")
    response_schema = load_json(root / "shared/standards/knowledge_graphs/triple_query_response.schema.json")

    core_ontology = load_json(root / "shared/ontology/core_ontology_v1.json")
    failures.extend(validate_schema(core_ontology, ontology_schema, "core_ontology_v1.json"))
    failures.extend(validate_ontology_semantics(core_ontology, core_ontology))

    for path_str in POSITIVE_ONTOLOGY_EXAMPLES:
        path = root / path_str
        ontology = load_json(path)
        failures.extend(validate_schema(ontology, ontology_schema, path_str))
        failures.extend(f"{path_str}: {msg}" for msg in validate_ontology_semantics(ontology, core_ontology))

    for path_str, expected_reason in NEGATIVE_ONTOLOGY_EXAMPLES.items():
        path = root / path_str
        ontology = load_json(path)
        messages = validate_schema(ontology, ontology_schema, path_str)
        messages.extend(validate_ontology_semantics(ontology, core_ontology))
        if not messages:
            failures.append(f"{path_str}: negative ontology fixture unexpectedly passed")
            continue
        observed_reason = ontology_failure_reason(messages)
        if observed_reason != expected_reason:
            failures.append(
                f"{path_str}: expected {expected_reason}, observed {observed_reason}"
            )

    for path_str, export_grade in POSITIVE_BUNDLE_EXAMPLES:
        path = root / path_str
        bundle = load_json(path)
        failures.extend(validate_schema(bundle, bundle_schema, path_str))
        failures.extend(f"{path_str}: {msg}" for msg in validate_bundle_semantics(bundle, export_grade))

    try:
        negative_bundle_results = validate_negative_bundle_examples(root)
    except ValueError as exc:
        failures.append(str(exc))
        negative_bundle_results = []

    roundtrip_report = validate_bundle_roundtrip(
        root,
        root / ROUNDTRIP_SOURCE,
        root / ROUNDTRIP_NORMALIZED,
    )
    if not roundtrip_report["normalized_match"]:
        failures.append("bundle roundtrip normalization does not match expected fixture")
    if not roundtrip_report["idempotent"]:
        failures.append("bundle roundtrip normalization is not idempotent")

    query_report = validate_query_fixture_matrix(root)
    failures.extend(query_report["failures"])
    for request_path_str, response_path_str in QUERY_CASES.values():
        request = load_json(root / request_path_str)
        response = load_json(root / response_path_str)
        failures.extend(validate_schema(request, request_schema, request_path_str))
        failures.extend(validate_schema(response, response_schema, response_path_str))

    failures.extend(validate_jsonld_projection(root, core_ontology))
    failures.extend(validate_shacl_checklist(root))
    failures.extend(validate_sparql_matrix(root))
    failures.extend(validate_topology_view(root))
    failures.extend(validate_freeze_report(root))
    try:
        load_reference_alignment_matrix(root)
    except ValueError as exc:
        failures.append(str(exc))

    if negative_bundle_results and len(negative_bundle_results) != len(NEGATIVE_BUNDLE_EXAMPLES):
        failures.append("negative bundle fixture coverage is incomplete")

    return failures


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate knowledge graph ontology, bundle, and query contracts."
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="Workspace root",
    )
    args = parser.parse_args()

    failures = validate_all(args.root)
    registry_report = build_registry_report(args.root)
    if failures:
        print("FAIL: knowledge graph contract validation failed")
        for failure in failures:
            print(f" - {failure}")
        return 1

    print(
        "PASS: knowledge graph contract package is consistent "
        f"({registry_report['schema_count']} schema entries, "
        f"{registry_report['artifact_count']} artifact entries, "
        f"{registry_report['discovered_files']} discovered files)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
