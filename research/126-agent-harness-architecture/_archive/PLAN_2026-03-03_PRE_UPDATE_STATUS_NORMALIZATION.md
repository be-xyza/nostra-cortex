---
id: "126"
name: "agent-harness-architecture"
title: "Agent Harness Architecture Implementation Plan"
type: "plan"
project: "nostra"
status: complete
execution_plane: "cortex"
authority_mode: "recommendation_only"
reference_topics: ["agent-systems"]
reference_assets: []
evidence_strength: "validated"
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

## Implementation Status

Implementation complete for initiative scope (`L0-L2` runtime enforcement, `L3-L4` fail-closed, lifecycle emission, replay artifacts, and sink rollout controls).

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
- [x] Align to workspace `AGENTS.md` constitutional and execution guidance.

### Phase 2: Core Primitives in Cortex
- [x] Define `AgentExecutionRecord` struct with lifecycle-correlation fields (`execution_id`, `attempt_id`, `phase`, `status`).
- [x] Implement core-v1 payload first; treat extension fields as non-blocking.
- [x] Wire `AgentExecutionRecord` emissions to canonical event stream mapping (`AgentExecutionLifecycle` + CloudEvent-compatible envelope).
- [x] Add event taxonomy update (`AgentExecutionLifecycle`) to shared standards before rollout.
- [x] Implement `Authority Level` checks in gateway path to enforce `L0-L2` behavior and `L3/L4` fail-closed.
- [x] Create and integrate evaluation-loop interface for L2 apply gating.

### Phase 3: Replay and Observability
- [x] Implement state snapshotting (input/output hash generation + model/tool fingerprints).
- [x] Persist replay artifacts with execution/attempt linkage and optional workflow replay refs.
- [x] Add targeted replay/lifecycle verification coverage in gateway and service tests.

### Out of Scope (V1)
- Full autonomous `L4` workflows.
- Cross-space escalation policy engine beyond authority ladder enforcement.
- Mandatory population of all extension fields in every lifecycle event.

---

## Verification Plan

### Automated Tests
```bash
# Core initiative test sweep
cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-desktop --lib
cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-desktop --test gateway_parity

# Gateway parity and protocol contract checks
bash /Users/xaoj/ICP/scripts/check_gateway_parity_inventory_sync.sh
bash /Users/xaoj/ICP/scripts/check_gateway_protocol_contract_coverage.sh
bash /Users/xaoj/ICP/scripts/check_gateway_contract_descriptors_strict.sh
```

### Manual Verification
1. Run a sample agent workflow at `L1` authority and verify that only a `Proposal` is created in Nostra.
2. Trigger an `Evaluation Loop` failure and confirm the agent output is gated and not promoted.
3. Replay a completed execution with fixed inputs/config and verify equivalent output snapshot hash.

---

## File Structure (Proposed)
```

## Completion Evidence (2026-02-24)
- Authority ladder behavior covered by targeted tests (`L0/L1/L2/L3/L4`) in `gateway/server.rs`.
- Lifecycle emission and sink behavior validated with service-level tests, including best-effort and fail-closed sink modes.
- Gateway parity and protocol contract coverage verified and synchronized (`inventory=158`, `contract=158`, `descriptors=158`).
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
