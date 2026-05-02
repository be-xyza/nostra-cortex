---
id: "138-validation"
initiative: "138"
type: "validation"
status: draft
created: "2026-05-01"
updated: "2026-05-02"
---

# Validation: Protected Resources and Secret Egress

## Conversation Scope Validation

The originating conversation defined two goals:

1. Protect secrets in the immediate scenario after a provider key appeared in an AI-visible terminal flow.
2. Build long-term trust so protected resources remain explicit, governed, and safe after stable production.

The implemented Phase 0-2 slice satisfies the immediate operational scope:

| Need | Current coverage | Status |
|---|---|---|
| Treat AI-visible exposure as compromise | `DEC-138-001` | Covered |
| Rotate exposed provider key | Operator completed rotation on the VPS | Covered |
| Verify runtime config without raw values | `scripts/inspect_runtime_config_redacted.sh` | Covered |
| Avoid raw env inspection guidance | `OPERATOR_RUNBOOK.md` | Covered |
| Scan repo/governed surfaces for high-confidence secrets | `scripts/check_secret_egress.py` plus CI static analysis | Covered |
| Prove scanner and inspector do not print raw values | `scripts/test_runtime_config_redaction.sh`, `scripts/test_secret_egress_scan.sh` | Covered |

The long-term trust goal is only partially covered. Initiative 138 must continue through Phase 3 and Phase 4 before it can be considered stable-production complete.

## Production Trust Gap Matrix

| Gap | Why it matters | Owning initiative relationship | Recommended next step |
|---|---|---|---|
| Gateway/worker runtime redaction helpers | Logs and error envelopes remain the highest-risk repeat leak path after env inspection is fixed | 118, 132 | Add a shared redaction helper and tests for provider errors, headers, auth bindings, env values, and PII |
| Governed evidence promotion scan | `promote_evidence_artifact.sh` now blocks promotion when `check_secret_egress.py` finds high-confidence secrets | 125 | Covered for the canonical promotion command; SIQ artifact checks remain a future hardening path |
| Protected-resource audit events | Users need to know when a secret was used without seeing it | 130, 134 | Define `ProtectedResourceUsedV1` audit shape with purpose, Space, tool, grant, expiry, render mode, and result |
| Sealed provider invocation | Agents should request provider use without receiving raw provider keys | 118, 132, 134 | Add a sealed provider invocation contract that resolves `SecretRef` only at the Cortex boundary |
| User-legible trust surface | Trust after stable production requires status and lineage, not invisible policy | 107, 108, 130 | Add redacted UI/operator status for protected-resource use and audit lineage |
| Provider locality/topology redaction | Local/Tunneled/Cloud provider visibility is useful but can expose sensitive topology if too detailed | 137 | Define a redacted provider-topology contract with badges and status but no keys, tunnel credentials, or auth bindings |
| Incident drill evidence | Stable production should prove rotate/revoke/scrub/audit before a real incident | 116, 125, 132 | Run and record a non-production protected-resource incident drill |

## Stable Production Exit Criteria

Initiative 138 should not be marked complete until all of these are true:

1. Runtime redaction tests cover gateway and worker logs, error envelopes, upstream provider errors, request headers, auth bindings, env values, and representative PII.
2. Repo, CI, and governed evidence promotion surfaces run secret egress scans.
3. `ProtectedResource`, `SecretRef`, capability grant, and audit event schemas are defined.
4. Sealed provider/render tools resolve protected values at a trusted boundary and return only status, reference, redacted preview, or fingerprint.
5. User/operator surfaces can explain protected-resource use without revealing protected values.
6. A rotate/revoke/scrub/audit drill is recorded with redacted evidence.

## Phase 3 Runtime Redaction Evidence

Status 2026-05-01:

| Runtime surface | Coverage | Evidence |
|---|---|---|
| Cortex provider-runtime HTTP error bodies | Redacted before returning provider runtime errors | `cortex/apps/cortex-eudaemon/src/services/provider_runtime/client.rs` |
| Cortex provider-runtime SSE parse diagnostics | Redacted before including frame data in parse errors | `cortex/apps/cortex-eudaemon/src/services/provider_runtime/client.rs` |
| Cortex runtime redaction helper | Unit coverage for provider keys, bearer-like values, private keys, SSN-like values, and safe operational metadata | `cargo test --manifest-path cortex/apps/cortex-eudaemon/Cargo.toml secret_redaction --lib` |
| Worker live-generation provider errors | Redacted before returning bad-gateway provider errors | `nostra/worker/src/live_generation.rs` |
| Worker runtime redaction helper | Unit coverage for provider keys, private keys, SSN-like values, and safe operational metadata | `cargo test --manifest-path nostra/worker/Cargo.toml secret_redaction --lib` |
| Provider-admin inventory diagnostics | Redacted before returning metadata, health payloads, provider runtime status, discovery errors, runtime host health, and auth-binding metadata | `cortex/apps/cortex-eudaemon/src/gateway/provider_admin/{records,contracts,service}.rs` |
| System log tail responses | Redacted before returning JSON snapshots, JSONL raw payloads, and raw text log lines | `cargo test --manifest-path cortex/apps/cortex-eudaemon/Cargo.toml system_logs --lib` |
| Terminal service output | Redacted before live PTY broadcast and ACP terminal output retrieval | `cargo test --manifest-path cortex/apps/cortex-eudaemon/Cargo.toml terminal_service --lib` |

Remaining Phase 3 gaps:

1. SIQ artifact checks should include secret egress validation for promoted evidence bundles.
2. User-facing protected-resource audit events still need the `SecretRef`/grant lifecycle from Phase 4.
3. Stable production still needs recorded rotate/revoke/scrub/audit incident-drill evidence.

## Evidence Promotion Scan Evidence

Status 2026-05-02:

| Surface | Coverage | Evidence |
|---|---|---|
| Canonical evidence promotion command | `scripts/promote_evidence_artifact.sh` scans source artifacts before copying into `research/<initiative>/<subdir>` | `bash scripts/test_promote_evidence_secret_scan.sh` |
| Secret-bearing promotion attempt | High-confidence fake provider key blocks promotion and does not print raw secret | `bash scripts/test_promote_evidence_secret_scan.sh` |
| Safe redacted promotion attempt | Redacted/fingerprint-only artifact promotes successfully | `bash scripts/test_promote_evidence_secret_scan.sh` |

## Phase 4 Contract Evidence

Status 2026-05-02:

| Contract surface | Coverage | Evidence |
|---|---|---|
| Protected resource authority | Draft schema defines kind, Space, authority, sensitivity, storage, policy, and lineage | `research/138-protected-resources-and-secret-egress/schemas/ProtectedResourceV1.schema.json` |
| Secret reference handle | Draft schema defines expiring non-value handle with fingerprint and render mode | `research/138-protected-resources-and-secret-egress/schemas/SecretRefV1.schema.json` |
| Capability grant | Draft schema binds reference to purpose, Space, tool, render mode, approval, expiry, and audit requirement | `research/138-protected-resources-and-secret-egress/schemas/ProtectedResourceGrantV1.schema.json` |
| Sealed tool invocation | Draft schema defines sealed provider/render/inspection invocation with `SecretRef` inputs | `research/138-protected-resources-and-secret-egress/schemas/SealedToolInvocationV1.schema.json` |
| Protected-resource audit | Draft schema records trusted boundary, result, emitted fields, and fingerprint without raw values | `research/138-protected-resources-and-secret-egress/schemas/ProtectedResourceUsedV1.schema.json` |
| Schema examples | Example payloads validate against the draft schemas without tracked raw secret-shaped values | `python3` JSON Schema validation over `schemas/*.json` and `examples/*.json` |
| Cortex runtime types | Runtime structs and validation helpers mirror Phase 4 contracts and enforce raw export/audit invariants | `cargo test --manifest-path cortex/apps/cortex-eudaemon/Cargo.toml protected_resources --lib` |

Remaining Phase 4 gaps:

1. Nostra authority persistence for protected resources, grants, and audit events is not implemented yet.
2. Cortex sealed provider/render tools do not yet consume `SecretRef` grants directly.
3. User/operator trust surfaces still need redacted status and audit lineage views.
