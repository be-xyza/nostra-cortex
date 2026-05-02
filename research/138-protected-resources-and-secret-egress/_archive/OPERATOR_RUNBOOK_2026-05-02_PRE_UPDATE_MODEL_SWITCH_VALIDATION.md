# Operator Runbook: Secret Exposure Response

Use this runbook when a live secret appears in terminal output, AI-visible context, CI logs, screenshots, generated artifacts, or runtime diagnostics.

## Immediate Response

1. Stop sharing or promoting the exposed output.
2. Rotate or revoke the exposed key in the upstream provider console.
3. Restart affected services with the replacement secret.
4. Verify service health through redacted inspection only.
5. Review recent usage and billing for unexpected activity.

## Scrub and Preserve

1. Remove local generated artifacts that contain raw secrets unless they are required for incident evidence.
2. If evidence must be preserved, store only redacted copies in governed repo paths.
3. Do not paste raw secrets into issues, PRs, chat, or model prompts.
4. Record the incident decision and mitigation summary in this initiative or the owning runtime initiative.

## Safe Verification

Use `scripts/inspect_runtime_config_redacted.sh` for runtime secret checks. The expected output is metadata only: name, presence, source class, value length, fingerprint, and policy.

## User Trust Closeout

After rotation or any protected-resource use, record a short redacted closeout that answers:

1. What protected resource class was involved?
2. Was the raw value ever emitted to terminal, transcript, logs, screenshot, CI, or governed artifacts?
3. Which trusted boundary resolved the value?
4. Which service or workflow used it?
5. What audit reference or evidence path records the action?
6. What did the user see: status, redacted preview, fingerprint, or failure?

Do not include host credentials, tunnel credentials, provider keys, bearer values, private keys, auth headers, PII values, or raw upstream provider payloads in the closeout.

## Production Drill

Before marking the protected-resource path stable-production ready, perform a dry-run incident drill:

1. Rotate a non-production provider key.
2. Restart services with the replacement value.
3. Verify health through redacted inspection only.
4. Run the repo and governed-artifact secret scanner.
5. Confirm logs and error envelopes redact the seeded fake secret.
6. Preserve only redacted drill evidence in governed initiative surfaces.
