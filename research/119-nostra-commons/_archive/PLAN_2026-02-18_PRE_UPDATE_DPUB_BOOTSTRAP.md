---
id: "119"
name: "nostra-commons"
title: "Nostra Commons Implementation Plan"
type: "plan"
project: "nostra"
status: completed
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

## Current Phase Status (2026-02-18 Final)

| Phase | Status | Notes |
|---|---|---|
| Phase 0: Semantic Foundation | ✅ Complete | Commons APIs and adoption edge model shipped. |
| Phase 1: Ruleset Schema | ✅ Complete (closure tranche) | Ruleset normalization, semver, templates, authority mode shipped. |
| Phase 2: Enforcement Integration | ✅ Complete (closure tranche) | Deterministic multi-commons aggregation + compliance query + UI read panel shipped. |
| Phase 3: Upgrade Protocol | ✅ Complete (Gated Waiver) | Core deterministic upgrade/state path shipped; external GlobalEvent pipeline completion waived for Initiative 119 closeout. |
| Phase 4: Automation Hooks | ✅ Complete (Gated Waiver) | Workflow-engine hook and policy gating shipped; full 013 governance bridge integration waived from 119 closure scope. |
| Phase 5: Canonical Commons v1 | ✅ Complete | Canonical v1 publish/read and auto-adopt controls shipped; baseline onboarding/ruleset path operational. |
| Phase 6: Simulation Validation Gate | ✅ Complete (Gated Waiver) | Policy/report/ratification gate surfaces shipped; full GSMS downstream coupling waived from 119 closure scope. |

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

## Waiver Closure Record (Phase 3-6 Gates)

The following follow-ons are explicitly waived from Initiative 119 closure scope and mapped to dependency tracks:

1. GlobalEvent cross-canister adapter hardening:
   - moved to shared specs + downstream implementation track.
2. Full workflow-governance bridge automation (013):
   - moved to Initiative 013 dependency closure path.
3. Full GSMS replay/risk coupling in governance approval flow:
   - moved to 118 GSMS downstream maturity path.

Waiver authority source:
1. Steward-directed closure instruction in this branch closeout session (2026-02-18).
2. Decision log entries `DEC-119-012` and `DEC-119-013`.

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
- ✅ `cargo check --manifest-path /Users/xaoj/ICP/nostra/Cargo.toml -p frontend` (warnings only)
- ✅ `cargo check --offline --manifest-path /Users/xaoj/ICP/nostra/Cargo.toml -p frontend`

## Closeout Gate

Closure criteria satisfied on 2026-02-18:
1. Phase 1-2 production-ready evidence recorded.
2. Phase 3-6 closed via implementation and/or explicit gated waivers with rationale.
3. `PLAN.md`, `VERIFY.md`, `DECISIONS.md`, and `RESEARCH_INITIATIVES_STATUS.md` synchronized to completed state.
4. Final closure artifact published (`PHASE_0_6_CLOSURE_EVIDENCE_2026-02-17.md`).

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
