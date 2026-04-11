# Governance Restoration Implementation

Updated: 2026-04-10
Base: `origin/main`
Restore source: `codex/governance-sync-recovery`
Planning reference: `codex/governance-contract-restoration-review`

## Purpose

This branch is the clean implementation lane for governed docs, research, standards, and verification surfaces that must be rescued from the governance recovery anchor.

## Owned Scope

- governed docs under `docs/`
- governed research surfaces under `research/`
- standards and schemas under `shared/`
- governance and verification scripts under `scripts/`
- `AGENTS.md` and related top-level governance surfaces when needed

## Restore Rule

Restore only authoritative governance material. Nested mirrors, local caches, reference clones, generated runtime artifacts, and mutable `logs/**` outputs remain excluded.

## Immediate Next Step

Promote the first bounded governance subset from `codex/governance-sync-recovery`, keeping reference-mirror spill and local-only material out of this branch.
