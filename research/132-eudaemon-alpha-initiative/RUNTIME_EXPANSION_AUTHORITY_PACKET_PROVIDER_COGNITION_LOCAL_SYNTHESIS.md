# Eudaemon Alpha Runtime Expansion Authority Packet: Provider Cognition Local Synthesis

## Status

- Packet id: `initiative-132-runtime-expansion-provider-cognition-local-synthesis-v1`
- Initiative: `132-eudaemon-alpha-initiative`
- Status: proposed
- Created: 2026-04-29
- Updated: 2026-04-29
- Authority mode: recommendation-only until separately approved for implementation
- Runtime identity: `agent:eudaemon-alpha-01`
- Depends on: `initiative-132-runtime-expansion-steward-reviewed-heap-emission-v1`

## Purpose

Define the next safe expansion after the steward-reviewed heap emission proof: a bounded one-shot worker mode that performs one operator-approved provider cognition pass and writes only a local redacted synthesis artifact.

This packet is intentionally a cognition gate, not a publication, workflow, polling, autonomous-planning, or execution gate. It lets Eudaemon Alpha prove it can call the live provider lane through the governed local runtime boundary while preserving the distinction between analysis, publication, proposal/workflow projection, and executable action.

## Preconditions

The following must remain true before implementation or activation:

1. Steward-reviewed heap emission fail-closed proof remains passed on the VPS.
2. Steward-reviewed heap emission authorized publication proof remains passed on the VPS.
3. `cortex-gateway.service` and `cortex-worker.service` are active.
4. Host-mode runtime authority check passes.
5. Production auth posture remains:
   - `NOSTRA_AUTHZ_DEV_MODE=0`
   - `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`
   - `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
   - `NOSTRA_AGENT_ID=agent:eudaemon-alpha-01`
6. The operator provides an explicit provider binding or provider profile identifier accepted by the implementation.
7. The operator provides an explicit model identifier or an explicit runtime-default model decision.
8. The operator provides an explicit prompt or audit unit input and an approval reference.
9. The operator explicitly enables provider cognition local synthesis for a single run.
10. Provider keys, auth bindings, and runtime topology remain operator-only infrastructure and must not be exposed in the output artifact.

## Authorization Prerequisite

Provider cognition may consume provider credentials or runtime bindings indirectly through the local Cortex runtime. The worker must not read, print, persist, or export raw provider keys, bearer tokens, cookies, SSH details, auth-binding secrets, or full runtime-host topology.

Before implementation, the selected provider call path must be documented as one of:

1. a loopback gateway endpoint that performs the provider invocation server-side;
2. a local Cortex runtime client that resolves provider bindings without exposing secrets to the worker artifact; or
3. an operator-mediated wrapper that injects exactly one approved prompt/model/provider configuration for the run.

The selected path must preserve these constraints:

1. provider secrets stay in operator/runtime custody;
2. the worker records only redacted provider metadata such as provider family, provider profile ID, model ID, latency, token/byte counts when available, and normalized error code;
3. if provider configuration is absent, invalid, or ambiguous, the worker must write `NEEDS_REVIEW` or equivalent failure state and skip provider invocation;
4. provider invocation success must not automatically publish to heap, create proposals, create workflow drafts, or start polling.

## Allowed Calls

The worker may call only:

1. `GET /api/system/whoami`
2. one explicitly selected provider-cognition invocation path documented by the implementation PR

Allowed provider-cognition request constraints:

1. The prompt or audit unit input must come from an operator-provided environment value, local operator packet, or explicit local file path.
2. The implementation must cap prompt/input bytes, with a recommended v1 cap of `12000` bytes.
3. The implementation must cap output persistence, with a recommended v1 cap of `8000` characters of model text in the local artifact.
4. The prompt must include an explicit instruction that output is local synthesis only and must not claim publication, execution, proposal, workflow, or steward approval.
5. The request may include only bounded text context. It must not include provider keys, cookies, bearer tokens, SSH details, full environment dumps, unredacted runtime topology, raw heap payloads beyond the operator-selected context, or repo-wide source dumps.
6. The implementation must write one local cognition observation artifact containing a redacted request summary, redacted provider summary, and bounded synthesis output.

## Forbidden Calls

The worker must not call:

1. `POST /api/cortex/studio/heap/emit`
2. heap pin, delete, feedback, steward-gate validate, or steward-gate apply endpoints
3. proposal endpoints
4. workflow draft or workflow instance endpoints
5. provider inventory, runtime-host inventory, auth-binding inventory, execution-binding inventory, or provider discovery diagnostics endpoints unless a separately approved operator-only diagnostics packet allows them
6. local gateway queue mutation endpoints
7. shell/code execution, terminal, ACP, or untrusted executor endpoints
8. batch-provider job submission, polling, cancellation, or queue-runner endpoints

This packet deliberately starts from an operator-approved provider prompt. It does not authorize the worker to discover tasks, choose heap blocks, publish results, or transform synthesis into executable plans.

## Allowed Behavior

The provider cognition local synthesis slice may:

1. Load existing worker configuration and HPKE key material.
2. Confirm gateway production-auth posture through `/api/system/whoami`.
3. Parse explicit operator-provided provider, model, prompt/input, and approval fields.
4. Validate input and output caps before invoking the provider.
5. Invoke exactly one provider cognition request through the selected governed local runtime path.
6. Record redacted provider metadata, timing, status, output length, and normalized error summary when present.
7. Write one local JSON cognition observation artifact under deployment state or logs.
8. Exit after one bounded pass.

## Forbidden Behavior

The provider cognition local synthesis slice must not:

1. Poll continuously.
2. Select tasks, source blocks, prompts, models, or providers autonomously.
3. Emit heap blocks.
4. Create proposals, workflow drafts, workflow instances, or agent contributions.
5. Submit or poll batch-provider jobs.
6. Read provider keys directly or print secrets.
7. Mutate the repo or deployment mirror.
8. Mutate production runtime state.
9. Mutate Nostra graph state.
10. Inspect operator-only topology surfaces by default.
11. Invoke shell/code execution or untrusted executor paths.
12. Persist raw provider request/response envelopes if they contain secrets, hidden chain-of-thought, tool-call payloads, or unbounded model output.
13. Treat success as authorization for publication, proposal/workflow projection, live polling, autonomous task selection, or execution.

## Proposed Activation Contract

Implementation should add an explicit opt-in flag rather than changing the default worker loop.

Suggested local controls:

```bash
NOSTRA_WORKER_PROVIDER_COGNITION_LOCAL_SYNTHESIS=1
NOSTRA_WORKER_PROVIDER_COGNITION_PROVIDER_ID=<operator-approved provider/profile id>
NOSTRA_WORKER_PROVIDER_COGNITION_MODEL=<operator-approved model id>
NOSTRA_WORKER_PROVIDER_COGNITION_PROMPT=<operator-approved prompt>
NOSTRA_WORKER_PROVIDER_COGNITION_INPUT_PATH=<optional local operator-provided input file>
NOSTRA_WORKER_PROVIDER_COGNITION_APPROVAL_REF=<approval id or evidence ref>
NOSTRA_WORKER_PROVIDER_COGNITION_PROMPT_LIMIT_BYTES=12000
NOSTRA_WORKER_PROVIDER_COGNITION_OUTPUT_LIMIT_CHARS=8000
```

Default behavior must remain passive heartbeat with runtime polling disabled. A one-shot provider cognition run should be easy to disable by unsetting the flag and restarting the worker service.

## Expected Cognition Observation Artifact

The output artifact should be local and immutable for the run, for example:

```text
/srv/nostra/eudaemon-alpha/state/observations/eudaemon-alpha-provider-cognition-local-synthesis-<UTC>.json
```

Required fields:

1. `schemaVersion`
2. `packetId`
3. `observedAt`
4. `agentId`
5. `gatewayBaseUrl`
6. `approvalRef`
7. `providerId`
8. `model`
9. `promptLengthBytes`
10. `promptLimitBytes`
11. `outputLengthChars`
12. `outputLimitChars`
13. `authzDevMode`
14. `allowUnverifiedRoleHeader`
15. `agentIdentityEnforcement`
16. `workerMode`
17. `providerCognition`
18. `checks`
19. `errors`
20. `exitStatus`

The `providerCognition` field should contain redacted status and bounded synthesis only. The artifact must not contain provider keys, bearer tokens, cookies, SSH details, full environment dumps, unredacted runtime topology, raw hidden reasoning, unbounded model output, proposal/workflow payloads, or executable commands intended for automatic execution.

## Verification Commands

Before implementation PR:

```bash
bash scripts/check_agent_preflight_contract.sh
bash scripts/check_dynamic_config_contract.sh
cargo check --manifest-path nostra/worker/Cargo.toml
```

After implementation PR:

```bash
NOSTRA_WORKER_PROVIDER_COGNITION_LOCAL_SYNTHESIS=1 \
NOSTRA_WORKER_PROVIDER_COGNITION_PROVIDER_ID=<operator-approved provider/profile id> \
NOSTRA_WORKER_PROVIDER_COGNITION_MODEL=<operator-approved model id> \
NOSTRA_WORKER_PROVIDER_COGNITION_PROMPT="Produce one local-only Initiative 132 synthesis note. Do not publish, propose, plan execution, or claim approval." \
NOSTRA_WORKER_PROVIDER_COGNITION_APPROVAL_REF=<approval ref> \
cargo run --manifest-path nostra/worker/Cargo.toml

bash scripts/check_vps_runtime_authority.sh --repo-contract
```

On VPS after promotion, if separately authorized:

```bash
systemctl is-active cortex-gateway.service cortex-worker.service
bash /srv/nostra/eudaemon-alpha/repo/scripts/check_vps_runtime_authority.sh
sudo -u nostra env \
  NOSTRA_WORKER_PROVIDER_COGNITION_LOCAL_SYNTHESIS=1 \
  NOSTRA_WORKER_PROVIDER_COGNITION_PROVIDER_ID=<operator-approved provider/profile id> \
  NOSTRA_WORKER_PROVIDER_COGNITION_MODEL=<operator-approved model id> \
  NOSTRA_WORKER_PROVIDER_COGNITION_PROMPT=<operator-approved prompt> \
  NOSTRA_WORKER_PROVIDER_COGNITION_APPROVAL_REF=<approval ref> \
  /srv/nostra/eudaemon-alpha/repo/nostra/worker/target/release/cortex_worker
```

## Acceptance Criteria

1. Passive default behavior is unchanged.
2. Provider cognition local synthesis mode requires explicit operator opt-in.
3. The worker exits after one bounded pass.
4. The worker calls only `/api/system/whoami` plus the selected provider-cognition invocation path.
5. Provider/model/prompt/input values come only from explicit operator controls.
6. Missing or ambiguous provider configuration fails closed before invocation.
7. The worker submits exactly one provider cognition request after validation passes.
8. The worker records local redacted cognition evidence without exposing secrets, raw provider envelopes, or unbounded model output.
9. No heap emit, proposal, workflow, provider inventory, runtime topology, repo, deploy, broad runtime, graph, polling, queue, batch-provider, or execution behavior occurs.
10. Host-mode authority checks remain green after deployment.
11. Any durable proof is promoted manually into Initiative 132 only after operator review.

## Follow-Up Gates

Only after this packet is implemented and validated should Initiative 132 consider a later packet for:

1. steward-reviewed heap publication from provider synthesis,
2. proposal or workflow-draft projection,
3. provider-backed cognition over operator-selected heap context,
4. live polling, or
5. execution-worker authority.

Each follow-up must define its own allowed reads, writes, identity checks, output artifacts, rollback path, and forbidden actions.
