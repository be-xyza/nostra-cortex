#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${NOSTRA_WORKSPACE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
REGISTRY_PATH="$ROOT_DIR/shared/standards/knowledge_graphs/schema_registry.toml"
ONTOLOGY_SCHEMA_PATH="$ROOT_DIR/shared/standards/knowledge_graphs/ontology_manifest.schema.json"
ONTOLOGY_PATH="$ROOT_DIR/shared/ontology/core_ontology_v1.json"
ONTOLOGY_JSONLD_GENERATOR="$ROOT_DIR/scripts/generate_core_ontology_jsonld.py"
RESEARCH_EXAMPLE_ONTOLOGY_PATH="$ROOT_DIR/shared/ontology/examples/research_space_ontology_v1.json"
OPERATIONS_EXAMPLE_ONTOLOGY_PATH="$ROOT_DIR/shared/ontology/examples/operations_space_ontology_v1.json"
BUNDLE_SCHEMA_PATH="$ROOT_DIR/shared/standards/knowledge_graphs/knowledge_bundle.schema.json"
RESEARCH_BUNDLE_SAMPLE_PATH="$ROOT_DIR/shared/standards/knowledge_graphs/examples/research_space_knowledge_bundle_v1.json"
OPERATIONS_BUNDLE_SAMPLE_PATH="$ROOT_DIR/shared/standards/knowledge_graphs/examples/operations_space_knowledge_bundle_v1.json"
TRIPLE_REQUEST_SCHEMA_PATH="$ROOT_DIR/shared/standards/knowledge_graphs/triple_query_request.schema.json"
TRIPLE_RESPONSE_SCHEMA_PATH="$ROOT_DIR/shared/standards/knowledge_graphs/triple_query_response.schema.json"
TRIPLE_REQUEST_SAMPLE_PATH="$ROOT_DIR/shared/standards/knowledge_graphs/examples/triple_query/research_space_query_request_v1.json"
TRIPLE_RESPONSE_SAMPLE_PATH="$ROOT_DIR/shared/standards/knowledge_graphs/examples/triple_query/research_space_query_response_v1.json"

python3 - "$ROOT_DIR" "$REGISTRY_PATH" "$ONTOLOGY_SCHEMA_PATH" "$ONTOLOGY_PATH" "$RESEARCH_EXAMPLE_ONTOLOGY_PATH" "$OPERATIONS_EXAMPLE_ONTOLOGY_PATH" "$BUNDLE_SCHEMA_PATH" "$RESEARCH_BUNDLE_SAMPLE_PATH" "$OPERATIONS_BUNDLE_SAMPLE_PATH" "$TRIPLE_REQUEST_SCHEMA_PATH" "$TRIPLE_RESPONSE_SCHEMA_PATH" "$TRIPLE_REQUEST_SAMPLE_PATH" "$TRIPLE_RESPONSE_SAMPLE_PATH" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    import tomllib
except Exception as exc:  # pragma: no cover
    print(f"FAIL: tomllib unavailable: {exc}")
    raise SystemExit(1)

try:
    import jsonschema
except Exception as exc:  # pragma: no cover
    print(f"FAIL: jsonschema unavailable: {exc}")
    raise SystemExit(1)

root = Path(sys.argv[1])
registry_path = Path(sys.argv[2])
ontology_schema_path = Path(sys.argv[3])
ontology_path = Path(sys.argv[4])
research_example_ontology_path = Path(sys.argv[5])
operations_example_ontology_path = Path(sys.argv[6])
bundle_schema_path = Path(sys.argv[7])
research_bundle_sample_path = Path(sys.argv[8])
operations_bundle_sample_path = Path(sys.argv[9])
triple_request_schema_path = Path(sys.argv[10])
triple_response_schema_path = Path(sys.argv[11])
triple_request_sample_path = Path(sys.argv[12])
triple_response_sample_path = Path(sys.argv[13])

required_paths = [
    registry_path,
    ontology_schema_path,
    ontology_path,
    research_example_ontology_path,
    operations_example_ontology_path,
    bundle_schema_path,
    research_bundle_sample_path,
    operations_bundle_sample_path,
    triple_request_schema_path,
    triple_response_schema_path,
    triple_request_sample_path,
    triple_response_sample_path,
]
missing = [str(path) for path in required_paths if not path.exists()]
if missing:
    print("FAIL: missing knowledge graph contract artifacts:")
    for path in missing:
        print(f" - {path}")
    raise SystemExit(1)

registry = tomllib.loads(registry_path.read_text(encoding="utf-8"))
schemas = registry.get("schemas", [])
artifacts = registry.get("artifacts", [])
if not isinstance(schemas, list) or not schemas:
    print("FAIL: schema registry missing schemas entries")
    raise SystemExit(1)
if not isinstance(artifacts, list) or not artifacts:
    print("FAIL: schema registry missing artifacts entries")
    raise SystemExit(1)

required_registry_keys = {"id", "title", "group", "path", "purpose", "owner", "aliases", "known_consumers", "status"}
valid_statuses = {"active", "draft", "legacy"}
observed_paths: set[str] = set()
observed_ids: set[str] = set()
failures: list[str] = []

def validate_entries(entries: list[dict], entry_type: str) -> None:
    for entry in entries:
        if not isinstance(entry, dict):
            failures.append(f"{entry_type} entry is not a table")
            continue

        missing_keys = sorted(required_registry_keys - entry.keys())
        if missing_keys:
            failures.append(f"{entry.get('id', '<unknown>')}: missing keys {', '.join(missing_keys)}")
            continue

        schema_id = str(entry.get("id", "")).strip()
        schema_path = str(entry.get("path", "")).strip()
        if not schema_id:
            failures.append(f"{entry_type} entry has empty id")
        elif schema_id in observed_ids:
            failures.append(f"duplicate registry id: {schema_id}")
        else:
            observed_ids.add(schema_id)

        if not schema_path:
            failures.append(f"{schema_id}: empty path")
        else:
            relative_path = Path(schema_path)
            if relative_path.is_absolute():
                failures.append(f"{schema_id}: path must be relative to the workspace, got {schema_path}")
            observed_paths.add(schema_path)
            full_path = root / relative_path
            if not full_path.exists():
                failures.append(f"{schema_id}: registry path missing on disk: {schema_path}")

            if entry_type == "schema" and not schema_path.endswith(".schema.json"):
                failures.append(f"{schema_id}: schema entry must point at a .schema.json file")
            if entry_type == "artifact" and schema_path.endswith(".schema.json"):
                failures.append(f"{schema_id}: artifact entry must not point at a .schema.json file")

        aliases = entry.get("aliases", [])
        consumers = entry.get("known_consumers", [])
        if not isinstance(aliases, list) or not aliases:
            failures.append(f"{schema_id}: aliases must be a non-empty list")
        if not isinstance(consumers, list):
            failures.append(f"{schema_id}: known_consumers must be a list")

        status = str(entry.get("status", "")).strip()
        if status not in valid_statuses:
            failures.append(f"{schema_id}: invalid status '{status}'")

validate_entries(schemas, "schema")
validate_entries(artifacts, "artifact")

schema_files = {
    str(path.relative_to(root)).replace("\\", "/")
    for path in (root / "shared" / "standards").rglob("*.schema.json")
    if "_archive" not in path.parts
}
artifact_files = {
    str(path.relative_to(root)).replace("\\", "/")
    for path in (root / "shared" / "standards").rglob("*.json")
    if "_archive" not in path.parts and not path.name.endswith(".schema.json")
}

registry_paths = observed_paths
missing_from_registry = sorted((schema_files | artifact_files) - registry_paths)
extra_registry_paths = sorted(registry_paths - (schema_files | artifact_files))

if missing_from_registry:
    failures.append("registry is missing schema files:")
    failures.extend(f" - {path}" for path in missing_from_registry)

if extra_registry_paths:
    failures.append("registry references paths that do not exist in the schema tree:")
    failures.extend(f" - {path}" for path in extra_registry_paths)

def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise SystemExit(f"FAIL: invalid JSON at {path}: {exc}")

def validate(instance: dict, schema_path: Path, name: str) -> None:
    schema = load_json(schema_path)
    try:
        jsonschema.validate(instance=instance, schema=schema)
    except jsonschema.ValidationError as exc:
        raise SystemExit(f"FAIL: schema validation failed for {name}: {exc.message}")

ontology_schema = load_json(ontology_schema_path)
ontology = load_json(ontology_path)
research_example_ontology = load_json(research_example_ontology_path)
operations_example_ontology = load_json(operations_example_ontology_path)
bundle_schema = load_json(bundle_schema_path)
research_bundle_sample = load_json(research_bundle_sample_path)
operations_bundle_sample = load_json(operations_bundle_sample_path)
triple_request_schema = load_json(triple_request_schema_path)
triple_response_schema = load_json(triple_response_schema_path)
triple_request_sample = load_json(triple_request_sample_path)
triple_response_sample = load_json(triple_response_sample_path)

validate(ontology, ontology_schema_path, "ontology")
validate(research_example_ontology, ontology_schema_path, "research_example_ontology")
validate(operations_example_ontology, ontology_schema_path, "operations_example_ontology")
validate(research_bundle_sample, bundle_schema_path, "research_bundle_sample")
validate(operations_bundle_sample, bundle_schema_path, "operations_bundle_sample")
validate(triple_request_sample, triple_request_schema_path, "triple_request_sample")
validate(triple_response_sample, triple_response_schema_path, "triple_response_sample")

bundle_sample = {
    "schema_version": "1.0.0",
    "bundle_id": "space-078-core-bundle",
    "bundle_kind": "space_knowledge_bundle",
    "space_id": "078",
    "ontology_ref": "nostra://ontology/core-knowledge-graph/v1",
    "graph_snapshot_ref": "nostra://graph/078/snapshot/latest",
    "embeddings_manifest_ref": "nostra://embeddings/078/manifest/latest",
    "provenance_root": "nostra://contribution/078",
    "retrieval_policy": {
        "max_hops": 3,
        "authority_weights": [
            {"scope": "system", "weight": 1.0},
            {"scope": "actor", "weight": 0.8},
            {"scope": "agent", "weight": 0.6},
        ],
        "freshness_window_seconds": 604800,
        "allowed_predicates": ["clarifies", "depends_on", "supports"],
        "named_graph_scopes": ["system", "actor", "agent"],
        "citation_required": True,
    },
    "compatibility": {
        "ontology_version": "1.0.0",
        "bundle_version": "1.0.0",
        "breaking_change_policy": "steward approval required",
    },
    "transport": {
        "interactive_protocol": "json",
        "bulk_protocol": "msgpack",
    },
    "generated_at": "2026-03-23T00:00:00Z",
    "notes": "Sample bundle used to validate the Phase D manifest contract.",
}
validate(bundle_sample, bundle_schema_path, "knowledge_bundle")

triple_request_sample = {
    "schema_version": "1.0.0",
    "query_id": "query-078-lineage",
    "ordering_strategy": "canonical",
    "scope": {"named_graph_scope": "actor", "scope_ref": "nostra://actor/did:nostra:alice"},
    "filters": {
        "subject": "nostra://contribution/078",
        "predicate": "relates_to",
        "object": "nostra://contribution/ontology",
        "include_provenance": True,
        "limit": 25,
        "offset": 0,
    },
}
validate(triple_request_sample, triple_request_schema_path, "triple_query_request")

triple_response_sample = {
    "schema_version": "1.0.0",
    "query_id": "query-078-lineage",
    "ordering_strategy": "canonical",
    "scope": {"named_graph_scope": "actor", "scope_ref": "nostra://actor/did:nostra:alice"},
    "result_count": 1,
    "triples": [
        {
            "ordinal": 0,
            "subject": "nostra://contribution/078",
            "predicate": "relates_to",
            "object": "nostra://contribution/ontology",
            "graph_scope": "actor",
            "provenance_scope": "actor",
            "source_ref": "nostra://event/route-lineage/078",
            "confidence": 0.94,
        }
    ],
}
validate(triple_response_sample, triple_response_schema_path, "triple_query_response")

if failures:
    print("FAIL: knowledge graph schema registry validation failed")
    for failure in failures:
        print(failure)
    raise SystemExit(1)

print(
    "PASS: knowledge graph contract package is consistent "
    f"({len(schemas)} schema entries, {len(artifacts)} artifact entries, "
    f"{len(schema_files) + len(artifact_files)} discovered files)"
)
PY

python3 "$ONTOLOGY_JSONLD_GENERATOR" --check
