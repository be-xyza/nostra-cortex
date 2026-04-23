---
id: '111'
name: cortex-distributed-collaboration-loop
title: 'Decisions: 111 Cortex Distributed Collaboration Loop'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-08'
updated: '2026-02-08'
---

# Decisions: 111 Cortex Distributed Collaboration Loop

## DEC-001: Source-State Telemetry Is Mandatory for Runtime Contract Governance
- Date: 2026-02-09
- Status: Accepted
- Decision: Add explicit source-state and drift-report APIs as first-class runtime governance surfaces.
- Rationale: Distributed readiness requires observable source-of-truth posture, not implicit filesystem assumptions.
- Consequence: Contract consumers can detect fallback mode and route/pattern drift before promotion.

## DEC-002: Artifact Collaboration Uses Lease + Optimistic Revision Control in Phase 3
- Date: 2026-02-09
- Status: Accepted
- Decision: Use lease-based single-writer guardrails with `expected_revision_id` conflict checks.
- Rationale: Provides deterministic edit integrity without introducing CRDT complexity in this phase.
- Consequence: Save/publish conflicts are explicit and lineage is preserved through immutable revisions.

## DEC-003: Feedback Lifecycle Must Reach Shipped and Remeasured States
- Date: 2026-02-09
- Status: Accepted
- Decision: Extend feedback automation beyond triage/approval into shipped/remeasured and overdue monitoring.
- Rationale: Governance loop is incomplete unless post-release impact is measured and surfaced.
- Consequence: Lifecycle transition events and remeasurement records become required evidence artifacts.

## DEC-004: Structural Promotion HITL Policy Remains Non-Negotiable
- Date: 2026-02-09
- Status: Accepted
- Decision: Keep structural promotion approval metadata mandatory; no auto-promotion bypass.
- Rationale: Preserves stewardship and constitutional authority boundaries.
- Consequence: Promotion APIs and lifecycle transitions enforce approval metadata and audit traceability.
