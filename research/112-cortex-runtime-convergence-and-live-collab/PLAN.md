---
id: "112-cortex-runtime-convergence-and-live-collab"
name: "cortex-runtime-convergence-and-live-collab"
title: "Phase 4: Cortex Runtime Convergence and Live Collaboration Governance"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authors:
  - "User"
  - "Codex"
tags: ["cortex", "desktop", "runtime", "vfs", "collaboration", "governance", "ci"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Cortex UX Runtime + Live Collaboration"
created: "2026-02-09"
updated: "2026-02-09"
---

# 112 Plan: Cortex Runtime Convergence and Live Collaboration Governance

## Objective
Converge Cortex UX runtime state domains to workflow-engine VFS primary persistence with explicit fallback telemetry, add controlled multi-actor artifact collaboration sessions, and enforce desktop/web drift compliance in CI.

## Locked Decisions
1. Workflow-engine VFS is the primary store for all Cortex UX domains; local JSON is fallback and replay cache only.
2. Collaboration model is deterministic op-log + lease + optimistic revision checks (no CRDT in this phase).
3. Structural UX promotions remain HITL-required with auditable approval metadata.

## Workstreams

### A. Workflow-Engine Primary Store Cutover
- Keep `CortexUxStore` abstraction with `WorkflowEngineVfsStore` primary and `LocalMirrorStore` fallback.
- Persist all Cortex UX state domains through store-backed I/O.
- Maintain replay queue for fallback writes with explicit runtime sync status and replay endpoints.

### B. Artifact Collaboration V2
- Add collaboration session APIs (`open`, `op`, `close`, `get`) with deterministic operation sequencing.
- Keep lease + optimistic revision enforcement for write safety.
- Maintain steward-gated publish path and artifact audit events.

### C. Feedback Automation Hardening
- Preserve lifecycle transitions with blocker states (`blocked_missing_baseline`, `blocked_missing_post_release`, `overdue_remeasurement`).
- Keep idempotent and auditable transition events and remeasurement records.

### D. Drift Enforcement in CI
- Promote `/Users/xaoj/ICP/shared/fixtures/cortex_ux_contract_fixture.json` as required parity fixture.
- Enforce route/capability/pattern checks and runtime diagnostic schema checks in CI.
- Fail CI on route/pattern drift unless approval metadata is present.

## Deliverables
- Runtime sync observability (`sync-status`, `sync/replay`) with fallback/replay telemetry.
- Controlled artifact collaboration session APIs and deterministic merge behavior.
- Phase 4 docs and governance records with archive-before-update compliance.
- CI checks for fixture parity and drift policy enforcement.

## Verification
- `cargo check --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml shared_contract_fixture_drift_requires_approval_metadata`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml cortex_runtime_sync_endpoints_report_schema`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml artifact_collab_session_open_op_close_roundtrip`
- `cargo test --manifest-path nostra/frontend/Cargo.toml shared_contract_fixture_contains_studio_and_artifacts_capabilities`

## Constitutional Alignment
- Recommendation-only remains default under ambiguous authority.
- Structural promotion remains steward/HITL gated.
- Runtime lineage, replay telemetry, and CI drift evidence are mandatory for governed evolution.

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
