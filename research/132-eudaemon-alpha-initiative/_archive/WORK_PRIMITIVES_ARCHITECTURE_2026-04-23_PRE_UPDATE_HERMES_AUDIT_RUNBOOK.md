# Eudaemon Alpha - Work Progression Primitive Architecture

Date: 2026-03-12
Scope: Initiative 132 readiness review before further implementation

## Current Cortex Handling

### 1. Notes and exploratory work
- Cortex handles working notes through the heap runtime, not through initiative artifacts.
- Canonical surfaces:
  - `POST /api/cortex/studio/heap/emit`
  - `GET /api/cortex/studio/heap/blocks`
  - `POST /api/cortex/studio/heap/blocks/context`
- Heap blocks are polymorphic and already support note-like, chart-like, pointer, rich text, and A2UI payloads.
- The web host defaults to note-oriented creation flows and exposes block types such as `note`, `task`, and `chart`, but the authority is still the generic heap block contract.

### 2. Tasks that move work forward
- Cortex currently has two task surfaces:
  - closeout ledgers for concrete follow-through work
  - workflow/user-task checkpoints for durable execution
- The closeout path is already implemented as a contribution-scoped ledger exposed by `GET /api/cortex/runtime/closeout/tasks`.
- This is the right operational task primitive for release gates, remediation follow-through, and initiative closeout work.
- Workflow tasks belong inside workflow instances rather than the heap.

### 3. Plans
- Cortex currently distinguishes between:
  - compiled navigation plans: where users should go
  - compiled action plans: what users can do in a given UI context
  - workflow artifacts: what the system can durably execute
- `GET /api/spaces/:space_id/navigation-plan` and `POST /api/spaces/:space_id/action-plan` are UI/runtime plans, not project plans.
- The executable plan stack is the governed workflow pipeline:
  - `WorkflowIntentV1`
  - `WorkflowDraftV1`
  - `WorkflowDefinitionV1`
  - `WorkflowInstanceV1`
- Initiative 134 is now the canonical architectural frame for this pipeline.

### 4. Initiatives
- Initiatives are not heap blocks and not workflow instances.
- They are governed research artifacts rooted in:
  - `research/*/PLAN.md`
  - `research/RESEARCH_INITIATIVES_STATUS.md`
  - ContributionGraph extraction from the research corpus
- An active initiative must have frontmatter, a portfolio role, and coherent status/index metadata.

### 5. Knowledge references
- Repo references live in `research/reference/index.toml`.
- Knowledge artifacts live in `research/reference/knowledge/index.toml` plus per-artifact `metadata.md`.
- These references should inform Initiative 132 decisions, but they do not replace Nostra or Cortex authority surfaces.

### 6. Hermes advisory batch context
- A local Hermes observer envelope may consume governed source bundles for batch-design planning.
- Hermes acts as a meta-observer for architecture observation, contradiction detection, drift detection, bounded audit-unit analysis, and recommendation synthesis.
- Doubleword and provider batch-policy material are read-only references for shaping audit units, source manifests, and synthesis rules.
- The activation guardrail surface lives in `/Users/xaoj/hermes/.hermes.md`; `~/.hermes` profiles and SOUL may shape ergonomics but do not redefine authority.
- Each Hermes session is one bounded, deterministic, auditable pass that consumes explicit source manifests and audit units and emits source-linked findings plus one synthesis artifact.
- Hermes is not a Cortex runtime primitive, workflow authority, provider adapter, or execution substrate at this stage.
- Live batch API submission/polling, provider credentials, and execution-control work remain out of scope.

### 7. Hermes capability discovery context
- A second local planning envelope may classify Hermes capabilities and define missing observer lanes without enabling new runtime behavior.
- Planned lanes include system cartography, component/dependency mapping, boundary integrity, pattern extraction, adapter translation, test synthesis, skill architecture, capability inventory, event/lifecycle logging, memory/continuity posture, batch design/aggregation, and final synthesis.
- Capability discovery may inventory skills, memory, session search, cron/webhooks, MCP, subagents, gateway delivery, terminal tooling, and batch-runner concepts, but feature enablement requires a later governed decision.
- Skill architecture follows the hybrid companion strategy: `nostra-cortex-dev-core` remains the lean governance gate, while `nostra-platform-knowledge` may be proposed as a progressive-disclosure knowledge skill with eval prompts.
- The capability discovery envelope produces local planning artifacts only; it does not create public Cortex APIs, workflow authority, runtime adapters, skill registry changes, or unattended execution.

## Resolved Stage Architecture

### Notes
- Use heap blocks as the Eudaemon working surface.
- Treat heap content as exploratory runtime material that may later be promoted into proposals, DPub updates, or other governed contributions.

### Tasks
- Use closeout task ledgers for concrete operational follow-through.
- Use workflow/user-task checkpoints only when Eudaemon is driving a durable execution flow.
- Do not invent a separate Eudaemon task model.

### Plans
- Use action/navigation plans for interface and operator affordances.
- Use the workflow artifact pipeline for executable plans.
- Do not treat research initiative plans as executable runtime plans.

### Initiatives
- Keep Initiative 132 as a governed research initiative with frontmatter, explicit dependencies, and portfolio registration.
- Use initiatives to record architecture intent, sequencing, and stewardship, not as runtime scratchpads.

### Developer isolation
- Use clean Git worktrees as an operator/developer process for protecting the system-definition layer.
- Root-worktree recovery, checkpointing, and evidence promotion are not heap primitives, closeout-ledger primitives, or workflow primitives.
- Mutable runtime outputs may still land in `logs/`, but durable evidence should be promoted into governed initiative surfaces.

### Kickoff approval
- Use heap kickoff approval only for bounded initiative kickoff packets that already have governed initiative metadata and a steward-backed execution boundary.
- The kickoff launcher must emit an approval-first `agent_solicitation`, not a direct execution task.
- Steward approval may mint a follow-up heap task, but execution still routes through the task router after approval.
- Kickoff approval does not replace steward-gated structural APIs, workflow authority, or initiative governance.

### Promotion path
- Heap note -> proposal / DPub / workflow draft / contribution update
- Initiative kickoff approval -> steward feedback -> routed kickoff task
- Closeout task -> verified completion artifact
- Workflow draft -> definition -> instance
- Initiative plan -> ContributionGraph lineage and portfolio governance
- Hermes advisory artifact -> source-linked findings + synthesis -> heap/proposal/closeout/workflow/chronicle candidate after steward review
- Hermes capability artifact -> lane/capability/skill proposal -> steward-reviewed governance candidate only

## Principles for This Stage

1. Boundary first: Nostra governs what exists; Cortex governs how active work is projected and executed.
2. Promotion over mutation: exploratory work starts on the heap and is promoted into governed artifacts when it matters.
3. Do not overload primitives: notes are not initiatives, and initiatives are not runtime tasks.
4. Prefer implemented contracts over speculative models: use existing heap, closeout, action-plan, and workflow surfaces before inventing anything new.
5. References inform design; they do not outrank local authority artifacts.
6. Kickoff approval is approval-first and bounded: it is not a generic "start initiative" affordance.
7. Developer hygiene stays operator-side: request worktree isolation and evidence promotion protect the repo, but they do not redefine Cortex runtime primitives.
8. Hermes stays observer-side: batch context can shape advisory audit design, but provider execution belongs to a later governed adapter path.
9. Hermes capability discovery stays planning-side: it can classify strengths and draft proposals, but it cannot enable tools, skills, scheduled jobs, memory authority, or execution behavior.

## Consequences for Initiative 132

1. The previous artifact-only workspace framing is no longer correct for this stage.
2. The previous git-branch submission framing is no longer the primary architecture.
3. Initiative 132 should integrate with Initiatives 124, 126, 127, 130, 133, and 134 as the active primitive stack.
4. The next implementation work for 132 should target heap integration, lifecycle emission, closeout linkage, workflow-backed promotion paths, and source-manifested Hermes advisory audit design.
