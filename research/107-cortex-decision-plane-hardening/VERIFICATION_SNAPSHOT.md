# Verification Snapshot

## Build/Check Targets
- `cargo check` for:
  - `nostra/backend/workflow_engine`
  - `nostra/backend/governance`
  - `cortex/apps/cortex-desktop`
  - `nostra/frontend`

## Test Targets
- `cargo test` for workflow/governance canister modules.
- `cargo test` for Cortex gateway decision surface tests.
- Simulation updates under `tests/simulations/offline/*` and `tests/simulations/governance/*`.
- `scripts/check_did_declaration_sync.sh`.
- `scripts/check_test_catalog_consistency.sh --mode blocking`.

## Assertions
1. Canister-first projection succeeds or emits explicit degraded metadata.
2. Mutation-gate envelopes include deterministic status, required actions, lineage, and policy references.
3. Risky decision actions reject unauthorized or policy-blocked requests deterministically.
4. DID/declaration drift check fails fast on mismatches.
