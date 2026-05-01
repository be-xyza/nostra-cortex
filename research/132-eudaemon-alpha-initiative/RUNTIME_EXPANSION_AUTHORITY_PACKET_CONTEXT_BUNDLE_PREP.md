# Eudaemon Alpha Runtime Expansion Authority Packet: Context Bundle Prep

## Status

- Packet id: `initiative-132-runtime-expansion-context-bundle-prep-v1`
- Initiative: `132-eudaemon-alpha-initiative`
- Status: proposed
- Created: 2026-04-28
- Authority mode: recommendation-only until separately approved for implementation
- Runtime identity: `agent:eudaemon-alpha-01`
- Depends on: `initiative-132-runtime-expansion-readonly-heap-delta-v1`

## Purpose

Define the next safe expansion after read-only heap delta: a bounded one-shot worker mode that prepares a local context-bundle observation from operator-selected heap block IDs.

This packet is intentionally a context-preparation gate, not a publication or execution gate. It lets Eudaemon Alpha verify the Cortex heap context endpoint against explicit operator-selected material while preserving the boundary against autonomous block selection, heap emission, proposal creation, workflow drafting, provider calls, polling, runtime mutation, repo mutation, and graph mutation.

## Preconditions

The following must remain true before implementation or activation:

1. Read-only heap delta worker proof remains passed on the VPS.
2. `cortex-gateway.service` and `cortex-worker.service` are active.
3. Host-mode runtime authority check passes.
4. Production auth posture remains:
   - `NOSTRA_AUTHZ_DEV_MODE=0`
   - `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`
   - `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
   - `NOSTRA_AGENT_ID=agent:eudaemon-alpha-01`
5. The operator provides an explicit finite list of heap block IDs.
6. The operator explicitly enables context bundle prep for a single run.

## Allowed Gateway Calls

The worker may call only these loopback gateway endpoints:

1. `GET /api/system/whoami`
2. `POST /api/cortex/studio/heap/blocks/context`

Allowed request constraints:

1. `block_ids` must come from an operator-provided environment value or equivalent local operator packet.
2. The implementation must cap submitted block IDs, with a recommended v1 cap of `5`.
3. Empty `block_ids` must fail closed before calling the context endpoint.
4. The request must not include free-form search text, autonomous selection criteria, provider prompts, workflow intent, or proposal text.
5. Response persistence must be summarized/redacted by default.

## Forbidden Gateway Calls

The worker must not call:

1. `POST /api/cortex/studio/heap/emit`
2. heap pin, delete, feedback, steward-gate validate, or steward-gate apply endpoints
3. proposal endpoints
4. workflow draft or workflow instance endpoints
5. provider, runtime-host, auth-binding, or execution-binding inventory endpoints
6. local gateway queue mutation endpoints
7. any external provider/model endpoint

This packet also does not authorize using `GET /api/cortex/studio/heap/blocks` or `GET /api/cortex/studio/heap/changed_blocks` for autonomous context selection. Those endpoints may remain useful as separate operator evidence, but this context-prep gate begins from explicit operator-selected IDs.

## Allowed Behavior

The context bundle prep slice may:

1. Load existing worker configuration and HPKE key material.
2. Confirm gateway production-auth posture through `/api/system/whoami`.
3. Parse an explicit bounded list of operator-provided heap block IDs.
4. Request a context bundle for those exact block IDs from the loopback gateway.
5. Record returned block count, prepared timestamp, stable block identifiers, titles, block types, tags count, mentions count, update timestamps, and approximate payload-size metadata when available.
6. Write one local JSON observation artifact under deployment state or logs.
7. Exit after one bounded pass.

## Forbidden Behavior

The context bundle prep slice must not:

1. Poll continuously.
2. Select block IDs autonomously.
3. Emit heap blocks.
4. Create proposals, workflow drafts, or agent contributions.
5. Submit provider jobs.
6. Call external model/provider APIs.
7. Read provider keys or print secrets.
8. Mutate the repo or deployment mirror.
9. Mutate production runtime state.
10. Mutate Nostra graph state.
11. Inspect operator-only topology surfaces by default.
12. Invoke shell/code execution or untrusted executor paths.
13. Persist raw large `surface_json` values in the observation artifact.
14. Treat success as authorization for heap emission, workflow/proposal projection, live polling, or execution.

## Proposed Activation Contract

Implementation should add an explicit opt-in flag rather than changing the default worker loop.

Suggested local controls:

```bash
NOSTRA_WORKER_CONTEXT_BUNDLE_PREP=1
NOSTRA_WORKER_CONTEXT_BLOCK_IDS=<comma-separated heap block ids>
NOSTRA_WORKER_CONTEXT_BLOCK_LIMIT=5
```

Default behavior must remain passive heartbeat with runtime polling disabled. A one-shot context prep run should be easy to disable by unsetting the flag and restarting the worker service.

## Expected Observation Artifact

The output artifact should be local and immutable for the run, for example:

```text
/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-context-bundle-prep-<UTC>.json
```

Required fields:

1. `schemaVersion`
2. `packetId`
3. `observedAt`
4. `agentId`
5. `gatewayBaseUrl`
6. `requestedBlockIds`
7. `blockLimit`
8. `authzDevMode`
9. `allowUnverifiedRoleHeader`
10. `agentIdentityEnforcement`
11. `workerMode`
12. `contextBundle`
13. `checks`
14. `errors`
15. `exitStatus`

The `contextBundle` field should contain counts and redacted item summaries only. It must not contain provider keys, bearer tokens, cookies, SSH details, full environment dumps, unredacted runtime topology, full raw heap payloads, or large `surface_json` bodies.

## Verification Commands

Before implementation PR:

```bash
bash scripts/check_agent_preflight_contract.sh
bash scripts/check_dynamic_config_contract.sh
cargo check --manifest-path nostra/worker/Cargo.toml
```

After implementation PR:

```bash
NOSTRA_WORKER_CONTEXT_BUNDLE_PREP=1 \
NOSTRA_WORKER_CONTEXT_BLOCK_IDS=<heap-block-id> \
NOSTRA_WORKER_CONTEXT_BLOCK_LIMIT=5 \
cargo run --manifest-path nostra/worker/Cargo.toml

bash scripts/check_vps_runtime_authority.sh --repo-contract
```

On VPS after promotion, if separately authorized:

```bash
systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
sudo -u nostra env \
  NOSTRA_WORKER_CONTEXT_BUNDLE_PREP=1 \
  NOSTRA_WORKER_CONTEXT_BLOCK_IDS=<comma-separated heap block ids> \
  NOSTRA_WORKER_CONTEXT_BLOCK_LIMIT=5 \
  /srv/nostra/eudaemon-alpha/repo/nostra/worker/target/release/cortex_worker
```

## Acceptance Criteria

1. Passive default behavior is unchanged.
2. Context bundle prep mode requires explicit operator opt-in.
3. The worker exits after one bounded pass.
4. The worker calls only allowed loopback gateway endpoints.
5. The worker submits only explicit operator-provided block IDs, capped by implementation.
6. The observation artifact is written under local deployment state or logs, not a governed repo path.
7. The artifact summarizes context visibility without exposing secrets or large raw payloads.
8. No heap emit, proposal, workflow, provider, repo, deploy, runtime, or graph mutation occurs.
9. Host-mode authority checks remain green after deployment.
10. Any durable evidence is promoted manually into Initiative 132 only after operator review.

## Follow-Up Gates

Only after this packet is implemented and validated should Initiative 132 consider a later packet for:

1. steward-reviewed heap emission,
2. proposal or workflow-draft projection,
3. provider-backed cognition,
4. live polling, or
5. execution-worker authority.

Each follow-up must define its own allowed reads, writes, identity checks, output artifacts, rollback path, and forbidden actions.
