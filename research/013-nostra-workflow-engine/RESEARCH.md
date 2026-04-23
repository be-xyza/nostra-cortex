---
id: '013'
name: nostra-workflow-engine
title: 'Research Initiative: Nostra Workflow Engine'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-17'
updated: '2026-02-17'
---

# Research Initiative: Nostra Workflow Engine

## 1. Core Objective
**To design and implement a composable, modular, and seamless Workflow Engine that powers the "Nostra on Nostra" ecosystem.**

The engine must automate and guide the complex processes of research, planning, and development, while abstracting this complexity away from the end-user. It should provide a "GPS-like" experience: users are guided to their destination (contribution) without needing to understand the mechanics of the route, yet they can zoom out to see the full map.

> [!IMPORTANT]
> **Interoperability Contract**: Workflows enable the *process*, but Contributions are the *result*. All significant steps (e.g., Voting, Reviewing) must produce a durable [Contribution Object](../008-nostra-contribution-types/PLAN.md) to ensure the Knowledge Graph reflects the history of work.

## 2. Problem Statement
The "Nostra on Nostra" vision requires coordinating diverse actors (Users, Architects, Developers, AI Agents) across complex lifecycles (Inception -> Exploration -> Materialization).
*   **Current State**: Ad-hoc coordination, manual handoffs, lack of visibility.
*   **Desired State**: Defined but flexible paths, automated transitions, clear "Next Actions".

## 3. Key Requirements

### 3.1. Composability & Modularity
The engine must not be a monolith of hardcoded logic.
*   **Atomic Actions**: Reusable units of work.
*   **Workflow Primitives**: Standard patterns (e.g., "Approval Chain").
*   **Dynamic Binding**: Workflows should be able to bind to different contexts.

### 3.2. Seamless End-User Interaction ("The GPS Experience")
*   **No Configuration Hell**: Users shouldn't need to write YAML.
*   **Context-Aware**: Users see a simple list of actions.
*   **Automated Transitions**: Move state forward automatically where possible.

### 3.3. High-Level View & Auditability ("The Map")
*   **Process Visualization**: A high-level visual representation (Quest Log / Subway Map).
*   **Audit Trail**: Every state change is recorded.
*   **Feedback Loop**: Users can propose optimizations to the process itself.

## 4. Proposed Architecture (Updated 2026-01-21)

This architecture integrates **Temporal's Durable Execution** model with **n8n/Node-RED's operational patterns**.

### 4.0 Architectural Pillars
1.  **Durable Execution** (Temporal): "If it's not in the History, it didn't happen."
2.  **Node Execution Stack** (n8n): Stack-based tracking of pending node operations.
3.  **Subflow Encapsulation** (Node-RED): Workflows as first-class reusable objects.
4.  **Expression Sandboxing** (n8n): Secure evaluation of user logic.


### 4.1. The Declarative Core (Serverless Workflow Spec)
We adopt the **CNCF Serverless Workflow Specification (v0.8+)** as the canonical format for Workflow Definitions (WFD).
*   **Why**: Standardization, robust state definition, event handling, and existing tooling support.
*   **Implementation**: The Engine parses these JSON/YAML definitions. We do **not** invent a custom DSL.

### 4.2. Execution Model (Actor-First)
*   **Logical Actors**: Each Workflow Instance behaves as an autonomous actor.
*   **State Machine**: A deterministic state machine (inspired by `workflow-rs`) manages transitions.
*   **Persistence**: Instance state is persisted via `IPersistenceAdapter`.
    *   *Default Adapter*: `StableBTreeMap` (ICP Stable Memory).

### 4.3. The Interaction Layer (A2UI)
*   **Problem**: How to render diverse tasks (Voting, Reviewing, Filling Forms) without custom frontend code for each?
*   **Solution**: **A2UI (Agent-to-User Interface)**.
*   **Implementation**:
    1.  **Protocol**: Agents emit **AG-UI JSON** intents (not raw HTML).
    2.  **Adapter**: The `AgentUIAdapter` maps intents to **Lit + Shoelace** components.
    3.  **Visualization**: Complex graphs use **D3.js** (Semantic/Interactive).
*   **Deep Dive**: See [Study: Space Dashboard & Knowledge Orchestration](./STUDY_SPACE_DASHBOARD.md) for the definition of the primary orchestration UI.
*   **Mechanism**:
    1.  Engine encounters a `UserTask`.
    2.  Engine produces an **AG-UI Message** (Form, Wizard, Dashboard) based on the task definition.
    3.  Frontend's **generic A2UI Renderer** displays the UI using **Shoelace components**.
    4.  User interacts and submits data.
    5.  Engine validates and transitions.

### 4.4. The Agent Layer (Standardized Actors)
*   **Standard**: Agents must expose `input_schema` and `output_schema` (JSON Schema) per `nostra-system-standards`.
*   **Handshake**: Engine reads Agent Schema -> Generates Input Form (A2UI) -> Submits Job.
*   **AsyncExternalOp**: Standard primitive for off-chain tasks.
*   **Agent Control Protocol (ACP)**:
    *   **Worker Implementation**: The `nostra_worker` implements a Rust version of `AcpConnection`.
    *   **Worker Implementation**: The `nostra_worker` implements a Rust version of `AcpConnection`.
    *   **Session Management**: Long-running agent sessions are managed as Temporal Workflows, maintaining the `sessionId` state and handling `session/prompt` signals.

### 4.4.1 Recursive Delegation (The "Boss" Pattern)
Derived from Agent Zero research, an Agent is not just an endpoint but a node in a hierarchy.
*   **System Call**: `spawn_subordinate(profile: AgentProfile, task: Text)`
*   **Mechanism**:
    1.  Parent Agent (Workflow A) calls `ChildWorkflow` (Workflow B).
    2.  Workflow B initializes with `ContextID` but new `MemoryID`.
    3.  Workflow B reports results back to A.
*   **Use Case**: A "Research Agent" spawns 3 "Search Agents" and 1 "Writer Agent" to produce a report.

### 4.5. Operational Patterns (Derived from n8n/Node-RED)
*   **Waiting Execution Registry**: Explicit state tracking for nodes waiting on multiple inputs (e.g. Merge nodes), avoiding "race to execute" bugs.
*   **Catch/Status Nodes**: First-class error handling nodes (`OnError`) that trap failures from their scope, inspired by Node-RED.
*   **Partial Execution**: Ability to "Resume from Checkpoint" by loading a persisted `NodeExecutionStack`.

### 4.6 The Function Layer (Inline Imperative Logic)
> Introduced via [Research 068](../068-clawdbot-integration-feasibility/RESEARCH.md) to bridge High-Level Declarative and Low-Level Imperative.

We introduce an **Inline Function Layer** to handle "glue logic" (JSON transformation, simple conditionals) without the overhead of async agents.

*   **Primitive**: `#inline_function` Step Type.
*   **Runtime**: Wasm-based JS/TS Interpreter (e.g. Boa, QuickJS) running *inside* the Engine Canister.
*   **Constraint**: Strictly deterministic, no network access, bounded fuel (`AbstractResourceUnits`).
    *   *Implementation*: Maps to Wasm Instructions or generic "Gas" before converting to Cycles.
*   **Use Case**: "If `user_input.contains('crisis')`, route to `emergency_flow`, else `standard_flow`."

### 4.7 Ingest Pipelines (Elasticsearch Adaptation)
Derived from Elasticsearch's `IngestNode` pattern, we define a specialized, high-throughput workflow type for data entry (RAG, ETL).
*   **Structure**: A linear chain of atomic `Processors` (Sanitize, Redact, Embed, Enrich).
*   **Requirement**: These execute *before* the data enters the System (Knowledge Graph/Vector Store).
*   **Configuration**: Defined via standard JSON pipelines, separate from complex Business Logic workflows.
*   **Use Case**: Agent "Thinking" loop (User Prompt -> Sanitizer -> Vector Embed -> Context Retrieval -> LLM).

### 4.8 Observability & Distributed Tracing
*   **Trace Context**: The Engine MUST propagate W3C `traceparent` headers to all executed Actions/Activities.
*   **Signals**: Every State Transition emits a standard OTel Span named `WorkflowState:{StateName}`.
*   **Correlation**: `trace_id` is linked to `workflow_execution_id` to allow debugging a full workflow across the cluster.

## 5. Use Case Scenarios

### Scenario A: The Feature Request (Standard)
1.  **User** reports a bug.
2.  **Engine** loads "Bug Triage" WFD (Serverless Workflow JSON).
3.  **Agent** attempts reproduction (AsyncOp).
4.  **Developer** gets `UserTask` -> A2UI renders a "Fix Submission" form.
5.  **User** gets `UserTask` -> A2UI renders a "Verify Fix" button.

### Scenario B: The Governance Proposal (High Stakes)
1.  **Member** proposes policy.
2.  **Engine** enforces strict states (Discussion -> Vote -> Delay -> Execution).
3.  **Community** votes via A2UI "Ballot" component.
4.  **Engine** tallies and executes.

## 6. Functional Primitives (Mapped to Spec)

We map Nostra's needs to Serverless Workflow constructs:

*   **States**: `Event`, `Operation` (Async), `Switch` (Branch), `Parallel`, `InlineFunction` (New).
*   **Components**:
    *   `UserTask` (Standard SW extension) -> Emits A2UI.
    *   `Functions` -> Map to System Ops (`Graph.Link`, `Ledger.Transfer`).
    *   `Events` -> Map to Nostra Events (`OnGraphChange`).
    *   `Script` -> Map to `#inline_function`.

## 7. Next Steps

> [!NOTE]
> See [PLAN.md](./PLAN.md) for the phased execution roadmap.

1.  **Foundation**: Implement minimal Serverless Workflow state machine in Rust.
2.  **Function Runtime**: Integrate Boa/QuickJS for `#inline_function`.
3.  **A2UI Renderer**: creating the Dioxus component to render schemas.
4.  **Governance Primitives**: Add Custom Types for Voting and Escrow.
5.  **Dashboard Schema**: Define `DashboardDefinition` schema for modular space layouts (see [Study](./STUDY_SPACE_DASHBOARD.md)).

## 8. Use Cases

This initiative includes specific use case implementations that consume the Workflow Engine:

| Use Case | Description | Location |
| :--- | :--- | :--- |
| **Nostra on Nostra** | Multi-user/agent orchestration for collaborative research & development of the Nostra platform itself. The "Innovation Loop" workflow. | [use-cases/nostra-on-nostra/](./use-cases/nostra-on-nostra/) |
| **Personal OS** | Individual user workflow (`Capture -> Route -> Store`). | [012-nostra-bootstrap-protocol](../012-nostra-bootstrap-protocol/) |

## 9. Workflow Constitutional Compliance

> [!IMPORTANT]
> Workflows are not above the law. All workflow operations that mutate entities must respect the **Constitutional Schema Constraints** defined in the Knowledge Graph.

### 9.1 Constitutional Integration

The Workflow Engine integrates with the [Constitutional Schema](../specs/constitutional-schema.md) system to enforce behavioral invariants:

1. **Pre-Execution Validation**: Before any `SystemOp` that creates or modifies entities, the engine validates the operation against the target schema's `constraints` block.
2. **State Machine Enforcement**: The `#stateMachine` constraint type is natively enforced by workflow state transitions.
3. **Immutability Protection**: Fields marked `#immutable` cannot be modified by any workflow step.
4. **Append-Only Enforcement**: Arrays marked `#appendOnly` can only have `#push` actions.

### 9.2 Workflow Definition Schema Enhancement

WorkflowDefinitions SHOULD include a `constitutionalPolicy` field:

```yaml
name: "Grant Application Workflow"
version: "1.0.0"
constitutionalPolicy:
  # Reference to constitutional framework documents
  references:
    - "nostra://constitutions/knowledge"
    - "nostra://constitutions/process"
  # Entity types this workflow may create/modify
  entityScope:
    - "nostra.contribution"
    - "nostra.proposal"
  # Override behavior (default: REJECT)
  onViolation: "REJECT" | "WARN" | "AUDIT_ONLY"
```

### 9.3 Enforcement Hierarchy

| Layer | Responsibility | Enforcement |
|:------|:---------------|:------------|
| **Schema Registry** | Define constraints | Static (on registration) |
| **KG Engine** | Validate mutations | Dynamic (on write) |
| **Workflow Engine** | Respect entity rules | Runtime (before SystemOp) |
| **A2UI Renderer** | Surface validations | Client-side (on input) |

### 9.4 Emergency Override Protocol

Per [DEC-011](./DECISIONS.md#dec-011-administrative-emergency-controls-god-mode), administrative emergency controls (`emergency_pause`, `emergency_cancel`) MUST:

1. Log as `EmergencyIntervention` Contribution type
2. Require multi-sig for governance workflows
3. Trigger compensation handlers automatically
4. Respect constitutional audit requirements

### 9.5 Constitutional References

- [Constitutional Schema Spec](../specs/constitutional-schema.md)
- [Nostra Constitutional Framework](./_bmad/docs/nostra-constitutional-framework.md)
- [DEC-011: Emergency Controls](./DECISIONS.md)

## 10. Implementation Reference
> [!NOTE]
> A reference implementation of the "Off-Chain Workflow Engine" (Parsing Markdown, executing Shell/DFX commands) is availble in **Cortex Desktop v0.2.0**.
> See [`cortex-desktop/src/services/workflow_executor.rs`](../../cortex/apps/cortex-desktop/src/services/workflow_executor.rs).
