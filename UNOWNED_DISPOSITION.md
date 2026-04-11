# Unowned Root Triage Disposition

Updated: 2026-04-10
Anchor commit: `87365e25`
Branch: `codex/unowned-root-triage`

## Purpose

This branch is a checkpoint-only triage lane for material that was removed from the root worktree but does not yet have a clear stewarded destination.

## Disposition Matrix

### `authoritative_source`

Use this disposition for authored repo material that should be routed into a real recovery branch after owner review.

- `ops/hetzner/icp/**`
- `tests/**`
- `WORKSPACE.md`
- `DECISIONS.md`
- `package.json`
- `package-lock.json`
- `Cargo.lock`
- `sdk/**`
- `ic-rmcp/**`
- `apps/**`
- `canisters/**`
- `tools/**`
- `KIP/**`
- `HRM/**`

### `local_only_ignore`

Use this disposition for machine-local state that should become ignore policy rather than Git history.

- `.playwright-cli/**`
- `.trae/**`
- `.shared/**`
- `.ignore`
- `.nvmrc`
- `worker_keys.json`
- `tmp_exec_probe.txt`

### `archive_only`

Use this disposition for historical carry-forward material that may be useful for lineage but is not active repo authority.

- `_archive/**`
- `receipts/**`
- `research_prompt_20260125_065309.txt`
- `hyeve.pdf`

### `drop`

Use this disposition only after steward review confirms the material is duplicate scratch or superseded local spill.

- ad hoc one-off helper outputs with no active owner
- duplicate local mirrors that are already preserved elsewhere

## Guardrails

- Do not commit unclassified files from this branch.
- Promote `authoritative_source` items into a dedicated request worktree before review.
- Convert repeated `local_only_ignore` patterns into root ignore policy only after they are confirmed not to be governed assets.

## Immediate Next Action

Walk the remaining untracked top-level paths against this matrix and move every `authoritative_source` group out of this lane before any cleanup or deletion.
