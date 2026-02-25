---
id: '013'
name: nostra-workflow-engine
title: 'Decisions Log: Workflow Engine'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-05'
---

# Decisions Log: Workflow Engine

**Context**: Strategic technical decisions for the Workflow Engine.

## DEC-001: Separation of Definition and Instance
*   **Context**: Need to reuse workflows.
*   **Decision**: Strictly separate `WorkflowDefinition` (Immutable Template) from `WorkflowInstance` (Stateful Execution).
*   **Status**: DECIDED

## DEC-002: Intention-First Builder
*   **Context**: Workflow editors are usually too complex for average users.
*   **Decision**: The Builder interaction model will be "Intention-First" (Prompt -> Template -> Customize), avoiding the "Blank Canvas" paralysis. Builder outputs standard Serverless Workflow definitions.
*   **Status**: DECIDED
*   **See Also**: [workflow-builder.md](./workflow-builder.md)

## DEC-003: Durable State on ICP
*   **Context**: Long-running workflows (e.g., governance votes) may take weeks.
*   **Decision**: All state must be persisted in Stable Memory (`StableBTreeMap`) to survive canister upgrades.
*   **Status**: DECIDED

## DEC-004: Explicit Context Object
*   **Context**: How do steps share data?
*   **Decision**: Use a "Traveler Context" (Key-Value map) that is passed from step to step. Steps read from Context and write results back to Context.
*   **Status**: DECIDED

## DEC-005: Agent Integration (014 Protocol)
*   **Context**: Agents need to pick up tasks from the workflow.
*   **Decision**: The Workflow Engine's `AsyncExternalOp` primitive will transparently publish jobs to the **AI Gateway**. The engine acts as the "Job Producer".
*   **Status**: DECIDED

## DEC-006: Optimization Strategy (Monolith First)
*   **Context**: Should the Workflow Engine be a separate canister or part of the Space canister?
*   **Decision**: **Logical Module within Monolith (Phase 1/2)**.
    *   **Rationale**: Atomic updates between Workflow State and Space Data are critical for v1.
*   **Status**: ENACTED

## DEC-007: Pricing Primitive (PaymentGate)
*   **Context**: How do we price services built on workflows?
*   **Decision**: **Use the `PaymentGate` Primitive**.
*   **Status**: ENACTED

## DEC-008: Adoption of Serverless Workflow Specification (CNCF)
*   **Context**: Need a robust, standard DSL for defining workflows.
*   **Decision**: **Adopt CNCF Serverless Workflow Specification (v0.8+)**.
    *   **Rationale**: Avoid inventing a bespoke DSL. Defines strict states, transitions, events, and error handling.
    *   **Implementation**: Use a Rust SDK (e.g., `serverlessworkflow`) to parse definitions.
*   **Status**: NEW

## DEC-009: A2UI for User Task Rendering
*   **Context**: The engine needs to present arbitrary forms (Vote, Review, Sign) to humans without custom frontend code for each.
*   **Decision**: **User Tasks emit A2UI Schemas**.
    *   **Mechanism**: The `UserTask` definition in the workflow includes an A2UI payload (or reference). The frontend A2UI Renderer displays it. The output is a JSON object submitted back to the engine.
*   **Status**: NEW

## DEC-010: Actor-Model Execution
*   **Context**: ICP is an actor model platform.
*   **Decision**: **Workflows Execute as Logical Actors**.
    *   Each instance is an isolated actor (state machine) that waits for signals.
    *   Aligns with `acts` and `workflow-rs` implementation patterns.
*   **Status**: DECIDED

## DEC-011: Administrative Emergency Controls ("God Mode")
*   **Context**: Workflows will inevitably get stuck (Agent failures, API downtime, locked funds). Non-technical DAO members seeing "Status: Running" for weeks have no recourse.
*   **Question**: Should Space Owners have a "Force Cancel" capability for workflows, even if "Code is Law"?
*   **Decision**: **Yes, with Constitutional Guardrails**.
    *   Space Owners (role: `SpaceAdmin`) can invoke `emergency_pause` and `emergency_cancel` on any workflow instance.
    *   **Constraint**: These actions are logged as `EmergencyIntervention` Contribution types in the Knowledge Graph.
    *   **Constraint**: Governance workflows (voting, proposals) require **multi-sig** (2-of-3 admins) for emergency actions.
    *   **Constraint**: Compensation handlers (`compensatedBy`) are triggered automatically on emergency cancel.
    *   **Rationale**: Pragmatic user experience trumps ideological purity. The audit trail maintains accountability.
*   **Status**: DECIDED
*   **See Also**: [CHALLENGES.md](./CHALLENGES.md) Question 1

## DEC-012: A2UI Renderer Investment Strategy
*   **Context**: The entire "Nostra on Nostra" experience depends on A2UI rendering quality. If the renderer feels "janky", all workflows, governance, and bounties feel like second-class citizens.
*   **Question**: Are we comfortable investing heavy engineering time into A2UI Dioxus/Tailwind components?
*   **Decision**: **Yes, A2UI Renderer is a Core Product**.
    *   The `A2UIRenderer` component is treated as a **first-party product**, not a utility.
    *   **Stack**: Dioxus + Shoelace/Tailwind for maximum portability (Web/Desktop/Mobile via WASM).
    *   **Quality Bar**: Must match or exceed native form UX (instant validation, smooth transitions, optimistic updates).
    *   **Ownership**: Dedicated engineering allocation in Phase 2 sprint capacity.
    *   **Client-Side Logic**: Implement CEL/JSON-Logic subset for field validation without server roundtrips.
*   **Status**: DECIDED
*   **See Also**: [CHALLENGES.md](./CHALLENGES.md) Question 2, [ANALYSIS_ADVANCED.md](./ANALYSIS_ADVANCED.md) Section 4

## DEC-013: Canonical Workflow Engine Canister
*   **Context**: Multiple implementation paths exist (monolith vs dedicated canister). We need a single canonical path for production alignment.
*   **Decision**: **Canonical implementation lives in the dedicated `workflow-engine` canister** at `nostra/backend/workflow_engine/`.
    *   **Note**: DEC-006 remains historical for early-phase monolith exploration, but is **superseded** for canonical implementation.
*   **Status**: DECIDED
*   **See Also**: `research/097-nostra-cortex-alignment/DECISIONS.md` (DEC-002)

## DEC-014: WorkflowOutcome Emission Contract
*   **Context**: Workflow execution must leave canonical traceable outcomes aligned with the Global Event Log.
*   **Decision**: Emit `GlobalEvent` records on workflow completion/failure and persist outcomes as Contribution-linked artifacts (schema defined in 040).
*   **Status**: DECIDED

## DEC-015: Synchronous Execution Mode (Fast Path)
*   **Context**: Service endpoint workflows (like "Sync Skills") require immediate synchronous responses, rather than long-running async execution.
*   **Decision**: Define a "Fast Path" mode for API-style workflows.
*   **Status**: DECIDED
*   **See Also**: [016-nostra-skills-sync-service-use-case](../016-nostra-skills-sync-service-use-case/RESEARCH.md)

## DEC-016: Optional Evaluator for AgentTask
*   **Context**: Semantic Merges and LLM generation steps output non-deterministic data requiring validation before the workflow progresses.
*   **Decision**: Enhance `AgentTask` primitives with an optional `Evaluator` function (a deterministic check or secondary agent validation) executed prior to step completion.
*   **Status**: DECIDED
*   **See Also**: [016-nostra-skills-sync-service-use-case](../016-nostra-skills-sync-service-use-case/RESEARCH.md)
