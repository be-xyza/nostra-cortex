---
id: "104-blackwell-guardrails-decisions"
name: "blackwell-guardrails-decisions"
title: "Decision Log: Blackwell Guardrails"
type: "decision"
project: "nostra"
status: active
authors:
  - "User"
  - "Codex"
tags: [epistemic, governance]
created: "2026-02-07"
updated: "2026-02-08"
---

# Decision Log: Blackwell Guardrails

## DEC-001: Initiative Placement
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Extend 013 directly
2. Extend 067 directly
3. Create dedicated 104 initiative

**Decision**: Create dedicated `104-blackwell-guardrails` initiative.

**Rationale**: Keeps epistemic safety work auditable as a distinct systems track while integrating with 013/067.

## DEC-002: Rollout Mode
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Hard gate immediately
2. Observe -> soft -> hard
3. Advisory-only forever

**Decision**: Observe -> soft -> hard staged rollout.

**Rationale**: Minimizes deployment risk and supports threshold calibration.

## DEC-003: Initial Hard-Gate Scope
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. All mutations
2. Governance only
3. Governance + Merge

**Decision**: Governance + Merge for phase 1.

**Rationale**: Highest impact classes get protection first while minimizing flow disruption.

## DEC-004: Policy Authority Model
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Keep workflow engine local admin forever
2. Immediate governance-only authority
3. Explicit authority mode (`LocalAdmin` or `GovernanceCanister(principal)`)

**Decision**: Add explicit authority mode in workflow engine with controller-only authority changes.

**Rationale**: Enables safe migration to governance ownership while preserving bootstrap/admin recoverability.

## DEC-005: Governance Sync Semantics
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Implicit sync on every policy write
2. Explicit sync endpoint with retry
3. External off-chain sync worker

**Decision**: Explicit governance sync endpoint (`sync_epistemic_policy_to_workflow_engine`) with persisted target principal.

**Rationale**: Keeps control flow auditable, minimizes accidental cross-canister writes, and supports deterministic retry handling.

## DEC-006: Projection Contract Enforcement
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Keep projection surfaces best-effort only
2. Validate projection schema as part of release blocker Playwright checks
3. Validate only in manual QA

**Decision**: Enforce projection schema checks inside `check_offline_simulation_playwright.sh` and treat failures as blocking.

**Rationale**: Ensures Cortex inbox consumers only receive well-formed `RenderSurface` payloads and prevents silent drift.

## DEC-007: Projection Polling and Degraded Mode
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Always-on polling with no control flag
2. Build-time flag to disable projection polling during incidents
3. Remove polling entirely outside tests

**Decision**: Keep polling enabled by default with `NOSTRA_ENABLE_TEST_PROJECTION_POLLING`, plus explicit degraded-mode test coverage for missing projection files.

**Rationale**: Preserves operator visibility while allowing safe incident mitigation and proving UI resilience under missing-data conditions.
