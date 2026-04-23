---
id: "105-cortex-test-catalog-decisions"
name: "cortex-test-catalog-decisions"
title: "Decision Log: Cortex Test Catalog"
type: "decision"
project: "nostra"
status: completed
authors:
  - "User"
  - "Codex"
tags: [testing, cortex, ci]
created: "2026-02-08"
updated: "2026-02-08"
---

# Decision Log: Cortex Test Catalog

## DEC-001: Canonical Source of Truth
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Filesystem artifacts
2. Canister-canonical storage
3. Desktop-local state only

**Decision**: Filesystem artifacts under `/Users/xaoj/ICP/logs/testing` are canonical for v1.

**Rationale**: Lowest migration risk and immediate interoperability with IDE agents + CI + Desktop.

**Implications**: Gateway/UI become read projections; canister persistence is deferred.

## DEC-002: Initial Product Scope
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Read-only catalog and run visibility
2. Full authoring and mutation controls in Desktop

**Decision**: Read-only catalog and run visibility first.

**Rationale**: Prioritize correctness and standardization before introducing mutation UX.

**Implications**: Testing view is non-mutating in v1.

## DEC-003: CI Enforcement Strategy
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Blocking on day one
2. Advisory forever
3. Advisory then blocking by date

**Decision**: Advisory through 2026-02-22, blocking on/after 2026-02-23.

**Rationale**: Stabilizes adoption while preserving a clear hardening deadline.

**Implications**: CI job computes mode from current UTC date.

## DEC-004: Cutover Readiness Criteria and Schema Freeze
**Date**: 2026-02-08
**Status**: ✅ Decided

**Decision**:
1. Cutover criteria are locked as:
   - false-positive rate <= 2% over trailing 7 days,
   - zero schema regressions,
   - zero ambiguous release-blocker verdicts.
2. `schema_version` is frozen at `1.0.0` for v1 cutover window.

**Rationale**: Stabilizes contract behavior before switching CI to blocking mode.

**Implications**: Any schema/interface extension moves to additive `v1.1` changes.

## DEC-005: Closeout Evidence Model
**Date**: 2026-02-08
**Status**: ✅ Decided

**Decision**:
1. Require at least one valid `local_ide` run artifact and one valid `ci` run artifact for closeout.
2. Require deterministic blocking rehearsal using known-good and known-bad artifacts.
3. Use fixture-based endpoint tests to validate `/api/testing/*` payloads and structured error mapping.

**Rationale**: Provides deterministic readiness evidence without waiting for long-running canister or UI mutation flows.

**Implications**:
- Closeout can be completed with reproducible, file-backed evidence.
- Operational advisory burn-in remains ongoing via normal CI execution up to date-based cutover.
