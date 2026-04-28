# Eudaemon Alpha VPS Passive Runtime and Production Auth Proof

## Summary

On 2026-04-28, Eudaemon Alpha was promoted to the merged production-auth authority-check commit and validated on the Hetzner VPS in passive runtime mode.

This evidence closes the previous host-mode authority and production-auth proof blocker for the passive runtime stage. It does not authorize live worker polling, autonomous execution, provider job submission, repo mutation, production graph mutation, or untrusted code execution.

## Promoted Commit

- Repo commit: `2cfbf65dbe2093666de443366d33626b1c325090`
- PR: `#74` (`Enforce Eudaemon production auth posture checks`)
- Prior deploy hardening: `#73` (`Harden Eudaemon Alpha deploy authority state`)

## VPS State Validated

- Host: `eudaemon-alpha-01`
- Runtime surface: loopback-local `cortex-gateway` plus `cortex_worker`
- `cortex-gateway.service`: active
- `cortex-worker.service`: active
- Runtime authority check: pass
- Manifest commit matched host repo `HEAD`
- `cortex-web` remained explicitly `not_deployed`
- Worker remained in passive preflight mode; runtime polling stayed disabled.

## Production Auth Posture Validated

The live VPS env was corrected and verified with:

- `NOSTRA_AUTHZ_DEV_MODE=0`
- `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`
- `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
- `NOSTRA_AGENT_ID=agent:eudaemon-alpha-01`

The host authority check now includes these posture checks, so future host-mode authority passes fail if the deployment drifts back to dev auth.

## Runtime Behavior Proof

Live gateway checks after promotion showed:

- `GET /api/system/whoami` returned a viewer fallback with `authzDevMode:false` and `allowUnverifiedRoleHeader:false`.
- An unverified `x-cortex-role: operator` request to provider inventory returned `403`.
- A bound operator principal request to provider inventory returned `200`.
- An unknown `x-cortex-agent-id` request to the agent-contribution endpoint returned `403`.

These prove the passive VPS runtime no longer relies on unverified operator headers and rejects unknown agent identities under enforcement.

## Durable Guardrails Added

- `ops/hetzner/deploy.sh` re-owns the deployment state tree for the service user during promotion so gateway session state and worker keys remain writable outside the Git mirror.
- `scripts/check_vps_runtime_authority.sh` validates the production auth posture in host mode without printing provider secrets.
- `scripts/test_vps_runtime_authority_host_mode.sh` includes fixture coverage for production auth drift.

## Current Boundary

Eudaemon Alpha is validated for:

- passive worker preflight
- host-mode service provenance
- production-auth posture
- operator-local promotion
- loopback-local gateway access

Still blocked until separately governed:

- live worker polling
- autonomous contribution execution
- provider job execution
- repo/runtime mutation by the worker
- untrusted execution or generated shell/code execution
- production graph mutation beyond steward-reviewed publication paths

## Verification Commands

On VPS:

```bash
systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
curl -sS http://127.0.0.1:3000/api/system/whoami
```

Operator-local / PR checks:

```bash
bash scripts/check_agent_preflight_contract.sh
bash scripts/check_dynamic_config_contract.sh
bash scripts/check_vps_runtime_authority.sh --repo-contract
bash scripts/test_vps_runtime_authority_host_mode.sh
```
