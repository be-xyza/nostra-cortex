# Shared Standards Discovery Index

This directory is the discovery entry point for governed standards in `shared/standards/`.

## Canonical Registry

- [standards_registry.toml](./standards_registry.toml): root discovery registry for `shared/standards/`
- [schema_guided_extraction_context.json](./schema_guided_extraction_context.json): extraction-oriented mapping from artifact families to schema ids

## Contract Classes

- `.schema.json` files: stable JSON Schema contracts
- versioned JSON files: governed artifacts and policy data
- `.toml` files: policy/configuration contracts

## Directory Map

| Path | Entry Point | Purpose |
|---|---|---|
| `./` | [README.md](./README.md) | Root discovery index for all governed standards. |
| `./branding/` | [branding/README.md](./branding/README.md) | Branding policy schema plus governed visual-state fixtures. |
| `./heap/` | [heap/README.md](./heap/README.md) | Structured heap block contracts. |
| `./knowledge_graphs/` | [knowledge_graphs/README.md](./knowledge_graphs/README.md) | Legacy motoko-graph artifacts and Phase D graph commons contracts. |
| `./siq/` | [siq/README.md](./siq/README.md) | SIQ governance and graph projection contracts. |
| `./testing/` | [testing/README.md](./testing/README.md) | Test catalog, run, and gate-summary contracts. |

## Core Standards Documents

| Document | Purpose |
|---|---|
| [ACCESSIBILITY.md](./ACCESSIBILITY.md) | Accessibility and inclusive UI guidance. |
| [LOCAL_FIRST.md](./LOCAL_FIRST.md) | Local-first execution and offline resilience guidance. |
| [ROLE_SEMANTICS.md](./ROLE_SEMANTICS.md) | Steward, reviewer, signer, and compiler role semantics. |
| [SEMANTIC_PRIMITIVES.md](./SEMANTIC_PRIMITIVES.md) | Two-axis naming, reservation, and promotion rules for high-signal primitives. |
| [TECHNOLOGY_NEUTRALITY.md](./TECHNOLOGY_NEUTRALITY.md) | Host-neutral architecture and adapter boundaries. |

## Root Contracts

| Contract | Type | Notes |
|---|---|---|
| [agent_failure_modes.toml](./agent_failure_modes.toml) | `.toml` | Failure-mode registry for agent execution and recovery behavior. |
| [agent_preflight_contract.toml](./agent_preflight_contract.toml) | `.toml` | Preflight contract for agent readiness and governance checks. |
| [alignment_contracts.toml](./alignment_contracts.toml) | `.toml` | Alignment contract for standards and hardcoded-path enforcement. |
| [skill_policy.toml](./skill_policy.toml) | `.toml` | Repo-managed skill policy for replacement and governance handling. |
| [dynamic_source_contract.toml](./dynamic_source_contract.toml) | `.toml` | Dynamic-source contract for governed runtime inputs. |
| [semantic_primitives_registry.toml](./semantic_primitives_registry.toml) | `.toml` | Registry for governed high-signal semantic primitives and reserved terms. |
| [alignment_contract_exceptions.json](./alignment_contract_exceptions.json) | versioned JSON | Exception registry for alignment contract hardcoded-path and parity checks. |
| [skill_policy_dispositions.json](./skill_policy_dispositions.json) | versioned JSON | Disposition registry for repo-managed skill policy handling. |
| [dynamic_source_exceptions.json](./dynamic_source_exceptions.json) | versioned JSON | Exception registry for dynamic-source contract bypass allowances. |

## Standards Groups

| Group | Canonical README | Primary Artifacts | Notes |
|---|---|---|---|
| `branding` | [branding/README.md](./branding/README.md) | [brand_policy.schema.json](./branding/brand_policy.schema.json), [brand_policy_v1.json](./branding/brand_policy_v1.json), [brand_visual_state_cases_v1.json](./branding/brand_visual_state_cases_v1.json) | Brand policy schema plus runtime policy/state fixtures. |
| `heap` | [heap/README.md](./heap/README.md) | `gate_summary_block.schema.json` | Generic structured heap block contracts. |
| `knowledge_graphs` | [knowledge_graphs/README.md](./knowledge_graphs/README.md) | [schema_registry.toml](./knowledge_graphs/schema_registry.toml), [ontology_manifest.schema.json](./knowledge_graphs/ontology_manifest.schema.json), [knowledge_bundle.schema.json](./knowledge_graphs/knowledge_bundle.schema.json), [triple_query_request.schema.json](./knowledge_graphs/triple_query_request.schema.json), [triple_query_response.schema.json](./knowledge_graphs/triple_query_response.schema.json) | Legacy motoko-graph operational contract plus Phase D commons layer. |
| `siq` | [siq/README.md](./siq/README.md) | `siq_governance_gate.schema.json`, `siq_graph_projection.schema.json` | Governance/execution integrity and graph-ready projection contracts. |
| `testing` | [testing/README.md](./testing/README.md) | `test_catalog.schema.json`, `test_run.schema.json`, `test_gate_summary.schema.json` | Local IDE, CI, and desktop test catalog contracts. |

## Discovery Rules

1. Core standards documents are the human entry point for shared governance guidance.
2. Root contract files are the authoritative policy/config and governed-artifact surfaces.
3. README files are the human entry point for each standards group.
4. `.schema.json` files are the canonical validation contracts.
5. Versioned JSON files remain discoverable artifacts, but are not schemas.
6. `knowledge_graphs/schema_registry.toml` is the canonical registry for the graph-contract package.
7. `standards_registry.toml` is the cross-standards discovery index and may delegate package-specific registries.
8. `schema_guided_extraction_context.json` maps extraction-oriented artifact families to the schema ids that govern them.
9. Discovery documentation must stay in sync with the actual `shared/standards/` tree.

## Validation

- `bash /Users/xaoj/ICP/scripts/check_standards_discovery_index.sh`
- `bash /Users/xaoj/ICP/scripts/check_knowledge_graph_contract.sh`
- `bash /Users/xaoj/ICP/scripts/check_dynamic_config_contract.sh`
- `bash /Users/xaoj/ICP/scripts/check_agent_preflight_contract.sh`
- `bash /Users/xaoj/ICP/scripts/check_alignment_contract_targets.sh`
- `bash /Users/xaoj/ICP/scripts/run_repo_python.sh scripts/check_semantic_primitives.py --strict`
