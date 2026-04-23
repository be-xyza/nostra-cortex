---
id: "133"
name: "eval-driven-orchestration"
title: "Eval-Driven Orchestration & Technical Enrichment Integration"
type: "plan"
project: "nostra"
status: active
execution_plane: "cortex"
authority_mode: "recommendation_only"
reference_topics: ["agent-systems", "evaluation", "orchestration", "a2ui"]
reference_assets:
  - "research/reference/knowledge/agent-systems/2026_Chang_KARL"
  - "research/reference/knowledge/workflow-orchestration/2026_Liu_MASFactory"
  - "research/reference/knowledge/agent-systems/2026_OpenAI_Doubleword_Batch_Strategy_Transcript"
evidence_strength: "draft"
handoff_target: ["cortex-runtime"]
authors:
  - "User"
tags: ["eval-loop", "subagents", "workflow-engine", "a2ui"]
stewardship:
  layer: "Cortex"
  primary_steward: "Systems Steward"
  domain: "Agent Harness"
created: "2026-03-08"
updated: "2026-03-19"
---

# Initiative 133: Eval-Driven Orchestration

## Overview
This initiative formalizes the "Eval-Driven Orchestration" paradigm, originally inspired by external "skill-creator" benchmarking scripts, into native Nostra Workflow and Cortex A2UI primitives. It bridges the Agent Harness (126), Workflow Engine (013), and Cortex Web Architecture (123) by treating subagent empirical evaluation as a first-class, durable platform capability rather than terminal script output.
External pattern sources now include KARL for multi-capability evaluation and trajectory semantics, MASFactory for governed workflow-draft and graph-observability patterns, and the Doubleword transcript for pass-based batch cognition plus meta-evaluation patterns that must remain downstream of workflow authority.

## Objective
To convert theoretical agent evaluation loops into production-grade Nostra primitives. This encompasses formalizing the **Evaluation Loop (Grader) Matrix**, extending the CNCF Serverless Workflow Engine to support comparative agent generation using **Parallel States**, and projecting structured feedback collection natively via **A2UI**.

## Technical Enrichment Opportunities (Workstreams)

### A. Formalizing the Evaluation Loop
Extend the conceptual Evaluation Loop from Initiative 126 into a strict assertion matrix.
- **Goal**: Transition from opaque human review to empirical, structured grading before an agent output reaches L3 promotion.
- **Mechanism**: When an agent produces an output snapshot, the gateway automatically executes a predefined Grader Workflow. This workflow evaluates the output against prompt requirements, computes latency, measures token costs, and emits an `AgentBenchmarkRecord` (Pass/Fail, Latency, Cost).

### B. Parallel Workflow State Exploration
Map the A/B comparative agent execution natively to the serverless Workflow Engine (Initiative 013).
- **Goal**: Enable the workflow engine to split execution context and instantiate multiple LLM sessions (subagents) with divergent system prompts simultaneously, utilizing the standard CNCF SW Spec `Parallel` state.
- **Mechanism**: Wrap multiple `AsyncExternalOp` agent calls in a `Parallel` workflow state. The engine runs N agents concurrently, waits for their completion snapshot, runs the Grader Matrix (DEC-013-016 Optional Evaluator) on all outputs, and either auto-selects the highest scorer or suspends for A2UI human selection.

### C. A2UI Feedback Projection (Evaluation Viewer)
Eliminate the need for detached local HTTP servers (like `eval-viewer`) by projecting the evaluation matrix natively to Cortex Web (Initiative 123).
- **Goal**: Provide a highly structured interface for the human steward to grade, compare, and provide routing feedback to agents natively within the Nostra client.
- **Mechanism**: When a workflow suspends in an A/B Agent state, it emits an A2UI payload containing the prompt, conflicting outputs, and the Grader Matrix scores. The user submits structured feedback (`feedback.json` equivalent), which resumes the workflow and routes the feedback back to the mutating agent.

### D. Skill "Trigger Optimization" Pipeline (CI/CD)
Extend the Skill Sync Service (Initiative 016) with an automated precision gate for the Nostra graph vocabulary.
- **Goal**: Guarantee routing precision across the ecosystem as the number of skills grows, preventing overlapping context triggers.
- **Mechanism**: Before a new Workflow, Intent, or Skill is deployed to the canonical graph, it must pass a "Trigger Optimization Matrix." A headless worker generates 20 positive and negative queries, proving the target skill's description triggers the model >90% of the time without false positives.

### E. Batch Cognition Audit Overlay
Use external batch backends for large-scale advisory analysis without letting them become workflow authority.
- **Goal**: Score structural, constitutional, risk, and optimization findings over typed audit units while keeping the results governable and replay-linked.
- **Mechanism**: A workflow emits a `SourceManifestV1` plus `AuditUnitV1` set, an execution adapter submits the independent units as an `AuditBatchRequestV1` to a batch backend, a meta-evaluation pass scores correctness/novelty/feasibility/contradiction density, and the resulting `AuditFindingV1` / `AuditSynthesisV1` evidence is routed back to Eudaemon or the steward for synthesis.
- **Boundary**: This workstream is secondary to the primary live cognition lane in Initiative 132 and must not become boot-critical for Phase 6 communication.

## Cross-Initiative Dependencies
| Initiative | Dependency Type | Relationship |
|---|---|---|
| `013-nostra-workflow-engine` | Mutates | Uses CNCF `Parallel` State mapped to `AsyncExternalOp` with `Evaluator` functions |
| `123-cortex-web-architecture` | Consumes | A2UI rendering for Evaluation Viewer |
| `126-agent-harness-architecture` | Extends | Implements exact Grader payload extension for the L2 evaluation gate (`AgentBenchmarkRecord`) |

## Exit Criteria
1. SW Spec `Parallel` state wrapping comparative subagents is validated in the engine.
2. The Evaluation Matrix / `AgentBenchmarkRecord` struct is added to the gateway.
3. A2UI schemas for the Feedback Projection screen (`feedback.json` UI) are codified.
4. The Trigger Optimization pipeline is defined as a standard CI script for local skill refinement.
5. Batch cognition results can be meta-evaluated and routed back into governed workflow evidence without bypassing steward review or workflow authority.
6. Typed `SourceManifestV1` and `AuditUnitV1` contracts are defined for the advisory batch lane.
