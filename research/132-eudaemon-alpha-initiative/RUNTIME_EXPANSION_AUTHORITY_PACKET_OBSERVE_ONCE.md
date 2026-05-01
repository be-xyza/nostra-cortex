# Eudaemon Alpha Runtime Expansion Authority Packet: Observe Once

## Status

- Packet id: `initiative-132-runtime-expansion-observe-once-v1`
- Initiative: `132-eudaemon-alpha-initiative`
- Status: proposed
- Created: 2026-04-28
- Authority mode: recommendation-only until separately approved for implementation
- Runtime identity: `agent:eudaemon-alpha-01`

## Purpose

Define the first safe expansion beyond passive `cortex_worker` preflight: a bounded observational one-shot mode that proves the worker can authenticate to the local Cortex runtime surface, collect a narrow runtime observation, and write a local evidence artifact without polling, executing tasks, mutating repos, mutating runtime state, or publishing graph changes.

This packet is the bridge between the validated passive VPS runtime and any future live worker loop. It does not itself authorize code changes, deploys, service activation, or recurring execution.

## Preconditions

The following must remain true before implementation or activation:

1. `cortex-gateway.service` is active on the VPS and bound to loopback.
2. `cortex-worker.service` is active or can run one-shot under `systemd`.
3. Host-mode runtime authority check passes.
4. Production auth posture remains:
   - `NOSTRA_AUTHZ_DEV_MODE=0`
   - `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`
   - `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
   - `NOSTRA_AGENT_ID=agent:eudaemon-alpha-01`
5. Worker key material lives outside the Git mirror under the deployment state root.
6. The operator explicitly enables the one-shot mode for a single run.

## Allowed Behavior

The observe-once worker slice may:

1. Load existing worker configuration and HPKE key material.
2. Read only explicitly configured loopback gateway endpoints required for runtime self-observation.
3. Confirm the gateway reports production-auth posture consistent with the host authority check.
4. Confirm the worker identity value it will use for later agent-bound calls.
5. Emit one local JSON observation artifact under the deployment state or logs root.
6. Exit after one bounded pass.

The first implementation should prefer read-only system or health endpoints. It must not require provider inventory, runtime-host inventory, auth-binding inventory, execution-binding state, or detailed discovery diagnostics unless a later packet explicitly grants an operator-only redacted observation contract.

## Forbidden Behavior

The observe-once worker slice must not:

1. Poll continuously.
2. Select work autonomously.
3. Submit provider jobs.
4. Call external model/provider APIs.
5. Read provider keys or print secrets.
6. Mutate the repo or deployment mirror.
7. Commit, push, deploy, or promote code.
8. Mutate production runtime state.
9. Mutate Nostra graph state.
10. Emit heap blocks, proposals, workflow drafts, or agent contributions.
11. Invoke shell/code execution or untrusted executor paths.
12. Inspect operator-only topology surfaces by default.
13. Treat success as authorization for live polling or execution.

## Proposed Activation Contract

Implementation should add an explicit opt-in flag rather than changing the default worker loop.

Suggested local control:

```bash
NOSTRA_WORKER_OBSERVE_ONCE=1
```

Default behavior must remain passive heartbeat with runtime polling disabled. A one-shot observe run should be easy to disable by unsetting the flag and restarting the worker service.

## Expected Observation Artifact

The output artifact should be local and immutable for the run, for example:

```text
/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-observe-once-<UTC>.json
```

Required fields:

1. `schemaVersion`
2. `packetId`
3. `observedAt`
4. `agentId`
5. `gatewayBaseUrl`
6. `authzDevMode`
7. `allowUnverifiedRoleHeader`
8. `agentIdentityEnforcement`
9. `workerMode`
10. `checks`
11. `errors`
12. `exitStatus`

The artifact must not contain provider keys, bearer tokens, cookies, SSH details, full environment dumps, or unredacted runtime topology.

## Verification Commands

Before implementation PR:

```bash
bash scripts/check_agent_preflight_contract.sh
bash scripts/check_dynamic_config_contract.sh
cargo check --manifest-path nostra/worker/Cargo.toml
```

After implementation PR:

```bash
NOSTRA_WORKER_RUN_ONCE=1 cargo run --manifest-path nostra/worker/Cargo.toml
NOSTRA_WORKER_OBSERVE_ONCE=1 cargo run --manifest-path nostra/worker/Cargo.toml
bash scripts/check_vps_runtime_authority.sh --repo-contract
```

On VPS after promotion, if separately authorized:

```bash
systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
sudo -u nostra env NOSTRA_WORKER_OBSERVE_ONCE=1 /srv/nostra/eudaemon-alpha/repo/nostra/worker/target/release/cortex_worker
```

## Acceptance Criteria

1. Passive default behavior is unchanged.
2. Observe-once mode requires explicit operator opt-in.
3. The worker exits after one bounded pass.
4. The observation artifact is written under local deployment state or logs, not a governed repo path.
5. The artifact proves production-auth posture without exposing secrets.
6. No heap, proposal, workflow, provider, repo, deploy, or graph mutation occurs.
7. Host-mode authority checks remain green after deployment.
8. Any durable evidence is promoted manually into Initiative 132 only after operator review.

## Follow-Up Gates

Only after this packet is implemented and validated should Initiative 132 consider a later packet for:

1. bounded heap read visibility,
2. steward-reviewed heap emission,
3. proposal or workflow-draft projection,
4. provider-backed cognition,
5. live polling, or
6. execution-worker authority.

Each follow-up must define its own allowed reads, writes, identity checks, output artifacts, rollback path, and forbidden actions.
