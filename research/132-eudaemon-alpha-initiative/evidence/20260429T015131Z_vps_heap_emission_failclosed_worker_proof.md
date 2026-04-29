# VPS Steward-Reviewed Heap Emission Fail-Closed Worker Proof

## Summary

- Initiative: `132-eudaemon-alpha-initiative`
- Commit deployed: `8a3b2fe35d818b66c4545a489f1e4bc21b328d66`
- Runtime identity: `agent:eudaemon-alpha-01`
- Host: `eudaemon-alpha-01`
- Observation artifact: `/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-steward-reviewed-heap-emission-2026-04-29T01-51-31-492815786+00-00.json`
- Result: pass for fail-closed behavior; publication was not attempted.

## Command Shape

The operator ran one bounded pass as the `nostra` service user with:

```bash
NOSTRA_WORKER_STEWARD_REVIEWED_HEAP_EMIT=1
NOSTRA_WORKER_HEAP_EMIT_SPACE_ID=initiative-132
NOSTRA_WORKER_HEAP_EMIT_TITLE="Eudaemon Alpha evidence note"
NOSTRA_WORKER_HEAP_EMIT_BODY="Operator-approved dry run publication. This pass must fail closed without verified operator authorization."
NOSTRA_WORKER_HEAP_EMIT_APPROVAL_REF=initiative-132-auth-preflight-dry-run
NOSTRA_WORKER_OBSERVATION_DIR=/srv/nostra/eudaemon-alpha/state/observations
NOSTRA_WORKER_KEYS_PATH=/srv/nostra/eudaemon-alpha/state/worker_keys.json
NOSTRA_AGENT_ID=agent:eudaemon-alpha-01
NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1
/srv/nostra/eudaemon-alpha/repo/nostra/worker/target/release/cortex_worker
```

No operator principal, signed session, proxy authorization, provider key, or bearer token was supplied.

## Observed Artifact Summary

```json
{
  "exitStatus": "needs_review",
  "errors": [
    "heap_emit_authz:verified_operator_or_higher_required"
  ],
  "authz": {
    "authMode": null,
    "effectiveRole": "viewer",
    "identityVerified": false,
    "identitySource": "read_fallback_viewer",
    "principalPresent": false,
    "verifiedOperatorOrHigher": false
  },
  "heapEmit": {
    "endpoint": "/api/cortex/studio/heap/emit",
    "attempted": false,
    "status": null,
    "accepted": null,
    "artifactId": null,
    "blockId": null,
    "opId": null,
    "idempotent": null,
    "sourceOfTruth": null,
    "fallbackActive": null
  },
  "authzDevMode": false,
  "allowUnverifiedRoleHeader": false,
  "agentIdentityEnforcement": true
}
```

## Authority Check

After the pass:

- `cortex-gateway.service`: active
- `cortex-worker.service`: active
- `/srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh`: `status=pass`

The runtime authority manifest matched the deployed repo head and preserved:

- `NOSTRA_AUTHZ_DEV_MODE=0`
- `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`
- `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
- `NOSTRA_AGENT_ID=agent:eudaemon-alpha-01`

## Interpretation

This is not a publication proof. It proves that the steward-reviewed heap emission mode fails closed when the worker has runtime agent identity but lacks verified operator-or-higher publication authority.

The next allowed validation is a separately approved run using a selected operator-authorized path, such as principal binding, signed/session authorization, or an operator-mediated local wrapper. Until that proof exists, Eudaemon Alpha remains unable to emit heap blocks on the VPS.
