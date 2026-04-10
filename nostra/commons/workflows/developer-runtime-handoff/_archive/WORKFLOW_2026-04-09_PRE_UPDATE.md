---
id: developer-runtime-handoff
title: Developer To Runtime Handoff
owner: Systems Steward
updated_at: 2026-04-09
---

# Developer To Runtime Handoff

## Purpose
Provide a canonical handoff path from IDE-based developer-agent work into runtime-facing validation and workflow execution surfaces without blurring the developer/runtime authority boundary.

## Triggers
- Changes to `cortex/**`, `nostra/**`, `shared/**`, or `scripts/**` that affect execution, transport, workflow declarations, or runtime authority contracts.
- Work that changes A2UI contracts, gateway routes, workflow execution semantics, or runtime operator surfaces.

## Inputs
- The active research plan for the initiative in scope.
- Preflight evidence from `nostra-cortex-dev-core`.
- Relevant runtime or workflow contract checks for the changed surface.

## Lanes
- `developer-local`: code and contract changes that remain inside IDE/operator authority.
- `runtime-impact`: changes that alter runtime behavior, host parity, or workflow execution semantics.
- `operator-review`: changes that touch operator-only surfaces or deployment authority.

## Analysis Focus
- Classify the touched surface by authority boundary.
- Determine which runtime/workflow evidence bundle is minimally sufficient.
- Identify whether host parity, operator review, or workflow declaration follow-up is required.

## Steps
1. Run developer-agent preflight and confirm the boundary contract still holds.
2. Classify whether the change affects:
   - runtime authority surfaces
   - workflow execution semantics
   - web/desktop host parity
   - provider-runtime or operator-only routes
3. Execute the narrowest contract checks that match the change surface.
4. Record whether the change remains developer-local or requires explicit runtime/operator review.
5. If runtime-facing behavior changed, attach the runtime evidence bundle to the closeout.

## Outputs
- A closeout that includes preflight evidence plus the relevant runtime/workflow evidence.
- Clear classification of whether the change stays in developer authority or requires runtime/operator follow-up.

## Observability
- Record the chosen lane in the closeout.
- Capture which contract checks were run and which evidence bundle was attached.
- Note repeated escalation patterns that indicate a missing dedicated workflow or guardrail.

## Self-Improvement
- If the same runtime-facing change class repeatedly routes through this workflow, spin out a narrower canonical workflow.
- If required evidence is routinely ambiguous, tighten the lane definitions or add explicit trigger examples.

## Required Checks
- `bash scripts/check_agent_preflight_contract.sh`
- `bash scripts/check_dynamic_config_contract.sh`
- `bash scripts/check_alignment_contract_targets.sh`

## Runtime Follow-Up
- For workflow declaration or execution changes, run the workflow-specific verification path.
- For gateway or provider-runtime changes, run the parity and authority checks that govern those surfaces.
