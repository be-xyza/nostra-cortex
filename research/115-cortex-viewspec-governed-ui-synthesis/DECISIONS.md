---
id: '115'
name: cortex-viewspec-governed-ui-synthesis
title: 115 Decision Log
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-09'
updated: '2026-02-09'
---

# 115 Decision Log

## DEC-115-001: Pattern Source Boundaries
**Date**: 2026-02-09
**Status**: Approved

**Decision**: Treat BISCUIT as a pattern seed (UI-centric HITL scaffolding), while implementing Nostra-native extensions for governance, lineage, and constraints.

**Rationale**:
- Local BISCUIT source supports UI scaffolding and HITL guidance.
- Core governance/fork/merge/lineage features are required by Nostra doctrine and are outside direct paper scope.

## DEC-115-002: Phase 1 Contract Priority
**Date**: 2026-02-09
**Status**: Approved

**Decision**: Implement `ViewSpecV1` + validator + compile + lock/fork/propose APIs before autonomous generation and learning.

**Rationale**:
- Ensures safe, governable substrate before higher autonomy.
- Preserves accessibility and constitutional authority boundaries.

## DEC-115-003: Phase 2 Candidate-Set Lifecycle + Hash-Gated HITL Staging
**Date**: 2026-02-09
**Status**: Approved

**Decision**: Implement controlled candidate-set synthesis with persisted candidate sets, per-candidate validation/preview envelopes, reload API, and hash-gated staging into ViewSpec storage. Maintain strict recommendation mode boundaries.

**Rationale**:
- Candidate generation needed continuity across operator sessions (`candidate_set_id` lifecycle).
- Hash-gated staging prevents stale/tampered candidate promotion.
- Existing lock/propose governance flows remain intact while enabling faster HITL iteration.

**Implications**:
- New APIs:
  - `POST /api/cortex/viewspecs/candidates`
  - `GET /api/cortex/viewspecs/candidates/:candidate_set_id`
  - `POST /api/cortex/viewspecs/candidates/:candidate_set_id/stage`
- Candidate sets persist under `/cortex/ux/viewspecs/candidates/*` with index support.
- Event stream now includes `viewspec_candidates_generated` and `viewspec_candidate_staged`.
- No auto-lock, auto-propose, auto-ratify, or learning writes in Phase 2.

## DEC-115-004: Phase 3 Space-Scoped Temporal Learning (Advisory-Only)
**Date**: 2026-02-09
**Status**: Approved

**Decision**: Add deterministic, auditable learning profiles scoped by `space_id`, sourced from human-approved ViewSpec lifecycle signals, with replayable confidence recommendations and no autonomous UI mutation.

**Rationale**:
- Phase 2 established safe candidate generation and HITL staging, but did not retain longitudinal correction signals.
- Phase 3 requires memory while preserving constitutional authority boundaries.
- Space-scoped profiles prevent cross-space style contamination and preserve sovereign UI norms.

**Implications**:
- New APIs:
  - `POST /api/cortex/viewspecs/learning/signals`
  - `GET /api/cortex/viewspecs/learning/profiles/:space_id`
  - `POST /api/cortex/viewspecs/learning/profiles/:space_id/recompute`
  - `POST /api/cortex/viewspecs/learning/profiles/:space_id/reset`
  - `POST /api/cortex/viewspecs/:view_spec_id/confidence/recompute`
- New storage paths:
  - `/cortex/ux/viewspecs/learning/signals/<YYYY-MM-DD>.jsonl`
  - `/cortex/ux/viewspecs/learning/profiles/<space_id>.json`
  - `/cortex/ux/viewspecs/learning/replay/<space_id>/<run_id>.json`
- Stage/lock/fork/propose handlers emit learning signals when space scope is present.
- Learning policy is hard-bounded:
  - `auto_apply_enabled=false`
  - `global_merge_enabled=false`
  - no automatic lock/propose/ratify actions.

## DEC-115-005: Phase 4-6 Governance Promotion, Canonical Authority, and Replay/Digest Hardening
**Date**: 2026-02-09
**Status**: Approved

**Decision**: Extend initiative 115 through Phase 6 with (a) proposal governance lifecycle endpoints, (b) canonical governance authority checks for privileged decisions, and (c) deterministic replay/digest artifacts as graduation gates.

**Rationale**:
- Phase 3 provided advisory learning but did not complete governed promotion into active scope adoption.
- Canonical governance enforcement is required for ratify/merge to preserve institutional authority.
- Replay/digest determinism is required for durable auditability and release confidence.

**Implications**:
- Added proposal lifecycle APIs for list/get/review/ratify/reject/merge and active-scope lookup.
- Added canonical decision metadata contract (`gate_level`, `gate_status`, `decision_gate_id`, `replay_contract_ref`, `source_of_truth`, `degraded_reason`).
- Added replay and digest endpoints/artifacts under `/cortex/ux/viewspecs/replay/*`.
- Ratify writes active scope adoption pointers; proposer self-ratify is blocked.
- Degraded mode permits read operations while blocking ratify/merge when canonical authority is unavailable.

## DEC-115-006: Phase 4-6 Closeout Verification Hygiene and Evidence Lock
**Date**: 2026-02-10
**Status**: Approved

**Decision**: Lock Phase 4-6 closeout evidence with deterministic decision-action test context, archived legacy-invalid run artifacts, and refreshed advisory-mode consistency checks.

**Rationale**:
- Governance decision tests must remain deterministic under local/non-canonical state variance.
- Legacy invalid run artifacts should not pollute active consistency scans for current release confidence.
- Initiative 115 requires explicit linkage between closeout execution and durable evidence locations.

**Implications**:
- Decision-action regression tests now pin test-only governance role/policy mocks for stable outcomes.
- Legacy-invalid run artifacts were archived under `/Users/xaoj/ICP/logs/testing/runs/_archive_legacy_invalid/2026-02-10/`.
- Phase closeout references:
  - `/Users/xaoj/ICP/logs/testing/runs/local_ide_cortex_desktop_closeout_20260210T051059Z.json`
  - `/Users/xaoj/ICP/logs/testing/test_gate_summary_latest.json`
  - `/Users/xaoj/ICP/logs/testing/cortex_ui_theme_conformance_latest.json`
