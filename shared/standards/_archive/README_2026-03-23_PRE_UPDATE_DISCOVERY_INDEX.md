# Shared Standards Discovery Index

This directory is the discovery entry point for governed standards in `shared/standards/`.

## Contract Classes

- `.schema.json` files: stable JSON Schema contracts
- versioned JSON files: governed artifacts and policy data
- `.toml` files: policy/configuration contracts

## Standards Groups

| Group | Canonical README | Primary Artifacts | Notes |
|---|---|---|---|
| `branding` | [branding/README.md](./branding/README.md) | `brand_policy.schema.json`, `brand_policy_v1.json`, `brand_visual_state_cases_v1.json` | Brand policy schema plus runtime policy/state fixtures. |
| `heap` | [heap/README.md](./heap/README.md) | `gate_summary_block.schema.json` | Generic structured heap block contracts. |
| `knowledge_graphs` | [knowledge_graphs/README.md](./knowledge_graphs/README.md) | `schema_registry.toml`, `ontology_manifest.schema.json`, `knowledge_bundle.schema.json`, `triple_query_request.schema.json`, `triple_query_response.schema.json` | Legacy motoko-graph operational contract plus Phase D commons layer. |
| `siq` | [siq/README.md](./siq/README.md) | `siq_governance_gate.schema.json`, `siq_graph_projection.schema.json` | Governance/execution integrity and graph-ready projection contracts. |
| `testing` | [testing/README.md](./testing/README.md) | `test_catalog.schema.json`, `test_run.schema.json`, `test_gate_summary.schema.json` | Local IDE, CI, and desktop test catalog contracts. |

## Discovery Rules

1. README files are the human entry point for each standards group.
2. `.schema.json` files are the canonical validation contracts.
3. Versioned JSON files remain discoverable artifacts, but are not schemas.
4. `knowledge_graphs/schema_registry.toml` is the canonical registry for the graph-contract package.
5. Discovery documentation must stay in sync with the actual `shared/standards/` tree.

## Validation

- `bash /Users/xaoj/ICP/scripts/check_standards_discovery_index.sh`
- `bash /Users/xaoj/ICP/scripts/check_knowledge_graph_contract.sh`
- `bash /Users/xaoj/ICP/scripts/check_dynamic_config_contract.sh`
- `bash /Users/xaoj/ICP/scripts/check_agent_preflight_contract.sh`
