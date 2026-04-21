# Root Untracked Recovery Triage - 2026-04-21

## Context

This evidence records the conservative follow-up after root `main` was reset to `origin/main` at `eff478c9f1480fb1f99ab2c87d5ea6d69e6be06e`.

Safety bundle:

- `/tmp/icp-root-sync-safety-20260421T152346Z/tracked_worktree.diff`
- `/tmp/icp-root-sync-safety-20260421T152346Z/untracked_overwritten_by_origin_main.tgz`
- `/tmp/icp-root-sync-safety-20260421T152346Z/untracked_remaining_before_clean.tgz`

## Promoted

The following recovered files were promoted because current repo docs or governance contracts already referenced them, or because their reference-intake records were complete enough to validate locally:

- `archive/motoko-maps-kg/spec.md`
- `docs/best-practices/general.md`
- `docs/reference/knowledge_taxonomy.toml`
- `docs/reference/topics.md`
- `research/019-nostra-log-registry/`
- `research/097-nostra-cortex-alignment/`
- `research/reference/analysis/ANALYSIS_TEMPLATE.md`
- `research/reference/analysis/larql.md`
- `research/reference/analysis/paper-2026-balakrishnan-logact.md`
- `research/reference/knowledge/PAPER_TEMPLATE.md`
- `research/reference/knowledge/agent-systems/2026_balakrishnan_logact/`
- `research/reference/repos/larql/`
- `scripts/check_reference_metadata_v2.py`
- `scripts/check_reference_taxonomy_integrity.py`

## Updated Catalogs

- `docs/reference/README.md` now reflects validator-backed local reference checks.
- `docs/_meta/catalog.toml` now registers the restored `docs/best-practices/` directory.
- `research/reference/index.toml` and `research/reference/index.md` register `larql`.
- `research/reference/knowledge/index.toml` registers the LogAct paper.
- `research/125-system-integrity-quality/VERIFY.md` no longer lists the restored reference validators as missing-script exceptions.

## Deferred

The Cortex web design-draft prototype remains deferred in the safety bundle because its untracked files were not enough for a complete integration on current `main`. The deferred paths include:

- `cortex/apps/cortex-web/src/components/a2ui/A2UISynthesisSpace.tsx`
- `cortex/apps/cortex-web/src/components/a2ui/DesignDraftPlane.tsx`
- `cortex/apps/cortex-web/src/components/a2ui/PlaygroundSurface.tsx`
- `cortex/apps/cortex-web/src/components/a2ui/designDraftCompiler.ts`
- `cortex/apps/cortex-web/tests/designDraftApiContract.test.ts`
- `cortex/apps/cortex-web/tests/designDraftCompiler.test.ts`

The `_spaces/nostra-governance-v0/` corpus, `nostra/Cargo.lock`, editor settings, and recovery-era archive snapshots also remain deferred in the safety bundle for steward review.
