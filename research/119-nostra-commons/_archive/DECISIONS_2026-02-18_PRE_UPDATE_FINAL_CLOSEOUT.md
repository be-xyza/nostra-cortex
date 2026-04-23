---
id: "119"
name: "nostra-commons"
title: "Decision Log: Nostra Commons"
type: "decision"
project: "nostra"
status: draft
authors:
  - "X"
tags:
  - "commons"
  - "governance"
  - "institution"
created: "2026-02-14"
updated: "2026-02-14"
---

# Decision Log: Nostra Commons

Track architectural decisions with rationale for future reference.

---

## DEC-119-001: Commons as Institution Usage Pattern (Not New Entity)
**Date**: 2026-02-14
**Status**: ✅ Decided

**Options Considered**:
1. New `Commons` ContributionType in the unified contribution model
2. Institution with `scope = "commons"` convention + charterRefs to SIQS rules
3. Standalone non-contribution entity

**Decision**: Option 2 — Institution usage pattern

**Rationale**: Institution already has lifecycle (6 phases), forking with lineage, governance strategy, version chaining, and graph edges (`governs`, `derives_from`, `charters`). Adding a new ContributionType would duplicate all of this infrastructure. Using Institution as the backing entity means zero contribution model changes and full reuse of existing CRUD, UI, and governance integration (094 Phases 1-6).

**Implications**: Commons are distinguishable only by convention (scope field content, charter content type), not by type system enforcement. This is a trade-off: simpler architecture but requires discipline in tooling to filter "Commons institutions" from "regular institutions."

---

## DEC-119-002: Adoption via Edge Metadata (Not Space Fields)
**Date**: 2026-02-14
**Status**: ✅ Decided

**Options Considered**:
1. Add `commonsRef` and `commonsMode` fields to Space type
2. Use existing `governs` edge with metadata for adoption semantics

**Decision**: Option 2 — Edge metadata

**Rationale**: Adding fields to Space couples the Space type to the Commons feature. Edge metadata is consistent with existing institutional relationship patterns and allows a Space to be governed by multiple Commons (future capability). The `governs` edge already exists in `EdgeTypes.GOVERNS`.

**Implications**: Adoption mode discovery requires edge traversal rather than direct field access. This is acceptable given graph queries are already used for institutional relationships.

---

## DEC-119-003: SIQS as Enforcement Engine (Not Custom Rules Engine)
**Date**: 2026-02-14
**Status**: ✅ Decided

**Options Considered**:
1. Custom Commons rules engine with its own DSL
2. Reuse SIQS `IntegrityRule` + `IntegrityPredicate` from Research 118
3. Embedded Motoko validation functions

**Decision**: Option 2 — SIQS reuse

**Rationale**: Research 118's SIQS defines `IntegrityRule`, `IntegrityPredicate`, `Constraint`, and `evaluate_all(rules, graph) → violations` — exactly the abstraction Commons enforcement needs. Building a separate rules engine would duplicate effort and create two incompatible validation systems. SIQS predicates are composable, serializable, and tested.

**Implications**: Phase 1-2 of Commons is gated on Research 118 delivering SIQS Layer 1. This is an acceptable dependency given SIQS has broader utility beyond Commons.

---

## DEC-119-004: No commonsRef Field on Space
**Date**: 2026-02-14
**Status**: ✅ Decided

**Options Considered**:
1. Add optional `commonsRef: ?InstitutionID` to Space
2. Rely on graph query (`getCommonsForSpace` via `governs` edge)

**Decision**: Option 2 — Graph query

**Rationale**: The `governs` edge already semantically represents this relationship. A redundant field creates a sync problem (field vs edge could disagree). Graph-first query is consistent with the existing architecture.

**Implications**: One graph traversal per Space render to check for governing Commons. This is trivially cacheable and consistent with existing Space → Institution relationship queries.

---

## DEC-119-005: Authority Rollout and Enforcement Progression
**Date**: 2026-02-17
**Status**: ✅ Decided

**Decision**:
1. Keep `upsertCommonsRuleset` for local/dev speed (`adminOnly` mode).
2. Require `upsertCommonsRulesetViaProposal` in production mode (`proposalRequired`).
3. Roll out enforcement with explicit progression: `shadow` -> `warnOrBlock`.

**Rationale**: This preserves local iteration speed while hardening production mutation authority and reducing false-positive blocking risk during initial rollout.

**Implications**:
- Production requires approved proposal linkage (`commons_id` metadata check) for ruleset mutation.
- Shadow period evidence is mandatory before warn/block promotion.

## DEC-119-006: Closeout Scaffolding for Phase 3/4/6 Gates
**Date**: 2026-02-17
**Status**: ✅ Decided

**Decision**: Implement gate-aware scaffolding now for later phases without claiming full phase closure:
1. Phase 3: adoption-state persistence + upgrade notice/history scan + backend global event surface.
2. Phase 4: automation policy and manual execution surfaces behind disabled-by-default policy.
3. Phase 6: simulation gate policy/report surfaces and proposal approval gate hook.

**Rationale**: This de-risks interface churn and allows incremental rollout while preserving external dependency gates (013 workflow integration and GSMS maturity).

**Implications**:
- APIs are stable for frontend/operator integration.
- Some behaviors remain intentionally partial until dependent initiatives are green.

## DEC-119-007: Frontend Workspace Manifest Restoration
**Date**: 2026-02-17
**Status**: ✅ Decided

**Decision**: Restore `/Users/xaoj/ICP/nostra/frontend/Cargo.toml` to resolve workspace topology and make `cargo check -p frontend` runnable again.

**Rationale**: `frontend` remained a workspace member in `/Users/xaoj/ICP/nostra/Cargo.toml`; missing manifest blocked repeatable build gates required for closeout.

**Implications**:
- Workspace-level and package-targeted Rust checks are executable.
- Future frontend dependency updates should be tracked via this manifest rather than lock-only drift.
