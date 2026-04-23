---
id: '112'
name: cortex-runtime-convergence-and-live-collab
title: 'Decisions: 112 Cortex Runtime Convergence and Live Collaboration Governance'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-08'
updated: '2026-02-08'
---

# Decisions: 112 Cortex Runtime Convergence and Live Collaboration Governance

## DEC-001: Workflow-Engine VFS is Primary for Cortex UX Runtime Domains
- Date: 2026-02-09
- Status: Accepted
- Decision: Runtime state writes target workflow-engine VFS first with local mirror fallback and replay.
- Rationale: Distributed source-of-truth is required for cross-client convergence and auditability.
- Consequence: Runtime APIs must expose fallback/replay posture for operators.

## DEC-002: Collaboration Uses Ordered Op-Log with Lease + Optimistic Concurrency
- Date: 2026-02-09
- Status: Accepted
- Decision: Multi-actor collaboration uses deterministic op ordering with lease-bound single writer and revision checks.
- Rationale: Prevents silent overwrite while avoiding CRDT complexity in this phase.
- Consequence: Non-head proposals return explicit merge outcomes and conflict guidance.

## DEC-003: Drift Compliance is CI-Enforced with Approval Metadata Gate
- Date: 2026-02-09
- Status: Accepted
- Decision: Shared fixture route/pattern drift fails CI unless approval metadata is present.
- Rationale: Structural UX changes must remain governance-backed and auditable.
- Consequence: Fixture and runtime checks become required gates on mainline.

## DEC-004: HITL Structural Promotion Rule Remains Non-Negotiable
- Date: 2026-02-09
- Status: Accepted
- Decision: No structural promotion bypass; HITL metadata is mandatory.
- Rationale: Stewardship and constitutional boundaries require explicit accountable approval.
- Consequence: Promotion flow remains blocked without complete approval evidence.
