---
id: '002'
name: nostra-v2-architecture
title: 'Requirements & Tech Stack: Nostra v2'
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements & Tech Stack: Nostra v2

## Overview
Technical requirements for evolving Nostra from a collaborative ideas engine to a **Collaborative Knowledge & Execution Engine** with unified contribution model, lifecycle flows, and graph-based architecture.

---

## Tech Stack

### Backend
| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| Runtime | Internet Computer | Latest | Decentralized compute |
| Language | Motoko or Rust | Motoko ≥0.10 / Rust ic-cdk ≥0.12 | Canister implementation |
| Storage | Stable Structures | Latest | Persistent upgrade-safe storage |
| Identity | Internet Identity | N/A | User authentication |
| Build | dfx | ≥0.15 | Local development & deployment |

### Frontend
| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| Framework | Dioxus | 0.5+ | UI framework (Rust) |
| State | Dioxus Signals | 0.5+ | State management |
| Routing | Dioxus Router | 0.5+ | Client-side routing |
| Styling | Tailwind CSS | 3.x | Utility-first CSS |
| IC Agent | ic-agent | Latest | Backend communication (Rust) |
| Graph Viz | D3.js | v7+ | Viz (via Dioxus bridge) |

---

## Functional Requirements

### FR-1: Unified Contribution Model
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | All contribution types share base properties (id, spaceId, type, title, description, status, tags, contributors, version, timestamps, phase, representations) | Must |
| FR-1.2 | System supports all contribution types: Idea, Question, Issue, Project, Initiative, Deliverable, Milestone, Artifact, Comment, Reflection, Decision, Poll, Bounty, Essay, Post, MediaEssay | Must |
| FR-1.3 | New contribution types can be added without breaking existing functionality | Should |
| FR-1.4 | Contributions maintain full version history | Must |
| FR-1.5 | Contributions support Contribution Phase metadata (Exploratory, Deliberative, Decisive, Executable, Archival) | Must |

### FR-2: Contribution Lifecycle
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | Ideas can evolve into Projects through explicit transition | Must |
| FR-2.2 | Lifecycle transitions are logged and auditable | Must |
| FR-2.3 | System supports forking and merging of contributions | Must |
| FR-2.4 | Outcomes feed back into new Ideas (cyclical model) | Should |
| FR-2.5 | Posts can evolve into Essays; Essays can evolve into MediaEssays (content evolution path) | Should |
| FR-2.6 | Reflections can be promoted to Artifacts or Decisions | Should |

### FR-3: Space Configuration
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | Spaces can enable/disable specific contribution types | Must |
| FR-3.2 | Spaces can define allowed transitions (e.g., Idea → Project) | Must |
| FR-3.3 | Spaces define visibility rules (public, private, member-only) | Must |
| FR-3.4 | Spaces can specify required metadata fields per type | Future (v3) |
| FR-3.5 | Governance model configuration (voting, quorum) | Future (v3) |

### FR-4: Graph Layer
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | System stores relationships between contributions as graph edges | Must |
| FR-4.2 | Graph supports edge types: evolves, contains, references, forks, merges | Must |
| FR-4.3 | Activity stream provides both temporal and relational views | Must |
| FR-4.4 | Graph visualization uses D3.js via Dioxus/WASM bridge | Must |

### FR-5: AI Readiness
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1 | Structured data export for AI consumption | Should |
| FR-5.2 | Summary aggregation endpoints | Could |
| FR-5.3 | Hooks for similarity/merge suggestions | Could |
| FR-5.4 | AI agents are read-only in v2.0; Draft mode enabled in v2.1 (Requires approval) | Must |

---

## Non-Functional Requirements

### NFR-1: Performance
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1.1 | Contribution creation latency | < 2s consensus finality |
| NFR-1.2 | Graph query response time | < 500ms for subgraphs ≤100 nodes |
| NFR-1.3 | Activity stream load time | < 1s for recent 50 items |

### NFR-2: Scalability
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-2.1 | Contributions per space | 10,000+ |
| NFR-2.2 | Graph edges per space | 50,000+ |
| NFR-2.3 | Concurrent users per canister | 1,000+ |

### NFR-3: Security
| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | All mutations verify caller identity | Must |
| NFR-3.2 | Role-based access control enforced server-side | Must |
| NFR-3.3 | Private space content inaccessible to non-members | Must |
| NFR-3.4 | AI agents cannot modify data without explicit permission | Must |

### NFR-4: Maintainability
| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-4.1 | Canister upgrades preserve all state | Must |
| NFR-4.2 | Modular code organization for logical separation | Must |
| NFR-4.3 | Candid interfaces versioned for backward compatibility | Should |

---

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| ic-agent (Rust) | Latest | Backend communication |
| dioxus | 0.5+ | Frontend framework |
| wasm-bindgen | Latest | JS Interop |
| d3-wasm | Custom | D3.js bindings |

---

## Constraints

| Constraint | Description |
|------------|-------------|
| ICP Canister Memory | 4GB heap, 64GB stable storage per canister |
| Cycles Budget | All operations consume cycles; avoid infinite loops |
| Candid Backward Compat | Breaking interface changes require migration strategy |
| Single Canister Start | Begin with logical modules, split later if needed |

---

## Open Questions → FEEDBACK.md

1. **Naming**: Confirm "Outputs" → "Artifacts" and "Thoughts" → "Reflections"
2. **Canister Split**: When should logical modules become physical canisters?
3. **Governance**: Is Phase 3 governance configuration in scope for v2 or a future iteration?
4. **AI Write Access**: Under what conditions (if any) should AI agents be able to write?

---

## References

- [PLAN.md](./PLAN.md) - Implementation phases
- [DECISIONS.md](./DECISIONS.md) - Architectural decisions
- [FEEDBACK.md](./FEEDBACK.md) - Open questions & responses
