---
id: "138"
name: "protected-resources-and-secret-egress"
title: "Protected Resources and Secret Egress"
type: "plan"
project: "nostra"
status: draft
portfolio_role: anchor
authority_mode: "recommendation_only"
execution_plane: "cortex"
depends_on:
  - "118"
  - "125"
  - "130"
  - "132"
  - "134"
  - "137"
authors:
  - "User"
  - "Codex"
tags:
  - "security"
  - "secrets"
  - "pii"
  - "agents"
  - "runtime"
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Security and Runtime Governance"
created: "2026-05-01"
updated: "2026-05-02"
---

# Initiative 138: Protected Resources and Secret Egress

## Objective

Establish the Nostra and Cortex security model for secrets, PII, provider credentials, and sealed outputs so AI agents can request approved uses of protected information without receiving raw values.

The immediate driver is an operator incident class: provider keys or sensitive runtime configuration can be exposed through terminal output, AI-visible transcripts, logs, CI artifacts, screenshots, or runtime diagnostics. The long-term target is a Nostra-governed protected-resource primitive with Cortex sealed-execution adapters.

## Boundary Model

- Nostra owns `ProtectedResource`, `SecretRef`, consent, policy, audit, lineage, and disclosure rules.
- Cortex owns sealed execution, redacted inspection, provider transport injection, runtime diagnostics, and egress controls.
- Agents may know protected resources exist and may request approved uses.
- Agents must not receive raw secret values unless a future governed contract explicitly grants a redacted or sealed output mode.
- Provider/runtime/auth topology remains operator-only unless an explicitly redacted contract says otherwise.

## Delivery Phases

### Phase 0: Incident Containment

- Rotate any provider key that appeared in terminal output, AI-visible transcript, CI log, screenshot, or artifact.
- Record the decision that AI-visible terminal exposure equals secret compromise.
- Add an operator runbook for rotate, revoke, scrub, and audit.

### Phase 1: Redacted Diagnostics

- Provide a safe runtime configuration inspection path that reports only presence, source class, value length, fingerprint, and policy state.
- Prohibit raw env inspection in agent-facing guidance: no broad `cat`, broad `grep`, service environment dumps, or terminal screenshots for secret-bearing files.
- Prefer deterministic redacted output that can be pasted into tickets or PR evidence without leaking secrets.

### Phase 2: Secret Egress Gates

- Add repo and artifact scans for high-confidence secrets before merge.
- Fail blocking checks for high-confidence credential leaks.
- Report lower-confidence PII patterns as warnings first to avoid blocking on noisy historical text.
- Keep generated runtime outputs local unless promoted after redaction.

### Phase 3: Runtime Redaction

- Ensure gateway and worker logs, error envelopes, upstream provider errors, and diagnostics redact secrets by default.
- Redact provider keys, auth bindings, request headers, bearer tokens, private keys, SSN-like values, and known sensitive env names.
- Preserve operator utility by reporting stable fingerprints and policy state rather than raw values.
- Status 2026-05-02: initial runtime redaction helpers are added for Cortex provider-runtime upstream errors/SSE parse diagnostics, worker live-generation provider errors, provider-admin inventory/diagnostic mapper boundaries, system-log tail responses, and terminal-service output. Evidence promotion now runs secret egress scanning before copying runtime artifacts into governed initiative paths.

### Phase 4: Protected Resource Primitive

- Define `ProtectedResource` kinds: provider key, credential, PII field, sealed document, and identity claim.
- Define `SecretRef` handles and capability grants with purpose, Space, tool, expiry, render mode, and audit requirements.
- Define sealed provider and document-rendering tools where values resolve only at a trusted boundary.
- Return status, `SecretRef`, or redacted previews to agents; never raw values.
- Status 2026-05-02: draft JSON schemas and example payloads now define `ProtectedResourceV1`, `SecretRefV1`, `ProtectedResourceGrantV1`, `SealedToolInvocationV1`, and `ProtectedResourceUsedV1`. The contracts remain draft and do not yet create a runtime vault, UI surface, or canister-backed authority.

## Initial Implementation Slice

This initiative starts with the narrow operational controls that reduce risk fastest:

1. Redacted runtime config inspection.
2. Secret egress scanning for governed repo surfaces.
3. CI coverage for the scanner.
4. Initiative documentation and decision records.

Full vault-backed protected resources remain a later phase after egress controls are in place.

## Verification

- Redacted inspection script tests prove raw secret values are never emitted.
- Secret egress scanner tests prove high-confidence credentials fail and low-confidence PII warns.
- CI invokes the scanner in the static-analysis job.
- Dynamic config checks pass after script additions.

## Scope Validation and Trust Gaps

The implemented Phase 0-2 slice matches the incident class from the originating conversation: a provider key or sensitive runtime value can leak through terminal output, AI-visible transcripts, logs, CI artifacts, screenshots, or diagnostics. The current controls close the fastest path to harm by treating AI-visible exposure as compromise, giving operators a redacted inspection command, and adding a repo/artifact scanner that fails on high-confidence credentials without printing the raw match.

For stable production trust, Initiative 138 must also close these gaps before it can be considered complete:

1. Runtime redaction must cover gateway and worker log/error envelopes, upstream provider errors, request headers, auth bindings, env values, and diagnostic payloads.
2. Evidence promotion must require secret scanning before mutable runtime outputs are copied into governed initiative surfaces.
3. User-facing trust surfaces must explain whether a protected resource was used, which tool boundary resolved it, and what audit event was written without revealing the value.
4. Capability grants must bind each `SecretRef` use to purpose, Space, tool, expiry, render mode, and audit requirement.
5. Operator diagnostics must avoid noisy secret classification so safe metadata such as token budgets or cost-per-token settings do not look like credentials.
6. Stable production must include incident-drill evidence for rotate, revoke, scrub, audit, and post-rotation runtime validation.

Current Phase 3 progress covers the highest-risk provider error path, provider-admin inventory diagnostics, system-log tailing, terminal output retrieval/broadcast, and scan-before-promotion for governed evidence. Remaining Phase 3 hardening should add SIQ multi-artifact evidence scanning and production-style incident drill evidence before Phase 4 protected-resource primitives are treated as complete.

Phase 4 contract progress now gives later implementation a stable target: Nostra records protected resources, issues expiring `SecretRef` handles, and approves grants; Cortex resolves values only inside sealed provider/render/inspection boundaries and emits `ProtectedResourceUsedV1` audit events. The next implementation slice should turn these draft contracts into runtime types and redacted operator/user status surfaces.

## Cross-Initiative Alignment

- Initiative 118 provides the runtime purity and adapter boundary. 138 must use that boundary to keep secret resolution in host-side sealed adapters, not in portable runtime/domain crates.
- Initiative 125 provides SIQ gates and artifact consistency. 138 should register secret egress scanning as a SIQ-aligned release control and use SIQ evidence promotion only after redaction.
- Initiative 130 provides the Space capability model. 138 should express protected-resource access as capability grants scoped to a Space and tool.
- Initiative 132 provides the active Eudaemon Alpha runtime context. 138 must keep live provider cognition operator-mediated until sealed provider invocation is implemented and audited.
- Initiative 134 provides the workflow adapter model. 138 should use workflow adapters for sealed render/provider operations so protected resources are consumed at a trusted boundary.
- Initiative 137 introduces Local/Tunneled/Cloud provider topology. 138 must ensure provider locality labels are visible without exposing provider keys, tunnel credentials, or auth bindings.
