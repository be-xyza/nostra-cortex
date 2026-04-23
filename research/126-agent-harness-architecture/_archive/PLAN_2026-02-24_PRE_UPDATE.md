---
id: "126"
name: "agent-harness-architecture"
title: "Agent Harness Architecture Implementation Plan"
type: "plan"
project: "nostra"
status: draft
execution_plane: "cortex"
authority_mode: "recommendation_only"
reference_topics: ["agent-systems"]
reference_assets: []
evidence_strength: "hypothesis"
handoff_target: ["cortex-runtime"]
authors:
  - "User"
tags: ["agent-harness", "governance", "temporal", "evaluation"]
stewardship:
  layer: "Cortex"
  primary_steward: ""
  domain: ""
created: "2026-02-24"
updated: "2026-02-24"
---

# Agent Harness Architecture Implementation Plan

## Overview
This initiative defines the Agent Harness Architecture for Cortex and Nostra. It formalizes Cortex as the execution environment (Agent Harness) and Nostra as the durable "system of record" (Knowledge Harness). The goal is to provide a structured environment, tools, and feedback loops for LLM-based agents, enabling reliable multi-step executions while preserving space sovereignty and constitutional authority.

---

## User Review Required

> [!IMPORTANT]
> - Ensure the 4 proposed Cortex primitives (`AgentExecutionRecord`, `Authority Levels`, `Evaluation Loop Interface`, `Replay Protocol`) align with your immediate engineering capacity.
> - Verify that `AgentExecutionRecord` structure maps correctly to the existing `GlobalEvent` envelope.

---

## Proposed Architecture

### 1. AgentExecutionRecord
A structured event stored in the `GlobalEvent` log that records the lifecycle and context of an agent session.
- **Fields**: `agent_id`, `workflow_id`, `input_snapshot_hash`, `output_snapshot_hash`, `confidence`, `authority_scope`, `promotion_level`
- **Purpose**: Provides durable observability and lineage for all agent actions.

### 2. Authority Levels (L0-L4)
An explicit escalation ladder governing agent write permissions.
- **L0**: Read-only
- **L1**: Suggestion-only (creates `Proposal` instead of direct write)
- **L2**: Limited write (space-scoped sandbox/branch)
- **L3**: Governed write (post-vote or human review)
- **L4**: Autonomous bounded workflow
- **Purpose**: Enforces the "AI agents consume Nostra data — they do not replace human judgment" principle.

### 3. Evaluation Loop Interface
A Temporal workflow attachment that gates agent outputs.
- **Flow**: Agent -> Output -> Evaluation -> Confidence Score -> Promotion Gate
- **Purpose**: Allows spaces to plug in validation rules (tests, static analysis) before an agent's work is promoted.

### 4. Replay Mechanism
Infrastructure for deterministic re-execution.
- **Components**: Input snapshot, model configuration, tool state logs.
- **Purpose**: Enables auditability, testing evaluation loops, and ensuring "History is Sacred".

---

## Implementation Phases

### Phase 1: Conceptual Formalization
- [x] Draft `PLAN.md` to define architecture and primitives
- [x] Draft `REQUIREMENTS.md`
- [ ] Establish explicit `AGENTS.md` guidelines for harness behavior

### Phase 2: Core Primitives in Cortex
- [ ] Define `AgentExecutionRecord` struct and wire it to Nostra's `GlobalEvent` stream.
- [ ] Implement `Authority Level` checks in Cortex router to enforce L1 (Proposal) vs L3 (Direct Write).
- [ ] Create base `EvaluationLoop` trait/interface for Temporal workflows.

### Phase 3: Replay and Observability
- [ ] Implement state snapshotting (Input/Output hash generation).
- [ ] Build basic deterministic replay runner for a specific `AgentExecutionRecord`.

---

## Verification Plan

### Automated Tests
```bash
# Verify AgentExecutionRecord serialization
cargo test -p cortex-agents test_agent_execution_record_serialization

# Verify Authority Level enforcement (L1 generates Proposal)
cargo test -p cortex-runtime test_authority_ladder_enforcement
```

### Manual Verification
1. Run a sample agent workflow at `L1` authority and verify that only a `Proposal` is created in Nostra.
2. Trigger an `Evaluation Loop` failure and confirm the agent output is gated and not promoted.

---

## File Structure (Proposed)
```
cortex/
├── packages/
│   ├── cortex-agents/
│   │   ├── src/harness/
│   │   │   ├── execution_record.rs
│   │   │   ├── authority.rs
│   │   │   └── replay.rs
│   └── cortex-runtime/
│       ├── src/evaluation/
```
