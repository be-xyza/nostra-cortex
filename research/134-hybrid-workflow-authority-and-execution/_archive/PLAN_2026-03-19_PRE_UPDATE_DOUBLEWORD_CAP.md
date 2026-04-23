---
id: "134"
name: "hybrid-workflow-authority-and-execution"
title: "Hybrid Workflow Authority and Execution"
type: "plan"
project: "nostra"
status: active
execution_plane: "cortex"
authority_mode: "recommendation_only"
reference_topics: ["agent-systems", "workflow-orchestration", "evaluation"]
reference_assets:
  - "research/reference/knowledge/agent-systems/2026_Chang_KARL"
  - "research/reference/knowledge/workflow-orchestration/2026_Liu_MASFactory"
evidence_strength: "moderate"
authors:
  - "Codex"
tags: ["workflow", "authority", "execution", "projections", "drift-review"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Workflow Architecture"
created: "2026-03-11"
updated: "2026-03-12"
---

# Initiative 134: Hybrid Workflow Authority and Execution

## Overview
Initiative 134 supersedes the assumption that [013](/Users/xaoj/ICP/research/013-nostra-workflow-engine/DECISIONS.md) is the canonical workflow architecture. It treats 013 as a historical pattern source, preserves its strongest decisions, and re-centers the architecture around Nostra authority artifacts plus Cortex execution adapters.
External pattern sources now include MASFactory for governed graph-draft and observability motifs, and KARL for trajectory and evaluation semantics that must remain downstream of workflow authority rather than replace it.

## Objective
Define and implement the greenfield workflow substrate where:
- Nostra defines workflow intents, drafts, definitions, governance, lineage, and outcomes.
- Cortex executes durable workflow instances through runtime adapters.
- Serverless Workflow is a deterministic projection/interchange format rather than the internal source of truth.
- A2UI is emitted from typed checkpoints and evaluation gates rather than embedded directly in canonical definitions.

## Architectural Principle
The canonical workflow substrate is an artifact pipeline plus execution adapter layer, not a single engine implementation.

## Phases
### Phase 0: Drift Reset
- Record the 013 supersession matrix.
- Mark 013 as a historical pattern source.
- Anchor durable execution decisions in 047 and 066.

### Phase 1: Canonical Contracts
- Add `WorkflowIntentV1`, `WorkflowDraftV1`, `WorkflowDefinitionV1`, projections, execution bindings, instances, trace, checkpoints, and outcomes.
- Mirror the proven governance patterns from 115.

### Phase 2: Deterministic Compiler
- Validate bounded motifs only.
- Emit deterministic normalized graph, Serverless Workflow projection, and A2UI/flow-graph projections.

### Phase 3: Governance Lifecycle
- Add candidate-set, staging, review, ratify/reject, replay, and active adoption patterns for workflow drafts and definitions.

### Phase 4: Execution Adapter Hardening
- Promote the local durable worker path to the first real adapter.
- Keep the canister path optional until build/runtime validation is real.

### Phase 5: Cross-Adapter Parity
- Run identical definitions across worker and canister adapters.
- Keep the canister as authority/projection service only if parity fails.

## Immediate Implementation Slice
- Successor initiative docs.
- `cortex-domain` workflow contracts, validation, synthesis, and deterministic compile.
- `cortex-runtime` workflow artifact runtime, digest, and store-key helpers.

## Dependencies
- [013](/Users/xaoj/ICP/research/013-nostra-workflow-engine/DECISIONS.md)
- [047](/Users/xaoj/ICP/research/047-temporal-architecture/DECISIONS.md)
- [066](/Users/xaoj/ICP/research/066-temporal-boundary/RESEARCH.md)
- [107](/Users/xaoj/ICP/research/107-cortex-decision-plane-hardening/PLAN.md)
- [108](/Users/xaoj/ICP/research/108-cortex-decision-plane-security-legibility/PLAN.md)
- [115](/Users/xaoj/ICP/research/115-cortex-viewspec-governed-ui-synthesis/DECISIONS.md)
- [126](/Users/xaoj/ICP/research/126-agent-harness-architecture/DECISIONS.md)
- [133](/Users/xaoj/ICP/research/133-eval-driven-orchestration/PLAN.md)

## Exit Criteria
- Canonical workflow contracts compile and test in `cortex-domain`.
- Deterministic compile emits stable projections for bounded motifs.
- Runtime artifact services can generate, persist, and reload candidate sets.
- No architecture claim assumes the dedicated `workflow_engine` canister is the canonical executor until local build/runtime parity exists.
