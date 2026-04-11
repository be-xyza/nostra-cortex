---
id: workflow-declaration-change
title: Workflow Declaration Change
owner: Systems Steward
updated_at: 2026-04-09
---

# Workflow Declaration Change

## Purpose
Govern canonical repo-managed workflow definitions, install parity across IDEs, and declaration integrity.

## Triggers
- Edits under `nostra/commons/workflows`
- Changes to `.codex/workflows` or `.claude/workflows`
- Changes to workflow registry or sync/check tooling

## Inputs
- Updated workflow registry entries
- Updated workflow assets and install roots

## Lanes
- `registry-change`: registry rows, targets, or source paths changed.
- `asset-change`: workflow body or required sections changed.
- `install-parity`: IDE install roots or sync behavior changed.

## Analysis Focus
- Registry validity, source-path correctness, and install parity across IDEs.
- Whether workflow assets remain structurally complete and semantically scoped.
- Whether a change is additive, deprecating, or behavior-shifting.

## Steps
1. Validate the workflow registry schema and source paths.
2. Confirm every repo-managed workflow validates semantically.
3. Confirm install parity across IDE roots.
4. Record any intended additions, removals, or deprecations in the workflow set.

## Outputs
- Workflow registry integrity result
- Install parity result across supported IDEs

## Observability
- Capture validation, drift, and install outcomes for each targeted IDE.
- Record workflow additions, removals, and deprecations in the change summary.
- Note recurring sync failures or validator misses as control-plane regressions.

## Self-Improvement
- If workflow drift keeps recurring, strengthen sync or validator coverage.
- If many workflow edits are metadata-only, consider a thinner registry update path.

## Required Checks
- `bash scripts/check_workflow_declarations.sh`
