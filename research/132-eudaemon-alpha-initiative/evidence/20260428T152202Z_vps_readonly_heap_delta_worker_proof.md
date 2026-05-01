# Eudaemon Alpha VPS Read-Only Heap Delta Worker Proof

## Summary

On 2026-04-28, Eudaemon Alpha was promoted to the merged read-only heap delta worker commit and validated on the Hetzner VPS with one bounded heap visibility pass.

This evidence proves the worker can run under the production runtime identity, confirm gateway auth posture, read the loopback heap changed-blocks endpoint, write a summarized local observation artifact, and exit without enabling polling or mutation.

## Promoted Commit

- Repo commit: `62ecf2d3d6c9112c4021b33b46344d41a8d3e387`
- PR: `#80` (`Add Eudaemon read-only heap delta mode`)
- Authority packet: `initiative-132-runtime-expansion-readonly-heap-delta-v1`

## VPS State Validated

- Host: `eudaemon-alpha-01`
- Runtime surface: loopback-local `cortex-gateway` plus `cortex_worker`
- `cortex-gateway.service`: active
- `cortex-worker.service`: active
- Runtime authority check: pass
- Manifest commit matched host repo `HEAD`
- `cortex-web` remained explicitly `not_deployed`

## Read-Only Heap Delta Run

The one-shot worker was run as the `nostra` service user with:

```bash
NOSTRA_WORKER_READONLY_HEAP_DELTA=1
NOSTRA_WORKER_HEAP_CHANGED_SINCE=2026-04-28T00:00:00Z
NOSTRA_WORKER_HEAP_LIMIT=25
NOSTRA_WORKER_OBSERVATION_DIR=/srv/nostra/eudaemon-alpha/state/observations
NOSTRA_WORKER_KEYS_PATH=/srv/nostra/eudaemon-alpha/state/worker_keys.json
NOSTRA_AGENT_ID=agent:eudaemon-alpha-01
NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1
```

Observation artifact:

```text
/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-readonly-heap-delta-2026-04-28T15-22-02-412628540+00-00.json
```

## Artifact Result

The observation artifact reported:

- `schemaVersion`: `1.0.0`
- `packetId`: `initiative-132-runtime-expansion-readonly-heap-delta-v1`
- `agentId`: `agent:eudaemon-alpha-01`
- `gatewayBaseUrl`: `http://127.0.0.1:3000`
- `changedSince`: `2026-04-28T00:00:00Z`
- `limit`: `25`
- `authzDevMode`: `false`
- `allowUnverifiedRoleHeader`: `false`
- `agentIdentityEnforcement`: `true`
- `workerMode`: `readonly_heap_delta`
- `heapRead.endpoint`: `/api/cortex/studio/heap/changed_blocks`
- `heapRead.count`: `0`
- `heapRead.changedCount`: `0`
- `heapRead.deletedCount`: `0`
- `heapRead.hasMore`: `false`
- `heapRead.nextCursorPresent`: `false`
- `exitStatus`: `pass`
- `errors`: empty

Checks recorded:

```text
packet:initiative-132-runtime-expansion-readonly-heap-delta-v1
agent_id:agent:eudaemon-alpha-01
mode:readonly_heap_delta
heap_limit:25
gateway_whoami:ok
authz_dev_mode:false
allow_unverified_role_header:false
agent_identity_enforcement:true
heap_changed_blocks:ok
```

## Boundary Confirmed

This proof validates only:

- explicit opt-in read-only heap delta mode,
- loopback self-observation through `/api/system/whoami`,
- loopback read through `/api/cortex/studio/heap/changed_blocks`,
- local summarized observation artifact emission,
- production-auth posture visibility, and
- one bounded pass with process exit.

Still not authorized:

- live polling,
- autonomous task selection,
- provider execution,
- heap emission,
- context bundle POSTs,
- proposal or workflow-draft projection,
- repo mutation,
- production runtime mutation,
- Nostra graph mutation,
- operator topology inspection by default, or
- untrusted shell/code execution.

## Verification Commands

Promotion and authority check:

```bash
NOSTRA_EUDAEMON_VPS_HOST=root@100.86.222.99 \
  bash scripts/promote_eudaemon_alpha_vps.sh 62ecf2d3d6c9112c4021b33b46344d41a8d3e387

systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
```

Read-only heap delta command:

```bash
install -d -o nostra -g nostra -m 750 /srv/nostra/eudaemon-alpha/state/observations
sudo -u nostra env \
  NOSTRA_WORKER_READONLY_HEAP_DELTA=1 \
  NOSTRA_WORKER_HEAP_CHANGED_SINCE=2026-04-28T00:00:00Z \
  NOSTRA_WORKER_HEAP_LIMIT=25 \
  NOSTRA_WORKER_OBSERVATION_DIR=/srv/nostra/eudaemon-alpha/state/observations \
  NOSTRA_WORKER_KEYS_PATH=/srv/nostra/eudaemon-alpha/state/worker_keys.json \
  NOSTRA_AGENT_ID=agent:eudaemon-alpha-01 \
  NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1 \
  /srv/nostra/eudaemon-alpha/repo/nostra/worker/target/release/cortex_worker
```

## Current Boundary

Eudaemon Alpha has now passed:

1. passive worker build/preflight,
2. host-mode VPS authority validation,
3. production-auth posture validation,
4. bounded observe-once worker validation, and
5. bounded read-only heap delta validation.

The next expansion decision should remain non-executing. The next safest packet is bounded context bundle preparation or steward-reviewed local evidence projection, with explicit allowed endpoints and no autonomous publication.
