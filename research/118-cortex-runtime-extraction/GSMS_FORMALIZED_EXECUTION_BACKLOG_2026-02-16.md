# Initiative 118 - GSMS Formalized Execution Backlog

Date: 2026-02-16
Updated: 2026-02-17
Status: partially activated (GSMS-0 implemented; GSMS-1/2 gated)

## Scope

This artifact formalizes the GSMS backlog described in `GSMS_ACTIVATION.md` into
implementation-ready, gate-ordered slices. It does not authorize execution while
cross-initiative gates remain red.

## Gate Snapshot (updated 2026-02-17)

1. Layer 0 complete: implemented and validated locally.
2. Layer 1 complete: implemented and validated locally.
3. GSMS-0 foundation complete: deterministic harness, scenario DSL, and bench contract artifacts are present.
4. 119 Phase 1-2 execution remains pending, but is no longer blocked on SIQS availability.
5. Layer 2 governance-wired enforcement remains pending downstream integration sequencing.
6. GSMS-1 and GSMS-2 remain gated until upstream initiative dependencies are explicitly closed.

GSMS-0 implementation has started and is validated under ADR-026 evidence discipline.
GSMS-1/GSMS-2 remain gated until upstream gates are explicitly closed.

## GSMS-0 (Deterministic Harness)

Deliverables:
1. `SimulationSession` core domain types in `cortex-domain`.
2. YAML Scenario DSL parser with canonical action sequence output.
3. Deterministic mutation replay engine over ephemeral graph state.
4. SIQS evaluation + structural diff packaging in session lifecycle.
5. Three canonical scenario suites:
   - constitutional unit
   - commons stress
   - governance replay
6. Resource-cap enforcement with explicit abort metadata.

Definition of done:
1. Same seed + same scenario => identical `SimulationResult` payload.
2. No wall-clock dependency in evaluation path.
3. No Chronicle event writes from simulation paths.
4. Bench contract returns at least `violation_counts`.

Implementation status (2026-02-17):
1. Implemented and locally validated under ADR-026.
2. Evidence: `GSMS_0_EXECUTION_PLAN_2026-02-16.md`, `GSMS_BENCH_MAPPING_CONTRACT_2026-02-16.md`, `GSMS_PREREQ_GATE_2026-02-16.md`.

## GSMS-1 (LLM Augmentation, future)

Deliverables:
1. Persona schema and bounded role constraints.
2. Token budget enforcement (agent + global).
3. Mapping from model output to `SimulationAction`.
4. Seeded randomness handling for reproducibility.
5. Bench metric expansion beyond violation counts.

Definition of done:
1. Budget violations hard-fail session execution.
2. Replay remains deterministic under fixed seed + fixed prompts.
3. Bench payload populates non-zero advanced metrics where available.

## GSMS-2 (Governance Staging, future)

Deliverables:
1. Production graph snapshot export contract.
2. What-if fork mode from real graph state.
3. Validated policy export to Commons artifact form.
4. Workflow Engine integration hooks.

Definition of done:
1. Snapshot replay is deterministic and isolated.
2. What-if runs never mutate production state.
3. Commons export includes version pinning + lineage refs.
4. Workflow hooks execute only after explicit governance authorization.

## Execution Policy

1. Authority mode remains recommendation-only while GSMS-1/GSMS-2 gates are red.
2. Any transition beyond GSMS-0 requires steward authorization record and updated gate evidence.
3. All GSMS phases must preserve deterministic replay constraints and no-Chronicle-write boundaries.
