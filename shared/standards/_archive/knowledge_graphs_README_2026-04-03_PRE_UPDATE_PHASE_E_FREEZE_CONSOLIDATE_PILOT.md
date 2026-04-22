# Knowledge Graph Commons and Legacy Operational Contracts v1

This directory now serves two roles:

1. Legacy filesystem-canonical operational artifacts for `motoko-graph`.
2. Phase D commons contract schemas for the schema registry, ontology manifest, knowledge bundle, and triple query facade.

## Legacy Operational Artifacts

- `/Users/xaoj/ICP/logs/knowledge_graphs/motoko_graph/snapshot_latest.json`
- `/Users/xaoj/ICP/logs/knowledge_graphs/motoko_graph/history/<event_id>.json`
- `/Users/xaoj/ICP/logs/knowledge_graphs/motoko_graph/decisions/pending/<decision_event_id>.json`
- `/Users/xaoj/ICP/logs/knowledge_graphs/motoko_graph/monitoring_runs/<run_id>.json`
- `/Users/xaoj/ICP/logs/knowledge_graphs/motoko_graph/monitoring_trend_latest.json`

## Legacy Schemas

- `motoko_graph_snapshot.schema.json`
- `motoko_graph_decision_event.schema.json`
- `motoko_graph_monitoring_run.schema.json`
- `motoko_graph_monitoring_trend.schema.json`

## Phase D Commons Artifacts

- `schema_registry.toml`
- `ontology_manifest.schema.json`
- `knowledge_bundle.schema.json`
- `triple_query_request.schema.json`
- `triple_query_response.schema.json`

## Phase D Example Fixtures

- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/research_space_knowledge_bundle_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/operations_space_knowledge_bundle_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/research_space_knowledge_bundle_export_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/operations_space_graph_only_bundle_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/roundtrip/research_space_export_bundle_roundtrip_source_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/roundtrip/research_space_export_bundle_roundtrip_normalized_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/negative/`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/research_space_query_request_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/research_space_query_response_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/system_scope_query_request_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/system_scope_query_response_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/agent_scope_query_request_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/agent_scope_query_response_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/any_scope_query_request_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/any_scope_query_response_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/zero_result_query_request_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/zero_result_query_response_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/provenance_disabled_query_request_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/provenance_disabled_query_response_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/scope_isolation_query_request_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/scope_isolation_query_response_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/multi_hop_planning_query_request_v1.json`
- `/Users/xaoj/ICP/shared/standards/knowledge_graphs/examples/triple_query/multi_hop_planning_query_response_v1.json`
- `/Users/xaoj/ICP/shared/ontology/examples/research_space_ontology_v1.json`
- `/Users/xaoj/ICP/shared/ontology/examples/operations_space_ontology_v1.json`
- `/Users/xaoj/ICP/shared/ontology/examples/adversarial_extension_ontology_v1.json`

## Versioned JSON Artifacts

- `shared/standards/alignment_contract_exceptions.json`
- `shared/standards/antigravity_rule_dispositions.json`
- `shared/standards/dynamic_source_exceptions.json`
- `shared/standards/branding/brand_policy_v1.json`
- `shared/standards/branding/brand_visual_state_cases_v1.json`

## Phase D Ontology Home

- `/Users/xaoj/ICP/shared/ontology/core_ontology_v1.json`

## Artifact Semantics

1. Legacy operational files remain append-only and preserve prior snapshots and decision events.
2. `schema_registry.toml` is the discovery index for schema artifacts and versioned JSON standards under `shared/standards/`.
3. `core_ontology_v1.json` is the minimal ontology manifest for the commons graph layer.
4. `knowledge_bundle.schema.json` packages ontology, graph snapshot, embeddings, provenance, and retrieval policy by reference.
5. `triple_query_request.schema.json` and `triple_query_response.schema.json` define the read-only S-P-O facade.
6. Example fixtures exist to prove the ontology, bundle, and triple query schemas can carry realistic Space-local contracts.
7. Export-grade bundle examples must use immutable portable refs; `latest` fixtures remain dev/example only.
8. The bundle and query facade are compared against TrustGraph and SPARQL-style patterns without promising full upstream compatibility.
9. Versioned JSON artifacts are discoverable alongside schemas, but remain a separate contract class from `.schema.json` files.

## Commands

```bash
bash /Users/xaoj/ICP/scripts/generate_motoko_graph_snapshot.sh
bash /Users/xaoj/ICP/scripts/summarize_motoko_graph_monitoring_trends.sh
bash /Users/xaoj/ICP/scripts/check_motoko_graph_contract.sh
bash /Users/xaoj/ICP/scripts/check_knowledge_graph_contract.sh
bash /Users/xaoj/ICP/scripts/apply_motoko_graph_decision_event.sh --event-file <path> --dry-run
```

## Status Vocabulary

- `active`
- `draft`
- `legacy`
