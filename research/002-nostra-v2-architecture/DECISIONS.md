---
id: '002'
name: nostra-v2-architecture
title: 'Decision Log: Nostra v2 Architecture'
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-15'
updated: '2026-02-15'
---

# Decision Log: Nostra v2 Architecture

Track architectural decisions with rationale for future reference.

---

## DEC-001: Unified Contribution Base Type
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. Keep separate types for each entity (Idea, Project, Issue, etc.)
2. Create abstract `Contribution` base type that all entities extend
3. Use a generic record with dynamic typing

**Decision**: Option 2 - Abstract base type

**Rationale**:
- Reduces code duplication across CRUD operations
- Simplifies activity stream aggregation
- Enables generic graph edge storage
- Type safety preserved through variants

**Implications**:
- All existing types need refactoring
- Candid interfaces updated
- Frontend type generation must account for union types

---

## DEC-002: Naming Convention Updates
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. Keep "Outputs" and "Thoughts"
2. Rename to "Artifacts" and "Reflections"
3. Partial rename: "Artifacts" only

**Decision**: Option 2 - Rename to "Artifacts" and "Reflections"

**Rationale**:
- "Artifacts" is more general and scales beyond academic contexts
- "Reflections" better describes the gating mechanism purpose
- Consistency in naming improves mental model
- User confirmed preference for both changes

**Implications**:
- UI labels, API endpoints, and documentation updated
- Migration script needed for existing data (field renaming)
- Update spec.md naming throughout (already done in v2 sections)

---

## DEC-003: Canister Architecture - Logical vs Physical
**Date**: 2026-01-14
**Status**: 🟡 Proposed (Pending Coordination)

**Options Considered**:
1. Single monolithic canister
2. Immediately split into 7 canisters (Space, Contribution, Activity, etc.)
3. Start with logical modules in single canister; split when scale demands

**Decision**: Coordinate with Nostra multi-project architecture

**Rationale**:
- Nostra already defines a multi-canister architecture pattern:
  - Registry Canister (project management)
  - Schema Registry (shared schemas)
  - KG Data Canister (template)
  - MCP Server (AI integration)
- Nostra can leverage similar patterns or share infrastructure
- Need unified approach across both projects

**Cross-Project Dependencies**:
- See `/Users/xaoj/ICP/research/001-multi-project-architecture/PLAN.md`
- Consider whether Nostra Spaces map to KG Projects
- Evaluate shared Schema Registry for contribution types

**Implications**:
- Defer physical canister split decision until coordination complete
- Document shared canister patterns in FRAMEWORK_STRATEGY.md
- Code organized by logical module within single canister initially

---

## DEC-004: Graph Storage Strategy
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. Adjacency list per contribution (store refs directly on each entity)
2. Separate edge table (ContributionRelation entries)
3. Hybrid: common relations on entity, detailed graph in edge table

**Decision**: Option 2 - Separate edge table

**Rationale**:
- Enables rich edge metadata (type, timestamp, author)
- Simplifies graph queries
- Decouples graph updates from contribution updates
- Better for future graph visualization
- Aligns with Nostra Relationship model

**Implications**:
- New `ContributionRelation` type: `{ source, target, relationType, createdAt, createdBy }`
- Graph queries traverse edge table
- Indexing needed for efficient traversal

---

## DEC-005: Space Configuration Scope
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. Minimal config: visibility only (current state)
2. Medium config: visibility + enabled types + allowed transitions
3. Full config: above + governance model + required fields

**Decision**: Option 2 - Medium configuration for v2

**Rationale**:
- Medium covers most use cases (DAO, research, product teams)
- Governance model is complex and deserves dedicated iteration
- Required fields can be added in v3

**Governance Placeholder**:
> [!NOTE]
> **Full Governance (v3 Scope)**:
> - Voting mechanisms (per-contribution, per-space, quorum)
> - Delegation models
> - SNS/DAO integration
> - Token-gated spaces
> - Proposal workflows for major decisions

**Implications**:
- `SpaceConfig` type includes: `enabledTypes`, `allowedTransitions`, `visibility`
- Validation layer for contribution creation and transitions
- UI for space owners to configure
- Governance extension points documented for v3

---

## DEC-006: AI Agent Access Model
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. AI agents are pure consumers (read-only)
2. AI agents can write with explicit user approval per action
3. AI agents have delegate roles with scoped permissions

**Decision**: Plan for controlled writes (Options 2/3)

**Rationale**:
- Read-only is insufficient for meaningful AI integration
- Controlled writes enable AI-assisted contribution drafting
- User maintains final approval authority
- Aligns with Nostra MCP integration pattern

**Implementation Approach**:
1. **v2.0**: Read-only AI endpoints (summaries, suggestions)
2. **v2.1**: Draft mode (AI proposes, user confirms)
3. **v2.2+**: Delegate principal with scoped permissions

**Security Guardrails**:
- AI-created contributions marked with `source: "ai-draft"`
- Require human approval before publishing
- Rate limiting on AI write operations
- Audit log for all AI actions

**References**:
- Nostra MCP integration: `/Users/xaoj/ICP/research/001-multi-project-architecture/PLAN.md#mcp-integration`

---

## DEC-007: Graph Visualization Library
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. D3.js - Maximum flexibility, steep learning curve
2. Sigma.js - Optimized for large graphs (WebGL)
3. Three.js - 3D immersive visualization
4. Dioxus-native (Rust/WASM approach)

**Decision**: D3.js (via Dioxus Bridge)

**Rationale**:
- User explicitly requested dropping Sigma.js/WebGL
- D3 provides maximum robustness and flexibility (user assumption confirmed)
- Frontend stack moved to Dioxus (Rust); D3 interactions handled via `wasm-bindgen`
- Simplifies dependency chain (pure SVG/Canvas manipulation via JS interop)

**Implications**:
- All graph rendering uses D3.js force-directed layouts
- Need robust Dioxus ↔ JS bindings for D3
- Performance limits of SVG/D3 must be managed (node limits ~1k visible)
- "Network View" will simply use zoomed-out D3 visualization

---

## DEC-008: Accessibility as Constitutional Invariant
**Date**: 2026-02-03
**Status**: ✅ Decided

**Options Considered**:
1. Treat accessibility as a "UX Guideline" (Manifesto only)
2. Add accessibility as a specific requirement for individual components
3. Elevate accessibility to a Constitutional Invariant (Standard)

**Decision**: Option 3 - Constitutional Invariant

**Rationale**:
- Inaccessible systems exert illegitimate authority by excluding users based on physical/cognitive variance.
- Governance legitimacy requires universal access (Ability to Perceive, Operate, Understand).
- Aligns with "Capabilities" model: The system must not assume a specific sensorium.

**Implications**:
- Created `shared/standards/ACCESSIBILITY.md` as a core doctrine.
- Updated `shared/specs.md` to include "Interaction" as a system plane.
- A2UI Schema must enforce semantic roles (Authority Containment).
- Motion Tokens must be governed (Vestibular safety).

---

## Pending Decisions

| ID | Topic | Blocking Phase | Awaiting |
|----|-------|----------------|----------|
| DEC-003 | Canister split strategy | Phase 4 | Cross-project coordination |
| DEC-004 (reserved) | Shared infrastructure with Cortex | Phase 4 | Architecture alignment meeting |

---

## Cross-Project References

| Topic | Related Research |
|-------|------------------|
| Multi-canister architecture | [Nostra PLAN.md](file:///Users/xaoj/ICP/research/001-multi-project-architecture/PLAN.md) |
| Schema Registry pattern | [Nostra PLAN.md §Schema Registry](file:///Users/xaoj/ICP/research/001-multi-project-architecture/PLAN.md#2-schema-registry-kg-schema) |
| MCP integration | [Nostra PLAN.md §MCP](file:///Users/xaoj/ICP/research/001-multi-project-architecture/PLAN.md#mcp-integration-architecture) |
| Design patterns (Graphiti/OpenSPG) | [Nostra DECISIONS.md DEC-006](file:///Users/xaoj/ICP/research/001-multi-project-architecture/DECISIONS.md) |
| Workflow Engine | [013-nostra-workflow-engine](file:///Users/xaoj/ICP/research/013-nostra-workflow-engine/REQUIREMENTS.md) |
| AI Agent Architecture | [014-ai-agents-llms-on-icp](file:///Users/xaoj/ICP/research/014-ai-agents-llms-on-icp/RESEARCH.md) |
| Open Source Library | [015-nostra-open-source-library](file:///Users/xaoj/ICP/research/015-nostra-open-source-library/PLAN.md) |
| Contribution Types Taxonomy | [008-nostra-contribution-types](file:///Users/xaoj/ICP/research/008-nostra-contribution-types/PLAN.md) |
| File Infrastructure & Version Chaining | [085-nostra-file-infrastructure](file:///Users/xaoj/ICP/research/085-nostra-file-infrastructure/DECISIONS.md) |
