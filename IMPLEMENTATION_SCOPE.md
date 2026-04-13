# Cortex Web Heap Shell Implementation

Updated: 2026-04-10
Base: `origin/main`
Restore source: `codex/execution-canvas-runtime-recovery`
Planning reference: `codex/cortex-web-heap-shell-review`

## Purpose

This branch is the clean implementation lane for web workbench shell, heap presentation, and related UI-layer changes after the hygiene merge.

## Owned Scope

- `cortex/apps/cortex-web/**`
- `cortex/apps/cortex-desktop/src/services/heap_mapper.rs`
- `cortex/apps/cortex-eudaemon/src/services/heap_mapper.rs`
- `cortex/apps/cortex-eudaemon/src/workflows/spatial_synthesis.rs`

## Restore Rule

Restore only the shell and heap slice. Provider-runtime backend changes, Nostra extraction churn, generated artifacts, and local logs are excluded.

## Immediate Next Step

Restore the owned UI-layer paths from `codex/execution-canvas-runtime-recovery`, validate `cortex-web` install/build behavior, and open a dedicated UI recovery PR from this branch.

## Validation Note

Validated against merged `origin/main` on 2026-04-11.

- restored only authored UI-layer source, test, and config surfaces from `codex/execution-canvas-runtime-recovery`
- removed a host-specific Rollup binary dependency drift from `package.json` and `package-lock.json`
- `npm install` and `npm run build` passed in `cortex/apps/cortex-web`
- targeted tests passed:
  - `tests/capabilityGraphApiContract.test.ts`
  - `tests/heapReviewQueueContract.test.ts`
- generated `dist/` and `node_modules/` outputs remained ignored and `check_tracked_generated_artifacts.sh` stayed green
