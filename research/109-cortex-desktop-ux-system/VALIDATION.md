# Validation: 109 Cortex Desktop UX System

## Baseline Freeze (Phase 0)
Baseline references captured from:
- `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/router.rs`
- `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/components/layout.rs`

### Baseline Route Surface
- Core: `/`, `/canisters`, `/system`, `/workflows`, `/testing`, `/console`
- Additional: `/sandbox`, `/schema`, `/settings`, `/kg/motoko-graph`, `/a2ui`
- Added in this phase: `/studio`, `/artifacts`

### Baseline Shell Composition
- Previous: hardcoded sidebar buttons.
- Current phase: manifest-driven navigation with route adapter and role filtering.

## Test Scenarios
1. Structural evaluation without approval metadata returns `blocked_hitl_required`.
2. Structural evaluation with approval metadata returns `eligible_hitl_approved` and emits promotion decision event.
3. Failing hard gate returns `blocked_auto_gate`.
4. Feedback endpoint persists event and returns explicit acknowledgement.
5. Layout spec endpoint includes Studio/Artifacts entries.
6. Capability matrix endpoint returns rubric-driven rows.
7. Artifacts bridge enforces role-based access denial when role is insufficient.

## Evidence Targets
- Desktop tests for new gateway contract handlers.
- Desktop `cargo check` green.
- Optional frontend contract consumer compile check.

## Exit Criteria
- APIs are additive and pass regression checks.
- Bridge lanes are visible and governed.
- HITL policy is enforced and test-covered.
