---
id: '002'
name: nostra-v2-architecture
title: 'Feedback Log: Nostra v2 Architecture'
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-06'
---

# Feedback Log: Nostra v2 Architecture

---

## 2026-01-14: Initial Research Questions
**Source**: Agent/Architecture Analysis

### Resolved Questions

1. **Naming Conventions**
   - [x] Should "Outputs" be renamed to "Artifacts"?
     - **Answer**: Yes
     - **Decision**: → See DEC-002 in DECISIONS.md
     - **Updated**: 2026-01-14
   - [x] Should "Thoughts" be renamed to "Reflections"?
     - **Answer**: Yes
     - **Decision**: → See DEC-002 in DECISIONS.md
     - **Updated**: 2026-01-14

2. **Space Configuration Depth**
   - [x] What level of space customization is needed for v2?
     - **Answer**: Medium (visibility + enabled types + transitions)
     - **Note**: Leave placeholder for full governance recommendations in v3
     - **Decision**: → See DEC-005 in DECISIONS.md
     - **Updated**: 2026-01-14

3. **Canister Split Timing**
   - [x] At what scale/complexity should we split logical modules into physical canisters?
     - **Answer**: Coordinate with Nostra multi-project architecture requirements
     - **Research needed**: Align canister patterns across both projects
     - **Decision**: → See DEC-003 in DECISIONS.md
     - **Updated**: 2026-01-14

4. **AI Agent Write Access**
   - [x] Should v2 support any AI write capabilities, or strictly read-only?
     - **Answer**: Plan for controlled writes
     - **Approach**: Read-only v2.0 → Draft mode v2.1 → Delegate principals v2.2+
     - **Decision**: → See DEC-006 in DECISIONS.md
     - **Updated**: 2026-01-14

5. **Graph Visualization Tech**
   - [x] Which library for graph visualization?
     - **Answer**: D3.js (via Dioxus bridge)
     - **Constraint**: User explicitly dropped Sigma.js/WebGL
     - **Decision**: → See DEC-007 in DECISIONS.md
     - **Updated**: 2026-01-14

6. **Governance Model Scope**
   - [x] Is governance configuration in scope for v2?
     - **Answer**: Not for v2; placeholder documentation for v3
     - **Decision**: → See DEC-005 governance placeholder in DECISIONS.md
     - **Updated**: 2026-01-14

---

## 2026-01-14: Graph Library Research Summary
**Source**: Web research on D3.js vs Sigma.js vs Three.js

### Findings

**D3.js Validation**:
- ✅ **Flexibility confirmed**: Unparalleled customization for diverse visualization types
- ✅ **Robustness**: Mature ecosystem, extensive documentation
- ⚠️ **Performance caveat**: Can struggle with large datasets (SVG DOM manipulation)
- ✅ **Best for**: Custom contribution detail views, lifecycle diagrams, interactive charts

**Sigma.js Benefits**:
- ✅ **Performance**: WebGL-based, handles 50K+ nodes and 1M+ edges
- ✅ **Graph-specific**: Purpose-built for network visualization
- ✅ **Best for**: Full space network views, similarity graphs

**Three.js Notes**:
- 3D is powerful but adds complexity
- Consider for future "immersive exploration" mode if requested
- Can combine with D3 for data handling

### Recommendation
> [!NOTE]
> **SUPERSEDED by DEC-007**: User explicitly requested dropping Sigma.js/WebGL. Using D3.js only.

~~Use hybrid approach:~~
1. ~~**D3.js** for detailed contribution views (< 500 nodes typical)~~
2. ~~**Sigma.js** for space-wide network visualization (potentially 1000s of nodes)~~

---

## Research Path Status

| Priority | Research | Status | Notes |
|----------|----------|--------|-------|
| 1 | R5: Naming Validation | ✅ Complete | User confirmed Artifacts/Reflections |
| 2 | R2: Graph Persistence | 🟡 In Progress | Align with Nostra patterns |
| 3 | R4: Governance Survey | ⏳ Deferred to v3 | Document placeholder in DEC-005 |
| 4 | R1: Reference Architecture | 🟡 Started | Nostra provides reference |
| 5 | R3: AI Integration | 🟡 Started | Controlled writes planned per DEC-006 |

---

## New Research Items

### R6: Cross-Project Canister Coordination
**Purpose**: Align Nostra canister architecture with core framework
**Activities**:
- Compare Nostra logical modules with core canister design
- Identify shared infrastructure opportunities (Registry, Schema Registry)
- Document where Nostra Spaces could integrate with KG Projects
- Define common MCP integration patterns

**Blocking**: DEC-003 (Canister split strategy)

**Reference**:
- [Nostra Reference PLAN.md](file:///Users/xaoj/ICP/research/001-multi-project-architecture/PLAN.md)
- [Nostra Reference DECISIONS.md](file:///Users/xaoj/ICP/research/001-multi-project-architecture/DECISIONS.md)

### R7: Dioxus ↔ JS Interop for Graph Libraries
**Purpose**: Define patterns for calling D3.js/Sigma.js from Dioxus/WASM
**Activities**:
- Research wasm-bindgen patterns for JS library calls
- Evaluate existing Dioxus graph components
- Document Nostra-specific interop requirements
- Create proof-of-concept integration

**Blocking**: Phase 5 (Graph Visualization)

---

## Remaining Open Questions

1. **Cross-Project Infrastructure**
   - [ ] Should Nostra and the core Registry share a Registry Canister?
   - [ ] Can Nostra contribution types be stored in a shared Schema Registry?
   - [ ] How do Nostra Spaces relate to KG Projects conceptually?

2. **Graph Performance Thresholds**
   - [ ] At what node count should UI switch from D3 to Sigma.js?
   - [ ] Recommended: < 500 nodes D3, > 500 nodes Sigma.js (validate with testing)

---

## Resolution Template

When questions are resolved, update entries as follows:

### Open Questions (RESOLVED)
- [x] [Question text]
  - **Answer**: [Resolution]
  - **Decision**: → See DEC-NNN in DECISIONS.md (if applicable)
  - **Updated**: YYYY-MM-DD
