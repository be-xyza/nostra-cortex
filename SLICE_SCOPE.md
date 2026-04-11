# Governance Contract Restoration Review

Updated: 2026-04-10
Base: `codex/governance-sync-recovery`

## Purpose

This branch is the first review lane for governed material preserved in the governance recovery branch. It exists to peel authoritative governance artifacts away from the larger salvage bucket.

## Primary Scope

- governed docs under `docs/`
- governed research surfaces under `research/`
- standards and schemas under `shared/`
- governance and verification scripts under `scripts/`
- `AGENTS.md` and related top-level governance documents when needed

## Exclusions

- nested local mirrors or cloned repos
- generated runtime artifacts and mutable `logs/**` outputs
- machine-local cache/tooling spill

## Review Goal

Promote authoritative governance material into bounded review slices while leaving reference mirrors, local caches, and ambiguous spill behind in the recovery lane until they are explicitly classified.
