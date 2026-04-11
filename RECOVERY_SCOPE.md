# Governance Sync Recovery Scope

Updated: 2026-04-10
Salvage anchor: `05362f4e`
Branch: `codex/governance-sync-recovery`

## Purpose

This branch preserves governance, research, standards, and script work that was too broad to mix into the hygiene migration. It is a recovery lane, not a merge-ready branch.

## In-Scope Material

- Governed docs under `docs/`
- Governed research and initiative surfaces under `research/`
- Standards and schemas under `shared/`
- Governance and verification scripts under `scripts/`
- Request-worktree and clean-state follow-on governance artifacts

## Out-of-Scope Material

- Generated runtime artifacts
- Frontend build outputs and local dependency trees
- Nested local mirrors or cloned repos that happen to live under `research/reference/` or other folders
- Machine-local scratch, cache, and tool-state spill

## Disposition Rules

- `authoritative_source`
  - Authored docs, standards, scripts, initiative plans, and governed evidence that belong in repo authority.
- `local_only_ignore`
  - Nested `.git` mirrors, local cache directories, and machine-specific tool state.
- `archive_only`
  - Archive-first backups and historical carry-forward snapshots kept for lineage only.
- `drop`
  - Temporary probes, duplicate scratch outputs, and non-governed spill with no steward owner.

## Next Review Splits

1. Governance contract restoration
   - Standards, contract files, verification scripts, and related tests.

2. Portfolio and initiative alignment
   - Research plans, README/DECISIONS updates, and initiative-specific evidence routing.

3. Reference-governance normalization
   - `research/reference` index/docs alignment only.
   - Nested mirrors remain `local_only_ignore` until explicitly promoted through reference intake.

## Immediate Next Action

Reduce this lane by promoting only `authoritative_source` groups into dedicated review branches and leave mirror/cache spill behind.
