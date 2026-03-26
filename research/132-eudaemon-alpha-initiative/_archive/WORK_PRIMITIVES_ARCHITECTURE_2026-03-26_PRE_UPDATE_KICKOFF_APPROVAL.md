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

### Promotion path
- Heap note -> proposal / DPub / workflow draft / contribution update
- Closeout task -> verified completion artifact
- Workflow draft -> definition -> instance
- Initiative plan -> ContributionGraph lineage and portfolio governance

## Principles for This Stage

1. Boundary first: Nostra governs what exists; Cortex governs how active work is projected and executed.
2. Promotion over mutation: exploratory work starts on the heap and is promoted into governed artifacts when it matters.
3. Do not overload primitives: notes are not initiatives, and initiatives are not runtime tasks.
4. Prefer implemented contracts over speculative models: use existing heap, closeout, action-plan, and workflow surfaces before inventing anything new.
5. References inform design; they do not outrank local authority artifacts.

## Consequences for Initiative 132

1. The previous artifact-only workspace framing is no longer correct for this stage.
2. The previous git-branch submission framing is no longer the primary architecture.
3. Initiative 132 should integrate with Initiatives 124, 126, 127, 130, 133, and 134 as the active primitive stack.
4. The next implementation work for 132 should target heap integration, lifecycle emission, closeout linkage, and workflow-backed promotion paths.
