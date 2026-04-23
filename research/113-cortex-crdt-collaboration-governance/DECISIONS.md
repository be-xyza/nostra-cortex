---
id: '113'
name: cortex-crdt-collaboration-governance
title: 'Decisions: 113 Cortex CRDT Collaboration Governance'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-09'
updated: '2026-02-09'
---

# Decisions: 113 Cortex CRDT Collaboration Governance

## DEC-001: Adopt Deterministic CRDT for Multi-Writer Artifact Editing
- Date: 2026-02-09
- Status: Accepted
- Decision: Use deterministic CRDT envelopes with idempotent op IDs and sequence ordering.
- Rationale: Ensures concurrent edits converge without silent overwrite.
- Consequence: Collaboration state and op history become first-class persisted artifacts.

## DEC-002: Streaming + Workflow Engine Runtime Base
- Date: 2026-02-09
- Status: Accepted
- Decision: Use streaming transport semantics for realtime collaboration and workflow-engine VFS for durable state.
- Rationale: Matches current architecture direction and avoids new canister introduction.
- Consequence: Collaboration must support fallback/replay under primary outage.

## DEC-003: Steward/HITL Guardrails Remain Mandatory
- Date: 2026-02-09
- Status: Accepted
- Decision: Keep publish and privileged conflict controls steward/HITL gated.
- Rationale: Preserves constitutional governance boundaries and accountability.
- Consequence: Force-resolve and structural changes require explicit approval metadata.
