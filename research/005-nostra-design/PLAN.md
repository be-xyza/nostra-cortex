---
status: archived
portfolio_role: satellite
stewardship:
  layer: "Product/UX"
  primary_steward: "UX Steward"
  domain: "UI Substrate"
updated: "2026-02-08"
---
# Plan: Nostra Design & Frontend Architecture (Legacy Doctrine)

## Status
This plan is retained as legacy design doctrine. Its visual direction is not a production stabilization baseline.

## Timeless Principles Retained
- Distinct visual language per mode/host is valuable.
- Component extraction and reusable primitives reduce drift.
- Performance and mobile behavior remain non-negotiable.

## Non-Binding Legacy Directives
- Dark-mode-only cinematic directives.
- Older font mandates and glassmorphism-heavy prescriptions.

## Active Canonical Sources
- `docs/architecture/a2ui-theme-policy.md`
- `docs/architecture/unified-inbox-enrichment.md`
- `research/074-cortex-ui-substrate/PLAN.md`

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
