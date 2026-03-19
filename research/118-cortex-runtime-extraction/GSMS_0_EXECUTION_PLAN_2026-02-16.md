# GSMS-0 Execution Plan

Date: 2026-02-16
Scope: deterministic harness only (no LLM augmentation)

## Objective

Implement and validate deterministic simulation replay in `cortex-domain` with SIQS evaluation and bench-compatible output.

## Deliverables

1. `SimulationSession`, `SimulationAction`, `SimulationResult` type surface.
2. YAML Scenario DSL parser + canonical action sequence projection.
3. Deterministic replay engine over ephemeral graph state.
4. SIQS evaluation + structural diff packaging per run.
5. Resource cap enforcement with deterministic abort reasons.
6. Canonical scenario suite files:
   - `scenarios/gsms0/constitutional_unit.yaml`
   - `scenarios/gsms0/commons_stress.yaml`
   - `scenarios/gsms0/governance_replay.yaml`

## Implementation Boundaries

1. No Chronicle writes from simulation paths.
2. No wall-clock dependency in replay/evaluation path.
3. Same seed + same scenario must produce identical `SimulationResult`.
4. Bench payload must include `violation_counts` and preserve forward-compatible placeholders.

## Verification

1. `cargo test --offline --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-domain`
2. Determinism test case: `simulation::session::tests::deterministic_session_replay_is_stable_for_same_seed`
3. DSL parse test case: `simulation::scenario::tests::parse_and_canonicalize_yaml_scenario`

## Exit

GSMS-0 exits complete when deterministic replay, SIQS packaging, and scenario DSL tests pass and evidence is captured in a dated completion artifact.
