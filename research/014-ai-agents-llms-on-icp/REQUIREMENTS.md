---
id: '014'
name: ai-agents-llms-on-icp
title: 'Requirements: AI Agents on ICP'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: AI Agents on ICP

**Context**: Technical specifications for the AI Agent ecosystem within Nostra.

## Functional Requirements

### FR-1: Job Orchestration
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | Canisters must be able to queue "AI Jobs" (e.g., Summarize, Extract). | Must |
| FR-1.2 | External workers must be able to securely poll for jobs and submit results. | Must |

### FR-2: Agent Memory (KIP)
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | The Knowledge Graph must expose a schema-compliant read interface. | Must |
| FR-2.2 | Agents must be able to traverse "Entity" and "Relationship" nodes. | Must |

### FR-3: Autonomous Actions
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | "Gardener" agents must be able to read the graph without human intervention. | Should |
| FR-3.2 | Agents must NOT be able to write to the graph without human verification (initially). | Must |

### FR-4: Agent Schema (BMAD Integration)
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | **AgentPersona**: The system must store Agent definitions (Role, Principles) as nodes in the graph. | Must |
| FR-4.2 | **ContextNode**: The system must allow linking specific graph nodes (Files, Epics) to an Agent as "Sidecar Memory". | Must |
| FR-4.3 | **Role Behavior**: Agents must adhere to the `017` Role definitions (Analyst, PM, etc.) in their system prompts. | Should |
| FR-4.4 | **NDL UI Integrity**: Agents emitting A2UI payloads MUST strictly adhere to NDL Surface Classifications (Constitutional vs. Execution) to prevent governance spoofing. | Must |

## Non-Functional Requirements
- **Latency**: Async job completion is acceptable (seconds to minutes).
- **Cost**: Minimize on-chain storage of large raw text; store summaries/embeddings only.
- **Trust**: Critical operations (signing) require TEEs (Future Scope).
