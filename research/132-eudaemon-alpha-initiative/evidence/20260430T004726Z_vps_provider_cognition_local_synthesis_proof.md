# VPS Provider Cognition Local Synthesis Proof

## Summary

On 2026-04-30 UTC, Eudaemon Alpha passed the provider cognition local synthesis gate on the VPS.

The operator promoted merge commit `ce91705c14ff7d61779794ca49ac461093a8ec2b` to `/srv/nostra/eudaemon-alpha/repo`, rebuilt `cortex-gateway` and `cortex_worker`, restarted both systemd services, and revalidated host runtime authority with `scripts/check_vps_runtime_authority.sh`.

The worker then ran one explicit provider-cognition pass through a transient loopback wrapper:

- `packet`: `initiative-132-runtime-expansion-provider-cognition-local-synthesis-v1`
- `agent_id`: `agent:eudaemon-alpha-01`
- `approval_ref`: `operator-session-2026-04-30-provider-cognition-local-synthesis`
- `provider_id`: `openrouter`
- `model`: `openai/gpt-5.4`
- `endpoint`: `http://127.0.0.1:39132/provider-cognition-local-synthesis`
- `prompt_length_bytes`: `363`
- `output_length_chars`: `2185`
- `exit_status`: `pass`

## Evidence

Immutable artifact:

- [20260430T004726Z_vps_provider_cognition_local_synthesis_artifact.json](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/evidence/20260430T004726Z_vps_provider_cognition_local_synthesis_artifact.json)

SHA-256:

```text
e14034e70748778775b0e4b03c107a13bf719603db38b0c5ea483a81aefa4d6b
```

The artifact records:

- `providerCognition.attempted: true`
- `providerCognition.status: 200`
- `errors: []`
- `checks` includes `gateway_whoami:ok`, `authz_dev_mode:false`, `allow_unverified_role_header:false`, and `provider_cognition:ok`

## Boundary Result

This proves one operator-approved provider cognition call through a loopback-local wrapper, with one local redacted artifact and no heap publication, proposal/workflow projection, polling loop, graph mutation, repo mutation, runtime execution, or autonomous task selection.

It does not authorize autonomous provider calls, retries, polling, publication, proposal/workflow projection, execution workers, provider job submission, or treating provider output as governance authority.

## Follow-Up

The next gate should decide whether to harden the transient wrapper into a governed local provider-cognition adapter or keep provider cognition as an operator-mediated validation lane only.
