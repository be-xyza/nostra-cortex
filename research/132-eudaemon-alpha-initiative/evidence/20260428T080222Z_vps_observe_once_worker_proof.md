# Eudaemon Alpha VPS Observe-Once Worker Proof

## Summary

On 2026-04-28, Eudaemon Alpha was promoted to the merged observe-once worker commit and validated on the Hetzner VPS with one bounded observational worker pass.

This evidence proves the worker can run under the production runtime identity, read the loopback gateway self-observation endpoint, write a local observation artifact, and exit without enabling live polling or mutation.

## Promoted Commit

- Repo commit: `a8af1afe312b521ab3d448b15716d9d6fd219312`
- PR: `#77` (`Add Eudaemon observe-once worker mode`)
- Authority packet: `initiative-132-runtime-expansion-observe-once-v1`

## VPS State Validated

- Host: `eudaemon-alpha-01`
- Runtime surface: loopback-local `cortex-gateway` plus `cortex_worker`
- `cortex-gateway.service`: active
- `cortex-worker.service`: active
- Runtime authority check: pass
- Manifest commit matched host repo `HEAD`
- `cortex-web` remained explicitly `not_deployed`

## Observe-Once Run

The one-shot worker was run as the `nostra` service user with:

```bash
NOSTRA_WORKER_OBSERVE_ONCE=1
NOSTRA_WORKER_OBSERVATION_DIR=/srv/nostra/eudaemon-alpha/state/observations
NOSTRA_WORKER_KEYS_PATH=/srv/nostra/eudaemon-alpha/state/worker_keys.json
NOSTRA_AGENT_ID=agent:eudaemon-alpha-01
NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1
```

Observation artifact:

```text
/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-observe-once-2026-04-28T08-02-22-468070976+00-00.json
```

## Artifact Result

The observation artifact reported:

- `schemaVersion`: `1.0.0`
- `packetId`: `initiative-132-runtime-expansion-observe-once-v1`
- `agentId`: `agent:eudaemon-alpha-01`
- `gatewayBaseUrl`: `http://127.0.0.1:3000`
- `authzDevMode`: `false`
- `allowUnverifiedRoleHeader`: `false`
- `agentIdentityEnforcement`: `true`
- `workerMode`: `observe_once`
- `exitStatus`: `pass`
- `errors`: empty

Checks recorded:

```text
packet:initiative-132-runtime-expansion-observe-once-v1
agent_id:agent:eudaemon-alpha-01
mode:observe_once
gateway_whoami:ok
authz_dev_mode:false
allow_unverified_role_header:false
agent_identity_enforcement:true
```

## Boundary Confirmed

This proof validates only:

- explicit opt-in observe-once mode,
- loopback self-observation through `/api/system/whoami`,
- local observation artifact emission,
- production-auth posture visibility, and
- one bounded pass with process exit.

Still not authorized:

- live polling,
- autonomous task selection,
- provider execution,
- heap/proposal/workflow emission,
- repo mutation,
- production runtime mutation,
- Nostra graph mutation,
- operator topology inspection by default, or
- untrusted shell/code execution.

## Verification Commands

Promotion and authority check:

```bash
NOSTRA_EUDAEMON_VPS_HOST=root@100.86.222.99 \
  bash scripts/promote_eudaemon_alpha_vps.sh a8af1afe312b521ab3d448b15716d9d6fd219312

systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
```

Observe-once command:

```bash
install -d -o nostra -g nostra -m 750 /srv/nostra/eudaemon-alpha/state/observations
sudo -u nostra env \
  NOSTRA_WORKER_OBSERVE_ONCE=1 \
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
3. production-auth posture validation, and
4. bounded observe-once worker validation.

The next expansion decision should not jump directly to broad execution. The next safest packet is bounded read-only heap/context visibility or steward-reviewed local evidence projection, with explicit allowed endpoints and forbidden actions.
