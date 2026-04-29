# Eudaemon Alpha Runtime Expansion Authority Packet: Steward-Reviewed Heap Emission

## Status

- Packet id: `initiative-132-runtime-expansion-steward-reviewed-heap-emission-v1`
- Initiative: `132-eudaemon-alpha-initiative`
- Status: proposed
- Created: 2026-04-29
- Authority mode: recommendation-only until separately approved for implementation
- Runtime identity: `agent:eudaemon-alpha-01`
- Depends on: `initiative-132-runtime-expansion-context-bundle-prep-v1`

## Purpose

Define the next safe expansion after real-heap context bundle prep: a bounded one-shot worker mode that emits exactly one steward-reviewed rich-text heap block from explicit operator-provided content.

This packet is intentionally a publication gate, not a cognition, planning, workflow, or execution gate. It lets Eudaemon Alpha prove that a prepared local observation can be promoted into the heap only when the operator supplies the publication payload and approval metadata.

## Preconditions

The following must remain true before implementation or activation:

1. Real-heap context bundle prep proof remains passed on the VPS.
2. `cortex-gateway.service` and `cortex-worker.service` are active.
3. Host-mode runtime authority check passes.
4. Production auth posture remains:
   - `NOSTRA_AUTHZ_DEV_MODE=0`
   - `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`
   - `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
   - `NOSTRA_AGENT_ID=agent:eudaemon-alpha-01`
5. The operator provides an explicit target space ID.
6. The operator provides an explicit block title, block body, and approval reference.
7. The operator explicitly enables steward-reviewed heap emission for a single run.

## Allowed Gateway Calls

The worker may call only these loopback gateway endpoints:

1. `GET /api/system/whoami`
2. `POST /api/cortex/studio/heap/emit`

Allowed emit constraints:

1. `schema_version` must be `1.0.0`.
2. `mode` must be `heap`.
3. `space_id` must come from an operator-provided environment value or equivalent local operator packet.
4. `source.agent_id` must be `agent:eudaemon-alpha-01`.
5. `source.request_id` must include the operator approval reference.
6. `block.type` must be a fixed v1 value such as `eudaemon_evidence_note`.
7. `block.title` must be operator-provided.
8. `content.payload_type` must be `rich_text`.
9. `content.rich_text.plain_text` must be operator-provided and capped by implementation, with a recommended v1 cap of `4000` characters.
10. `block.attributes` must include the packet ID, approval reference, and source observation artifact path when provided.
11. The implementation must write one local publication observation artifact containing the emit response summary.

## Forbidden Gateway Calls

The worker must not call:

1. `GET /api/cortex/studio/heap/blocks`
2. `GET /api/cortex/studio/heap/changed_blocks`
3. `POST /api/cortex/studio/heap/blocks/context`
4. heap pin, delete, feedback, steward-gate validate, or steward-gate apply endpoints
5. proposal endpoints
6. workflow draft or workflow instance endpoints
7. provider, runtime-host, auth-binding, or execution-binding inventory endpoints
8. local gateway queue mutation endpoints
9. any external provider/model endpoint

This packet deliberately starts from an operator-supplied publication payload. It does not authorize the worker to discover, select, summarize, or synthesize heap content during the emission pass.

## Allowed Behavior

The steward-reviewed heap emission slice may:

1. Load existing worker configuration and HPKE key material.
2. Confirm gateway production-auth posture through `/api/system/whoami`.
3. Parse explicit operator-provided publication fields.
4. Build one `EmitHeapBlockRequest` with rich-text content only.
5. Submit exactly one `POST /api/cortex/studio/heap/emit` request to the loopback gateway.
6. Record the response fields `accepted`, `artifactId`, `blockId`, `opId`, `idempotent`, `sourceOfTruth`, and `fallbackActive` when present.
7. Write one local JSON publication observation artifact under deployment state or logs.
8. Exit after one bounded pass.

## Forbidden Behavior

The steward-reviewed heap emission slice must not:

1. Poll continuously.
2. Select source blocks autonomously.
3. Build context bundles.
4. Summarize or synthesize new conclusions from heap content.
5. Create proposals, workflow drafts, or agent contributions.
6. Submit provider jobs.
7. Call external model/provider APIs.
8. Read provider keys or print secrets.
9. Mutate the repo or deployment mirror.
10. Mutate production runtime state outside the single heap emit request.
11. Mutate Nostra graph state.
12. Inspect operator-only topology surfaces by default.
13. Invoke shell/code execution or untrusted executor paths.
14. Emit more than one heap block per run.
15. Treat success as authorization for autonomous publication, provider cognition, live polling, or execution.

## Proposed Activation Contract

Implementation should add an explicit opt-in flag rather than changing the default worker loop.

Suggested local controls:

```bash
NOSTRA_WORKER_STEWARD_REVIEWED_HEAP_EMIT=1
NOSTRA_WORKER_HEAP_EMIT_SPACE_ID=<space id>
NOSTRA_WORKER_HEAP_EMIT_TITLE=<operator-approved title>
NOSTRA_WORKER_HEAP_EMIT_BODY=<operator-approved body>
NOSTRA_WORKER_HEAP_EMIT_APPROVAL_REF=<approval id or evidence ref>
NOSTRA_WORKER_HEAP_EMIT_SOURCE_ARTIFACT=<optional local observation artifact path>
NOSTRA_WORKER_HEAP_EMIT_BODY_LIMIT=4000
```

Default behavior must remain passive heartbeat with runtime polling disabled. A one-shot heap emission run should be easy to disable by unsetting the flag and restarting the worker service.

## Expected Publication Observation Artifact

The output artifact should be local and immutable for the run, for example:

```text
/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-steward-reviewed-heap-emission-<UTC>.json
```

Required fields:

1. `schemaVersion`
2. `packetId`
3. `observedAt`
4. `agentId`
5. `gatewayBaseUrl`
6. `spaceId`
7. `approvalRef`
8. `sourceObservationArtifact`
9. `bodyLength`
10. `bodyLimit`
11. `authzDevMode`
12. `allowUnverifiedRoleHeader`
13. `agentIdentityEnforcement`
14. `workerMode`
15. `heapEmit`
16. `checks`
17. `errors`
18. `exitStatus`

The `heapEmit` field should contain the gateway response summary only. The artifact must not contain provider keys, bearer tokens, cookies, SSH details, full environment dumps, unredacted runtime topology, raw source heap payloads, or external model output.

## Verification Commands

Before implementation PR:

```bash
bash scripts/check_agent_preflight_contract.sh
bash scripts/check_dynamic_config_contract.sh
cargo check --manifest-path nostra/worker/Cargo.toml
```

After implementation PR:

```bash
NOSTRA_WORKER_STEWARD_REVIEWED_HEAP_EMIT=1 \
NOSTRA_WORKER_HEAP_EMIT_SPACE_ID=initiative-132 \
NOSTRA_WORKER_HEAP_EMIT_TITLE="Eudaemon Alpha evidence note" \
NOSTRA_WORKER_HEAP_EMIT_BODY="Operator-approved test publication." \
NOSTRA_WORKER_HEAP_EMIT_APPROVAL_REF=<approval ref> \
cargo run --manifest-path nostra/worker/Cargo.toml

bash scripts/check_vps_runtime_authority.sh --repo-contract
```

On VPS after promotion, if separately authorized:

```bash
systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
sudo -u nostra env \
  NOSTRA_WORKER_STEWARD_REVIEWED_HEAP_EMIT=1 \
  NOSTRA_WORKER_HEAP_EMIT_SPACE_ID=<space id> \
  NOSTRA_WORKER_HEAP_EMIT_TITLE=<operator-approved title> \
  NOSTRA_WORKER_HEAP_EMIT_BODY=<operator-approved body> \
  NOSTRA_WORKER_HEAP_EMIT_APPROVAL_REF=<approval ref> \
  /srv/nostra/eudaemon-alpha/repo/nostra/worker/target/release/cortex_worker
```

## Acceptance Criteria

1. Passive default behavior is unchanged.
2. Steward-reviewed heap emission mode requires explicit operator opt-in.
3. The worker exits after one bounded pass.
4. The worker calls only allowed loopback gateway endpoints.
5. The worker submits exactly one heap emit request.
6. The emitted block uses operator-provided rich-text content only.
7. The worker records local publication evidence without exposing secrets or raw source heap payloads.
8. No context bundle prep, proposal, workflow, provider, repo, deploy, broad runtime, graph, polling, or execution behavior occurs.
9. Host-mode authority checks remain green after deployment.
10. Any durable proof is promoted manually into Initiative 132 only after operator review.

## Follow-Up Gates

Only after this packet is implemented and validated should Initiative 132 consider a later packet for:

1. provider-backed cognition without publication authority,
2. proposal or workflow-draft projection,
3. steward-gate validation/apply integration,
4. live polling, or
5. execution-worker authority.

Each follow-up must define its own allowed reads, writes, identity checks, output artifacts, rollback path, and forbidden actions.
