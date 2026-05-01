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
  - "132"
  - "134"
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
updated: "2026-05-01"
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

### Phase 4: Protected Resource Primitive

- Define `ProtectedResource` kinds: provider key, credential, PII field, sealed document, and identity claim.
- Define `SecretRef` handles and capability grants with purpose, Space, tool, expiry, render mode, and audit requirements.
- Define sealed provider and document-rendering tools where values resolve only at a trusted boundary.
- Return status, `SecretRef`, or redacted previews to agents; never raw values.

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

