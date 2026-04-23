---
id: "144-meta-workbench-refactoring"
name: "meta-workbench-refactoring"
title: "Requirements & Tech Stack: Meta Workbench Refactoring"
type: "requirements"
project: "cortex"
status: draft
authors:
  - "Antigravity Agent"
tags: ["workbench", "space", "meta", "ui"]
created: "2026-03-13"
updated: "2026-03-13"
---

# Requirements & Tech Stack

## Overview
This document outlines the requirements and technical assumptions for the "Meta Workbench" refactoring. It describes the necessary changes to enable a unified, cross-space view of a user's data and workflows.

---

## Tech Stack

### Backend
| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| `cortex-eudaemon` | Rust | Workspace | Local gateway handling block retrieval and viewspec compilation |
| `cortex_runtime` | Rust | Workspace | Workflow Engine execution environment |

### Frontend
| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| `cortex-web` | React | Workspace | Standard unified frontend |
| `uiStore.ts` | Zustand | Workspace | Global state management for active spaces and contexts |

---

## Functional Requirements

### FR-1: Semantic Optionality
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | `activeSpaceId` must be nullable or support a reserved "meta" state in UI stores. | Must |
| FR-1.2 | The Space Selector must present a "Global" or "All Spaces" option. | Must |
| FR-1.3 | Data fetching hooks (e.g., `useHeapActionPlan`) must conditionally omit `space_id` when the Meta Workbench is active. | Must |
| FR-1.4 | When creating new related blocks, the relation composer must provide a "Target Context" selector to explicitly choose the block's destination space. | Must |

### FR-2: Backend Aggregation
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | `get_cortex_heap_blocks` must return cross-space blocks when `space_id` is omitted. | Must |
| FR-2.2 | `get_space_navigation_plan` must intercept `"meta"` space IDs and return a globally scoped navigation plan. | Must |
| FR-2.3 | Transmitted blocks and tasks must retain their origin `workspace_id`. | Must |

---

## Non-Functional Requirements

### NFR-1: Performance
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1.1 | Cross-space local queries must not exceed linear degradation per connected space. | < 200ms latency |

### NFR-2: Security & Governance
| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-2.1 | Meta routing must strict-evaluate the global `actor_role` rather than relying on individual space capability graphs for global views. | Must |

---

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `cortex_domain::spaces` | Capability Graph resolution |
| A2UI Protocol | Viewspec compilation |

---

## Constraints

| Constraint | Description |
|------------|-------------|
| **Local Only Iteration**| The initial multi-space aggregation relies entirely on data synchronized locally to the Eudaemon. Remote, unsynced spaces limit the completeness of the Meta Workbench. |

---

## References

- [PLAN.md](./PLAN.md) - Implementation phases
- [DECISIONS.md](./DECISIONS.md) - Architectural decisions
