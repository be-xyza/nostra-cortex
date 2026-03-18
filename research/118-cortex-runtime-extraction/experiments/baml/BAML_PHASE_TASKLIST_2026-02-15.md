# BAML Pattern Extraction Tasklist (Phase-Mapped)

Date: 2026-02-15  
Initiative: `research/118-cortex-runtime-extraction`

## Intent
Translate the BAML pattern findings into concrete extraction tasks aligned with Initiative 118 phase boundaries.

## Phase 3 Tasks (Governance, Workflow, Streaming)
1. Add a runtime policy surface for external operation strategy selection:
   - `single`, `fallback`, `round_robin`.
2. Introduce typed retry policy model for runtime execution:
   - `max_retries` + `constant_delay` / `exponential_backoff`.
3. Bind policy model to `AsyncExternalOp` orchestration path (runtime, not platform schema).
4. Emit structured attempt telemetry events:
   - `provider`, `attempt`, `outcome`, `backoff_ms`, `fallback_transition`.
5. Preserve deterministic replay metadata for provider selection decisions.

## Phase 4 Tasks (Agent, ViewSpec, UX)
1. Add typed function contract envelope for agent-facing operations:
   - function id, input schema ref, output schema ref.
2. Surface partial-vs-final output semantics in UI projection contracts.
3. Add deterministic provider-attempt projections to UX evidence panes for operator review.
4. Define policy visualization mapping in decision surfaces:
   - strategy mode, retry profile, last fallback reason.

## Current Implementation Stub
- Feature-gated prototype landed in:
  - `cortex/libraries/cortex-runtime/src/policy_experiments.rs`
  - feature: `baml-policy-experiments` in `cortex/libraries/cortex-runtime/Cargo.toml`

## Verification Gates
1. `cargo test --manifest-path cortex/libraries/cortex-runtime/Cargo.toml --features baml-policy-experiments`
2. Validate no impact on default build (`default = []` remains unchanged).
3. Confirm phase tasks stay within Runtime Purity Contract boundaries (no forbidden APIs or deps introduced).
