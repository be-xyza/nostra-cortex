---
id: "111-cortex-distributed-collaboration-loop"
name: "cortex-distributed-collaboration-loop"
title: "Phase 3: Cortex Distributed Runtime, Collaborative Artifacts, and Automated UX Governance"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authors:
  - "User"
  - "Codex"
tags: ["cortex", "desktop", "studio", "artifacts", "ux", "governance", "distributed"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Cortex UX Runtime + Collaboration"
created: "2026-02-09"
updated: "2026-02-09"
---

# 111 Plan: Cortex Distributed Runtime, Collaborative Artifacts, and Automated UX Governance

## Objective
Move Cortex UX runtime from local-only persistence posture to distributed-ready source governance with explicit source-state telemetry, revisioned artifact collaboration, and lifecycle-complete feedback automation.

## Locked Decisions
1. Structural promotions remain HITL-gated and auditable.
2. Artifact collaboration for this phase is lease + optimistic concurrency (single-writer at a time).
3. Fallback remains deterministic and explicitly surfaced via runtime source-state telemetry.

## Workstreams

### A. Distributed Persistence Adapter
- Introduce source-state and drift-report APIs (`/api/cortex/layout/source-state`, `/api/cortex/layout/drift-report`).
- Keep additive fallback to local JSON while exposing `source_of_truth` and fallback state.
- Preserve persisted shell contract as governed payload.

### B. Artifact Revision Lineage and Collaboration
- Upgrade artifact model to document v2 with revision head pointers and content hash.
- Add revision APIs and save flow with `expected_revision_id` conflict protection.
- Add lease lifecycle APIs (`checkout`, `renew`, `release`) to enforce deterministic single-writer behavior.

### C. Closed-Loop Feedback Automation
- Add lifecycle APIs for promote candidate, mark shipped, mark remeasured, and overdue visibility.
- Persist lifecycle transitions and remeasurement records.
- Enforce baseline/post-release metric timestamp policy across lifecycle transitions.

### D. Drift and Governance Hardening
- Extend contract docs and capability matrix with phase 3 endpoints and lifecycle semantics.
- Keep additive-only API posture and existing endpoint compatibility.
- Record decisions and validation evidence under initiative 111 artifacts.

## Deliverables
- Distributed runtime source-state + drift API surfaces.
- Artifact document v2 + revisions + lease collaboration controls.
- Feedback automation endpoints and overdue queue visibility.
- Initiative 111 plan/requirements/decisions/validation records.

## Verification
- `cargo check --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml artifacts_save_requires_active_lease_and_matching_revision`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml feedback_overdue_returns_shipped_items_past_threshold`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml cortex_layout_source_state_reports_fallback_without_workflow_engine_id`

## Constitutional Alignment
- Recommendation-only remains default under ambiguous authority.
- Structural promotion remains explicitly HITL-controlled.
- Lineage, telemetry, and remeasurement evidence are required for stateful UX evolution.

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
