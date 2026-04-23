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
updated: "2026-02-14"
---

# Nostra Commons Implementation Plan

## Overview

A Nostra Commons is a **portable constitutional bundle** of rules, transitions, metadata requirements, governance hooks, and automation workflows that a Space can adopt, fork, pin to, or detach from.

Architecturally, a Commons is **not a new entity type**. It is an Institution whose `charterRefs` point to a set of SIQS `IntegrityRule` definitions. This reuses the existing Institution lifecycle, forking, governance strategy, and lineage infrastructure.

### Core Insight

> Encoding coordination patterns as portable constitutional artifacts — because Contributions are first-class, Governance is action-on-target, and Time is executable.

### Dependencies

| Dependency | Initiative | Status | Blocks Phase |
|---|---|---|---|
| Institution module | 094 (completed) | ✅ | Phase 0 |
| Spaces concept | 007 (active) | ✅ | Phase 0 |
| SIQS Engine | 118 Layer 1 | 🟡 Designed | Phase 1-2 |
| GlobalEvent pipeline | shared/specs.md → Motoko | 🔴 Spec'd only | Phase 3 |
| Workflow ↔ Governance | 013 + 094 Phase 7 | 🔴 Not integrated | Phase 4 |

---

## User Review Required

> [!IMPORTANT]
> **Decision: Commons as Institution usage pattern vs. new entity type.**
> This plan treats Commons as a usage pattern of the existing Institution entity (via `charterRefs` → SIQS rules), not as a new ContributionType. This avoids contribution model changes but means Commons are distinguishable only by convention (scope field, charter content), not by type system enforcement.

> [!WARNING]
> **Phases 1-4 are gated on external initiatives.** Phase 0 is the only immediately executable phase. If 118 (SIQS) or 013 (Workflow Engine) are delayed, Commons enforcement and automation will be deferred accordingly.

---

## Proposed Architecture

### Commons = Institution + SIQS Rules

```
Institution (scope = "commons")
├── charterRefs → [IntegrityRule artifacts]  // SIQS rules
├── governanceStrategy → voting/quorum config
├── lifecyclePhase → emergent → operational
├── version chain → previousVersionId
└── parentInstitutionId → lineage for forks
```

### Adoption via Edge Metadata

| Mode | Mechanism | Edge Metadata |
|---|---|---|
| **Adopted** | `governs` edge (Commons → Space) | `{ mode: "adopted", autoUpgrade: "minor" }` |
| **Pinned** | `governs` edge (Commons → Space) | `{ mode: "pinned", version: "1.2.3" }` |
| **Forked** | `derives_from` edge (New → Original) | Tracked via `parentInstitutionId` |
| **Detached** | No `governs` edge | Space has no commons relationship |

### Rule Layers

| Layer | Content | Enforcement |
|---|---|---|
| **Structural** | Required metadata, enabled types, field constraints | SIQS `IntegrityPredicate` |
| **Lifecycle** | Status transitions, inactivity thresholds, endorsement gates | SIQS + Temporal timers |
| **Automation** | Event triggers, auto-proposals, SLO enforcement | Workflow Engine + ACP events |
| **Upgrade** | Migration policy, version compatibility, pledge conditions | Governance proposals |

---

## Implementation Phases

### Phase 0: Semantic Foundation
> **Gate**: None — immediately executable
> **Goal**: Make Commons visible and queryable without enforcement.

- [x] Define Commons convention: `Institution.scope = "commons"` + marker in `charterRefs`
- [x] Add `getCommonsForSpace` query helper (implemented in `commons.mo`, wired in `main.mo`)
- [x] Add `adoptCommons(spaceId, commonsId, mode)` function to `main.mo`
- [x] Define edge metadata schema for adoption modes (composite type: `governs:commons:adopted|pinned`)
- [x] Add `detachCommons`, `listCommons`, `getCommonsAdoptions` queries
- [x] Create "Nostra Core Commons v0" as seed Institution in bootstrap
- [ ] Add Commons section to Space detail view *(deferred: frontend, out of scope for Phase 0 backend)*
- [x] Document convention in spec.md

### Phase 1: Ruleset Schema
> **Gate**: Research 118 delivers SIQS Layer 1
> **Goal**: Define Commons rules using SIQS predicate language.

- [ ] Define `CommonsRuleset` type mapping to `Vec<IntegrityRule>`
- [ ] Create standard rule templates (structural, lifecycle)
- [ ] Build ruleset serialization/deserialization
- [ ] Create UI for viewing Commons rules
- [ ] Map existing Space Configuration options to SIQS predicates

### Phase 2: Enforcement Integration
> **Gate**: SIQS evaluation engine built (118 Layer 1)
> **Goal**: Validate contributions against Commons rules at create/update time.

- [ ] Wire SIQS `evaluate_all(rules, graph)` into contribution create path
- [ ] Return violations as warnings or blocks based on rule severity
- [ ] Add Commons compliance indicator to Space dashboard
- [ ] Implement violation reporting in activity stream

### Phase 3: Upgrade Protocol
> **Gate**: GlobalEvent pipeline implemented
> **Goal**: Enable versioned Commons with upgrade detection.

- [ ] Implement `CommonsUpgradeAvailable` event type
- [ ] Build version comparison logic (semver for Commons)
- [ ] Create upgrade proposal auto-generation workflow
- [ ] Implement adoption mode responses (auto-upgrade for Adopted, notify for Pinned)
- [ ] Add upgrade history to Commons detail view

### Phase 4: Automation Hooks
> **Gate**: 013 Workflow Engine + Governance integration
> **Goal**: Connect Commons to workflow engine for reactive automation.

- [ ] Implement auto-proposal generation on governance events
- [ ] Wire SLO enforcement via ACP automation events
- [ ] Implement pledge-based conditional adoption (ConditionalGovernanceSubscription)
- [ ] Create standard automation workflow templates
- [ ] Connect inactivity/milestone timers to lifecycle rules

### Phase 5: Canonical Commons
> **Gate**: Phases 0-4 complete
> **Goal**: Ship "Nostra Core Commons v1" for ecosystem coherence.

- [ ] Define Nostra Core Commons v1 ruleset:
  - Minimum accessibility enforcement
  - Mandatory version chaining
  - Proposal-based breaking change policy
  - Auto-archive rule (configurable inactivity window)
  - Reflection gating on controversial decisions
- [ ] Create adoption onboarding flow
- [ ] Document Commons authoring guide
- [ ] Ship with default Space templates

### Phase 6: Simulation Validation Gate
> **Gate**: 118 Layer 3 (GSMS) activated
> **Goal**: Require simulation validation before Commons ratification.

- [ ] Define minimum simulation coverage for Commons approval
- [ ] Implement simulation report attachment on governance proposals
- [ ] Create canonical stress scenarios for Nostra Core Commons
- [ ] Wire GSMS `SimulationResult.risk_score` into governance decision support
- [ ] Document simulation validation requirements in Commons authoring guide

> [!NOTE]
> This phase integrates with 118 Layer 3 (GSMS). See
> `research/118-cortex-runtime-extraction/GSMS_ACTIVATION.md` for the
> simulation infrastructure specification.


---

## Verification Plan

### Automated Tests
```bash
# Phase 0 verification
dfx build          # Clean compilation
cargo test         # Rust test suite

# SIQS integration (Phase 1-2)
# Run integrity rule evaluation tests
cargo test -p cortex-domain --lib integrity
```

### Manual Verification
1. Create a Commons Institution with scope = "commons"
2. Adopt it from a Space via `adoptCommons`
3. Verify `getCommonsForSpace` returns the correct Commons
4. Fork the Commons, verify lineage preserved
5. Verify Space detail view shows governing Commons

### Phase-Gate Verification
- Phase 0: Commons CRUD + adoption without enforcement
- Phase 1: Rules serialize/deserialize correctly, map to SIQS
- Phase 2: Contribution creation blocked/warned by SIQS violations
- Phase 3: Upgrade events emitted and responded to correctly
- Phase 4: Auto-proposals generated from workflow triggers
- Phase 5: Nostra Core Commons v1 adopted by test space, all rules enforced
- Phase 6: Simulation validation gate operational, risk scores integrated into governance

---

## File Structure (Proposed)

```
nostra/backend/modules/
├── institution.mo          # Existing — commons convention via scope field
├── governance.mo           # Existing — no changes in Phase 0
└── commons.mo              # NEW Phase 0: types, adoption logic, edge builders, query helpers

cortex-domain/src/integrity/ # Phase 1-2 (from Research 118)
├── rule.rs                 # IntegrityRule definitions
├── predicate.rs            # IntegrityPredicate types
└── engine.rs               # evaluate_all() function

research/119-nostra-commons/
├── PLAN.md                 # This file
├── RESEARCH.md             # Ideation analysis and reference synthesis
├── REQUIREMENTS.md         # Functional and non-functional requirements
├── DECISIONS.md            # Architectural decision log
├── FEEDBACK.md             # Feedback tracking
├── VERIFY.md               # Verification records
└── _archive/
```

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
- Composability: Actions are workflow-compatible and emit standard events.
- Data Confidence & Integrity: Confidence/reliability metadata is required where applicable.
- Portability: Data must be exportable and WASM-safe; avoid OS-specific dependencies in core logic.
- Durable Execution: State is persisted via stable memory; workflows are replayable.
- Visibility Decoupling: Index/search are async from source of truth.
- Outbox Pattern: External calls are queued with idempotency and retry semantics.
- Verification: Each initiative includes verification steps and records results.
