---
id: "134"
name: "hybrid-workflow-authority-and-execution"
title: "Hybrid Workflow Authority and Execution"
type: "plan"
project: "nostra"
status: archived
execution_plane: "cortex"
authority_mode: "recommendation_only"
reference_topics: ["agent-systems", "workflow-orchestration", "evaluation"]
reference_assets:
  - "research/reference/knowledge/agent-systems/2026_Chang_KARL"
  - "research/reference/knowledge/workflow-orchestration/2026_Liu_MASFactory"
  - "research/reference/knowledge/agent-systems/2026_OpenAI_Doubleword_Batch_Strategy_Transcript"
evidence_strength: "moderate"
completion_status: "complete"
archived_at: "2026-04-27"
archive_reason: "Exit criteria satisfied; canister-backed workflow execution adapter merged, live-validated, and evidence-backed."
authors:
  - "Codex"
tags: ["workflow", "authority", "execution", "projections", "drift-review"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Workflow Architecture"
created: "2026-03-11"
updated: "2026-04-27"
---

# Initiative 134: Hybrid Workflow Authority and Execution

## Completion Summary

**Status 2026-04-27**: Complete and archived.

Initiative 134 delivered the governed workflow substrate and execution-adapter pattern required for the current Nostra/Cortex slice. The canonical workflow contracts, deterministic compiler/projection surfaces, governance lifecycle, local durable worker adapter, explicit `workflow_engine_canister_v1` adapter, persisted gateway registry, workbench loading, typed frontend workflow-definition contract, ICP CLI deployment path, and live cross-adapter parity evidence are all in place.

The default runtime remains `local_durable_worker_v1`; `workflow_engine_canister_v1` remains explicit opt-in. That is an operational default decision, not an unresolved architecture gap.

## Overview
Initiative 134 supersedes the assumption that [013](/Users/xaoj/ICP/research/013-nostra-workflow-engine/DECISIONS.md) is the canonical workflow architecture. It treats 013 as a historical pattern source, preserves its strongest decisions, and re-centers the architecture around Nostra authority artifacts plus Cortex execution adapters.
External pattern sources now include MASFactory for governed graph-draft and observability motifs, KARL for trajectory and evaluation semantics that must remain downstream of workflow authority rather than replace it, and the Doubleword transcript for advisory batch-cognition patterns that must stay behind execution adapters.

## Objective
Define and implement the greenfield workflow substrate where:
- Nostra defines workflow intents, drafts, definitions, governance, lineage, and outcomes.
- Cortex executes durable workflow instances through runtime adapters.
- Serverless Workflow is a deterministic projection/interchange format rather than the internal source of truth.
- A2UI is emitted from typed checkpoints and evaluation gates rather than embedded directly in canonical definitions.
- Low-latency live cognition adapters and slower batch audit adapters remain distinct runtime lanes with shared governance boundaries.
- External batch cognition backends may appear only as execution adapters or activities that consume approved manifests and emit advisory artifacts. They are never canonical workflow or governance authority.

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
- Implement the canister execution adapter path behind explicit adapter selection.
- Keep the local durable worker as the default until live cross-adapter parity is proven.
- **Status 2026-04-27**: Complete for implementation, static verification, main-tree promotion, and live validation. `workflow_engine_canister_v1` is implemented, gateway/workbench routing is adapter-aware, and ICP CLI-first operator surfaces are aligned.

### Phase 5: Cross-Adapter Parity
- Deploy the workflow canister through the current ICP CLI path (`icp`).
- Run identical definitions across worker and canister adapters.
- Assert parity on trace, checkpoints, snapshots, cancellation, and outcomes.
- Keep the canister path gated until live gateway-to-canister execution and parity are demonstrated.
- **Status 2026-04-27**: Complete. ICP CLI deployment, direct canister lifecycle calls, gateway-to-canister execution, cancellation, fail-fast unsupported-node behavior, and local-vs-canister semantic parity were demonstrated on 2026-04-26.

## Immediate Implementation Slice
- Successor initiative docs.
- `cortex-domain` workflow contracts, validation, synthesis, and deterministic compile.
- `cortex-runtime` workflow artifact runtime, digest, and store-key helpers.
- Activity/adapter boundaries for external cognition providers used by live evaluation or audit workflows.
- Distinct boundary contracts for `LiveCognitionProvider` and `BatchAuditProvider` lanes.

## Dependencies
- [013](/Users/xaoj/ICP/research/013-nostra-workflow-engine/DECISIONS.md)
- [047](/Users/xaoj/ICP/research/047-temporal-architecture/DECISIONS.md)
- [066](/Users/xaoj/ICP/research/066-temporal-boundary/RESEARCH.md)
- [107](/Users/xaoj/ICP/research/107-cortex-decision-plane-hardening/PLAN.md)
- [108](/Users/xaoj/ICP/research/108-cortex-decision-plane-security-legibility/PLAN.md)
- [115](/Users/xaoj/ICP/research/115-cortex-viewspec-governed-ui-synthesis/DECISIONS.md)
- [126](/Users/xaoj/ICP/research/126-agent-harness-architecture/DECISIONS.md)
- [133](/Users/xaoj/ICP/research/133-eval-driven-orchestration/PLAN.md)

## Cross-Initiative Consumers

- **Initiative 132 (Eudaemon Alpha)**: Uses the workflow substrate for advisory observation and bounded synthesis passes. Hermes audit units may assess 134 readiness gates, but this is advisory review, not a blocking dependency.
- **Future initiatives**: May build on the validated adapter pattern for multi-adapter orchestration, evaluation gates, and batch audit workflows.

## Exit Criteria
- [x] Canonical workflow contracts compile and test in `cortex-domain`.
- [x] Deterministic compile emits stable projections for bounded motifs.
- [x] Runtime artifact services can generate, persist, and reload candidate sets.
- [x] Execution adapter routing supports both `local_durable_worker_v1` and explicit `workflow_engine_canister_v1`.
- [x] The dedicated `workflow_engine` canister remains an explicit adapter, with live ICP CLI deployment and cross-adapter parity evidence recorded before any default-adapter change.
- [x] Subscription-auth sidecars such as ZeroClaw remain provider brokers only and do not become workflow authority.

## Final Validation

Validation evidence:

- PR #52 merged to `origin/main` as `7fedf5741d667534386c7fbb75aad5f9a44e5f5e`.
- PR #52 CI was green before merge: Static Analysis, Rust Unit & Integration Tests, Cortex Runtime Freeze Gates, ACP Gateway Integration, Motoko Canister Tests, Test Catalog Consistency, SIQ Observe, SIQ Softgate Promotion, Initiative 118 Evidence Gate, Offline Simulations Playwright, Vercel, and canister-ID checks passed.
- Live ICP CLI deployment and direct canister lifecycle evidence is recorded under `research/134-hybrid-workflow-authority-and-execution/evidence/`.
- Gateway-mediated canister execution and local-vs-canister parity evidence is recorded under the same evidence directory.
- Hermes readiness gate `initiative-134-hermes-integration-readiness` records all five gates as `MET`.

Residual gaps after archival:

- Automated local-vs-canister gateway parity regression coverage can still be expanded in future maintenance work. This is a hardening opportunity, not an Initiative 134 completion blocker, because live parity was demonstrated and CI already covers the merged static/runtime surfaces.
- Unsupported canister node kinds remain intentionally rejected for `EvaluationGate`, `Parallel`, `Switch`, and `SubflowRef`. Future initiatives may expand the canister adapter surface, but the Initiative 134 contract requires fail-fast behavior rather than silent degradation.
- Changing the default adapter from `local_durable_worker_v1` to `workflow_engine_canister_v1` remains out of scope and would require a separate rollout decision.
