# Decisions: Protected Resources and Secret Egress

## DEC-138-001: Treat AI-visible secret exposure as compromise

**Date**: 2026-05-01

**Decision**: Any live provider key, credential, token, private key, or sensitive personal identifier exposed through terminal output, AI-visible transcript, CI log, screenshot, generated artifact, or runtime diagnostic is considered compromised and must be rotated or revoked.

**Rationale**: AI/tool transcripts and generated evidence can be copied, indexed, retained, or replayed beyond the operator's intended trust boundary. Prompt instructions are not a reliable containment boundary.

**Implications**:
- Debugging must use redacted inspection tools.
- Incident response must prioritize rotation and audit before deeper architecture work.
- Runtime and CI surfaces must prefer fingerprints and redacted previews over raw values.

## DEC-138-002: Egress controls precede vault construction

**Date**: 2026-05-01

**Decision**: The first implementation slice will focus on redacted diagnostics and secret egress gates before building a full protected-resource vault.

**Rationale**: The immediate risk is accidental disclosure through existing operational surfaces. A vault reduces storage risk, but it does not by itself stop secrets from leaking through terminal, logs, CI artifacts, or upstream error envelopes.

**Implications**:
- Phase 1 delivers safe operator inspection.
- Phase 2 delivers scanning and CI protection.
- Nostra protected-resource primitives follow after current egress surfaces are controlled.

## DEC-138-003: Trust evidence must be user-legible without value disclosure

**Date**: 2026-05-01

**Decision**: Protected-resource use must produce user-legible trust evidence without exposing the protected value. Evidence should identify purpose, Space, tool, grant, expiry, render mode, trusted boundary, result status, and audit reference.

**Rationale**: After stable production, trust cannot rely only on internal policy claims. Users need to understand when sensitive resources were used and why, while still being protected from transcript, terminal, log, screenshot, and artifact leakage.

**Implications**:
- Agents receive references, redacted previews, fingerprints, or status objects rather than raw values.
- Runtime adapters must emit redacted audit events for protected-resource use.
- UI/operator surfaces should explain protection state and audit lineage, not secret contents.

## DEC-138-004: Provider topology is not secret value, but remains protected metadata

**Date**: 2026-05-01

**Decision**: Provider locality, model identity, tunnel status, auth binding presence, and runtime topology are protected operational metadata. They may be shown only through redacted contracts that avoid credentials, host-sensitive details, and mutation affordances.

**Rationale**: Initiatives 132 and 137 require Local/Tunneled/Cloud provider visibility for operations and trust. That visibility must not recreate the original failure mode by exposing provider keys, tunnel credentials, auth bindings, or privileged topology.

**Implications**:
- Provider dashboards may show safe badges and status summaries.
- Detailed provider inventory remains operator-only unless redacted.
- Secret egress controls apply to topology diagnostics as well as credential values.

## DEC-138-005: Runtime provider errors must be redacted before serialization

**Date**: 2026-05-01

**Decision**: Cortex provider-runtime and worker live-generation errors must pass through runtime redaction before being serialized into HTTP responses, logs, diagnostics, or test artifacts.

**Rationale**: Upstream provider error bodies can echo request headers, bearer tokens, provider keys, PII, or submitted payload fragments. Redacted env inspection does not protect users if runtime provider errors can still serialize raw upstream content.

**Implications**:
- Provider-runtime HTTP error bodies and SSE parse diagnostics are redacted before return.
- Worker live-generation provider failures are redacted before bad-gateway responses are emitted.
- Broader gateway/admin/system-log surfaces must adopt the same helper pattern before Initiative 138 is stable-production complete.

## DEC-138-006: Provider admin diagnostics redact at mapper boundaries

**Date**: 2026-05-01

**Decision**: Provider-admin inventory and diagnostic responses must redact metadata and JSON diagnostic payloads at mapper boundaries before serialization.

**Rationale**: Provider inventory is operator-only, but operator-only is not sufficient protection once outputs can be copied into tickets, AI-visible transcripts, screenshots, or evidence bundles. Redaction at the mapper boundary reduces the chance that stored metadata, upstream health payloads, auth-binding metadata, or runtime-host health echo sensitive values.

**Implications**:
- Provider records, discovery records, runtime host records, auth-binding metadata, and runtime status diagnostics apply runtime redaction before returning API responses.
- Raw auth secrets remain excluded from response contracts.
- Provider topology remains visible only as redacted operational metadata.

## DEC-138-007: Evidence promotion must scan before copy

**Date**: 2026-05-02

**Decision**: The canonical evidence promotion command must run the secret egress scanner against the source artifact before copying it into governed initiative paths.

**Rationale**: Runtime outputs are mutable operator surfaces and may contain provider errors, terminal text, logs, headers, PII, or other sensitive values. Copying them into research evidence without a blocking scan can preserve a leak in Git-governed history.

**Implications**:
- `scripts/promote_evidence_artifact.sh` blocks on high-confidence secret findings.
- The scanner must not print raw matched secret values.
- Safe redacted artifacts remain promotable.
- SIQ artifact checks should later include the same protection for multi-artifact evidence bundles.

## DEC-138-008: System logs and terminal output must redact before visibility

**Date**: 2026-05-02

**Decision**: Cortex system-log tail responses and terminal-service output must pass through runtime redaction before being returned, broadcast, copied into transcripts, or promoted as evidence.

**Rationale**: System logs and terminal output are high-risk repeat leak paths because they can echo env values, headers, provider errors, auth bindings, PII, command output, and debug dumps. Operator-only access is not sufficient once visible output can enter AI transcripts, tickets, screenshots, or governed artifacts.

**Implications**:
- JSON log snapshots and JSONL raw payloads are redacted before serialization.
- Raw text log lines are redacted before response.
- Live PTY output broadcasts and ACP terminal output retrieval are redacted before visibility.
- Raw execution internals may remain in process memory briefly, but visible surfaces must use the redacted contract.

## DEC-138-009: Protected resources use references and grants, not raw values

**Date**: 2026-05-02

**Decision**: Phase 4 protected-resource work will use `ProtectedResource`, `SecretRef`, `ProtectedResourceGrant`, `SealedToolInvocation`, and `ProtectedResourceUsed` contracts as the canonical handoff between Nostra authority and Cortex sealed execution.

**Rationale**: The system needs a trust model that is explicit enough for users to understand and strict enough that agents do not receive raw values. References and grants allow Nostra to govern purpose, Space, tool, expiry, render mode, consent, and audit requirements while Cortex resolves protected values only at trusted execution boundaries.

**Implications**:
- `ProtectedResource` records ownership, policy, storage class, fingerprint, and lineage.
- `SecretRef` is an expiring handle and must not contain raw protected values.
- `ProtectedResourceGrant` binds a handle to purpose, Space, tool, expiry, render mode, approver, and audit requirement.
- `SealedToolInvocation` lets Cortex provider/render/inspection adapters resolve values only inside trusted boundaries.
- `ProtectedResourceUsed` is the user-legible audit event and may emit status, fingerprint, redacted preview, sealed artifact reference, and audit reference, never the raw value.
