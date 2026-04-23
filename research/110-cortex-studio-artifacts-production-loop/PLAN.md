---
id: "110-cortex-studio-artifacts-production-loop"
name: "cortex-studio-artifacts-production-loop"
title: "Phase 2: Cortex Studio + Artifacts Production Integration and Closed-Loop UX Governance"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authors:
  - "User"
  - "Codex"
tags: ["cortex", "desktop", "studio", "artifacts", "ux", "governance", "hitl"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Cortex UX + Capability Runtime"
created: "2026-02-09"
updated: "2026-02-09"
---

# 110 Plan: Cortex Studio + Artifacts Production Integration and Closed-Loop UX Governance

## Objective
Advance from bridge-only Studio/Artifacts lanes to production-governed capability routes while completing feedback triage/promotion closure and persisted UX contract governance.

## Locked Decisions
1. Studio/Artifacts are in-scope now as production capability lanes.
2. Structural promotions require explicit HITL metadata.
3. Feedback loop must support intake, triage, promotion decision, and re-measurement trace.

## Workstreams

### A. Persisted UX Contract Runtime
- Promote shell/navigation/capability contract from code defaults to persisted contract store.
- Add write path validation and fallback adapter behavior.
- Ensure desktop and web read from common gateway contract.

### B. Studio/Artifacts Production Lane
- Replace bridge lane behavior with production route behavior in desktop.
- Add artifact create/read/publish API flows with role-gated permissions.
- Emit artifact audit events for traceability.

### C. Closed Feedback Loop Completion
- Add feedback queue states (`new`, `deduped`, `triaged`, `candidate`, `approved`, `shipped`, `remeasured`, `rejected`).
- Add triage and promotion approval/rejection endpoints.
- Enforce metric date capture for approved promotion path.

### D. Qualification and Validation
- Reclassify Studio/Artifacts as production lanes in capability matrix.
- Keep additive API compatibility and fallback paths.
- Validate role enforcement and HITL constraints in gateway tests.

## Deliverables
- Persisted UX contract load/save endpoint contract.
- Production Studio/Artifacts routes and artifact audit trail.
- Feedback queue + triage + promotion history API surfaces.
- Updated architecture docs and initiative status index.

## Verification
- `cargo check --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- `cargo check --manifest-path nostra/frontend/Cargo.toml`

## Constitutional Alignment
- Recommendation-only remains default under authority ambiguity.
- Structural changes remain HITL-gated and auditable.
- Role and policy boundaries remain explicit on production capability lanes.

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
