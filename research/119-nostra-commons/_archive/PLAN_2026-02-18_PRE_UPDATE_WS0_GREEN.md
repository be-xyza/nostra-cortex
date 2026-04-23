---
id: "119"
name: "nostra-commons"
title: "Nostra Commons Implementation Plan"
type: "plan"
project: "nostra"
status: active
execution_plane: "nostra"
authority_mode: "recommendation_only"
reference_topics:
  - "governance"
  - "institutions"
  - "constitutional-framework"
reference_assets:
  - "shared/specs.md"
  - "nostra/backend/modules/institution.mo"
  - "nostra/backend/modules/governance.mo"
evidence_strength: "validated"
handoff_target:
  - "094-institution-modeling"
  - "013-nostra-workflow-engine"
authors:
  - "X"
tags:
  - "commons"
  - "governance"
  - "institution"
  - "constitutional"
  - "automation"
stewardship:
  layer: "Architectural"
  primary_steward: "Economic/Governance Steward"
  domain: "Governance & Economics"
created: "2026-02-14"
updated: "2026-02-18"
---

# Nostra Commons Implementation Plan

## Overview

A Nostra Commons is a portable constitutional bundle (institution + SIQS ruleset + governance/automation policy). Commons remains an Institution usage pattern (`scope = "commons"`) and does not add a new ContributionType.

## Current Phase Status (2026-02-18)

| Phase | Status | Notes |
|---|---|---|
| Phase 0: Semantic Foundation | ✅ Complete | Commons APIs and adoption edge model shipped. |
| Phase 1: Ruleset Schema | ✅ Complete (closure tranche) | Ruleset normalization, semver, templates, authority mode shipped. |
| Phase 2: Enforcement Integration | ✅ Complete (closure tranche) | Deterministic multi-commons aggregation + compliance query + UI read panel shipped. |
| Phase 3: Upgrade Protocol | 🟡 In Progress (gated) | Deterministic notice state transitions and acknowledgment shipped; final cross-canister event pipeline and steward UX remain. |
| Phase 4: Automation Hooks | 🟡 In Progress (gated by 013) | Workflow-engine canister hook + manual trigger shipped behind policy gate; governance bridge sequencing remains. |
| Phase 5: Canonical Commons v1 | 🟡 In Progress | `publishNostraCoreCommonsV1` + optional auto-adopt on space create shipped; final rollout/onboarding sign-off remains. |
| Phase 6: Simulation Validation Gate | 🟡 In Progress (gated) | Policy/report/ratification gate shipped; GSMS replay-coupled governance evidence path remains. |

## Dependency Status

| Dependency | Initiative | Status | Blocks |
|---|---|---|---|
| Institution module | 094 | ✅ complete | none |
| SIQS Engine | 118 Layer 1 | ✅ available | none |
| GlobalEvent Motoko pipeline | shared/specs.md | 🔴 partial | Phase 3 completion |
| Workflow ↔ Governance integration | 013 | 🔴 pending | Phase 4 completion |
| GSMS full integration | 118 Layer 3+ | 🟡 prerequisite active, full coupling pending | Phase 6 completion |

## Delivered in Closeout Tranche

### Phase 1-2 closure
- [x] `getCommonsForSpace` recognizes adoption edges and returns deterministic ordering.
- [x] Ruleset persistence hardening:
  - semver check (`major.minor.patch`)
  - duplicate `rule.id` rejection
  - deterministic rule sort by `rule.id`
  - endpoint argument `commonsId` is canonical
- [x] Rule template registry and query API.
- [x] Authority dual path:
  - admin-only path retained
  - proposal-linked upsert path enforced when mode=`proposalRequired`
- [x] Compliance query surface (`getCommonsComplianceForSpace`) with bounded violations.
- [x] Deterministic namespaced rule IDs (`{commonsId}::{ruleId}`).
- [x] Admin ingest enforcement toggle (`enforceCommonsOnAdminIngest`) applied to batch/library ingest paths.
- [x] Frontend read parity (types/api + institutions panel + compliance summary).

### Phase 3/4/6 scaffolding (gated)
- [x] Persistent `CommonsAdoptionState` map with migration fallback from legacy edges.
- [x] Upgrade notice/history scan API and global event emission surface (`commonsUpgradeAvailable`).
- [x] Automation policy + manual automation trigger + last-run surface.
- [x] Simulation gate policy + simulation report attachment/listing + proposal approval gate validator.

### Final closeout execution pass (2026-02-18)
- [x] Added upgrade notice acknowledgment API (`acknowledgeCommonsUpgradeNotice`).
- [x] Added workflow-engine canister configuration APIs (`set/getWorkflowEngineCanisterId`).
- [x] Added canonical commons v1 publish/read APIs (`publishNostraCoreCommonsV1`, `getNostraCoreCommonsV1`).
- [x] Added auto-adopt toggle for new spaces (`set/getAutoAdoptCoreCommonsV1OnSpaceCreate`).
- [x] Added workflow-linked automation run tracking (`workflowInstanceId` in automation run).
- [x] Synced backend DID and declarations from generated service DID.
- [x] Extended web institutions read UI with deterministic upgrade notice status panel.

## Remaining Work to Close Initiative

### WS-0 cargo closure blocker
- [x] Remove unused `ic-agent` dependency from `nostra-test-kit` to eliminate yanked `keccak` resolution path.
- [ ] Resolve remaining frontend workspace compile blockers (missing crate/module parity and API mismatches) in CI-connected environment.
- [ ] Regenerate lockfile in networked CI and re-run `cargo check --manifest-path /Users/xaoj/ICP/nostra/Cargo.toml -p frontend`.

### Phase 3 completion tasks
- [ ] Finalize canonical GlobalEvent adapter integration for commons upgrade events across canister boundaries.
- [ ] Complete adopted/pinned execution workflows with explicit proposal lifecycle and steward UX.
- [ ] Add upgrade history UI detail surfaces beyond minimal APIs.

### Phase 4 completion tasks (gated by 013)
- [ ] Replace manual automation trigger with workflow-engine backed scheduled/triggered runs.
- [ ] Integrate governance/workflow event subscriptions and retry/idempotency semantics.

### Phase 5 completion tasks
- [ ] Publish Nostra Core Commons v1 canonical ruleset package.
- [ ] Ship adoption onboarding defaults in template spaces.
- [ ] Publish commons authoring and operations guide.

### Phase 6 completion tasks (gated)
- [ ] Bind GSMS scenario outputs directly into ratification proposal flow.
- [ ] Enforce scenario coverage/risk thresholds from GSMS evidence artifacts.
- [ ] Publish deterministic replay validation guidance for commons ratification.

## Verification Commands

```bash
cd /Users/xaoj/ICP/nostra
TERM=xterm-256color NO_COLOR=1 dfx build backend
bash /Users/xaoj/ICP/scripts/check_did_declaration_sync.sh
cargo check --manifest-path /Users/xaoj/ICP/nostra/Cargo.toml -p frontend
```

## Latest Gate Results (2026-02-18)

- ✅ `TERM=xterm-256color NO_COLOR=1 dfx build backend`
- ✅ `bash /Users/xaoj/ICP/scripts/check_did_declaration_sync.sh`
- ❌ `cargo check --manifest-path /Users/xaoj/ICP/nostra/Cargo.toml -p frontend` (DNS/index access failure in this environment)
- ❌ `cargo check --offline --manifest-path /Users/xaoj/ICP/nostra/Cargo.toml -p frontend` (fails on broader frontend compile parity issues; yanked `keccak` chain no longer present)

## Closeout Gate

Initiative 119 is closure-complete only when:
1. Phase 1-2 production rollout evidence (shadow -> warn/block) is recorded.
2. Phase 3-6 are complete or formally waived with steward-approved gate rationale.
3. `PLAN.md`, `VERIFY.md`, `DECISIONS.md`, and `RESEARCH_INITIATIVES_STATUS.md` reflect implementation truth.
4. A final closure artifact (`PHASE_0_6_CLOSURE_EVIDENCE_<date>.md`) is published and initiative status is moved to completed.

## Alignment Addendum (Constitution + System Standards)

- Labs Constitution: Default to Production patterns unless explicitly labeled as Labs; experiments remain fork-safe and documented.
- Knowledge Integrity & Memory: Preserve lineage, log decisions, and avoid rewriting history; summaries are additive, not replacements.
- Spaces Constitution: All authority and data are space-scoped; cross-space effects are explicit links, not merges.
- Stewardship & Roles: Identify accountable roles per change; unclear authority defaults to recommendation-only.
- Contribution Lifecycle: Renames, merges, archives, and scope changes require explicit rationale and approval.
- Agent Behavior & Authority: Agents operate in observe/recommend/simulate unless execution is explicitly approved.
- Security & Privacy: Least authority, explicit consent, and scoped access; minimize sensitive data exposure.
- Governance & Escalation: Disputes and irreversible actions follow escalation pathways and steward review.
- UI/UX Manifesto: Interfaces must surface provenance, time, and agency; avoid dark patterns.
- Modularity: Strict interfaces, no hardcoded canister IDs, and clean boundary contracts.
