---
id: workflow-engine-runtime-change
title: Workflow Engine Runtime Change
owner: Systems Steward
updated_at: 2026-04-09
---

# Workflow Engine Runtime Change

## Purpose
Cover changes to workflow execution semantics, adapters, and `nostra-workflow-core` integration points that affect deterministic runtime behavior.

## Triggers
- Changes to `nostra/backend/workflow_engine/**`
- Changes to `nostra/libraries/nostra-workflow-core/**`
- Changes to workflow execution adapters or orchestration behavior in runtime services

## Inputs
- Relevant workflow-engine research plan and decisions
- Updated execution semantics, adapter contracts, and declaration artifacts

## Lanes
- `declaration-change`: workflow declarations or registry-linked definitions changed.
- `engine-semantics`: execution behavior, retries, adapters, or orchestration semantics changed.
- `boundary-risk`: substrate neutrality or Nostra/Cortex authority boundaries may have drifted.

## Analysis Focus
- Declaration integrity versus runtime semantics changes.
- Adapter boundaries and deterministic execution behavior.
- Whether the workflow substrate still respects the Nostra/Cortex split.

## Steps
1. Classify whether the change affects declarations, execution semantics, or adapter boundaries.
2. Confirm the workflow substrate remains boundary-consistent with Cortex execution and Nostra authority.
3. Run the declaration-integrity path if declarations changed.
4. Record any deterministic execution or adapter fallout for runtime hosts.

## Outputs
- Workflow-engine change classification
- Required declaration/runtime follow-up evidence

## Observability
- Record whether the change was declarative, semantic, or boundary-related.
- Capture declaration-integrity or runtime-check results attached to the change.
- Note recurring execution regressions that indicate weak adapter boundaries.

## Self-Improvement
- If engine changes regularly spill into declarations, split declaration and execution workflows further.
- If deterministic execution regressions recur, add narrower checks around retries, adapters, or substrate purity.

## Required Checks
- `bash scripts/check_workflow_declarations.sh`
