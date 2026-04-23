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
This initiative defines the Agent Harness Architecture for Cortex and Nostra. It formalizes Cortex as the execution environment (Agent Harness) and Nostra as the durable system of record (Knowledge Harness). The goal is to provide a structured environment, tools, and feedback loops for LLM-based agents, enabling reliable multi-step executions while preserving space sovereignty and constitutional authority.
It extends initiative `122-cortex-agent-runtime-kernel` from MVK architecture into governed runtime contracts without replacing 122's loop model.

---

## User Review Required

> [!IMPORTANT]
> - Ensure the 4 proposed Cortex primitives (`AgentExecutionRecord`, `Authority Levels`, `Evaluation Loop Interface`, `Replay Protocol`) align with immediate engineering capacity.
> - Confirm `AgentExecutionRecord` -> `GlobalEvent` mapping contract below is acceptable as the canonical envelope strategy.

---

## Proposed Architecture

### 1. AgentExecutionRecord
A structured record emitted as lifecycle events in the canonical `GlobalEvent` stream.
- **Core v1 fields (required)**: `schema_version`, `execution_id`, `attempt_id`, `agent_id`, `workflow_id`, `phase`, `status`, `authority_scope`, `input_snapshot_hash`, `output_snapshot_hash`, `timestamp`
- **Extension fields (optional in v1)**: `space_id`, `model_fingerprint`, `tool_state_hash`, `confidence`, `promotion_level`, `started_at`, `ended_at`
- **Purpose**: Provides durable observability, deterministic lifecycle correlation, and lineage for all agent actions.

### 2. Authority Levels (L0-L4)
An explicit escalation ladder governing agent write permissions.
- **L0**: Read-only
- **L1**: Suggestion-only (creates `Proposal` instead of direct write)
- **L2**: Limited write (space-scoped sandbox/branch)
- **L3**: Governed write (post-vote or human review)
- **L4**: Autonomous bounded workflow
- **v1 cutline**: Runtime implementation scope is `L0-L2`; `L3-L4` are schema-defined but deferred for later initiatives.
- **Purpose**: Enforces the "AI agents consume Nostra data; they do not replace human judgment" principle.

### 3. Evaluation Loop Interface
A Temporal workflow attachment that gates agent outputs.
- **Flow**: Agent -> Output -> Evaluation -> Confidence Score -> Promotion Gate
- **Purpose**: Allows spaces to plug in validation rules (tests, static analysis) before an agent's work is promoted.

### 4. Replay Protocol
Infrastructure for deterministic re-execution.
- **Components**: Input snapshot, model configuration fingerprint, tool-state log, and bounded environment metadata.
- **Purpose**: Enables auditability, replay testing of evaluation loops, and preservation of event lineage.

### GlobalEvent Envelope Mapping Contract
`AgentExecutionRecord` MUST be emitted through the canonical `GlobalEvent` envelope (`shared/specs.md`).
- `GlobalEvent.id`: unique event UUID per emission.
- `GlobalEvent.source`: `#Agent(agent_id)` for agent-originated records.
- `GlobalEvent.type`: `AgentExecutionLifecycle`.
- `GlobalEvent.resource`: `nostra://workflow/<workflow_id>/execution/<execution_id>`.
- `GlobalEvent.payload`: versioned `AgentExecutionRecord` payload (minimum keys: `schema_version`, `execution_id`, `attempt_id`, `phase`, `status`, `authority_scope`, `input_snapshot_hash`, `output_snapshot_hash`, `timestamp`).

### Alignment and Non-Overlap
- **Relationship to 122**: Initiative 126 extends `122-cortex-agent-runtime-kernel` with governance contracts, replay protocol, and lifecycle observability.
- **No overlap intent**: 126 does not replace 122's MVK loop or introduce planner/evaluator framework complexity outside the defined evaluation gate interface.
- **Event taxonomy control**: `AgentExecutionLifecycle` must be registered in `shared/specs.md` before production adoption.

---

## Implementation Phases

### Phase 1: Conceptual Formalization
- [x] Draft `PLAN.md` to define architecture and primitives.
- [x] Draft `REQUIREMENTS.md`.
- [x] Create `DECISIONS.md` and `VERIFY.md` scaffolding for initiative continuity.
- [ ] Establish explicit `AGENTS.md` guidance for harness behavior.

### Phase 2: Core Primitives in Cortex
- [ ] Define `AgentExecutionRecord` struct with lifecycle-correlation fields (`execution_id`, `attempt_id`, `phase`, `status`).
- [ ] Implement core-v1 payload first; treat extension fields as non-blocking.
- [ ] Wire `AgentExecutionRecord` emissions to Nostra `GlobalEvent` stream using the mapping contract.
- [ ] Add event taxonomy update (`AgentExecutionLifecycle`) to shared standards before rollout.
- [ ] Implement `Authority Level` checks in Cortex router to enforce L1 (`Proposal`) vs L3 (governed direct write).
- [ ] Create base `EvaluationLoop` trait/interface for Temporal workflows.

### Phase 3: Replay and Observability
- [ ] Implement state snapshotting (input/output hash generation + model/tool fingerprints).
- [ ] Build deterministic replay runner for a specific `AgentExecutionRecord`.
- [ ] Add replay verification cases that assert deterministic outcomes for fixed inputs/config.

### Out of Scope (V1)
- Full autonomous `L4` workflows.
- Cross-space escalation policy engine beyond authority ladder enforcement.
- Mandatory population of all extension fields in every lifecycle event.

---

## Verification Plan

### Automated Tests
```bash
# Verify AgentExecutionRecord serialization in cortex workspace
cargo --manifest-path cortex/Cargo.toml test -p cortex-agents test_agent_execution_record_serialization

# Verify Authority Level enforcement (L1 generates Proposal)
cargo --manifest-path cortex/Cargo.toml test -p cortex-runtime test_authority_ladder_enforcement

# Verify GlobalEvent envelope mapping contract for lifecycle emissions
cargo --manifest-path cortex/Cargo.toml test -p cortex-runtime test_agent_execution_global_event_mapping
```

### Manual Verification
1. Run a sample agent workflow at `L1` authority and verify that only a `Proposal` is created in Nostra.
2. Trigger an `Evaluation Loop` failure and confirm the agent output is gated and not promoted.
3. Replay a completed execution with fixed inputs/config and verify equivalent output snapshot hash.

---

## File Structure (Proposed)
```
cortex/
├── libraries/
│   ├── cortex-agents/
│   │   └── src/harness/
│   │       ├── execution_record.rs
│   │       ├── authority.rs
│   │       └── replay.rs
│   └── cortex-runtime/
│       └── src/evaluation/
```
