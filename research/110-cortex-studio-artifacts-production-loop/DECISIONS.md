---
id: '110'
name: cortex-studio-artifacts-production-loop
title: 'Decisions: 110 Cortex Studio + Artifacts Production Loop'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-08'
updated: '2026-02-08'
---

# Decisions: 110 Cortex Studio + Artifacts Production Loop

## DEC-001: Persisted Contract Source of Truth
- Date: 2026-02-09
- Status: Accepted
- Decision: Use persisted UX contract payload as primary runtime source, with code defaults as fallback only.
- Rationale: Prevents desktop/web drift and enables governed contract promotion.
- Consequence: Adds contract read/write validation and compatibility adapter behavior.

## DEC-002: Production Studio/Artifacts Now
- Date: 2026-02-09
- Status: Accepted
- Decision: Promote Studio and Artifacts routes from bridge semantics to production capability lanes in this phase.
- Rationale: Required to validate current/future capability fit and close workflow loop.
- Consequence: Adds artifact APIs, route policy enforcement, and audit telemetry.

## DEC-003: Feedback Loop Must Be Closed-Loop
- Date: 2026-02-09
- Status: Accepted
- Decision: Extend feedback from write-only ingestion into triage and promotion lifecycle management.
- Rationale: Without query/triage/promotion history, enrichment and re-measurement loops remain incomplete.
- Consequence: Adds queue states, triage endpoint, approval/rejection endpoints, and promotion history endpoint.

## DEC-004: HITL Structural Promotion Gate Is Mandatory
- Date: 2026-02-09
- Status: Accepted
- Decision: Structural promotions are blocked unless HITL metadata is present.
- Rationale: Aligns with stewardship and prevents uncontrolled structural drift.
- Consequence: Gateway rejects/blocks incomplete structural promotion requests and logs explicit reasons.
