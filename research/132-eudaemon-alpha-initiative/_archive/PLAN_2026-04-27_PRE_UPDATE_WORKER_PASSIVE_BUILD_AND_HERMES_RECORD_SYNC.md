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
  - "research/reference/knowledge/agent-systems/2026_OpenAI_Doubleword_Batch_Strategy_Transcript"
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
  - "125"
  - "080"
  - "121"
  - "122"
  - "124"
  - "126"
  - "127"
  - "130"
  - "133"
informs:
  - "134"
structural_pivot_impact: "major"
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Institutional Agent Architecture"
created: "2026-03-07"
updated: "2026-04-23"
---

# Initiative 132: Eudaemon Alpha

## Overview

Initiative 132 establishes Eudaemon Alpha as the first institutional research agent aligned to the current Nostra/Cortex runtime stack. The objective is not to invent a new workspace or planning model. The objective is to bind the agent to the primitives already implemented or actively governed in the portfolio.

Current validated reality for this pass:
- Gateway parity passes locally in this checkout.
- The root repo is the authoritative source for this planning pass.
- The companion `eudaemon-alpha/` repo has been deprecated. Initiative 132 authority lives entirely in the root ICP tree. Older references to `eudaemon-alpha/` in archived documents are historical only.
- The active VPS deployment contract is `cortex-gateway` plus `cortex_worker`, rendered and checked through the Hetzner runbook and runtime authority manifest.
- Prompt override remains a target hypothesis, not a validated runtime dependency.
- Meta-Harness adoption is recommendation-only and does not bypass Nostra or Cortex authority boundaries.
- Hermes is allowed only as a local, read-only advisory meta-observer for batch-design planning; live batch-provider execution and execution-adapter logic remain out of scope.

## Phase 6 Runtime Resolution

For the current implementation slice, the Phase 6 target is:

1. **Host**: Hetzner VPS
2. **Gateway**: Rust `cortex-gateway` on the same host, bound to `127.0.0.1:3000`
3. **Worker**: Rust `cortex_worker` in `nostra/worker`; older Python companion worker references are historical/unvalidated in this checkout
4. **Service model**: Linux `systemd`
5. **Auth posture**:
   - `NOSTRA_AUTHZ_DEV_MODE=0`
   - `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`
   - `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
6. **Migration posture**: Rust-native `cortex-eudaemon` remains the Phase 7+ parity target; do not shift runtime authority until parity is proven

At this stage:
- Nostra remains authority for initiatives, contributions, DPub lineage, and institutional identity.
- Cortex remains authority for heap workspaces, lifecycle execution, closeout tracking, action/navigation compilation, and workflow runtime behavior.
- Initiative 134 supersedes treating older workflow-engine assumptions as the canonical workflow architecture.
- Developer worktree isolation is an operator/developer governance process for protecting the system definition. It is not a heap primitive, closeout primitive, or workflow primitive.

## Cross-Initiative Relationship: 132 and 134

Initiative 132 (Eudaemon Alpha) and Initiative 134 (Hybrid Workflow Authority and Execution) have a consumer-provider relationship, not a blocker dependency.

- **Initiative 134 provides**: The canonical workflow substrate (definitions, adapters, execution, parity).
- **Initiative 132 consumes**: The workflow substrate for advisory observation, bounded passes, and synthesis artifacts. Hermes advisory passes may reference Initiative 134 contracts and gates, but they do not block on 134 completion.
- **Direction of influence**: 132 informs 134 through architectural observation, drift detection, and readiness-gate assessment. 134 enables 132 by providing a stable substrate against which to evaluate.

This relationship is asymmetric:
- 134 can reach completion without 132.
- 132 can operate in bounded advisory mode without 134 being complete, though its observations are more valuable when 134 contracts are stable.

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

## Cognitive Audit Resolution

The Doubleword transcript is adopted in 132 only as a recommendation-only pattern source for a future Cognitive Audit Pipeline.

1. Phase 6 steward communication and main-cycle analysis must use a native live cognition path first.
2. Eudaemon Alpha should define audit units, pass structure, cadence, and synthesis rules for a later advisory batch lane.
3. External batch cognition may perform large parallel analysis, but it is not an authority source and not the canonical workflow substrate.
4. ZeroClaw is relevant here only as a possible auth/provider sidecar pattern source for Codex subscription access. It does not replace the gateway, lifecycle, heap, or workflow contracts.
5. ChatGPT Pro does not act as a generic API-credit source for Eudaemon. If adopted, it must enter through an explicit Codex subscription adapter path.
6. Durable orchestration belongs to Cortex workflow artifacts and adapters under Initiative 134.
7. Current Phase 6 runs must enter and exit through the real stack already present in the repo:
   - Rust gateway
   - `cortex_worker`
   - low-latency live cognition provider invocation
   - heap/context endpoints
   - lifecycle and benchmark artifacts
8. Findings publish first as heap blocks, proposals, closeout follow-through, workflow drafts, or chronicle drafts. They do not directly mutate Nostra graph authority.
9. Core-graph bootstrap ideas from the transcript are limited to semantic discovery and normalization. They do not authorize direct graph creation from external batch outputs.
10. Hermes source bundles may include Doubleword and provider batch-policy references only to design `AuditUnit`/`SourceManifest` boundaries; they must not include provider credentials, API submission logic, polling instructions, execution-adapter logic, or repo mutation authority.
11. Hermes activation must run from a dedicated local workspace under `/Users/xaoj/hermes`, using a workspace-local `.hermes.md` guardrail file; each session is one bounded, deterministic, auditable pass that emits a session record, optional source-linked findings, and one synthesis artifact.

See `WORK_PRIMITIVES_ARCHITECTURE.md` for the full readiness analysis.

## Scope

1. Bind Eudaemon to heap-mode workspace contracts from Initiative 124.
2. Bind Eudaemon to lifecycle and authority constraints from Initiative 126.
3. Restrict code and file operations to the sandbox and memory-fs boundaries from Initiatives 121 and 127.
4. Use Initiative 130 capability overlays and compiled plans as the space activation model.
5. Use Initiative 134 as the governing workflow substrate for executable plans and future durable execution migration.
6. Preserve DPub lineage and promotion paths for chronicle outputs under Initiative 080.
7. Integrate a recommendation-only cognitive audit path that uses typed audit units, batch reasoning, and governed synthesis without bypassing SIQ, workflow, or steward review.
8. Formalize a live-provider boundary that supports `api_key` and `codex_subscription` lanes without making browser-subscription auth the runtime authority.
9. Treat clean request worktrees, durable checkpointing, and immutable evidence promotion as operator safety controls for the system-definition layer.
10. Define a local Hermes observer envelope as a planned advisory environment rather than a Cortex runtime component, with a dedicated activation workspace and local synthesis outputs.

## Out of Scope

1. Inventing a new Eudaemon-only workspace primitive.
2. Treating research initiative plans as executable workflow plans.
3. Giving external references or upstream model behavior authority over local governance contracts.
4. Autonomous promotion beyond recommendation-only boundaries.
5. Direct creation or mutation of the Nostra core graph from external batch analysis output.
6. Treating developer Git worktree state as a Cortex runtime primitive.
7. Running live Doubleword or other batch-provider APIs, queue workers, polling loops, or execution-control extraction as part of the Hermes advisory pass.

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

### Phase E: Live Provider and Auth Boundary
- Define the `LiveCognitionProvider` boundary for the Phase 6 primary lane.
- Keep `api_key` provider support deployment-ready for Hetzner.
- Treat Codex subscription access as an explicit sidecar/profile path rather than a generic API-credit shortcut.
- Do not require the advisory batch lane for boot-critical steward communication.
- Treat provider/runtime/auth inventory and runtime status as operator-only Cortex execution infrastructure rather than agent-facing default-readable state.
- Use split operator inventory/status contracts for steward diagnostics, and preserve `403 -> access_denied` behavior on non-operator paths.
- Require server-side execution eligibility for any default binding or runtime override so discovered inventory does not become executable implicitly.

### Phase F: Runtime Migration Readiness
- Keep external runtime adapters provisional.
- Evaluate migration from external agent host to native Cortex execution only through parity-backed slices.
- Refuse architecture claims that bypass the governed workflow/runtime stack already defined in 124, 126, 130, 133, and 134.

### Phase G: Cognitive Audit Pipeline
- Define an `AuditUnit` manifest over governed sources such as architecture standards, active initiative plans, heap context bundles, lifecycle events, and workflow artifacts.
- Define a `SourceManifest` contract that records exact sources and authority tier for every audit unit.
- Define `HermesObserverSession` as the local advisory session envelope for pass name, model config, input audit units, and output artifact references.
- Define `HermesSourcePacketV1` as a local operator-prepared bounded excerpt/fact packet for passes that should not rely on direct Hermes file inspection.
- Use Hermes locally as a meta-observer for architecture observation, contradiction detection, drift detection, bounded audit-unit analysis, and recommendation synthesis.
- Add a local Hermes Capability & Discovery Envelope for system cartography, component/dependency mapping, boundary integrity, pattern extraction, adapter translation, test synthesis, skill architecture, capability inventory, event/lifecycle logging, memory/continuity posture, batch design/aggregation, and final synthesis.
- Add a visibility/approval projection lane that maps Hermes outputs into existing Cortex Web heap, A2UI approval, steward-gate, and proposal-review surfaces without granting direct execution authority.
- Activate Hermes only through the dedicated `/Users/xaoj/hermes` workspace and its workspace-local `.hermes.md`; treat promoted root `ICP` docs as authority and `~/.hermes` profiles/SOUL as non-authoritative convenience state.
- Keep each Hermes session to one bounded, deterministic, auditable pass with explicit `SourceManifest` and `AuditUnit` inputs, `maxSteps = 1`, and `writeAccess = false`.
- Allow normal advisory inference for Hermes reasoning, but keep batch-provider submission, polling, queue runners, and execution-adapter behavior out of scope.
- Require each Hermes pass to emit one session record, zero or more source-linked findings, and one local synthesis artifact containing summary, contradictions or drift, recommendations, and source references.
- Treat `HermesLaneCatalogV1`, `HermesCapabilityMatrixV1`, and `SkillImprovementProposalV1` as local planning contracts only, not public runtime APIs.
- Define `HermesAuditRunbookV1` as a local runbook contract for operator-mediated preflight, one bounded pass, postflight, optional evidence drafting, and manual promotion.
- Define explicit Hermes stage stabilization signals so update, upgrade, and deprecate decisions are made from evidence rather than intuition.
- Use the hybrid companion skill strategy: keep `nostra-cortex-dev-core` lean as the governance/preflight gate and draft `nostra-platform-knowledge` as a progressive-disclosure proposal before any skill registry change.
- Use Hermes locally to reason over batch-design references without provider execution; any later external batch backend must remain advisory cognition behind Initiative 134.
- Re-enter Eudaemon Alpha for synthesis, prioritization, contradiction review, and recommendation drafting.
- Publish outputs through heap blocks, proposals, closeout work, workflow drafts, or chronicle drafts instead of direct authority mutation.
- Keep deterministic SIQ checks and steward review as the release-gating authority.

### Phase H: Repo Hygiene and Evidence Promotion
- Require clean request worktrees for developer/operator implementation work outside bounded repo-wide stewardship operations.
- Reserve the shared root worktree for recovery, portfolio alignment, and other repo-wide structural tasks.
- Treat mutable runtime outputs under `logs/` as local operational surfaces rather than Git authority.
- Preserve durable evidence by promoting immutable copies into governed initiative surfaces instead of tracking mutable `*_latest.*` outputs.
- Use checkpoint bundles or WIP commits before handoff or context switching so steward-facing updates are not stranded in a dirty tree.
- Keep this hygiene layer aligned with Initiative 125 controls, Initiative 133 evidence routing, and Initiative 134 workflow authority boundaries.

## Exit Criteria

1. Initiative 132 is portfolio-consistent and structurally governed.
2. Eudaemon documentation no longer conflicts with the canonical heap/runtime/workflow architecture.
3. The agent workspace model clearly uses heap notes for working material and governed promotion paths for durable outputs.
4. The task and planning model clearly distinguishes closeout/workflow tasks from initiatives and notes.
5. Initiative 132 is ready to continue implementation against current Cortex contracts rather than stale architecture assumptions.
6. The cognitive audit path is explicitly routed through current heap, lifecycle, SIQ, and workflow surfaces rather than treated as a direct mutation engine.
7. The live cognition lane is explicitly primary for Phase 6 communication, while the batch audit lane remains secondary and advisory.
8. Developer worktree isolation, closeout hygiene, and evidence promotion are explicitly separated from heap, closeout-ledger, and workflow primitives.
9. Hermes activation resolves only promoted root `ICP` authority sources, runs through the dedicated local activation workspace, and emits bounded source-linked session/synthesis artifacts without repo or runtime mutation.
10. Hermes capability discovery is documented as a local planning layer that classifies features and drafts lane/skill proposals without enabling tools, skills, scheduled jobs, batch infrastructure, execution adapters, or runtime mutation.
11. Hermes runbooks standardize repeatable operator-mediated passes without enabling unattended execution, background scheduling, runtime adapters, or direct mutation.
12. Hermes source packets and stabilization signals make bounded passes practical, legible, and governable enough to decide when Hermes should be maintained, expanded, or retired for this stage.
13. Any future user-facing Hermes visibility is routed through existing Cortex Web heap/proposal/approval surfaces rather than a new standalone control plane.

### 6.2 LLM API Connection
Promote the live LLM path from an activity-side workaround into the canonical pattern-detection contract:
- Treat direct live request/response analysis as the primary Phase 6 cognition path.
- Support explicit provider boundaries for `api_key` and `codex_subscription` lanes.
- Do not assume ChatGPT Pro provides transferable generic API billing.
- Load the system prompt from `config/system_prompt.md`.
- Prefer `prompt_override` from Heap when present and verified, with local file fallback.
- Format the payload as `{"model": route.model, "system": active_prompt, "messages": [{"role": "user", "content": bundle_context + code_context}]}`.
- Parse `score`, `rationale`, `proposed_nodes`, and `proposed_edges` from the returned JSON.
- Track actual `latency_ms`, `token_usage`, and `token_cost` in the benchmark path.

### 6.3 Gateway Connection
Use canonical heap endpoints and contracts end to end:
- Query recent blocks via `GET /api/cortex/studio/heap/blocks`.
- Package selected context via `POST /api/cortex/studio/heap/blocks/context`.
- Emit proposals and execution records through `POST /api/cortex/studio/heap/emit`.
- Validate that the agent ingests steward comments and, when verified, prompt overrides from the canonical bundle path.

### 6.4 Graph-Native PROMPT Seeding
- Create the first real `ContributionType.PROMPT` node for the Code Analysis system prompt on the Heap.
- If a live `prompt_override` path is available, test that `PatternDetector` fetches and uses it instead of `config/system_prompt.md`.
- Until that path is validated, keep the local file as the fallback and record the override path as unverified.
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

### 7.5 Rust-Native Runtime Parity & Protocol Hardening
Transition the Eudaemon-facing capabilities into parity-backed `cortex-eudaemon` Rust slices. Validation against the current repo and external pattern sources supports this as a constrained migration program, not a wholesale Phase 7 replatform. This phase mandates:
- **Parity-Backed Migration**: Preserve the active Phase 6 `cortex_worker` + Rust gateway deployment authority until Rust slices prove behavioral parity against current heap, lifecycle, workflow, and provider contracts.
- **Existing Boundary Reuse**: Follow the current workspace seams in `cortex-domain`, `cortex-runtime`, `cortex-ic-adapter`, and `cortex-eudaemon` rather than inventing a new core crate taxonomy.
- **Strict Payload Boundaries**: Harden external Gateway and A2UI DTOs with discriminated unions where the domain is variant-shaped, consistent `camelCase` serialization, and contract-synchronized TypeScript bindings.
- **Protocol Fidelity**: Remove ad hoc cross-language payload drift on the highest-risk network boundaries first, while preserving explicitly governed compatibility fields where current contracts still require them.
- **Measured Extraction Pressure**: Drive crate or module extraction from observed bloat and reuse pressure, not from abstract modularity goals alone.

Immediate Phase 7 seam candidates validated in the current repo:
1. **Provider Runtime Surface**: isolate provider registry/runtime/client/policy logic so live-provider and batch-audit adapters can evolve without further inflating the main gateway surface.
2. **ACP / Terminal Execution Surface**: separate ACP protocol, terminal orchestration, permission ledger, and execution-policy enforcement into a narrower execution-control slice.
3. **Workbench UX / Heap Projection Surface**: extract the large heap/workbench/viewspec projection and UX orchestration logic into a dedicated application slice with clearer projection contracts.

### 7.6 Execution Isolation Hardening
To safely expand Eudaemon Alpha toward higher-authority execution:
- **Phase 6 Posture Preservation**: Keep the current Hetzner `systemd` deployment and operator-local SSH promotion model as the Phase 6 baseline. Do not treat Phase 7 as a blanket replacement of that runtime contract.
- **Immediate Service Hardening**: Tighten service-level restrictions first through host policy, working-directory minimization, repo/state path separation, and operator-only execution-infrastructure visibility.
- **Executor-Specific Sandbox Trigger**: Require OS-level sandboxing when Eudaemon begins executing untrusted code, generated shell/code evaluation loops, or broader autonomous contribution paths. This control applies to the executor slice that runs those actions, not automatically to the entire gateway/runtime stack.
- **Privilege Degradation**: Ensure any sandboxed execution path operates with explicit network, filesystem, and capability restrictions plus bounded resource controls before higher-authority automation is enabled.
- **Infrastructure Protection**: Preserve the runtime host topology boundary by keeping provider/runtime/auth inventory operator-only and by ensuring discovered inventory never becomes executable implicitly.

### 7.7 Validated Phase 7 Sequencing
The validated progression for Initiative 132 is:
1. **Contract Hardening Now**: tighten DTOs, payload discriminators, and TypeScript/Rust contract sync on the currently exposed Gateway and A2UI boundaries.
2. **Parity Slice Next**: port the highest-value Eudaemon loop capabilities into Rust behind existing `cortex-domain` / `cortex-runtime` seams and verify parity before shifting runtime authority.
3. **Isolation Before Untrusted Execution**: add executor-specific OS-level sandboxing before enabling untrusted code execution or broader autonomous contribution flows.

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

### Phase 5.8: Infrastructure Hardening

**P1 — Current Resolution:**
- [x] **Space Membership Model**: `SpaceRecord.members` and `SpaceRecord.archetype` exist in `cortex-domain`
- [x] **Actor Registry**: `ActorRegistry` exists for persistent user/agent identity management
- [x] **Storage Checkpoint**: `SpaceRegistry.save_to_path()` uses backup-on-write semantics
- [x] **Gateway Identity Enforcement**: `resolve_agent_identity()` validates `x-cortex-agent-id` when enforcement is enabled
- [ ] **Graph-Native Config Refresh**: keep `execution_strategy.refresh_from_heap()` and `resource_governor.refresh_from_heap()` driven by live Heap strategy inputs in production runs
- [ ] **Hetzner Governance Bootstrap**: create or validate actor registry entry plus Space membership/archetype before go-live

### Phase 5.9: Hetzner Deployment Normalization

Blocking preflight work before live deployment:

- [x] **Canonical Runtime Split**: `cortex_worker` + Rust gateway is the active VPS deployment target; the older companion repo path remains historical/unvalidated in this checkout
- [x] **Canonical Env Contract**: loopback gateway URL, production auth flags, explicit Temporal target
- [x] **Linux Service Assets**: root repo owns `cortex-gateway.service` and `cortex-worker.service`
- [x] **Hetzner Runbook**: promotion, deploy, systemd, SSH, and runtime-authority guidance live in the root repo
- [x] **Deployment Hygiene**: ignore workstation artifacts (`venv`, `__pycache__`, `.pytest_cache`, `.DS_Store`, `.worktrees`)
- [x] **Repo Boundary**: Initiative 132 stays in the root repo; `eudaemon-alpha/` companion-repo references are historical only
- [ ] **Live Box Validation**: verify SSH alias/host, install deps on Hetzner, enable services, and complete one end-to-end bootstrap cycle

### Phase 6.5: Production Identity Enforcement

- [x] Validate `x-cortex-agent-id` header against `ActorRegistry` in `resolve_agent_identity()`
- [x] Require `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1` for Hetzner production services
- [ ] Treat the deployment as unhealthy until the registered agent is accepted with enforcement enabled

### Phase 5.9: Cortex-Web Contribution Experience (Pre-VPS UI Enrichments)

To ensure the Eudaemon Alpha loop can be tested end-to-end without mocks before VPS deployment, the following UI enrichments must be completed against the local `cortex-gateway`:
- [x] Inline Agent Solicitation block creation UI in cortex-web
- [x] Reduce hardcoded default-space assumptions that block steward testing across non-preview spaces
- [x] Agent activity notification panel (GlobalEvent streaming)
- [x] A2UI Feedback Projection form binding for Human-in-the-Loop loops
- [x] `ChronicleWriter` local file persistence for Phase 6 drafting
- [ ] Heap-backed chronicle promotion path once DPub wiring is scheduled

---

## Verification Plan

### Automated (Phases 1-5)
1. Hetzner-ready gateway process boots on `127.0.0.1:3000` and authenticates with production auth settings.
2. `AgentExecutionRecord` successfully emitted and visible in `GlobalEvent` log.
3. Heap Mode emission test: `POST /heap/emit` successfully generates a polymorphic block.
4. Token budget stops agent execution if limit exceeded (both local and graph layers).
5. Live provider preflight fails when the selected Phase 6 cognition lane is under-configured.
6. Main-cycle analysis uses the native live cognition lane; any batch audit lane remains optional and advisory.
7. Security boundary test rejects path traversal outside sandbox.
8. Historical Python companion tests are run only if the companion repo is restored and validated; current repo gates use Rust/gateway checks.
9. Grader assertions correctly PASS/FAIL/PARTIAL grade analysis outputs.
10. `AgentBenchmarkRecord` serializes/deserializes correctly via Pydantic.
11. SelfOptimizer proposes PROMPT edits after 3 consecutive FAIL benchmarks.
12. Trigger optimization rejects descriptions with <90% precision.
13. End-to-end Temporal integration: bootstrap → steward approval → bound.
14. Memory persistence: trajectory log contains valid ExecutionRecordBlock committed to Git.
15. HeapClient sends correct `EmitHeapBlockRequest` envelope with `x-cortex-agent-id` header.
16. Git-backed memory: commits survive working directory resets; linear history maintained.
17. Context bundle assembly uses `POST /api/cortex/studio/heap/blocks/context`.
18. Governance bootstrap writes or validates `agent:eudaemon-alpha-01` in actor and space registries.
19. Hermes source manifests resolve only promoted root `ICP` authority paths and do not fall back to request-worktree governance files.
20. Hermes activation loads its workspace-local `.hermes.md` guardrail file from `/Users/xaoj/hermes`.
21. `HermesObserverSession` enforces `maxSteps = 1` and `writeAccess = false` for the advisory pass.
22. Each Hermes activation pass emits a local session record, optional source-linked findings, and one synthesis artifact with summary, contradictions or drift, recommendations, and source references.
23. Hermes may use a normal advisory inference lane, but no batch submission, polling, queueing, or execution-adapter behavior is invoked.

### Manual (Phases 6-7)
1. SSH login works to the Hetzner host using the documented alias or explicit host and local private key.
2. If and when the companion repo is restored, it is validated separately before becoming a deployment dependency again.
3. `cortex-gateway.service` and `cortex-worker.service` start cleanly under `systemd`.
4. The actor registry contains `agent:eudaemon-alpha-01`.
5. The target Space includes the agent in `members` with the intended `archetype`.
6. Steward reviews the `ConfigProposalBlock` after first boot.
7. The agent discovers a solicitation block and emits a `ConfigProposalBlock`.
8. Users comment/interact with blocks, and Eudaemon ingests these signals in the next context bundle.
9. One end-to-end bootstrap cycle completes on Hetzner without dev auth flags.
10. If a Codex subscription lane is adopted, the sidecar/profile path is validated without bypassing the heap/gateway/workflow authority model.
