---
id: navigation-contract-change
title: Navigation Contract Change
owner: Systems Steward
updated_at: 2026-04-09
---

# Navigation Contract Change

## Purpose
Govern changes to navigation and layout authority so hosts do not drift into local route scanning or private ordering logic.

## Triggers
- Changes to `/api/cortex/layout/spec`
- Changes to `/api/spaces/:space_id/navigation-plan`
- Changes to capability graph promotion logic or navigation slot vocabulary

## Inputs
- Navigation-slot documentation and relevant initiative plans
- Updated capability graph or layout/navigation projections

## Lanes
- `layout-spec`: `/api/cortex/layout/spec` changed.
- `navigation-plan`: route promotion, ranking, or slot mapping changed.
- `graph-promotion`: capability graph or contextual hinting changed and may affect surfacing.

## Analysis Focus
- Whether navigation authority still lives in server-backed contracts.
- Whether role visibility and drill-down semantics remain deterministic.
- Whether host-local ordering logic has leaked back in.

## Steps
1. Identify whether the change affects layout authority, ranking, or capability promotion.
2. Confirm host routing still follows server-backed contracts rather than local scanning.
3. Verify role visibility and drill-down semantics remain contract-driven.
4. Record any graph or layout contract updates required by the change.

## Outputs
- Navigation contract classification and drift result
- Required follow-up on graph/layout projections

## Observability
- Capture which navigation authority surface changed and what host impact it has.
- Record if route placement changed because of contract rules or host-local logic.
- Note recurring slot or ranking churn that suggests unstable policy.

## Improvement Loop
- Use repeated labs discovery evidence to decide whether a navigation rule belongs in canonical slot policy.
- Route slot-promotion changes through steward review rather than allowing host-local ordering fixes to accumulate.

## Required Checks
- `bash scripts/check_alignment_contract_targets.sh`
