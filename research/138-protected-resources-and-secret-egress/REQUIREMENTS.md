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

## Security Requirements

1. Any live secret shown to an AI-visible terminal, transcript, CI log, screenshot, or artifact is considered compromised and must be rotated.
2. Provider keys, tokens, private keys, bearer values, auth bindings, and sensitive env values must not appear in logs or error envelopes.
3. Redaction output must preserve enough metadata for operations: key name, presence, value length, fingerprint, source class, and policy state.
4. Secret scans must not print the detected raw secret.
5. Provider/runtime/auth topology remains operator-only unless a redacted contract explicitly permits disclosure.

## Non-Goals

1. This initiative does not build a full vault in the first implementation slice.
2. This initiative does not grant runtime agents new authority to read code, env files, credentials, or provider topology.
3. This initiative does not make PII generally available to agents through prompt instructions.

