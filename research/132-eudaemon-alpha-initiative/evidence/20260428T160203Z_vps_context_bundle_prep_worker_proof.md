# Eudaemon Alpha VPS Context Bundle Prep Worker Proof

## Summary

On 2026-04-28, Eudaemon Alpha was promoted to the merged context bundle prep worker commit and validated on the Hetzner VPS with one bounded context-preparation pass.

This evidence proves the worker can run under the production runtime identity, confirm gateway auth posture, submit an explicit operator-provided heap block ID list to the loopback context endpoint, write a summarized local observation artifact, and exit without enabling polling, autonomous selection, publication, provider calls, or mutation.

This proof used a harmless sentinel block ID and therefore validates endpoint authority and artifact shape, not real heap-context usefulness.

## Promoted Commit

- Repo commit: `6cbf62c6e5d4977e7e1eb41b953aa35d22e25de4`
- PR: `#83` (`Add Eudaemon context bundle prep mode`)
- Authority packet: `initiative-132-runtime-expansion-context-bundle-prep-v1`

## VPS State Validated

- Host: `eudaemon-alpha-01`
- Runtime surface: loopback-local `cortex-gateway` plus `cortex_worker`
- `cortex-gateway.service`: active
- `cortex-worker.service`: active
- Runtime authority check: pass
- Manifest commit matched host repo `HEAD`
- `cortex-web` remained explicitly `not_deployed`

## Context Bundle Prep Run

The one-shot worker was run as the `nostra` service user with:

```bash
NOSTRA_WORKER_CONTEXT_BUNDLE_PREP=1
NOSTRA_WORKER_CONTEXT_BLOCK_IDS=eudaemon-context-prep-smoke-nonexistent
NOSTRA_WORKER_CONTEXT_BLOCK_LIMIT=5
NOSTRA_WORKER_OBSERVATION_DIR=/srv/nostra/eudaemon-alpha/state/observations
NOSTRA_WORKER_KEYS_PATH=/srv/nostra/eudaemon-alpha/state/worker_keys.json
NOSTRA_AGENT_ID=agent:eudaemon-alpha-01
NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1
```

Observation artifact:

```text
/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-context-bundle-prep-2026-04-28T16-02-03-572688415+00-00.json
```

## Artifact Result

The observation artifact reported:

- `schemaVersion`: `1.0.0`
- `packetId`: `initiative-132-runtime-expansion-context-bundle-prep-v1`
- `agentId`: `agent:eudaemon-alpha-01`
- `gatewayBaseUrl`: `http://127.0.0.1:3000`
- `requestedBlockIds`: `["eudaemon-context-prep-smoke-nonexistent"]`
- `blockLimit`: `5`
- `authzDevMode`: `false`
- `allowUnverifiedRoleHeader`: `false`
- `agentIdentityEnforcement`: `true`
- `workerMode`: `context_bundle_prep`
- `contextBundle.endpoint`: `/api/cortex/studio/heap/blocks/context`
- `contextBundle.blockCount`: `0`
- `contextBundle.requestedCount`: `1`
- `contextBundle.returnedCount`: `0`
- `contextBundle.totalSurfaceJsonBytes`: `0`
- `exitStatus`: `pass`
- `errors`: empty

Checks recorded:

```text
packet:initiative-132-runtime-expansion-context-bundle-prep-v1
agent_id:agent:eudaemon-alpha-01
mode:context_bundle_prep
context_block_limit:5
context_requested_count:1
gateway_whoami:ok
authz_dev_mode:false
allow_unverified_role_header:false
agent_identity_enforcement:true
heap_blocks_context:ok
```

## Boundary Confirmed

This proof validates only:

- explicit opt-in context bundle prep mode,
- loopback self-observation through `/api/system/whoami`,
- loopback context packaging through `/api/cortex/studio/heap/blocks/context`,
- operator-provided block IDs capped by implementation,
- local summarized observation artifact emission,
- production-auth posture visibility, and
- one bounded pass with process exit.

Still not authorized:

- live polling,
- autonomous block or task selection,
- provider execution,
- heap emission,
- proposal or workflow-draft projection,
- repo mutation,
- production runtime mutation,
- Nostra graph mutation,
- operator topology inspection by default,
- raw large heap payload persistence, or
- untrusted shell/code execution.

## Verification Commands

Promotion and authority check:

```bash
NOSTRA_EUDAEMON_VPS_HOST=root@100.86.222.99 \
  bash scripts/promote_eudaemon_alpha_vps.sh 6cbf62c6e5d4977e7e1eb41b953aa35d22e25de4

systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
```

Context bundle prep command:

```bash
sudo -u nostra env \
  NOSTRA_WORKER_CONTEXT_BUNDLE_PREP=1 \
  NOSTRA_WORKER_CONTEXT_BLOCK_IDS=eudaemon-context-prep-smoke-nonexistent \
  NOSTRA_WORKER_CONTEXT_BLOCK_LIMIT=5 \
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
4. bounded observe-once worker validation,
5. bounded read-only heap delta validation, and
6. bounded context bundle prep validation with a sentinel operator-provided ID.

The next expansion decision should remain non-executing unless the operator deliberately moves into a separate publication or cognition gate. The next safest runtime proof is a context bundle prep pass with real operator-selected heap block IDs, still local-artifact-only.
