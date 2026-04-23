---
id: '109'
name: cortex-desktop-ux-system
title: 'Decisions: 109 Cortex Desktop UX System'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-08'
updated: '2026-02-08'
---

# Decisions: 109 Cortex Desktop UX System

## DEC-001: Bridge-First Studio/Artifacts Integration
- Date: 2026-02-09
- Status: Accepted
- Decision: Add Studio and Artifacts as governed bridge lanes in Cortex Desktop now; defer full editor platform migration.
- Rationale: Preserves momentum and validation coverage without conflating shell-governance work with full editor-productization.
- Consequence: `/studio` and `/artifacts` exist as capability lanes with explicit role and contract metadata.

## DEC-002: HITL Required for Structural Promotions
- Date: 2026-02-09
- Status: Accepted
- Decision: CUQS may rank candidates automatically, but structural promotions require human approval metadata.
- Rationale: Prevents unauthorized structural drift and aligns with stewardship authority doctrine.
- Consequence: Evaluations without approval metadata are blocked with deterministic reason codes.

## DEC-003: Additive Gateway Contract Expansion
- Date: 2026-02-09
- Status: Accepted
- Decision: Introduce `/api/cortex/*` endpoints without breaking existing routes.
- Rationale: Enables phased rollout and low-risk adoption by existing clients.
- Consequence: Desktop and web can adopt contract consumption incrementally.

## DEC-004: Local UX Event Store
- Date: 2026-02-09
- Status: Accepted
- Decision: Persist feedback/evaluation/promotion events in JSONL under local Cortex UX log root.
- Rationale: Fast traceability and auditability with minimal operational overhead.
- Consequence: Adds local operational evidence for post-release loop closure.
