# Eudaemon Alpha — Implementation Plan

**Target**: Deploy Eudaemon Alpha (Stage 1) on Hostinger VPS (Simulating the 122 MVK)
**Runtime**: ZeroClaw minimal orchestration (Python prototype for the 122 Rust Kernel)
**Scope**: Observation, graph queries, heap block emission, chronicle generation
**Authority**: L1 (Suggestion-only)

---

## Phase 1: Infrastructure Setup

### 1.1 VPS Provisioning
- Hostinger VPS: 2-4 vCPU, 8-16GB RAM, 80-120GB SSD, Ubuntu 24.04
- **Tailscale VPN**: Install Tailscale for zero-trust networking. Bind SSH exclusively to the Tailnet interface.
- SSH key authentication (password auth disabled)
- UFW firewall (default deny all ingress; allow SSH over Tailscale `tailscale0` interface; allow standard outbound HTTPS)
- **`.env` file**: Contains `NOSTRA_GATEWAY_URL`, LLM API keys, and Doubleword credentials. Not committed to version control. A `.env.example` template is maintained in the repo.

### 1.2 Containerization
```yaml
# docker-compose.yml
services:
  eudaemon:
    build: ./agent
    user: "1000:1000"  # non-root
    volumes:
      - ./repos:/repos:ro                                    # read-only canonical code
      - ./sandboxes:/repos/cortex-memory-fs/sandboxes:rw     # read-write experiment sandbox
      - ./memory:/memory                                     # Git-backed episodic store (Init 121)
      - ./logs:/logs                                         # local execution logs
    environment:
      - NOSTRA_AGENT_ID=agent:eudaemon-alpha-01
      - NOSTRA_AUTHORITY_LEVEL=L1
      - NOSTRA_GATEWAY_URL=${NOSTRA_GATEWAY_URL}
      - NOSTRA_LOCAL_MAX_SPEND=20.00       # Immutable hard cap (Layer 1)
      - NOSTRA_LOCAL_MAX_TOKENS=500000     # Immutable hard cap (Layer 1)
      - NOSTRA_LOCAL_MAX_BLOCKS=20         # Immutable hard cap (Layer 1)
    env_file: .env
    networks:
      - eudaemon-net

networks:
  eudaemon-net: {}  # External access enabled for PyPI, LLM APIs, Gateway
```

**Key changes from original spec:**
- `read_only: true` removed — agent needs write access for pip installs and scratch files.
- `internal: true` removed — agent needs outbound HTTPS to reach Gateway, PyPI, and LLM APIs.
- `.venv` created in Dockerfile owned by user `1000:1000` for dependency autonomy.
- `./memory:/memory` added for Initiative 121 Git-backed episodic store.
- `./sandboxes` mounted `rw` for Initiative 127 sandbox experimentation.

---

## Phase 2: Interfacing with Nostra-Cortex

ZeroClaw must implement the following architectural contracts:

### 2.1 Agent Context Loading (Initiative 124)
Instead of manual index building for the workspace, Eudaemon pulls structured context:
```bash
POST /api/cortex/studio/heap/blocks/context
# Returns an AgentContextBundle of the active Research Space heap
```

### 2.2 Workspace Operations (Initiative 124)
Eudaemon emits working thoughts, charts, polls, and reflections directly to the spatial heap:
```bash
POST /api/cortex/studio/heap/emit
# Payload: EMIT_HEAP_BLOCK.schema.json
# Content: Polymorphic blocks (a2ui, rich_text, charts, structured_data)
```

### 2.3 Global Lifecycle Auditing (Initiative 126)
Every cycle must hit the GlobalEvent stream with an execution record:
```json
// AgentExecutionRecord Payload wrapper
{
  "schema_version": "v1",
  "execution_id": "<uuid>",
  "agent_id": "agent:eudaemon-alpha-01",
  "phase": "analysis",
  "status": "completed",
  "authority_scope": "L1",
  "input_snapshot_hash": "<hash>",
  "output_snapshot_hash": "<hash>",
  "timestamp": "<iso8601>"
}
```

---

## Phase 3: Core Runtime (Workflow-as-Agent)

Per Initiative 047, the Agent is a Durable Workflow. ZeroClaw simulates this on the VPS:

### 3.1 Initialization (System Prompt Injection)
The agent's identity and governance boundaries are explicitly loaded into the LLM context prior to any inference. The system prompt is stored as a version-controlled config file (`agent/config/system_prompt.md`) and loaded by `main.py` before any LLM call.
- **Constitutional Context Loader**: Injects the 4 Core Principles of Nostra, the definition of the Stewardship Institute, and the instruction: *"Do not produce generic AI summaries. Produce highly opinionated, structurally sound architectural insights."*
- **Model Constitution Disclosure (062)**: Enforces the `AgentDisclosurePattern`. If the base model refuses an exploratory task due to upstream safety training, it must explicitly annotate this as a `ModelBiasAnnotation` in the Heap, not as a Nostra policy decision.
- **Bootstrap Check**: On first boot, if no `ExecutionStrategyBlock` exists on the Heap, runs the Bootstrap Wizard (§3.5) before entering the main loop.

### 3.2 The Execution Loop
```python
while True:
    # --- Config Refresh ---
    # Read graph-native policy blocks from Heap (fallback to local config if unreachable)
    execution_strategy.refresh_from_heap(heap_client)
    resource_governor.refresh_from_heap(heap_client)
    cycle_interval = config_manager.get("cycle_interval", default=1800)  # Heap-adjustable
    insight_threshold = config_manager.get("insight_threshold", default=0.7)  # Heap-adjustable
    allowed_types = config_manager.get("allowed_types", default=["Architecture", "Governance"])

    # --- Observation ---
    events = observation_engine.fetch_filtered_events(allowed_types=allowed_types)
    user_signals = heap_client.get_context_bundle("cortex-stewardship")
    
    if not events and not user_signals:
        lifecycle_auditor.emit(phase="idle", status="sleeping")
        sleep(cycle_interval)
        continue

    # --- Tool-Assisted Codebase Access ---
    # SECURITY GATE (127): Restricted to cortex-memory-fs/sandboxes/
    code_context = tool_executor.run_code_search_if_needed(
        events, user_signals, target_dir="/repos/cortex-memory-fs/sandboxes/"
    )

    # --- Durable Execution Step ---
    input_hash = hash(events + user_signals + code_context)
    lifecycle_auditor.emit(phase="analysis", status="started", input_hash=input_hash)

    # Route to appropriate model via execution strategy
    route = execution_strategy.resolve("code_analysis")
    analysis = pattern_detector.analyze(events, user_signals, code_context, model=route.model)

    if analysis.score >= insight_threshold:
        heap_client.emit_block(analysis.as_polymorphic_block())
        
    chronicle_writer.append_draft_chapter()

    if analysis.warrants_governance:
        submission_queue.propose_contribution(analysis)

    output_hash = hash(analysis)
    lifecycle_auditor.emit(phase="analysis", status="completed", output_hash=output_hash)

    # --- Governance ---
    resource_governor.check_budget_and_enforce_caps()
    heap_client.emit_block(resource_governor.build_usage_report())  # Surface usage on Heap

    # --- Memory Persistence (Initiative 121) ---
    memory_store.persist_cycle(
        events=events, signals=user_signals, code_context=code_context,
        analysis=analysis, route=route
    )

    # --- Self-Optimization ---
    self_optimizer.evaluate_cycle(analysis, resource_governor, execution_strategy)
    if self_optimizer.has_proposals():
        for proposal in self_optimizer.drain_proposals():
            heap_client.emit_block(proposal.as_heap_block())  # L1-gated, never auto-applied
    
    sleep(cycle_interval)
```

### 3.3 Execution Strategy (Adaptive Routing)
The agent routes tasks to real-time or batch inference based on task type and codebase volatility.
- **`ExecutionStrategyBlock`** on the Heap defines routing rules (model, priority, SLA, provider).
- **`volatility_mode`**: Agent tracks commit frequency in sandbox. High churn → real-time. Stable → batch (up to 75% cheaper via services like Doubleword Batched).
- Local fallback: `config/model_registry.json` used if Heap is unreachable.

### 3.4 Self-Optimization Loop & Graph-Native Evolution
The agent monitors its own performance and proposes improvements via the same L1 proposal mechanism. To strictly enforce Initiative 124 (Graph-Native Codecs), the agent's actual "behavior" must be hosted on the Graph, not hardcoded in the container:
- **Evolvable Nodes**: The agent's LLM Instructions, JSON schemas, and Tool logic exist as `ContributionType.PROMPT`, `SCHEMA`, and `SKILL` nodes on the Space Heap.
- **The Optimization Cycle**: If pattern detection fails, the agent's `SelfOptimizer` submits a `SelfOptimizationProposal` specifically proposing a branched edit to its own `PROMPT` node on the graph.
- All proposals emitted as `SelfOptimizationProposal` blocks on Heap. Agent never auto-applies. Once a human steward merges the `PROMPT` edit, the agent adopts the new logic on the next execution cycle.

### 3.5 Bootstrap Wizard
On first boot, if no `ExecutionStrategyBlock` exists on the Heap:
1. Agent probes configured API providers for available models, capabilities, and pricing.
2. Agent emits a `ConfigProposalBlock` to the Heap with recommended defaults.
3. Human steward reviews, adjusts, and approves on the Heap Board.
4. Agent reads approved config and begins normal operation.

---

## Phase 4: Integration Architecture

### 4.1 The Institutional Space
- The `Cortex Stewardship Institute` maintains a public Nostra Space named **Cortex Stewardship**.
- The Space Archetype is **Research**.
- Space capability graphs (`130`) activate **Heap Mode** (`124`) and **DPub Rendering** (`080`).

### 4.2 The "Space Steward" Standardization (Initiative 126 Extension)

Eudaemon Alpha serves as the prototype for a repeatable **Space-Aligned Agent** capability. A Space (representing a business idea, institution, or project) can invite a constitutional agent to act as a "Steward".

* **The Solicitation**: The human Space Steward emits an `AgentSolicitationBlock` defining the role, capabilities, budget, and authority scope.
* **The Interview (Bootstrap Wizard)**: A remote agent booting up reads the solicitation, injects the Space's specific constitution into its system prompt, and emits a `ConfigProposalBlock` (the "Resume").
* **The Binding**: Once approved, the Gateway binds the agent to the Space's Heap.

**The Role of the Agent Steward:**
1. **Analyze**: Read the Heap and ContributionGraph to understand the business intent and ecosystem landscape.
2. **Converse**: Emit structured reflections, questions, and architectural proposals to the Heap Board to help human operators clarify their aims.
3. **Integrate**: Help human processes translate unstructured goals into formalized Nostra-Cortex primitives (e.g., suggesting a Temporal workflow definition to solve a stated business problem).

Like Eudaemon Alpha, these stewards operate via Anthropic's "Harness" pattern: they carry no internal state, using the Space's Heap as their structured external memory across isolated execution cycles.

### 4.3 The Chronicle System (DPub)
- Eudaemon maintains a `Contribution<DPub>` representing its timeline.
- **Living Layer**: Daily operations update the `HEAD` chapters.
- **Editions**: Monthly, the human steward triggers the `Publish Edition` workflow to freeze a Merkle-dag root hash of the month's insights.

---

## Phase 5: Hardening & Security Validation

### 5.1 Security Boundary Tests (Initiative 127)
Enforce the sandbox boundary promises made in Phase 1:
- **Path Traversal Rejection**: Test that `ToolExecutor.search_codebase()` with inputs like `../../etc/passwd` is rejected before reaching `subprocess.run`.
- **Sandbox Write Confinement**: Test that file write operations are restricted to the mounted `/repos/cortex-memory-fs/sandboxes/` volume.
- **Import Sandboxing**: Verify Temporal's workflow sandbox correctly blocks non-deterministic imports in workflow code.

### 5.2 Budget & Resource Governor Integration Tests
Validate the economic guardrails from Phase 1 (Section 1.2):
- **Hard Cap Enforcement**: Start main loop with `NOSTRA_LOCAL_MAX_SPEND=0.01`. Assert the loop `break`s after the first tracked inference exceeds the cap.
- **Token Budget Gate**: Set `NOSTRA_LOCAL_MAX_TOKENS=100`. Assert the `ResourceGovernor` halts execution.
- **Block Emission Cap**: Set `NOSTRA_LOCAL_MAX_BLOCKS=1`. Assert the agent emits exactly one proposal block then stops.
- **Usage Report Emission**: Assert that `resource_governor.build_usage_report()` produces a valid `UsageReportBlock` emitted to the Heap every cycle.

### 5.3 Memory Persistence Validation (Initiative 121)
- After a cycle, assert `memory/trajectory_logs/` contains a JSON file with the `ExecutionRecordBlock` including a valid `AgentBenchmarkRecord`.
- Test that `MemoryStore.persist_cycle()` correctly Git-commits the trajectory log.
- Validate replay: corrupt the log file and assert the system recovers gracefully on next cycle.

### 5.4 End-to-End Temporal Integration Test
- Boot a real Temporal dev server (`temporal server start-dev`).
- Start `AgentBootstrapWorkflow`, signal approval, transition to `AgentMainLoopWorkflow`.
- Execute exactly one cycle with a mocked HeapClient fallback.
- Assert the `ExecutionRecordBlock` has a valid `AgentBenchmarkRecord` with `overall_grade` set.

### 5.5 SelfOptimizer Benchmark Degradation Test
- Feed 3 consecutive `FAIL`-graded `AgentBenchmarkRecord`s via `record_benchmark()`.
- Assert `evaluate_cycle()` produces a `SelfOptimizationProposalBlock` targeting `PatternDetector.PROMPT` with `action: branch_and_edit`.

### 5.6 Trigger Optimization CI Validation
- Create a sample `SKILL` node description.
- Generate 20 positive/negative test queries.
- Assert `validate_trigger_precision()` correctly gates bad descriptions below 90% precision.

---

## Phase 6: Live Integration

### 6.1 VPS Deployment
- Deploy `docker-compose.yml` to Hostinger VPS.
- Validate Tailscale VPN connectivity and SSH binding to `tailscale0`.
- Boot the Temporal server and the `eudaemon` container.
- Confirm the agent completes one full bootstrap → sleep cycle with local fallback.

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

