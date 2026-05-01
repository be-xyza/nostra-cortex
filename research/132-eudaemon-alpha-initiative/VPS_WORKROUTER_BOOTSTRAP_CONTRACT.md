# VPS WorkRouter Bootstrap Contract

**Initiative**: 132 Eudaemon Alpha
**Status**: Draft contract
**Created**: 2026-05-01
**Authority mode**: recommendation-only
**Scope**: D0-D1 WorkRouter deployment posture on the existing VPS

## Decision

WorkRouter v1 should be installed on the already provisioned VPS as a separate, low-authority Cortex service beside the existing gateway and worker runtime. It does not require new infrastructure for v1.

Recommended service name:

```text
cortex-workrouter.service
```

Repo template:

- [cortex-workrouter.service](/Users/xaoj/ICP/ops/hetzner/systemd/cortex-workrouter.service)

Local validation:

```bash
bash scripts/check_vps_workrouter_bootstrap.sh
```

This separation is operational, not philosophical. WorkRouter remains part of the Cortex execution layer, but its authority boundary differs from `cortex-gateway` and `cortex_worker`.

## Placement

```text
Existing VPS
- cortex-gateway
- cortex_worker / eudaemon-alpha runtime candidate
- cortex-workrouter.service
- local WorkRouter logs and run evidence
- transport adapter credentials only when explicitly enabled
```

Local operator machine:

```text
- Codex/operator code mutation authority
- Hermes profiles for advisory and patch-prep handoffs
- local validation and fixture development
- optional direct transport adapter testing
```

## Allowed V1 Actions

The VPS WorkRouter may:

1. load `AgentHarnessRegistryV1`, `AgentProfileV1`, `AgentPromotionGateV1`, and `AgentRunEvidenceV1` records
2. observe approved queues, pending dispatches, and health summaries
3. classify work into D0-D1 only
4. create `DispatchRequestV1` records
5. export `DispatchTransportEnvelopeV1` messages
6. ingest replies only when tied to a specific request id or pending envelope
7. emit `DispatchDecisionV1` records within the dispatch ceiling
8. capture unknown replies into unknown-response review records
9. produce handoff records for Codex/operator or advisory profiles
10. notify through configured adapters

## Forbidden V1 Actions

The VPS WorkRouter must not:

1. edit source files
2. stage, commit, push, or open pull requests
3. deploy services
4. call canisters or mutate runtime state
5. mutate the Nostra graph
6. expose provider inventory, runtime topology, auth bindings, or discovery diagnostics to agent-facing or general routes
7. treat Hermes, Telegram, Cortex Web, CLI, Matrix, email, or webhook messages as implicit authority
8. raise authority above the profile or dispatch ceiling through transport metadata
9. store repo mutation credentials, deploy credentials, or canister mutation credentials

## Minimum Runtime Configuration

The service should fail closed unless the equivalent settings are present:

```text
WORK_ROUTER_MAX_DISPATCH_LEVEL=D1
WORK_ROUTER_SOURCE_MUTATION_ALLOWED=0
WORK_ROUTER_RUNTIME_MUTATION_ALLOWED=0
WORK_ROUTER_REQUIRE_REQUEST_ID=1
WORK_ROUTER_UNKNOWN_REPLIES_REVIEW_ONLY=1
WORK_ROUTER_TRANSPORTS_ENABLED=cli
```

Live transport credentials are not required for bootstrap. The first VPS pass may run with CLI or file-based outbox mode only.

## Promotion Gates

Before enabling outbound live transport:

1. local WorkRouter dispatch checks pass
2. agent operating model checks pass
3. VPS runtime authority check passes
4. service account has no mutation credentials
5. logs and evidence paths are writable
6. unknown replies are captured without state changes
7. duplicate replies are idempotent
8. operator-only topology surfaces remain protected

Before enabling Hermes Gateway or Telegram live ingestion:

1. adapter is declared as `DispatchTransportAdapterV1`
2. adapter max authority is D0-D1
3. replies require a matching request id or envelope
4. ambiguous text routes to unknown-response review
5. transport-specific fields cannot raise authority

## Initial Bootstrap Sequence

1. Deploy the validated repo revision to the existing VPS through the governed operator-local promotion path.
2. Install the WorkRouter service with live transports disabled and CLI/file-outbox only.
3. Run the observe-loop service in one-shot mode before enabling the long-running unit.
4. Validate that it emits heartbeat output and `AgentRunEvidenceV1` records under local runtime logs.
5. Enable outbound notification only.
6. Enable structured reply ingestion only after duplicate and unknown-reply checks pass.
7. Keep code mutation in Codex/operator until a separate D2 implementation lane is governed.

## Observe-Loop V1

The initial service loop is [work_router_observe_service.py](/Users/xaoj/ICP/scripts/work_router_observe_service.py), launched through [work_router_service_stub.sh](/Users/xaoj/ICP/scripts/work_router_service_stub.sh). The shell wrapper refuses to start unless the fail-closed D0-D1 environment is present.

The observe loop may:

1. list pending WorkRouter runs
2. export pending dispatch envelopes to a local outbox
3. write a local service heartbeat
4. write `AgentRunEvidenceV1` for the WorkRouter observe pass

The observe loop may not send live messages, ingest live replies, mutate source, mutate runtime state, deploy, call canisters, or expose topology.

## Alignment

WorkRouter is the bootstrapper for routing and orchestration, not an executor for code or runtime mutation. Hermes profiles, Telegram direct, Hermes Gateway, Cortex Web, CLI, Matrix, email, and webhooks remain replaceable adapters under Cortex contracts.
