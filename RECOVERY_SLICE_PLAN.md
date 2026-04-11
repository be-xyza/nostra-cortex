# Execution Canvas Runtime Recovery Slice Plan

Updated: 2026-04-10
Salvage anchor: `b71e1b26`
Branch: `codex/execution-canvas-runtime-recovery`

## Purpose

This branch is a salvage anchor for authored runtime work that was externalized from the root worktree during the clean-state migration. It is not a merge-ready PR branch.

## Reviewable Slice Order

1. Provider runtime and gateway boundary changes
   - Scope: runtime/provider selection, gateway boundary enforcement, execution-host coupling, auth/runtime operator surfaces.
   - Exit condition: changes are isolated from unrelated UI and declaration churn.

2. `cortex-web` heap and shell changes
   - Scope: heap-mode shell, execution workbench surfaces, route/surface integration, UI shell fixes.
   - Exit condition: web-facing runtime changes can be reviewed without backend/runtime diff noise.

3. Nostra/runtime extraction and declaration churn
   - Scope: declaration sync, shared type movement, extraction cleanup, generated-binding alignment that remains authored.
   - Exit condition: declaration and extraction changes can be reviewed as a bounded contract-maintenance slice.

## Operating Rules

- Do not merge this salvage anchor directly.
- Each review slice should be created from this branch into a dedicated request worktree.
- Keep generated artifacts out of Git authority.
- Use `bash scripts/checkpoint_request.sh` before any handoff or context switch.

## Immediate Next Action

Create three child branches from this salvage anchor, one per slice above, and move only the relevant authored files into each review branch.
