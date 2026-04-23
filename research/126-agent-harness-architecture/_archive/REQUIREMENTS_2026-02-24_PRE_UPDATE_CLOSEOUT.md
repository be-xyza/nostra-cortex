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
This document outlines the technical requirements for formalizing Cortex as an Agent Harness. It specifies the infrastructure needed to support structured, durable, and governed agent execution loops.

---

## Tech Stack

### Backend
| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| Execution Runtime | Rust | Stable channel (CI pinned) | Core Cortex environment containing harness logic |
| Durable Workflows | `squads-temporal-*` crates | `0.3.1` (locked in `cortex/Cargo.lock`) | Long-running evaluation loops and promotion gates |
| Event Storage Contract | Nostra `GlobalEvent` envelope | v2 (`shared/specs.md`) | Stores `AgentExecutionRecord` lifecycle and replay metadata |

---

## V1 Scope Envelope
- Runtime authority implementation is limited to `L0-L2`.
- `L3-L4` remain defined contract levels but are deferred and non-runnable in this initiative.
- `AgentExecutionRecord` extension fields are optional in v1 and must not block release.

---

## Functional Requirements

### FR-1: Execution Traceability
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | The system MUST emit an `AgentExecutionRecord` lifecycle event at execution start and terminal phase (`completed`, `failed`, or `aborted`). | Must |
| FR-1.2 | Every lifecycle event MUST include `execution_id`, `attempt_id`, `phase`, `status`, and current `authority_scope`. | Must |
| FR-1.3 | Start and terminal lifecycle events MUST be correlatable through shared `execution_id` and monotonic timestamps. | Must |
| FR-1.4 | `AgentExecutionRecord` MUST be encoded in the canonical `GlobalEvent` envelope (`id`, `source`, `type`, `resource`, `payload`, `timestamp`). | Must |
| FR-1.5 | The record MUST include `input_snapshot_hash` and `output_snapshot_hash` for replay lineage. | Must |
| FR-1.6 | Core v1 payload keys MUST be present (`schema_version`, `execution_id`, `attempt_id`, `agent_id`, `workflow_id`, `phase`, `status`, `authority_scope`, `input_snapshot_hash`, `output_snapshot_hash`, `timestamp`); extension keys are optional. | Must |
| FR-1.7 | `GlobalEvent.type` value `AgentExecutionLifecycle` MUST be registered in shared standards before production enablement. | Must |

### FR-2: Authority Governance
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | The harness MUST enforce explicit Authority Levels on all agent actions, with v1 runtime support for `L0-L2`. | Must |
| FR-2.2 | Agents operating at L1 MUST only create `Proposal` entities and MUST NOT perform direct writes. | Must |
| FR-2.3 | Write actions exceeding the agent's current authority MUST be blocked or converted to recommendations. | Must |
| FR-2.4 | Any requested `L3-L4` execution in v1 MUST fail closed (blocked/deferred) and emit a governance-visible recommendation record. | Must |

### FR-3: Evaluation and Promotion
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | The system MUST support pluggable evaluation workflows per space. | Must |
| FR-3.2 | Agent outputs MUST pass the evaluation loop before being promoted or committed. | Must |

### FR-4: Replayability
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | The system MUST capture sufficient state (inputs, model fingerprint, configs, tool results) to allow deterministic replay of an agent session. | Must |
| FR-4.2 | Replay MUST reference the same `execution_id` lineage and produce a verifiable replay result record. | Must |

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

### NFR-3: Determinism & Auditability
| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | Replay output comparisons MUST be reproducible under identical inputs/configuration. | Must |
| NFR-3.2 | Lifecycle and replay records MUST include schema versioning for forward-compatible decoding. | Must |

---

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `nostra-core` | Workspace | `GlobalEvent` and `Proposal` contract integration |
| `cortex-runtime` | Workspace | Intercepts and wraps agent executions |
| `cortex-agents` | Workspace | Agent harness primitives and execution record models |

---

## Constraints

| Constraint | Description |
|------------|-------------|
| Modularity | Harness logic remains in Cortex (execution layer) and does not leak into Nostra (platform authority layer). |
| Governance | No global hardcoded evaluation oracle; evaluation remains space-governed. |
| Envelope Contract | Agent harness events must use canonical `GlobalEvent` rather than introducing a parallel event envelope. |
| Scope Control | V1 implementation is limited to `L0-L2` and core payload keys to prevent overbuild. |

---

## References

- [PLAN.md](./PLAN.md) - Implementation phases and mapping contract
- [DECISIONS.md](./DECISIONS.md) - Architectural decision log
- [VERIFY.md](./VERIFY.md) - Verification evidence plan
- [AGENTS.md](../../AGENTS.md) - Agent constitutional framework
