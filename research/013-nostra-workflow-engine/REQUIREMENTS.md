---
id: '013'
name: nostra-workflow-engine
title: 'Build Requirements: Nostra Workflow Engine'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Build Requirements: Nostra Workflow Engine

> **Status**: DRAFT
> **Source**: [RESEARCH.md](./RESEARCH.md)
> **Context**: "Nostra on Nostra" - Enabling self-building organizations.

## 1. Introduction
This document defines the technical build requirements for the Nostra Workflow Engine. The engine is a deterministic execution kernel for **Serverless Workflow** definitions, serving as the central nervous system for "Nostra on Nostra" coordination.

## 2. Functional Requirements

### 2.1 Core Engine Capabilities
- **FR-01: Deterministic State Machine**:
    - The engine MUST implement the **CNCF Serverless Workflow Specification (v0.8+)** state machine semantics.
    - Transitions must be deterministic given the same inputs and history.
- **FR-02: Workflow Definition (WFD) Management**:
    - **Format**: JSON/YAML adhering to the SW Spec.
    - **Validation**: Strict schema validation upon registration.
    - **Versioning**: Support SW Spec semantic versioning.
- **FR-03: Instance Management**:
    - **Isolation**: Each instance runs as a logical actor.
    - **Persistence**: All state (Instruction Pointer, Data Model) MUST be stored in strict Stable Memory (`StableBTreeMap`).

### 2.2 Control Flow Logic (Supported States)
The engine must support these specific SW Spec states:
- **FR-04: Operation State** (Actions): Execute System functions or External Service calls.
- **FR-05: Switch State** (Logic): Data-based branching (`if/else`).
- **FR-06: Event State** (Waits): Pause execution until a signal is received.
- **FR-07: Parallel State**: Fork/Join execution.
- **FR-08: Sleep State**: Time-based delays.

### 2.3 Task Management Ecosystem
- **FR-09: User Task (A2UI Extension)**:
    - **Extension**: We define a custom extension to the SW Spec: `x-nostra-user-task`.
    - **Payload**: This extension MUST contain or reference an **A2UI Schema** (JSON) that defines the interface.
    - **Output**: The engine provides the schema to the frontend; the frontend provides the user submission back to the engine.
- **FR-10: System Operations**:
    - Native Drivers: `Graph.Create`, `Graph.Link`, `Ledger.Transfer`.
    - Monetization: `Escrow.Lock`, `Escrow.Release`.

### 2.4 Async Workers (Agents)
- **FR-11: AsyncExternalOp**:
    - The standard `Operation` state is used for Agents.
    - **Protocol**:
        1. Engine emits `JobCreated`.
        2. Agent polls/receives job.
        3. Agent performs work.
        4. Agent calls `complete_job` with signed result.

### 2.5 Advanced Robustness Patterns
- **FR-12: Saga Pattern (Compensation)**:
    - **Context**: Critical for multi-step Ledger/Escrow operations on ICP where atomicity is not guaranteed across canisters.
    - **Requirement**: Any state with a side-effect (e.g., `LockFunds`) MUST define a `compensatedBy` transition (e.g., `UnlockFunds`).
    - **Enforcement**: The Engine MUST automatically trigger the compensation path if a subsequent step fails or times out.
- **FR-13: Sub-Workflow Composability**:
    - **Requirement**: Support the `subFlowRef` state to spawn a child workflow instance.
    - **Behavior**: Parent pauses; Child executes; Parent resumes with Child's output.
    - **Versioning**: References MUST be version-pinned (e.g., `v1.2`) to prevent breaking changes.
- **FR-14: Signal Batching (Optimization)**:
    - **Context**: High-volume voting can DOS the canister with thousands of wakeups.
    - **Requirement**: `Vote` steps must support an `accumulation_strategy` where signals are buffered and processed in batches.

## 3. Technical Requirements (ICP Architecture)

### 3.1 Data Structures & Storage
- **TR-01: Stable Storage**: **Mandatory**. No heap-only state for workflows.
- **TR-02: Interoperability**:
    - All significant outcomes (Approvals, Votes) MUST be recorded as `Contribution` entities in the Knowledge Graph.

### 3.2 Security & Governance
- **TR-03: Role Enforcement**:
    - The engine must verify that the actor performing a `UserTask` holds the `AllowedRoles` specified in the definition.
    - Role definitions are sourced from the Space Configuration.

## 4. User Experience Requirements

### 4.1 A2UI Rendering
- **UX-01**: The Frontend MUST implement a generic **A2UI Renderer** that can display Forms, Wizards, and Dashboards sent by the engine.
- **UX-02**: No custom Typescript should be required to render a new Workflow Step type.

### 4.2 Visualization
- **UX-03**: The engine must provide a "Graph View" export of the workflow definition (Nodes/Edges) for the UI.

### 4.3 Client-Side Logic
- **UX-04**: The A2UI Renderer MUST support basic client-side expression evaluation (e.g., JSON Logic or CEL) for field validation and visibility toggles to reduce latency.
