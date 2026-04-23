---
id: "113-cortex-crdt-collaboration-governance"
name: "cortex-crdt-collaboration-governance"
title: "Phase 5: Cortex Real-Time CRDT Collaboration and Governed Promotion Operations"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authors:
  - "User"
  - "Codex"
tags: ["cortex", "desktop", "crdt", "collaboration", "streaming", "governance"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Cortex Collaboration Runtime + Governance"
created: "2026-02-09"
updated: "2026-02-09"
---

# 113 Plan: Cortex Real-Time CRDT Collaboration and Governed Promotion Operations

## Objective
Upgrade Artifacts collaboration from single-writer deterministic ops to concurrent multi-writer CRDT convergence over streaming transport, with workflow-engine durability and steward/HITL governance boundaries.

## Locked Decisions
1. Runtime base: streaming transport + workflow-engine persistence.
2. Scope: collaboration core only (AI-assist and mobile deferred).
3. Structural UX promotions and privileged artifact controls remain HITL/steward governed.

## Workstreams

### A. CRDT Core
- Introduce CRDT document engine module for deterministic operation application and markdown materialization.
- Keep additive compatibility with existing `ArtifactDocumentV2` surfaces.

### B. Realtime Session + Presence
- Preserve existing collaboration session APIs and add CRDT batch/presence/state APIs.
- Enforce operation idempotency and monotonic sequence guardrails.

### C. Durable Snapshot + Recovery
- Persist CRDT snapshots/ops/presence through Cortex UX store manager with VFS-primary paths.
- Add checkpoint and replay-ready reconstruction flow.

### D. Governance Boundaries
- Keep publish steward-gated and structural promotions HITL-gated.
- Add force-resolve endpoint with steward + explicit approval metadata requirements.

### E. Contract + CI Validation
- Extend shared fixture and desktop/frontend parity checks for collaboration expectations.
- Keep drift gate and approval metadata enforcement active.

## Deliverables
- New CRDT engine/service module.
- New collaboration APIs (`state`, `op/batch`, `ops`, `presence`, `checkpoint`, `force-resolve`).
- Phase 5 initiative artifacts and architecture updates.
- Added deterministic convergence/idempotency/recovery tests.

## Verification
- `cargo check --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml crdt_converges_three_client_sequence`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml artifact_collab_batch_ops_state_presence_and_ordering_roundtrip`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml artifact_collab_force_resolve_requires_steward_and_records_governance`
- `cargo test --manifest-path nostra/frontend/Cargo.toml shared_contract_fixture_contains_studio_and_artifacts_capabilities`
- `cargo test --manifest-path nostra/frontend/Cargo.toml shared_contract_fixture_declares_phase5_collaboration_endpoints`

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
