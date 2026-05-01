# Architecture Standards (Modularity and Confidence)

This document is the canonical index for architecture-level standards referenced by governance checks and portfolio contracts.

## Core Contract
1. **Boundary first**: Nostra defines what exists, Cortex defines how work runs.
2. **Host neutrality**: runtime logic is substrate/host agnostic; hosts are adapters.
3. **Deterministic projections**: derived artifacts must be reproducible from authority sources.
4. **Steward-gated structural change**: sensitive mutations require explicit approval and lineage.
5. **Parity over duplication**: shared contracts and runtime logic, host-specific rendering only.
6. **Preventative correction when drift is known**: when semantics are already known to be misleading and the migration surface is still small, correct the primitive before drift compounds.

## Source Standards
1. `/Users/xaoj/ICP/docs/architecture/nostra-cortex-boundary.md`
2. `/Users/xaoj/ICP/docs/architecture/repo-clean-state.md`
3. `/Users/xaoj/ICP/shared/standards/TECHNOLOGY_NEUTRALITY.md`
4. `/Users/xaoj/ICP/shared/standards/LOCAL_FIRST.md`
5. `/Users/xaoj/ICP/shared/standards/ROLE_SEMANTICS.md`
6. `/Users/xaoj/ICP/shared/standards/SEMANTIC_PRIMITIVES.md`
7. `/Users/xaoj/ICP/docs/architecture/promotion-migration-rubric.md`
8. `/Users/xaoj/ICP/shared/specs.md`
9. `/Users/xaoj/ICP/shared/standards/agent_preflight_contract.toml`
10. `/Users/xaoj/ICP/shared/standards/dynamic_source_contract.toml`
11. `/Users/xaoj/ICP/shared/standards/agent_failure_modes.toml`
12. `/Users/xaoj/ICP/nostra/commons/skills/registry.toml`
13. `/Users/xaoj/ICP/nostra/commons/workflows/registry.toml`
14. `/Users/xaoj/ICP/shared/standards/skill_policy.toml`
15. `/Users/xaoj/ICP/shared/standards/skill_policy_dispositions.json`
16. `/Users/xaoj/ICP/shared/standards/cortex_runtime_authority.schema.json`
17. `/Users/xaoj/ICP/research/120-nostra-design-language/schemas/SpaceDesignProfileV1.schema.json`
18. `/Users/xaoj/ICP/research/120-nostra-design-language/schemas/DesignElementImportV1.schema.json`
19. `/Users/xaoj/ICP/research/120-nostra-design-language/schemas/SpaceTemplatePackV1.schema.json`

## Enforcement Hooks
1. `/Users/xaoj/ICP/shared/standards/alignment_contracts.toml`
2. `/Users/xaoj/ICP/scripts/check_nostra_cortex_terminology.sh`
3. `/Users/xaoj/ICP/scripts/check_gateway_parity_inventory_sync.sh`
4. `/Users/xaoj/ICP/scripts/check_gateway_protocol_contract_coverage.sh`
5. `/Users/xaoj/ICP/scripts/check_contribution_graph_bootstrap.sh`
6. `/Users/xaoj/ICP/scripts/run_siq_checks.sh`
7. `/Users/xaoj/ICP/shared/standards/siq/siq_graph_projection.schema.json`
8. `/Users/xaoj/ICP/scripts/check_alignment_contract_targets.sh`
9. `/Users/xaoj/ICP/scripts/check_workspace_merge_integrity.sh`
10. `/Users/xaoj/ICP/scripts/check_no_hardcoded_workspace_paths.sh`
11. `/Users/xaoj/ICP/scripts/check_agent_preflight_contract.sh`
12. `/Users/xaoj/ICP/scripts/check_dynamic_config_contract.sh`
13. `/Users/xaoj/ICP/scripts/check_skill_registry_integrity.sh`
14. `/Users/xaoj/ICP/scripts/check_skill_policy.sh`
15. `/Users/xaoj/ICP/scripts/check_workflow_declarations.sh`
16. `/Users/xaoj/ICP/scripts/check_semantic_primitives.py`
17. `/Users/xaoj/ICP/scripts/check_semantic_alignment_surfaces.py`
18. `/Users/xaoj/ICP/scripts/check_role_semantics.py`
19. `/Users/xaoj/ICP/scripts/check_knowledge_graph_contract.sh`
20. `/Users/xaoj/ICP/scripts/check_standards_discovery_index.sh`
21. `/Users/xaoj/ICP/scripts/check_vps_runtime_authority.sh`
22. `/Users/xaoj/ICP/scripts/check_clean_worktree.sh`
23. `/Users/xaoj/ICP/scripts/check_tracked_generated_artifacts.sh`
24. `/Users/xaoj/ICP/scripts/check_ndl_design_profiles.py`
