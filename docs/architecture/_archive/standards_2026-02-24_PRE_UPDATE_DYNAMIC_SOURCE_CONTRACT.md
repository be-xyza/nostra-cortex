# Architecture Standards (Modularity and Confidence)

This document is the canonical index for architecture-level standards referenced by governance checks and portfolio contracts.

## Core Contract
1. **Boundary first**: Nostra defines what exists, Cortex defines how work runs.
2. **Host neutrality**: runtime logic is substrate/host agnostic; hosts are adapters.
3. **Deterministic projections**: derived artifacts must be reproducible from authority sources.
4. **Steward-gated structural change**: sensitive mutations require explicit approval and lineage.
5. **Parity over duplication**: shared contracts and runtime logic, host-specific rendering only.

## Source Standards
1. `/Users/xaoj/ICP/docs/architecture/nostra-cortex-boundary.md`
2. `/Users/xaoj/ICP/shared/standards/TECHNOLOGY_NEUTRALITY.md`
3. `/Users/xaoj/ICP/shared/standards/LOCAL_FIRST.md`
4. `/Users/xaoj/ICP/shared/standards/ROLE_SEMANTICS.md`
5. `/Users/xaoj/ICP/shared/specs.md`

## Enforcement Hooks
1. `/Users/xaoj/ICP/shared/standards/alignment_contracts.toml`
2. `/Users/xaoj/ICP/scripts/check_nostra_cortex_terminology.sh`
3. `/Users/xaoj/ICP/scripts/check_gateway_parity_inventory_sync.sh`
4. `/Users/xaoj/ICP/scripts/check_gateway_protocol_contract_coverage.sh`
5. `/Users/xaoj/ICP/scripts/check_initiative_graph_bootstrap.sh`
6. `/Users/xaoj/ICP/scripts/run_siq_checks.sh`
7. `/Users/xaoj/ICP/shared/standards/siq/siq_graph_projection.schema.json`
8. `/Users/xaoj/ICP/scripts/check_alignment_contract_targets.sh`
9. `/Users/xaoj/ICP/scripts/check_workspace_merge_integrity.sh`
10. `/Users/xaoj/ICP/scripts/check_no_hardcoded_workspace_paths.sh`
