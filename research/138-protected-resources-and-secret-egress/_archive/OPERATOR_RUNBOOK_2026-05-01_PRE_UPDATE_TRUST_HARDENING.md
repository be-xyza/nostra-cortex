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

