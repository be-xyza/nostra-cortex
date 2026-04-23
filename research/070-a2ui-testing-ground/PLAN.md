---
id: "070-a2ui-testing-ground"
name: "a2ui-testing-ground"
title: "A2UI Testing Ground Plan"
type: "plan"
project: "nostra"
status: superseded
portfolio_role: satellite
authors:
  - "User"
  - "Codex"
tags: ["a2ui", "testing", "conformance"]
stewardship:
  layer: "Experimental"
  primary_steward: "Research Steward"
  domain: "UI Substrate"
created: "2026-02-05"
updated: "2026-02-08"
---

# A2UI Testing Ground Plan (Absorbed)

## Status
Testing-ground scope is absorbed into `074-cortex-ui-substrate` hardening and release gates.

## What Was Absorbed
- A2UI fixture validation patterns.
- Renderer conformance checks.
- Cross-host semantic parity checkpoints for metadata-driven rendering.

## Current Role
Historical lineage only. New A2UI conformance and closeout checks are tracked under 074.

## Successor
`research/074-cortex-ui-substrate/PLAN.md`

## Alignment Addendum (Constitution + System Standards)
- Labs Constitution: Default to Production patterns unless explicitly labeled as Labs; experiments remain fork-safe and documented.
- Knowledge Integrity & Memory: Preserve lineage, log decisions, and avoid rewriting history; summaries are additive, not replacements.
- Spaces Constitution: All authority and data are space-scoped; cross-space effects are explicit links, not merges.
- Stewardship & Roles: Identify accountable roles per change; unclear authority defaults to recommendation-only.
- Contribution Lifecycle: Renames, merges, archives, and scope changes require explicit rationale and approval.
- Agent Behavior & Authority: Agents operate in observe/recommend/simulate unless execution is explicitly approved.
- Security & Privacy: Least authority, explicit consent, and scoped access; minimize sensitive data exposure.
- Governance & Escalation: Disputes and irreversible actions follow escalation pathways and steward review.
- UI/UX Manifesto: Interfaces must surface provenance, time, and agency; avoid dark patterns.
- Modularity: Strict interfaces, no hardcoded canister IDs, and clean boundary contracts.
- Composability: Actions are workflow-compatible and emit standard events.
- Data Confidence & Integrity: Confidence/reliability metadata is required where applicable.
- Portability: Data must be exportable and WASM-safe; avoid OS-specific dependencies in core logic.
- Durable Execution: State is persisted via stable memory; workflows are replayable.
- Visibility Decoupling: Index/search are async from source of truth.
- Outbox Pattern: External calls are queued with idempotency and retry semantics.
- Verification: Each initiative includes verification steps and records results.
