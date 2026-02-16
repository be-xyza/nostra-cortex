# Phase 2 Entry Packet (Prepared, Not Executed)

## Purpose
This packet defines readiness requirements to start Initiative 118 Phase 2.
It does not authorize or execute Phase 2 extraction.

## Phase 2 Entry Checklist

All items must be true before kickoff:

- [ ] `cortex-runtime-freeze-gates` CI job is green on latest `main`
- [ ] `cortex-runtime-freeze-gates` CI job is green on latest PR candidate
- [ ] No parity inventory gaps (`endpoint_inventory.tsv` == `parity_cases/*.json` + exemptions)
- [ ] `approved_exemptions.json` remains `{"exemptions": []}` unless explicitly approved by ADR
- [ ] No unresolved ACP shadow mismatches in freeze-gate logs
- [ ] `PHASE_0_1_CLOSURE_EVIDENCE_2026-02-15.md` remains valid and reproducible

## Candidate Extraction Order (Phase 2)

Mapped to current crate/module boundaries and pre-Phase-2 constraints.

1. `acp_meta_policy.rs`
- Destination: `cortex-domain/src/policy/meta.rs`
- Why first: pure policy validation, lowest coupling, zero host IO.

2. `artifact_collab_crdt.rs`
- Destination: `cortex-domain/src/collaboration/crdt.rs`
- Constraint: inject time inputs; no wall-clock calls.

3. `acp_adapter.rs` split
- Runtime trait surface: `cortex-runtime/src/policy/adapter.rs`
- Host implementation stays in desktop services.

4. `acp_session_store.rs`
- Destination: `cortex-runtime/src/policy/sessions.rs`
- Constraint: storage through `StorageAdapter` only.

5. `acp_permission_ledger.rs`
- Destination: `cortex-runtime/src/policy/permissions.rs`
- Constraint: deterministic ordering in externally visible outputs.

6. `acp_metrics.rs`
- Destination: `cortex-runtime/src/policy/metrics.rs`
- Constraint: remove OnceLock/env dependencies from extracted paths.

7. `acp_protocol.rs` split (last in Phase 2)
- Domain protocol types: `cortex-domain/src/policy/types.rs`
- Runtime orchestrator: `cortex-runtime/src/policy/protocol.rs`
- Host bridge remains in desktop until parity proves complete.

## Rollback Note for Phase 2 First PR

Rollback strategy must be included in the first Phase 2 PR description:

- Keep legacy ACP path as default authority.
- Keep `cortex_runtime_v0` feature flag default-off.
- Keep `CORTEX_RUNTIME_V0` / `CORTEX_RUNTIME_SHADOW` toggles functional.
- If regressions occur:
  - disable `CORTEX_RUNTIME_V0`
  - preserve legacy behavior
  - retain failing fixtures/tests as evidence
- Required tests in first PR:
  - `gateway_parity`
  - ACP update-kind parity matrix
  - ACP shadow strictness tests
  - domain/runtime purity scripts
  - wasm checks for `cortex-domain` and `cortex-runtime --no-default-features`
