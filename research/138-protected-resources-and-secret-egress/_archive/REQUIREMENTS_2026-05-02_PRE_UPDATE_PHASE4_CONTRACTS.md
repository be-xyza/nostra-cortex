---
id: "138-requirements"
initiative: "138"
type: "requirements"
status: draft
created: "2026-05-01"
updated: "2026-05-01"
---

# Requirements: Protected Resources and Secret Egress

## Functional Requirements

1. Operators must have a safe way to inspect secret-bearing runtime configuration without printing raw values.
2. Secret-bearing output intended for AI agents, PR evidence, CI logs, or tickets must use redacted fields or fingerprints.
3. Repo and governed artifact scans must detect high-confidence credential leaks.
4. Lower-confidence PII patterns must be visible to operators without blocking early adoption.
5. Future protected-resource APIs must expose references and capabilities rather than raw secret values.
6. User-facing trust surfaces must show protected-resource use as status, purpose, grant, and audit metadata rather than raw values.
7. Runtime diagnostics must distinguish secret-bearing config from non-secret operational metadata such as token budgets, model limits, and pricing fields.

## Security Requirements

1. Any live secret shown to an AI-visible terminal, transcript, CI log, screenshot, or artifact is considered compromised and must be rotated.
2. Provider keys, tokens, private keys, bearer values, auth bindings, and sensitive env values must not appear in logs or error envelopes.
3. Redaction output must preserve enough metadata for operations: key name, presence, value length, fingerprint, source class, and policy state.
4. Secret scans must not print the detected raw secret.
5. Provider/runtime/auth topology remains operator-only unless a redacted contract explicitly permits disclosure.
6. Gateway and worker error envelopes must redact request headers, provider upstream errors, auth bindings, env values, private keys, bearer values, and sensitive PII before they reach operators, agents, CI artifacts, or promoted evidence.
7. Secret resolution must occur only inside trusted sealed boundaries; agents receive `SecretRef`, redacted previews, fingerprints, or status objects.

## Production Trust Requirements

1. Stable production readiness requires a repeatable rotate/revoke/scrub/audit drill with evidence.
2. Stable production readiness requires CI and SIQ-aligned gates for repo files and promoted governed evidence.
3. Stable production readiness requires an audit event for each protected-resource use, including purpose, Space, tool, grant expiry, render mode, and result status.
4. Stable production readiness requires user-legible reassurance: the system can explain that a secret was protected, where it was resolved, and what was emitted.

## Non-Goals

1. This initiative does not build a full vault in the first implementation slice.
2. This initiative does not grant runtime agents new authority to read code, env files, credentials, or provider topology.
3. This initiative does not make PII generally available to agents through prompt instructions.
4. This initiative does not treat local, tunneled, or cloud provider topology as generally readable runtime status unless a redacted contract permits it.
