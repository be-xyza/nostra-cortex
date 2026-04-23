---
id: "109-cortex-desktop-ux-system"
name: "cortex-desktop-ux-system"
title: "Cortex Desktop UX System + Studio/Artifacts Bridge"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authors:
  - "User"
  - "Codex"
tags: ["cortex", "desktop", "ux", "governance", "studio", "artifacts"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Cortex UX"
created: "2026-02-09"
updated: "2026-02-09"
---

# 109 Plan: Cortex Desktop UX System + Studio/Artifacts Bridge

## Objective
Deliver a governed Cortex Desktop UX contract with schema-driven shell composition, capability qualification, CUQS scoring, and HITL promotion gates, while introducing a Studio/Artifacts bridge lane without forcing full editor migration.

## Locked Decisions
1. Studio/Artifacts integration is bridge-first in this phase.
2. Structural promotions require HITL approval metadata.
3. Operator decision clarity is the top objective.

## Workstreams

### A. UX Contract + Shell System
- Define `ShellLayoutSpec`, `NavigationGraphSpec`, `ViewCapabilityManifest`, `PatternContract`.
- Move desktop sidebar composition to manifest-driven navigation.
- Preserve compatibility via route adapter (`route_id` -> `Route`).

### B. Capability Qualification + Matrix
- Define weighted rubric matrix with route-level capability scoring.
- Publish machine-readable and human-readable matrix outputs.
- Prioritize operator-critical routes (`/workflows`, `/console`, `/testing`, `/system`).

### C. Studio/Artifacts Bridge Lane
- Add `/studio` and `/artifacts` in desktop router and shell contract.
- Add role-based route contract (`operator` for Studio, `steward` for Artifacts).
- Add bridge metadata surfaces and feedback submission path.

### D. Feedback Loop + HITL Scoring
- Add `UxFeedbackEvent`, `UxLayoutEvaluationRequest`, `UxCandidateEvaluation`, `UxPromotionDecision`.
- Add gateway APIs:
  - `GET /api/cortex/layout/spec`
  - `POST /api/cortex/layout/evaluate`
  - `POST /api/cortex/feedback/ux`
  - `GET /api/cortex/views/capability-matrix`
- Persist feedback/evaluation/promotion logs to local UX event store.

## Non-Goals (This Phase)
- Full production Artifacts Editor replacement in desktop.
- Breaking route/API changes.
- Plugin/platform rewrite.

## Acceptance Targets
1. Desktop and web can read the same UX contract endpoints.
2. Studio/Artifacts bridge exists in desktop and is role-governed.
3. Structural promotions are blocked without HITL metadata.
4. UX feedback + evaluation logs are persisted with deterministic schema.
5. Capability matrix is available from gateway and docs.

## Verification
- `cargo check --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- Optional web check: `cargo check --manifest-path nostra/frontend/Cargo.toml`

## Constitutional Alignment
- Recommendation-only remains default under authority ambiguity.
- Structural promotions require steward/operator decision trace.
- All bridge capability actions preserve lineage and role boundaries.

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
