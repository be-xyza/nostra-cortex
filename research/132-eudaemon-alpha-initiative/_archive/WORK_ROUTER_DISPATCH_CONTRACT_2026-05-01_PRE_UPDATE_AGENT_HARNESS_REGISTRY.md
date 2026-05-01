# Work Router Dispatch Contract

**Initiative**: 132 Eudaemon Alpha
**Status**: Draft contract
**Created**: 2026-04-30
**Authority mode**: recommendation-only
**Scope**: Transport-neutral work dispatch, approval, and code-change routing for low-risk forward motion

## Purpose

This contract defines a transport-neutral dispatch lane for moving approved work forward without making a chat surface, Hermes profile, or VPS process into hidden authority.

The goal is to support fast interaction through Telegram, Cortex Web, CLI, email, Matrix, or future transports while keeping the actual authority chain explicit:

```text
Intent -> WorkRouterDecisionV1 -> DispatchRequestV1 -> DispatchDecisionV1 -> CodeChangeRequestV1 -> verification -> review gate
```

The dispatch transport carries prompts and decisions. It does not own authority, mutate code, deploy services, or bypass steward gates.

Schemas:

- [DispatchRequestV1.schema.json](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/schemas/DispatchRequestV1.schema.json)
- [DispatchDecisionV1.schema.json](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/schemas/DispatchDecisionV1.schema.json)
- [CodeChangeRequestV1.schema.json](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/schemas/CodeChangeRequestV1.schema.json)
- [WorkRouterDecisionV1.schema.json](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/schemas/WorkRouterDecisionV1.schema.json)
- [DispatchTransportReceiptV1.schema.json](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/schemas/DispatchTransportReceiptV1.schema.json)

Local validation command:

```bash
bash scripts/check_work_router_dispatch.sh
```

## Naming

Use generic dispatch language in governed contracts and code.

| Term | Meaning |
|---|---|
| `DispatchTransport` | Replaceable carrier for prompts and decisions, such as Telegram, Cortex Web, CLI, Matrix, email, or webhook relay |
| `WorkRouterV1` | Cortex routing process that classifies task shape, risk, authority ceiling, and next action |
| `DispatchRequestV1` | Prompt sent to a human or steward through a transport |
| `DispatchDecisionV1` | Structured response to a dispatch request |
| `CodeChangeRequestV1` | Bounded request for patch preparation or isolated implementation |
| `DispatchTransportReceiptV1` | Delivery/read/reply receipt from the transport adapter |

Avoid naming authority levels after a specific messenger. Telegram is one adapter, not the model.

## Dispatch Authority Levels

| Level | Meaning | Default Status |
|---|---|---|
| `D0` | Observe, summarize, and report status only | allowed |
| `D1` | Prepare plan, advisory summary, or developer handoff | allowed |
| `D2` | Create a patch in an isolated request worktree after explicit approval | future-gated |
| `D3` | Stage or commit after explicit approval | future-gated |
| `D4` | Push branch or open PR after explicit approval | future-gated |
| `D5` | Deploy, runtime mutation, canister call, or production operation | steward/operator-gated |

For the first implementation slice, only `D0` and `D1` are enabled. `D2` may be designed but must remain fail-closed until the isolated worktree, lifecycle, and verification gates are implemented and validated.

## Boundary

Nostra defines what exists:

- governed contributions
- spaces
- proposals
- workflow definitions
- policies and authority records
- durable lineage and promotion surfaces

Cortex defines how work runs:

- routing decisions
- dispatch prompts
- transport adapters
- lifecycle records
- heap and closeout projections
- verification and replay evidence
- execution adapters after governance approval

Hermes remains advisory:

- `hermes132` may inform advisory synthesis and drift review.
- `hermescortexdev` may produce patch-prep handoffs.
- No Hermes profile may dispatch unattended code mutation, commit, push, deploy, or runtime mutation.

The VPS agent may host the router loop only after runtime authority validation. In v1 it should orchestrate and notify; it should not mutate source code.

## Routing Rules

Every routed item must preserve:

1. source task or intent reference
2. risk level
3. authority ceiling
4. requested dispatch level
5. transport used
6. human/steward decision
7. required verification
8. evidence or lifecycle reference

Default route matrix:

| Condition | Route |
|---|---|
| Intent unclear | `D0` status plus clarification request |
| Advisory synthesis needed | `D1` Hermes advisory or heap note |
| Patch-prep needed | `D1` `hermescortexdev` handoff |
| Low-risk code change, approved | `D2` candidate only after implementation gate exists |
| Medium-risk code change | `D1` plan plus human review |
| High-risk or structural change | steward gate |
| Auth/provider/runtime topology | operator-only path |
| Deploy, canister call, production operation | `D5` separate steward/operator approval |

## Risk Classifier

| Risk | Examples | Max Default Level |
|---|---|---|
| `low` | bounded docs, tests, examples, narrow UI copy, isolated fixture updates with checks | `D1` now, `D2` later |
| `medium` | shared services, API/client contracts, noncritical runtime behavior, broad refactors | `D1` |
| `high` | auth, provider routing, runtime host status, execution bindings, workflow authority, canister interfaces | steward gate |
| `structural` | governance policy, schemas, constitutions, deployment authority, graph mutation, naming standards | steward gate |

Low-risk classification is not enough by itself. The router must also know the allowed file/module scope and required checks.

## Code Change Dispatch

Code changes must flow through a `CodeChangeRequestV1`.

Allowed v1 outputs:

- patch-prep handoff
- candidate patch plan
- verification command list
- risk note
- blocked-state summary

Future `D2` outputs, once governed:

- isolated request worktree path
- changed file list
- patch diff reference
- verification results
- lifecycle record
- review prompt

Implementation approval does not imply commit, push, PR, deploy, or evidence promotion approval. Each higher action requires its own dispatch decision.

## Transport Requirements

A `DispatchTransport` adapter must:

1. preserve the request id
2. preserve the decision id
3. identify the transport without embedding transport-specific authority semantics
4. record delivery and reply timestamps
5. support idempotency or duplicate detection
6. never expose operator-only runtime topology by default
7. never turn a message reply into execution without a matching dispatch request and authority ceiling

## VPS Runtime Role

The VPS agent may eventually run:

- queue observation
- status digest generation
- dispatch request delivery
- decision receipt ingestion
- lifecycle/event emission
- approved `D0` and `D1` routing

The VPS agent must not run source mutation in v1. A later `D2` host lane requires:

1. host-mode VPS authority validation
2. isolated request worktrees
3. explicit allowed write scope
4. deterministic verification
5. lifecycle and replay evidence
6. fail-closed denylist for sensitive surfaces
7. separate commit/push/deploy approvals

## Forbidden Route Collapse

The router must reject:

- dispatch decisions without a matching request
- requested level higher than the authority ceiling
- Telegram-specific or transport-specific authority fields
- `D2+` code mutation for high or structural risk
- deploy/runtime/canister operations below `D5`
- auth/provider/runtime topology reads on general or agent-facing paths
- Hermes as executor
- `hermescortexdev` as advisory observer
- automatic commit, push, PR, deploy, evidence promotion, or graph mutation

## Minimal Implementation Slices

### Slice 1: D0-D1 Dry Run

- Validate schemas and fixtures.
- Route one example task into a transport-neutral dispatch request.
- Accept a structured decision fixture.
- Produce a `D1` patch-prep `CodeChangeRequestV1`.
- Generate a non-mutating developer handoff from the approved `CodeChangeRequestV1`.
- Emit a local `WorkRouterRunV1` artifact that links input, request, receipt, decision, approved bundle, and handoff references.
- Support a pending-run queue where a run can be created before a decision and completed later by applying a matching `DispatchDecisionV1`.
- Emit no repo mutation and no runtime mutation.

Implementation note: the dry-run router must not emit a `CodeChangeRequestV1` before an approval exists. It should emit the router decision and dispatch request first. A separate decision-application step may mint `CodeChangeRequestV1` only after a valid `DispatchDecisionV1` references the request and stays within the authority ceiling.

### Slice 2: Transport Adapter

- Implement a Telegram adapter behind `DispatchTransport`.
- Keep router core transport-neutral.
- Send compact prompts and ingest structured decisions.
- Mirror details into Cortex Web/heap for inspection.

Before a live adapter, a dry-run transport renderer should prove that any `DispatchRequestV1` can become a compact human-facing prompt and that delivery can be represented as `DispatchTransportReceiptV1` without embedding messenger-specific authority semantics.

Pending dispatch messages may be exported as `DispatchTransportEnvelopeV1` records into a local outbox. A real Telegram, Matrix, email, or Cortex Web adapter should consume those envelopes rather than deriving authority from transport-specific state.

The first adapter should be `DispatchTransportAdapterV1` in CLI dry-run mode. It may read outbox envelopes, print/send the message body, validate a structured local `DispatchDecisionV1`, and call the pending-run decision application command. This proves the adapter lifecycle before any live messenger API is introduced.

A transport reply parser may convert constrained replies such as `approve`, `reject`, `revise`, `escalate`, or `pause` into `DispatchDecisionV1`, but only in the context of a specific `DispatchTransportEnvelopeV1`. Free-form chat text must not become execution authority.

The Telegram adapter must remain a guarded transport adapter:

- dry-run mode is the default safe test path
- live mode requires explicit `WORK_ROUTER_TELEGRAM_BOT_TOKEN`
- chat routing must use the envelope channel or explicit operator configuration
- replies must become `DispatchDecisionV1` through the generic parser before they can affect a run
- no Telegram message may authorize beyond the request authority ceiling

Telegram receive must be proven fixture-first. A dry-run ingester may read saved `getUpdates` payloads, extract reply text, match exactly one pending envelope for the chat, parse known commands or decision aliases, and record processed update ids for idempotency. Unknown replies are routed to the local unknown-response log.

## D0 Query and Unknown Reply Handling

Read-only chat commands are allowed at `D0`:

- `help`
- `pending`
- `status`
- `latest`
- `show <run_id>`

Decision aliases such as `approved`, `yes`, `yea`, `yeah`, `ok`, `continue`, or `proceed` may normalize to `approve`, but they must still be attached to a matching dispatch envelope before they can affect a run.

Unknown text must be recorded locally for routing review and must not become a decision or task by implication. This unknown-response log is a training/routing improvement surface, not authority.

Aliases live in `dispatch_aliases.v1.json` so routing can improve through reviewed edits instead of hidden code changes. Unknown route reviews may recommend a mapping, but applying the review does not automatically edit the alias registry.

### Slice 3: D2 Candidate Patch Lane

- Add isolated request worktree creation.
- Add allowed write-scope enforcement.
- Add verification gates.
- Return patch evidence for review.
- Keep commit/push/deploy separate.

## Acceptance Criteria

This contract is valid only if:

1. transport remains generic
2. authority levels are not named after Telegram or any other messenger
3. a dispatch decision cannot exceed the request authority ceiling
4. low-risk code work still uses explicit `CodeChangeRequestV1`
5. `D0-D1` can run without source mutation
6. `D2+` remains fail-closed until governed implementation gates exist
7. VPS router participation starts with orchestration and notification only
8. all route decisions are replayable from structured records
