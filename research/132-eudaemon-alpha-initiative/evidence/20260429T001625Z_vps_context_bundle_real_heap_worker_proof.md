# Eudaemon Alpha VPS Context Bundle Real Heap Worker Proof

## Summary

On 2026-04-29 UTC, Eudaemon Alpha was validated on the Hetzner VPS with one bounded context-preparation pass over real heap blocks.

This evidence proves the worker can run under the production runtime identity, confirm gateway auth posture, submit an explicit bounded heap block ID list to the loopback context endpoint, receive matched context summaries, write a summarized local observation artifact, and exit without enabling polling, autonomous publication, provider calls, repo/runtime mutation, or graph mutation.

The block IDs used here came from a bounded read-only heap list inspection immediately before the pass. They were then supplied explicitly to the worker as `NOSTRA_WORKER_CONTEXT_BLOCK_IDS`.

## Promoted Commit

- Repo commit: `6cbf62c6e5d4977e7e1eb41b953aa35d22e25de4`
- PR: `#83` (`Add Eudaemon context bundle prep mode`)
- Authority packet: `initiative-132-runtime-expansion-context-bundle-prep-v1`

## Source Block Selection

A bounded read-only heap list query returned 10 available heap blocks. The first three returned IDs were used for this context-prep validation:

| Artifact ID | Title | Block Type | Updated At |
| --- | --- | --- | --- |
| `01KMAQFV3R9T5S8YEVCWRT0B8C` | `usage_report block` | `usage_report` | `2026-03-22T12:14:38.052222594+00:00` |
| `01KMAQFH6P8SWRA1GS5TF712YG` | `self_optimization_proposal block` | `self_optimization_proposal` | `2026-03-22T12:14:28.033512943+00:00` |
| `01KMAQF74S9TG99TWFY1T08QPP` | `agent_execution_record block` | `agent_execution_record` | `2026-03-22T12:14:17.849158798+00:00` |

This selection was used only for validation. It does not authorize the worker to choose future block IDs autonomously.

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
NOSTRA_WORKER_CONTEXT_BLOCK_IDS=01KMAQFV3R9T5S8YEVCWRT0B8C,01KMAQFH6P8SWRA1GS5TF712YG,01KMAQF74S9TG99TWFY1T08QPP
NOSTRA_WORKER_CONTEXT_BLOCK_LIMIT=5
NOSTRA_WORKER_OBSERVATION_DIR=/srv/nostra/eudaemon-alpha/state/observations
NOSTRA_WORKER_KEYS_PATH=/srv/nostra/eudaemon-alpha/state/worker_keys.json
NOSTRA_AGENT_ID=agent:eudaemon-alpha-01
NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1
```

Observation artifact:

```text
/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-context-bundle-prep-2026-04-29T00-16-25-230752195+00-00.json
```

## Artifact Result

The observation artifact reported:

- `schemaVersion`: `1.0.0`
- `packetId`: `initiative-132-runtime-expansion-context-bundle-prep-v1`
- `agentId`: `agent:eudaemon-alpha-01`
- `gatewayBaseUrl`: `http://127.0.0.1:3000`
- `requestedBlockIds`: three explicit heap block IDs
- `blockLimit`: `5`
- `authzDevMode`: `false`
- `allowUnverifiedRoleHeader`: `false`
- `agentIdentityEnforcement`: `true`
- `workerMode`: `context_bundle_prep`
- `contextBundle.endpoint`: `/api/cortex/studio/heap/blocks/context`
- `contextBundle.blockCount`: `3`
- `contextBundle.requestedCount`: `3`
- `contextBundle.returnedCount`: `3`
- `contextBundle.totalSurfaceJsonBytes`: `2576`
- `exitStatus`: `pass`
- `errors`: empty

Returned block summaries:

| Artifact ID | Title | Block Type | Surface JSON Bytes |
| --- | --- | --- | --- |
| `01KMAQFV3R9T5S8YEVCWRT0B8C` | `usage_report block` | `usage_report` | `446` |
| `01KMAQFH6P8SWRA1GS5TF712YG` | `self_optimization_proposal block` | `self_optimization_proposal` | `747` |
| `01KMAQF74S9TG99TWFY1T08QPP` | `agent_execution_record block` | `agent_execution_record` | `1383` |

The artifact recorded payload sizes only. It did not persist raw `surface_json` bodies.

## Boundary Confirmed

This proof validates only:

- explicit opt-in context bundle prep mode,
- loopback self-observation through `/api/system/whoami`,
- loopback context packaging through `/api/cortex/studio/heap/blocks/context`,
- explicit bounded block IDs supplied to the worker,
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

Real-block context bundle prep command:

```bash
sudo -u nostra env \
  NOSTRA_WORKER_CONTEXT_BUNDLE_PREP=1 \
  NOSTRA_WORKER_CONTEXT_BLOCK_IDS=01KMAQFV3R9T5S8YEVCWRT0B8C,01KMAQFH6P8SWRA1GS5TF712YG,01KMAQF74S9TG99TWFY1T08QPP \
  NOSTRA_WORKER_CONTEXT_BLOCK_LIMIT=5 \
  NOSTRA_WORKER_OBSERVATION_DIR=/srv/nostra/eudaemon-alpha/state/observations \
  NOSTRA_WORKER_KEYS_PATH=/srv/nostra/eudaemon-alpha/state/worker_keys.json \
  NOSTRA_AGENT_ID=agent:eudaemon-alpha-01 \
  NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1 \
  /srv/nostra/eudaemon-alpha/repo/nostra/worker/target/release/cortex_worker
```

Post-run authority check:

```bash
systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
```

## Current Boundary

Eudaemon Alpha has now passed:

1. passive worker build/preflight,
2. host-mode VPS authority validation,
3. production-auth posture validation,
4. bounded observe-once worker validation,
5. bounded read-only heap delta validation,
6. sentinel context bundle prep validation, and
7. real-heap context bundle prep validation over three explicit block IDs.

The next expansion decision should move only by a separate governed packet. The nearest candidates are steward-reviewed local evidence projection, heap emission under approval, or provider-backed cognition with no publication authority.
