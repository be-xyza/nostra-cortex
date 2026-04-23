---
id: "126"
name: "agent-harness-architecture"
title: "Requirements & Tech Stack: Agent Harness Architecture"
type: "requirements"
project: "nostra"
status: draft
authors:
  - "User"
tags: ["agent-harness", "governance", "temporal", "evaluation"]
created: "2026-02-24"
updated: "2026-02-24"
---

# Requirements & Tech Stack

## Overview
This document outlines the technical requirements for formalizing Cortex as an Agent Harness. It specifies the necessary infrastructure to support structured, durable, and governed agent execution loops.

---

## Tech Stack

### Backend
| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| Execution Runtime | Rust | Latest | Core Cortex environment containing the harness logic |
| Durable Workflows | Temporal | Latest | Managing long-running evaluation loops and promotion gates |
| Event Storage | Nostra (GlobalEvent) | v2 | Storing `AgentExecutionRecord` and state snapshots |

---

## Functional Requirements

### FR-1: Execution Traceability
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | The system MUST emit an `AgentExecutionRecord` at the start and end of every agent session. | Must |
| FR-1.2 | The record MUST include `input_snapshot_hash`, `output_snapshot_hash`, and current `authority_scope`. | Must |

### FR-2: Authority Governance
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | The harness MUST enforce explicit Authority Levels (L0-L4) on all agent actions. | Must |
| FR-2.2 | Agents operating at L1 MUST only be able to create `Proposal` entities, not perform direct writes. | Must |
| FR-2.3 | Write actions exceeding the agent's current authority MUST be blocked or converted to recommendations. | Must |

### FR-3: Evaluation and Promotion
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | The system MUST support plugging in custom evaluation workflows per space. | Must |
| FR-3.2 | Agent outputs MUST pass the evaluation loop before being promoted or committed. | Must |

### FR-4: Replayability
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | The system MUST capture sufficient state (inputs, configs, tool results) to allow deterministic replay of an agent session. | Should |

---

## Non-Functional Requirements

### NFR-1: Performance
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1.1 | Emitting `AgentExecutionRecord` MUST NOT block the critical execution path of the agent workflow. | < 50ms overhead |

### NFR-2: Security & Sovereignty
| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-2.1 | The harness MUST NOT override space-level governance rules; it must enforce them. | Must |

---

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `nostra-core` | Workspace | For `GlobalEvent` and `Proposal` types |
| `cortex-runtime` | Workspace | To intercept and wrap agent executions |

---

## Constraints

| Constraint | Description |
|------------|-------------|
| Modularity | The harness logic must remain in Cortex (execution layer) and not leak into Nostra (platform/authority layer). |
| Governance | Cannot hardcode global evaluation oracles; must allow space governance. |

---

## References

- [PLAN.md](./PLAN.md) - Implementation phases
- [AGENTS.md](../../AGENTS.md) - Agent Constitutional Framework
