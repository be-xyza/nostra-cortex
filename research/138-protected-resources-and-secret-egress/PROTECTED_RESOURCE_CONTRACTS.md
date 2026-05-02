---
id: "138-protected-resource-contracts"
initiative: "138"
type: "contract"
status: draft
created: "2026-05-02"
updated: "2026-05-02"
---

# Protected Resource Contracts

## Purpose

This contract set defines the Phase 4 boundary for protected resources. It makes sensitive values explicit and governable without giving agents raw secret values.

## Contract Files

| Contract | Authority | Purpose |
|---|---|---|
| `schemas/ProtectedResourceV1.schema.json` | Nostra | Defines the governed resource, kind, Space, policy, storage class, fingerprint, and lineage. |
| `schemas/SecretRefV1.schema.json` | Nostra-issued, Cortex-consumed | Provides an expiring handle to a protected resource. The handle is not the value. |
| `schemas/ProtectedResourceGrantV1.schema.json` | Nostra | Binds a `SecretRef` to purpose, Space, tool, render mode, expiry, approval, and audit requirement. |
| `schemas/SealedToolInvocationV1.schema.json` | Cortex | Describes a sealed provider/render/inspection request that resolves protected values only inside a trusted boundary. |
| `schemas/ProtectedResourceUsedV1.schema.json` | Cortex-emitted, Nostra-audited | Records that a protected resource was used, which boundary resolved it, and what redacted/status fields were emitted. |

## Boundary Rules

1. `ProtectedResource` is Nostra authority. It records that a protected thing exists, who governs it, which Space it belongs to, and which policies control use.
2. `SecretRef` is a handle. It must never contain the raw secret, raw PII, credential material, provider key, document content, or auth binding value.
3. `ProtectedResourceGrant` is required before sealed use. It must bind purpose, Space, tool, expiry, render mode, approver, and audit requirement.
4. `SealedToolInvocation` is a Cortex boundary request. Cortex may resolve protected values only inside the trusted tool boundary named by the invocation.
5. `ProtectedResourceUsed` is the user-legible audit surface. It may emit status, fingerprint, redacted preview, sealed artifact reference, and audit reference. It must not emit raw values.

## Render Modes

| Mode | Agent-visible output |
|---|---|
| `status_only` | Success, denial, expiry, revocation, or boundary error. |
| `fingerprint` | Status plus stable fingerprint such as `sha256:...`. |
| `redacted_preview` | Status plus redacted preview with sensitive values removed. |
| `sealed_output` | Status plus sealed artifact reference; contents remain behind the trusted boundary. |

## Trusted Boundaries

| Boundary | Owner | Allowed resolution |
|---|---|---|
| `cortex_provider_transport` | Cortex | Provider key injection into outbound provider transport. |
| `cortex_sealed_renderer` | Cortex | Template/document rendering where protected fields resolve inside the renderer. |
| `cortex_redacted_inspector` | Cortex | Presence, length, fingerprint, source class, and policy state inspection. |
| `external_secret_manager` | Operator/Cortex adapter | Retrieval from a vault or external secret manager into a sealed Cortex boundary. |

## Non-Emission Invariant

No contract in this set may serialize raw secret values. Raw value export is forbidden by policy, and audit evidence must describe protected-resource use without reproducing the value.

