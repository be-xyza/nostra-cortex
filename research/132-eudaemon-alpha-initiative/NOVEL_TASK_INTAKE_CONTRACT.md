# Nostra Cortex Novel Task Intake Contract

**Initiative**: 132 Eudaemon Alpha
**Status**: Draft contract
**Created**: 2026-04-27
**Authority mode**: recommendation-only
**Scope**: Bounded intake path for new or novel tasks using existing Nostra/Cortex primitives

## Purpose

This contract defines how a new or novel task should enter Nostra Cortex without inventing a new Eudaemon-specific task, workspace, or execution primitive.

The goal is to answer:

> Given a novel task, how should Nostra Cortex decide which primitive, authority level, planning path, execution path, evaluation path, and memory/promotion path should apply?

This is an intake and routing contract. It does not create a new runtime engine, workflow authority, Hermes authority lane, or autonomous promotion path.

Schema draft:

- [NovelTaskIntakeV1.schema.json](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/schemas/NovelTaskIntakeV1.schema.json)

Local validation command:

```bash
bash scripts/check_novel_task_intake.sh
```

This command validates the positive fixture and confirms the negative `hermescortexdev` advisory fixture is rejected.

## Boundary

Nostra defines what exists:

- governed contributions
- spaces
- proposals
- policies
- workflow definitions and lineage
- institutional memory and promotion surfaces

Cortex defines how work runs:

- heap runtime and context projection
- closeout ledgers
- workflow execution adapters
- capability/action/navigation projection
- lifecycle records
- evaluation and approval surfaces

Hermes remains outside the runtime boundary. It may provide local advisory synthesis over explicit source manifests and audit units, but it cannot execute, mutate, schedule, install skills, run subagents, or act as workflow authority.

The current Hermes split must stay explicit:

| Hermes Profile | Intent | Allowed Output | Forbidden |
|---|---|---|---|
| `hermes132` | Initiative 132 advisory review, architecture synthesis, contradiction/drift detection, source-manifested audit units | session records, source-linked findings, synthesis artifacts, reviewed heap/evidence candidates | repo mutation, runtime mutation, provider execution, subagents, skill activation, workflow authority |
| `hermescortexdev` | Cortex developer patch-prep handoffs from approved local task packets | one developer handoff artifact with patch plan, likely files, verification commands, risks, acceptance criteria | claiming `hermes132` observer outputs, editing files, committing, pushing, deploying, runtime mutation, provider jobs, autonomous task selection |

`hermescortexdev` is the correct profile name. `hermecortexdev` is a typo and must be rejected rather than treated as an alias. Intake packets must name which Hermes profile, if any, is being invoked.

### Hermes Graph Context Boundary

ContributionGraph and knowledge graph storage may inform Hermes only as bounded, read-only context.

- `hermes132` may request `memoryPolicy.*.surface = "contribution_graph"` only for `recommendedPrimitive = "hermes_advisory"`.
- A Hermes graph-context intake must include `no_runtime_mutation` and `no_repo_mutation` authority constraints.
- `hermescortexdev` must not request `contribution_graph` memory as Hermes authority. It may mention graph findings only inside a local patch-prep handoff after operator/Codex review.
- No Hermes profile may directly write graph artifacts, call graph mutation endpoints, promote graph-derived evidence, or bypass steward review.

## Intake Object

A novel task intake should be represented as a bounded packet before execution:

```text
NovelTaskIntakeV1
```

Minimum fields:

| Field | Purpose |
|---|---|
| `task_id` | Stable local id for the intake packet |
| `space_id` | Target Space or explicit `unknown` |
| `requester` | Human, steward, agent, or system source |
| `intent` | Plain-language goal |
| `desired_outcome` | What success should look like |
| `known_context_refs` | Heap blocks, artifacts, proposals, workflow refs, files, or evidence refs |
| `constraints` | Budget, deadline, authority, privacy, runtime, or policy constraints |
| `required_capabilities` | Candidate capabilities from the space capability overlay |
| `uncertainties` | Missing facts, ambiguous scope, unknown authority, or unresolved dependencies |
| `risk_level` | `low`, `medium`, `high`, or `structural` |
| `recommended_primitive` | Heap, closeout, workflow, proposal, chronicle, steward review, or stop |
| `authority_ceiling` | Maximum allowed authority level for this intake |
| `evaluation_requirements` | Checks needed before promotion or execution |
| `memory_policy` | Short-term, working, execution, or institutional memory handling |
| `promotion_path` | How output may become durable governed material |
| `hermes_profile` | Optional; only `hermes132`, `hermescortexdev`, or `none` for this stage |

The packet may be projected as a heap artifact, proposal attachment, workflow intent input, or Hermes source packet, depending on route.

## Intake Flow

### 1. Capture Intent

Start with the user's goal and desired outcome. Do not choose a workflow or tool yet.

Required questions for the intake layer:

1. What is the user trying to accomplish?
2. What should exist when the task is complete?
3. Is this exploratory, operational, executable, structural, or publishable?
4. What Space owns the context and authority?

Default route:

- If the intent is vague, create a heap note or solicitation for clarification.
- If the intent is clear but authority is unclear, use proposal-first or steward-review routing.

### 2. Assemble Context

Use the narrowest sufficient context bundle.

Preferred sources:

1. Heap context bundle via `POST /api/cortex/studio/heap/blocks/context`
2. Relevant workflow definition, instance, checkpoint, or outcome
3. Closeout task and evidence refs
4. Governed initiative docs and promoted evidence
5. Space capability graph and navigation/action plans
6. Hermes source packet only when the pass is advisory and manifest-scoped

Avoid raw repo scanning as the default runtime context path. Developer codebase inspection remains an operator/developer action, not a runtime primitive.

### 3. Classify Task Shape

Classify the task before selecting a path.

| Shape | Use When | Primary Primitive |
|---|---|---|
| Exploratory note | The task is sensemaking, drafting, or early synthesis | Heap block |
| Operational follow-through | The task is concrete remediation, release cleanup, or checklist work | Closeout ledger |
| Executable process | The task needs durable steps, checkpoints, signals, or replay | Workflow artifact pipeline |
| Governance change | The task changes policy, schema, authority, or durable platform state | Proposal / steward gate |
| Publication or memory | The task should become institutional narrative or durable report | Chronicle / DPub / governed evidence |
| Advisory audit | The task asks for contradiction, drift, or architecture synthesis | Hermes advisory pass, then heap/proposal review |
| Unsafe or unclear | Authority, scope, identity, or risk is unresolved | Stop or steward escalation |

### 4. Select Authority Ceiling

Use the smallest authority that can satisfy the request.

| Level | Meaning | Allowed Route |
|---|---|---|
| L0 | Read-only | Context assembly, advisory analysis |
| L1 | Suggestion-only | Heap proposal, recommendation, draft |
| L2 | Limited write | Space-scoped sandbox or bounded runtime write |
| L3 | Governed write | Steward-approved promotion or reviewed mutation |
| L4 | Autonomous bounded workflow | Deferred except where explicitly governed |

Defaults:

- Novel tasks start at L0 or L1.
- Hermes is always advisory and local for this stage.
- Structural mutations require steward review.
- Provider/runtime/auth topology remains operator-only.

### 5. Select Capability Path

Capability selection should be explicit, not implied by the agent's preference.

Inputs:

- `required_capabilities`
- space capability overlay
- role and claim eligibility
- budget constraints
- confidence and uncertainty
- deterministic check availability
- whether the task is exploratory, operational, executable, or structural

Decision rules:

1. If no capability matches, route to proposal or steward review instead of improvising.
2. If a capability exists but required claims are missing, stop or request authorization.
3. If multiple capabilities match, prefer the lowest-authority path with the strongest evaluation route.
4. If execution eligibility is unclear, stay recommendation-only.

### 6. Select Planning Path

Use the planning primitive that matches task shape.

| Need | Planning Path |
|---|---|
| Clarify or synthesize | Heap note or advisory artifact |
| Follow up on known work | Closeout task |
| Draft durable executable steps | `WorkflowIntentV1` -> `WorkflowDraftV1` |
| Compare alternatives | Workflow candidate set or Hermes advisory synthesis |
| Require human review | Human checkpoint, proposal review, or steward gate |
| Preserve narrative | Chronicle/DPub draft |

Do not treat a research initiative plan as an executable workflow instance.

### 7. Evaluate Before Promotion

Every route needs an evaluation posture, even if lightweight.

Minimum evaluation by route:

| Route | Evaluation |
|---|---|
| Heap note | Source links, confidence, uncertainty notes |
| Closeout task | Completion evidence and blocker state |
| Workflow draft | Validation, compile result, policy flags, review status |
| Proposal | Steward review, decision record, lineage |
| Hermes advisory output | Source manifest, audit unit, session record, synthesis artifact |
| Runtime execution | Lifecycle record, trace, checkpoint/outcome, benchmark where applicable |

Evaluation does not automatically grant promotion.

### 8. Assign Memory Policy

Use the existing hierarchy:

| Memory Layer | Surface |
|---|---|
| Short-term context | Active context bundle |
| Working memory | Heap blocks and workflow context |
| Execution memory | Lifecycle records, trace, replay artifacts, closeout evidence |
| Institutional memory | Proposals, governed evidence, Chronicle/DPub, ContributionGraph |

Rules:

1. Heap material is exploratory until promoted.
2. Runtime logs remain mutable/local unless promoted as immutable evidence.
3. Hermes source packets are input-side aids, not authority.
4. Institutional memory requires governed review or promotion.

### 9. Decide Promotion Path

Outputs should move through explicit promotion routes:

| Output | Promotion Path |
|---|---|
| Insight | Heap note -> proposal or evidence |
| Recommendation | Heap/proposal -> steward review |
| Executable process | Workflow draft -> definition -> instance |
| Completed remediation | Closeout task -> evidence |
| Architecture finding | Evidence note -> decision/proposal if needed |
| Narrative record | Chronicle/DPub draft -> governed publication |
| Hermes synthesis | Local artifact -> heap/proposal/evidence after review |

No output from a novel task directly mutates Nostra graph authority without the appropriate governed path.

## Routing Matrix

| Condition | Route |
|---|---|
| Intent unclear | Heap solicitation or clarification note |
| Context missing | Context bundle request or Hermes source-packet prep |
| Authority unclear | Recommendation-only plus steward escalation |
| Capability missing | Proposal for capability gap or steward review |
| High confidence, low risk | Heap task, closeout task, or workflow draft |
| Durable execution required | Workflow artifact pipeline |
| Human decision required | Human checkpoint, steward gate, or proposal review |
| Structural change requested | Proposal and steward gate |
| Runtime/provider/admin topology involved | Operator-only path |
| Advisory architecture synthesis requested | Hermes bounded pass, then reviewed projection |
| Publication or lineage needed | Governed evidence or Chronicle/DPub route |

## Uncertainty Policy

Uncertainty must affect routing.

| Uncertainty State | Required Behavior |
|---|---|
| Low uncertainty | Continue with bounded route and record confidence |
| Medium uncertainty | Add evaluation requirement or request human clarification |
| High uncertainty | Stay recommendation-only |
| Authority uncertainty | Steward escalation |
| Runtime safety uncertainty | Stop before execution |
| Source uncertainty | Require source manifest, evidence, or context bundle |

The intake packet should preserve uncertainty rather than hide it behind a confident plan.

## Cost Policy

Cost and latency should influence execution path.

Defaults:

1. Prefer deterministic checks before model calls when they answer the question.
2. Prefer live cognition for Phase 6 Eudaemon communication and low-latency loops.
3. Keep batch cognition advisory and secondary.
4. Require explicit budget labels for agent solicitations and benchmark-bearing runs.
5. Record token, latency, and cost evidence where provider execution is involved.
6. Do not use subscription or provider access as governance authority.

## Hermes-Specific Rule

Hermes may be used only when the route is advisory audit, contradiction detection, drift review, architecture synthesis, capability discovery, or source-bounded recommendation.

Use `hermes132` for these Initiative 132 advisory routes.

Use `hermescortexdev` only for developer patch-prep handoffs. A `hermescortexdev` output may inform Codex/operator implementation work, but it is not an Initiative 132 observer-lane session, finding, synthesis artifact, workflow artifact, or runtime approval.

Hermes pass requirements:

1. one explicit `SourceManifestV1`
2. one explicit `AuditUnitV1`
3. `maxSteps = 1`
4. `writeAccess = false`
5. one session record
6. optional source-linked findings
7. one synthesis artifact

Forbidden for Hermes in this stage:

- repo mutation
- runtime mutation
- provider execution
- batch submission or polling
- queue runners
- subagents
- skill activation or installation
- memory authority
- unattended execution
- workflow authority

## Minimal Example

User asks:

> Use Nostra to investigate whether a new agent capability is needed for reviewing workflow failures.

Intake result:

| Field | Value |
|---|---|
| Shape | Advisory audit / capability gap review |
| Primitive | Heap solicitation plus optional Hermes bounded pass |
| Authority | L0/L1 |
| Capability path | Space capability overlay lookup for workflow/evaluation/capability review |
| Planning path | SourceManifest + AuditUnit, then synthesis artifact |
| Evaluation | Source-linked findings and steward review |
| Memory | Local Hermes artifact -> heap/evidence after review |
| Promotion | Proposal only if a real capability gap survives review |

Rejected routes:

- direct skill activation
- direct workflow adapter creation
- direct runtime mutation
- autonomous agent execution

## Acceptance Criteria

This contract is useful only if:

1. a novel task can be classified before execution
2. primitive selection is explicit
3. authority ceiling is visible
4. uncertainty changes routing
5. capability selection is tied to Space overlays and claims
6. evaluation is attached before promotion
7. memory/promotion path is explicit
8. Hermes stays advisory-only
9. no bespoke Eudaemon-only task/workspace primitive is introduced

## Follow-Through

Recommended next implementation slices:

1. Add a heap projection pattern for intake packets.
2. Add a bounded Hermes audit unit using this contract as input.
3. Connect task routing UI language to this matrix without changing runtime authority.
4. Add tests that reject unsafe route collapse: initiative plan as workflow, Hermes as executor, provider inventory as agent-readable, and structural mutation without steward gate.
