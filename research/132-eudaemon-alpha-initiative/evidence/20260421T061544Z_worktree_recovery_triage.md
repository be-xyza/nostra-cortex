# Worktree Recovery Triage

Generated: 2026-04-21T06:15:44Z

## Baseline

- Canonical baseline: `origin/main` at `df8270fab4ad1c9715cbe198b26a4b99136c3d80`
- Recovery branch: `codex/worktree-recovery-conservative`
- Root snapshot: `/tmp/icp-main-root-recovery-20260421T061544Z`

## Promoted

No source files were promoted in this conservative pass.

The high-confidence candidate branches were inspected against `origin/main`; their safe source/docs content was either already present on `origin/main` or had remaining dirty deltas that would regress newer baseline behavior.

## Already Present On Origin Main

The following planned promote candidates were inspected and found to have their high-confidence source/docs content already present on `origin/main`, so no extra file promotion was needed in this branch:

- `docs-governance-sync`
- `boundary-terminology-sync`
- `execution-canvas-sync`
- `nostra-worker-sync`
- `knowledge-runbook-sync`

## Quarantined

- `knowledge-graph-phase-e-sync`: most source/docs paths matched `origin/main`; the remaining dirty differences regressed newer registry/validation behavior or referenced a missing `knowledge_graph_query_adapter` test file. Checkpoint bundle: `/tmp/icp-checkpoint-knowledge-graph-phase-e-sync-20260421T061544Z`.
- `provider-runtime-admin-sync`: most provider-admin/runtime source files matched `origin/main`; the remaining dirty differences included a large `gateway/server.rs` delta and were not promoted in this conservative pass. Checkpoint bundle: `/tmp/icp-checkpoint-provider-runtime-admin-sync-20260421T061544Z`.
- `nostra-worker-sync`: `nostra/worker/Cargo.lock` was the only absent dirty file and was not promoted because the source changes are already on `origin/main` and lockfile policy needs steward review.
- `governance-sync-recovery`: broad, noisy governance/docs/source recovery candidate; deferred for steward review. Checkpoint bundle: `/tmp/icp-checkpoint-governance-sync-recovery-20260421T061544Z`.
- `nostra-core-boundary-realign`: core boundary refactor with additions/deletions; deferred for steward review. Checkpoint bundle: `/tmp/icp-checkpoint-nostra-core-boundary-realign-20260421T061544Z`.
- `a2ui-terminal-experiment`: experiment content mixed with generated frontend output; deferred for steward decision on archive vs promotion. Checkpoint bundle: `/tmp/icp-checkpoint-a2ui-terminal-experiment-20260421T061544Z`.
- Generated-heavy recovery/review worktrees remain deferred because their filtered source signal was low relative to `dist`, `node_modules`, `.vite`, logs, or test-output churn.

## Deferred Actions

- Do not reset or fast-forward `/Users/xaoj/ICP` root until this recovery branch is reviewed.
- Do not prune stale worktrees until quarantine bundles and branch state are accepted.
- Do not commit or push this branch without steward approval.
