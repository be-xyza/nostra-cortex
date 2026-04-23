---
id: "074-cortex-ui-substrate"
name: "cortex-ui-substrate"
title: "Cortex UI Substrate Stabilization Plan"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authors:
  - "User"
  - "Codex"
tags: ["ui", "ux", "a2ui", "theming", "stabilization"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "UI Substrate"
created: "2026-02-05"
updated: "2026-02-08"
---

# Cortex UI Substrate Stabilization Plan

## Overview
This initiative is the canonical UI substrate track for Cortex Desktop + Web. It consolidates prior exploratory initiatives (028, 045, 070) and focuses on stabilization before further customization expansion.

## Canonical Baseline
- `docs/architecture/a2ui-theme-policy.md`
- `docs/architecture/unified-inbox-enrichment.md`
- `docs/architecture/a2ui-spec-v1.md`
- `docs/architecture/standards.md`

## Scope
### In Scope
- Web/Desktop A2UI parity for metadata and rendering semantics.
- Theme policy hardening: safe mode allowlist, token-version handshake, motion policy, conformance tests.
- Accessibility and operability gates tied to desktop closeout checks.
- Portfolio and plan-state normalization for consolidated UI initiatives.

### Out of Scope
- Structural repository merges/renames/deletes.
- New visual platform rewrite.
- Per-space runtime theme governance as a release blocker.

## Initiative Disposition (Resolved)
- `028-a2ui-integration-feasibility`: superseded-by-074 (historical reference retained).
- `045-component-library-labs`: labs-proven, production-migrated into 074.
- `070-a2ui-testing-ground`: testing scope absorbed by 074 hardening track.
- `005-nostra-design`: legacy design doctrine; timeless principles only.
- `088-accessibility-strategy`: active-hardening until verification gates pass.
- `096-offline-sync`: remains active and integrated into offline theme/state fallback checks.
- `106/107/108`: active decision-surface tracks; included in portfolio index.

## Workstreams
### A. Plan and Spec Hygiene
- Replace placeholder plans with explicit disposition and cross-links.
- Correct stale references in implementation artifacts.
- Record reconciled authority in architecture docs and decision log.

### B. A2UI Metadata Parity (Web/Desktop)
- Preserve full `RenderSurface` metadata end-to-end in desktop pipeline.
- Eliminate component-only projection in console path.
- Keep additive A2UI v1 compatibility.

### C. Theme Policy Hardening
- Add metadata fields:
  - `token_version`
  - `motion_policy` (`system|reduced|full`)
  - `safe_mode`
  - `theme_allowlist_id`
  - `contrast_preference` (`system|more|less`)
- Enforce safe-mode allowlist and token-version compatibility fallback.
- Add policy conformance unit tests.

### D. Accessibility + Operability Gates
- Enforce reduced-motion handling, focus visibility, contrast preference handling, keyboard-safe controls, and ARIA metadata preservation.
- Integrate gate checks into `scripts/cortex-desktop-closeout-check.sh`.

## Targeted Enrichments (Post-Baseline)
1. Theme profile packs aligned to host posture (`cortex-ops`, `cortex-focus`, `nostra-knowledge`).
2. User preference persistence for motion/contrast/theme overrides with offline fallback.
3. Decision-surface legibility kit for 106/107/108 operator surfaces.
4. Offline resilience polish for cached token + metadata fallbacks (with 096 parity checks).
5. Labs-to-production provenance mapping to reduce recurrence of plan/spec drift.

## Acceptance Criteria
1. Desktop and Web consume shared metadata semantics and preserve `surface.meta`.
2. Unknown metadata fields are safely ignored and preserved through typed decode (`serde flatten` extensions).
3. Theme policy fallback is deterministic for untrusted allowlist IDs and invalid token versions.
4. Accessibility/operability source checks pass in closeout script.
5. `research/RESEARCH_INITIATIVES_STATUS.md` reflects normalized statuses including 106/107/108.

## Verification Plan
### Automated
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml --tests`
- Targeted tests for metadata merge/parsing and theme policy conformance.
- `bash scripts/cortex-desktop-closeout-check.sh`

### Manual
- Validate operator-console rendering of conflict/decision surfaces includes expected semantics (`theme`, `priority`, `context`, gate metadata).
- Validate fallback behavior when `safe_mode=true` and allowlist/token version fails.

## Deliverables
- Stabilized `research/074-cortex-ui-substrate/PLAN.md`.
- Updated implementation references and consolidated initiative notes.
- Desktop/Web A2UI parity code updates.
- Theme policy and standards doc updates.
- Decision log entry for the stabilization resolution.

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
