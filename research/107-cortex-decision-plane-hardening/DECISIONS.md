---
id: '107'
name: cortex-decision-plane-hardening
title: Decisions
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-08'
updated: '2026-02-08'
---

# Decisions

## D1 — Canonical-First Projection
Adopted. Gateway system projections query canisters first, then cache, then fallback with explicit `source_of_truth` and `degraded_reason`.

## D2 — Deterministic Mutation Gate Envelope
Adopted. Mutation gate surfaces synthesize epistemic assessment, governance evaluation, replay contract, and latest test-gate results into a single deterministic response.

## D3 — Policy-Enforced Decision Actions
Adopted. `decision_ack` and `decision_escalate` require role authorization and governance policy gate pass/review semantics before persistence.

## D4 — DID Drift Gate
Adopted. `scripts/check_did_declaration_sync.sh` is a release gate for workflow/governance declaration integrity.
