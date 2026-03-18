# Initiative 118 - GSMS Formalized Execution Backlog

Date: 2026-02-16
Status: gated (no implementation PR authorized)

## Scope

This artifact formalizes the GSMS backlog described in `GSMS_ACTIVATION.md` into
implementation-ready, gate-ordered slices. It does not authorize execution while
cross-initiative gates remain red.

## Gate Snapshot (as of 2026-02-16)

1. Layer 0 complete: designed, not built.
2. Layer 1 complete: designed, not built.
3. 119 Phase 1-2 complete: gated on Layer 1.
4. Layer 2 complete: gated on 119.
5. Layer 3 (GSMS): gated on Layer 2.

No GSMS phase may start until upstream gates are explicitly closed.

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

1. Authority mode remains recommendation-only while GSMS gates are red.
2. Any transition from design to implementation requires steward authorization record.
3. When gates become green, create a dedicated GSMS execution plan artifact before coding.
