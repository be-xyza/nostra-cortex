# Eudaemon Alpha Runtime Expansion Authority Packet: Read-Only Heap Delta

## Status

- Packet id: `initiative-132-runtime-expansion-readonly-heap-delta-v1`
- Initiative: `132-eudaemon-alpha-initiative`
- Status: proposed
- Created: 2026-04-28
- Authority mode: recommendation-only until separately approved for implementation
- Runtime identity: `agent:eudaemon-alpha-01`
- Depends on: `initiative-132-runtime-expansion-observe-once-v1`

## Purpose

Define the next safe expansion after observe-once: a bounded one-shot worker mode that reads only heap block list/delta metadata from the loopback Cortex gateway and writes one local observation artifact.

This packet is intentionally narrower than task execution. It gives Eudaemon Alpha visibility into recent working material without authorizing heap emission, context bundling, proposal creation, workflow drafting, provider calls, live polling, or mutation.

## Preconditions

The following must remain true before implementation or activation:

1. Observe-once worker proof remains passed on the VPS.
2. `cortex-gateway.service` and `cortex-worker.service` are active.
3. Host-mode runtime authority check passes.
4. Production auth posture remains:
   - `NOSTRA_AUTHZ_DEV_MODE=0`
   - `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`
   - `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
   - `NOSTRA_AGENT_ID=agent:eudaemon-alpha-01`
5. The operator provides an explicit bounded query window.
6. The operator explicitly enables the read-only heap delta mode for a single run.

## Allowed Gateway Reads

The worker may call only these loopback gateway endpoints:

1. `GET /api/system/whoami`
2. `GET /api/cortex/studio/heap/blocks`
3. `GET /api/cortex/studio/heap/changed_blocks`

Allowed query constraints:

1. `spaceId` must be operator-provided when supported by the endpoint.
2. `changedSince` or equivalent timestamp cursor must be operator-provided for delta reads.
3. A page/limit value must be capped by implementation, with a recommended v1 cap of `25`.
4. Response persistence must be summarized/redacted by default.

## Forbidden Gateway Calls

The worker must not call:

1. `POST /api/cortex/studio/heap/emit`
2. `POST /api/cortex/studio/heap/blocks/context`
3. heap pin, delete, feedback, steward-gate validate, or steward-gate apply endpoints
4. proposal endpoints
5. workflow draft or workflow instance endpoints
6. provider, runtime-host, auth-binding, or execution-binding inventory endpoints
7. local gateway queue mutation endpoints
8. any external provider/model endpoint

## Allowed Behavior

The read-only heap delta slice may:

1. Load existing worker configuration and HPKE key material.
2. Confirm gateway production-auth posture through `/api/system/whoami`.
3. Read a bounded heap block list or changed-block delta from the loopback gateway.
4. Count returned items and record stable block identifiers, titles, timestamps, space IDs, and source/type labels when present.
5. Write one local JSON observation artifact under deployment state or logs.
6. Exit after one bounded pass.

## Forbidden Behavior

The read-only heap delta slice must not:

1. Poll continuously.
2. Select work autonomously.
3. Emit heap blocks.
4. Build context bundles.
5. Create proposals, workflow drafts, or agent contributions.
6. Submit provider jobs.
7. Call external model/provider APIs.
8. Read provider keys or print secrets.
9. Mutate the repo or deployment mirror.
10. Mutate production runtime state.
11. Mutate Nostra graph state.
12. Inspect operator-only topology surfaces by default.
13. Invoke shell/code execution or untrusted executor paths.
14. Treat success as authorization for live polling or execution.

## Proposed Activation Contract

Implementation should add an explicit opt-in flag rather than changing the default worker loop.

Suggested local controls:

```bash
NOSTRA_WORKER_READONLY_HEAP_DELTA=1
NOSTRA_WORKER_HEAP_CHANGED_SINCE=<RFC3339 timestamp>
NOSTRA_WORKER_HEAP_SPACE_ID=<space id>
NOSTRA_WORKER_HEAP_LIMIT=25
```

Default behavior must remain passive heartbeat with runtime polling disabled. A one-shot heap delta run should be easy to disable by unsetting the flag and restarting the worker service.

## Expected Observation Artifact

The output artifact should be local and immutable for the run, for example:

```text
/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-readonly-heap-delta-<UTC>.json
```

Required fields:

1. `schemaVersion`
2. `packetId`
3. `observedAt`
4. `agentId`
5. `gatewayBaseUrl`
6. `spaceId`
7. `changedSince`
8. `limit`
9. `authzDevMode`
10. `allowUnverifiedRoleHeader`
11. `agentIdentityEnforcement`
12. `workerMode`
13. `heapRead`
14. `checks`
15. `errors`
16. `exitStatus`

The `heapRead` field should contain counts and redacted item summaries only. It must not contain provider keys, bearer tokens, cookies, SSH details, full environment dumps, unredacted runtime topology, or large raw heap payloads.

## Verification Commands

Before implementation PR:

```bash
bash scripts/check_agent_preflight_contract.sh
bash scripts/check_dynamic_config_contract.sh
cargo check --manifest-path nostra/worker/Cargo.toml
```

After implementation PR:

```bash
NOSTRA_WORKER_READONLY_HEAP_DELTA=1 \
NOSTRA_WORKER_HEAP_CHANGED_SINCE=2026-04-28T00:00:00Z \
NOSTRA_WORKER_HEAP_LIMIT=25 \
cargo run --manifest-path nostra/worker/Cargo.toml

bash scripts/check_vps_runtime_authority.sh --repo-contract
```

On VPS after promotion, if separately authorized:

```bash
systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
sudo -u nostra env \
  NOSTRA_WORKER_READONLY_HEAP_DELTA=1 \
  NOSTRA_WORKER_HEAP_CHANGED_SINCE=<RFC3339 timestamp> \
  NOSTRA_WORKER_HEAP_LIMIT=25 \
  /srv/nostra/eudaemon-alpha/repo/nostra/worker/target/release/cortex_worker
```

## Acceptance Criteria

1. Passive default behavior is unchanged.
2. Read-only heap delta mode requires explicit operator opt-in.
3. The worker exits after one bounded pass.
4. The worker calls only allowed loopback gateway endpoints.
5. The observation artifact is written under local deployment state or logs, not a governed repo path.
6. The artifact summarizes heap visibility without exposing secrets or large raw payloads.
7. No heap emit, context bundle POST, proposal, workflow, provider, repo, deploy, runtime, or graph mutation occurs.
8. Host-mode authority checks remain green after deployment.
9. Any durable evidence is promoted manually into Initiative 132 only after operator review.

## Follow-Up Gates

Only after this packet is implemented and validated should Initiative 132 consider a later packet for:

1. bounded context bundle preparation,
2. steward-reviewed heap emission,
3. proposal or workflow-draft projection,
4. provider-backed cognition,
5. live polling, or
6. execution-worker authority.

Each follow-up must define its own allowed reads, writes, identity checks, output artifacts, rollback path, and forbidden actions.
