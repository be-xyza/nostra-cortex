---
id: "132"
name: "eudaemon-alpha-initiative"
title: "Eudaemon Alpha - Institutional Agent Readiness Plan"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authority_mode: "recommendation_only"
execution_plane: "cortex"
reference_topics:
  - "agent-systems"
  - "workflow-orchestration"
  - "evaluation"
reference_assets:
  - "research/reference/knowledge/agent-systems/2026_Chang_KARL"
  - "research/reference/knowledge/workflow-orchestration/2026_Liu_MASFactory"
authors:
  - "User"
  - "Codex"
tags:
  - "agents"
  - "institution"
  - "heap"
  - "workflow"
  - "governance"
depends_on:
  - "080"
  - "121"
  - "122"
  - "124"
  - "126"
  - "127"
  - "130"
  - "133"
  - "134"
structural_pivot_impact: "major"
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Institutional Agent Architecture"
created: "2026-03-07"
updated: "2026-03-12"
---

# Initiative 132: Eudaemon Alpha

## Overview

Initiative 132 establishes Eudaemon Alpha as the first institutional research agent aligned to the current Nostra/Cortex runtime stack. The objective is not to invent a new workspace or planning model. The objective is to bind the agent to the primitives already implemented or actively governed in the portfolio.

At this stage:
- Nostra remains authority for initiatives, contributions, DPub lineage, and institutional identity.
- Cortex remains authority for heap workspaces, lifecycle execution, closeout tracking, action/navigation compilation, and workflow runtime behavior.
- Initiative 134 supersedes treating older workflow-engine assumptions as the canonical workflow architecture.

## Architecture Resolution

The current work progression model for 132 is:

1. Notes and exploratory work:
   - Use the heap runtime.
   - Canonical contracts:
     - `POST /api/cortex/studio/heap/emit`
     - `GET /api/cortex/studio/heap/blocks`
     - `POST /api/cortex/studio/heap/blocks/context`

2. Tasks:
   - Use contribution-scoped closeout ledgers for concrete follow-through.
   - Use workflow/user-task checkpoints only for durable execution paths.
   - Do not introduce a bespoke Eudaemon task primitive.

3. Plans:
   - Use action/navigation plans for UI and operator affordances.
   - Use the workflow artifact pipeline for executable plans:
     - `WorkflowIntentV1`
     - `WorkflowDraftV1`
     - `WorkflowDefinitionV1`
     - `WorkflowInstanceV1`

4. Initiatives:
   - Keep Initiative 132 as a governed research artifact with frontmatter, dependencies, stewardship, and status-index registration.

5. References:
   - Treat KARL and MASFactory as pattern sources registered under `research/reference/knowledge/`.
   - Do not treat external references as authority over local workflow, heap, or governance contracts.

See `WORK_PRIMITIVES_ARCHITECTURE.md` for the full readiness analysis.

## Scope

1. Bind Eudaemon to heap-mode workspace contracts from Initiative 124.
2. Bind Eudaemon to lifecycle and authority constraints from Initiative 126.
3. Restrict code and file operations to the sandbox and memory-fs boundaries from Initiatives 121 and 127.
4. Use Initiative 130 capability overlays and compiled plans as the space activation model.
5. Use Initiative 134 as the governing workflow substrate for executable plans and future durable execution migration.
6. Preserve DPub lineage and promotion paths for chronicle outputs under Initiative 080.

## Out of Scope

1. Inventing a new Eudaemon-only workspace primitive.
2. Treating research initiative plans as executable workflow plans.
3. Giving external references or upstream model behavior authority over local governance contracts.
4. Autonomous promotion beyond recommendation-only boundaries.

## Delivery Phases

### Phase A: Governance Normalization
- Add governed initiative metadata and dependency clarity for 132.
- Align README and supporting architecture docs to the current primitive stack.
- Register all active successor dependencies needed before implementation continuation.

### Phase B: Workspace and Context Binding
- Emit and read heap blocks through canonical heap endpoints.
- Treat heap blocks as exploratory working material that can later be promoted.
- Use heap context bundles rather than ad hoc workspace scans as the default context packaging path.

### Phase C: Lifecycle and Task Binding
- Emit `AgentExecutionLifecycle` records under Initiative 126.
- Link follow-through work to closeout task ledgers where applicable.
- Reserve workflow/user-task checkpoints for durable execution slices rather than generic note-taking.

### Phase D: Planning and Promotion
- Route executable plans through the workflow artifact pipeline defined by Initiative 134.
- Use compiled navigation/action plans only as UI/runtime guidance surfaces.
- Promote mature outputs from heap notes into proposals, DPub updates, workflow drafts, or other governed contributions.

### Phase E: Runtime Migration Readiness
- Keep external runtime adapters provisional.
- Evaluate migration from external agent host to native Cortex execution only through parity-backed slices.
- Refuse architecture claims that bypass the governed workflow/runtime stack already defined in 124, 126, 130, 133, and 134.

## Exit Criteria

1. Initiative 132 is portfolio-consistent and structurally governed.
2. Eudaemon documentation no longer conflicts with the canonical heap/runtime/workflow architecture.
3. The agent workspace model clearly uses heap notes for working material and governed promotion paths for durable outputs.
4. The task and planning model clearly distinguishes closeout/workflow tasks from initiatives and notes.
5. Initiative 132 is ready to continue implementation against current Cortex contracts rather than stale architecture assumptions.

### 6.2 LLM API Connection
Replace the mock `AnalysisResult` in `PatternDetector.analyze()` with a real LLM API call:
- Load the system prompt from `config/system_prompt.md` (already implemented).
- Format the payload: `{"model": route.model, "system": active_prompt, "messages": [{"role": "user", "content": bundle_context + code_context}]}`.
- Wire the response parser to extract `score`, `rationale`, `proposed_nodes` from the LLM output.
- Track actual `latency_ms` and `token_cost` in the `AgentBenchmarkRecord`.

### 6.3 Gateway Connection
Flip the `HeapClient` from graceful-fallback mode to live:
- Test `GET /heap/context` → `GraphContextBundle` deserialization.
- Test `POST /heap/emit` → `ProposalBlock` / `ExecutionRecordBlock` emission.
- Test `GET /heap/blocks` → `AgentSolicitationBlock` polling for the bootstrap wizard.
- Validate that the agent correctly ingests steward comments from the next context bundle.

### 6.4 Graph-Native PROMPT Seeding
- Create the first real `ContributionType.PROMPT` node for the Code Analysis system prompt on the Heap.
- Test that `PatternDetector` correctly fetches and uses `prompt_override` from the Heap instead of the local `config/system_prompt.md`.
- Validate the agent gracefully falls back to the local file when the PROMPT node is unavailable.

---

## Phase 7: Strategic Enrichment (Initiative 133 Capstone)

### 7.1 A2UI Feedback Projection (Init 133 Workstream C)
When the Cortex Web A2UI rendering is available:
- After emitting a `ProposalBlock`, the workflow suspends using Temporal's `wait_condition` (pattern proven in `AgentBootstrapWorkflow`).
- The Gateway emits an A2UI payload containing the proposal, the `AgentBenchmarkRecord`, and a structured feedback form.
- When the steward submits feedback, the workflow resumes with the `feedback.json` payload.
- The agent incorporates structured feedback into the next cycle's analysis context.

### 7.2 Parallel A/B Subagent Execution (Init 133 Workstream B)
Requires real LLM APIs (Phase 6.2 must be complete):
- `ExecutionStrategy` forks into a `Parallel` workflow state (CNCF SW Spec).
- Two `PatternDetector` instances run with divergent system prompts (e.g., "conservative analysis" vs. "exploratory analysis").
- The `Grader` evaluates both outputs and selects the empirically better one.
- `ResourceGovernor` enforces that both parallel branches share the same cumulative spend counter.

### 7.3 Chronicle DPub Integration (Initiative 080)
- Wire `ChronicleWriter` to emit DPub chapters to the Heap.
- Implement the "Living Layer" daily update of `HEAD` chapters.
- Implement the "Publish Edition" Temporal workflow triggered monthly by the steward.
- Validate Merkle-dag root hash freezing of monthly insights.

### 7.4 Capability Audit Workflow
Implement the self-scaffolding pipeline from the Graph-Native Architecture design:
- `SelfOptimizer` periodically queries the graph for `SKILL` nodes and audits the container's `/services` directory.
- On gap detection, the agent scaffolds new service code in `sandboxes/`.
- On test pass, the agent packages the scaffold as a `ContributionType.DELIVERABLE` (PR) to the host repo.
- Human developer merges → container rebuilds → agent awakens with new capability.

---

### Phase 5.7: Pre-Live Remediation (Gap Analysis — March 2026)

Critical items identified during Initiative 132 gap analysis that must be resolved before Phase 6 live deployment.

**P0 — Completed:**
- [x] **Agent Identity Fix**: Default config `agent_id` corrected from `agent:zeroclaw-01` to `agent:eudaemon-alpha-01`
- [x] **HeapClient ↔ Gateway Alignment**: Headers (`X-Agent-Id` → `x-cortex-agent-id`), context bundle URL (GET → POST), emit payload wrapped in `EmitHeapBlockRequest` envelope
- [x] **MemoryStore Git Integration** (Init 121): Added `_git_init_if_needed()` and `_git_commit()` — each `persist_cycle()` now creates a Git commit
- [x] **Temporal Activities Conversion** (Init 126): Created `activities.py` with 6 `@activity.defn` functions, converted `bootstrap.py` to `workflow.execute_activity()`, fixed `asyncio.sleep()` → `workflow.sleep()` in main loop
- [x] **Type Correctness**: Fixed `SubmissionQueue.propose_contribution()` return type, removed invalid `asyncio.sleep(0.1)` in `LifecycleAuditor`
- [x] **Deployment Readiness**: Added `pyproject.toml` and `.env.example`

### Phase 5.8: Infrastructure Hardening (Pending)

**P1 — Next Sprint:**
- [ ] **Graph-Native Config Refresh**: Wire `execution_strategy.refresh_from_heap()` and `resource_governor.refresh_from_heap()` to actually query the HeapClient instead of using static env-var config
- [ ] **Space Membership Model**: Add `members: Vec<String>` and `archetype: Option<String>` to `SpaceRecord` in `cortex-domain`
- [ ] **Actor Registry**: Create `ActorRegistry` for persistent user/agent identity management
- [ ] **Storage Checkpoint**: Add backup-on-write to `SpaceRegistry.save_to_path()` (temp file + atomic rename)

### Phase 6.5: Agent Authentication Enforcement (Deferred)

- [ ] Validate `x-cortex-agent-id` header against `ActorRegistry` in `resolve_agent_identity()`
- [ ] Requires `ActorRegistry` from Phase 5.8

### Phase 5.9: Cortex-Web Contribution Experience (Pre-VPS UI Enrichments)

To ensure the Eudaemon Alpha loop can be tested end-to-end without mocks before VPS deployment, the following UI enrichments must be completed against the local `cortex-gateway`:
- [ ] Inline Agent Solicitation block creation UI in cortex-web
- [ ] Multi-space switching (removing hardcoded `SPACE_ID`)
- [ ] Agent activity notification panel (GlobalEvent streaming)
- [ ] A2UI Feedback Projection form binding for Human-in-the-Loop loops
- [ ] `ChronicleWriter` local file persistence (currently stub)

---

## Verification Plan

### Automated (Phases 1-5)
1. ZeroClaw container boots and authenticates to Nostra Gateway.
2. `AgentExecutionRecord` successfully emitted and visible in `GlobalEvent` log.
3. Heap Mode emission test: `POST /heap/emit` successfully generates a polymorphic block.
4. Token budget stops agent execution if limit exceeded (both local and graph layers).
5. Execution strategy routes governance alerts to real-time and chronicle drafts to batch.
6. Security boundary test rejects path traversal outside sandbox.
7. `pytest agent/tests/` — all 55 test files pass.
8. Grader assertions correctly PASS/FAIL/PARTIAL grade analysis outputs.
9. `AgentBenchmarkRecord` serializes/deserializes correctly via Pydantic.
10. SelfOptimizer proposes PROMPT edits after 3 consecutive FAIL benchmarks.
11. Trigger optimization rejects descriptions with <90% precision.
12. End-to-end Temporal integration: bootstrap → steward approval → bound.
13. Memory persistence: trajectory log contains valid ExecutionRecordBlock committed to Git.
14. HeapClient sends correct `EmitHeapBlockRequest` envelope with `x-cortex-agent-id` header.
15. Git-backed memory: commits survive working directory resets; linear history maintained.

### Manual (Phases 6-7)
1. Nostra users browse to the Cortex Stewardship Research space.
2. Users observe Eudaemon's generated heap blocks (charts, notes).
3. Users comment/interact with blocks, and verify Eudaemon ingests these signals.
4. Steward validates DPub Edition publication.
5. Steward reviews `ConfigProposalBlock` after first boot.
6. Steward validates `SelfOptimizationProposal` blocks are L1-gated.
7. A2UI feedback form correctly resumes suspended workflow.
8. Parallel A/B execution produces two divergent analyses with grader selection.
