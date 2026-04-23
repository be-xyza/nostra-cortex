---
id: '115'
name: cortex-viewspec-governed-ui-synthesis
title: '115 Requirements: ViewSpec Governed UI Synthesis + Space Learning'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-09'
updated: '2026-02-09'
---

# 115 Requirements: ViewSpec Governed UI Synthesis + Space Learning

## Functional Requirements
1. System MUST expose a `ViewSpecV1` schema with:
- `view_spec_id`, `scope`, `intent`, `constraints`, `layout_graph`, `style_tokens`, `component_refs`, `confidence`, `lineage`, `policy`, `provenance`.
2. System MUST validate ViewSpecs against A2UI component catalog allowlist.
3. System MUST enforce interactive a11y labels when `policy.a11y_hard=true`.
4. System MUST compile valid ViewSpecs to A2UI v1 `RenderSurface` envelopes.
5. System MUST persist ViewSpecs to current/history/event storage keys in `/cortex/ux/viewspecs/`.
6. System MUST provide lock/fork/propose APIs with deterministic responses.
7. System MUST reject structural lock requests without HITL metadata.
8. System MUST expose candidate-set APIs:
- `POST /api/cortex/viewspecs/candidates`
- `GET /api/cortex/viewspecs/candidates/:candidate_set_id`
- `POST /api/cortex/viewspecs/candidates/:candidate_set_id/stage`
9. Candidate generation MUST return candidate envelopes including:
- `candidate_id`, `view_spec`, `validation`, optional `preview_surface`, `generation_trace`, `input_hash`.
10. Candidate staging MUST require `candidate_id`, `staged_by`, `rationale`, and `expected_input_hash`.
11. Candidate staging MUST reject hash mismatches and invalid candidate re-validation.
12. Candidate staging MUST persist staged ViewSpecs via existing current/history/event lifecycle without auto-lock.
13. Candidate sets MUST be persisted to `/cortex/ux/viewspecs/candidates/<scope>/<candidate_set_id>.json`.
14. Candidate set index MUST be maintained at `/cortex/ux/viewspecs/candidates/index.json`.
15. Candidate event stream MUST include `viewspec_candidates_generated` and `viewspec_candidate_staged`.
16. System MUST expose learning APIs:
- `POST /api/cortex/viewspecs/learning/signals`
- `GET /api/cortex/viewspecs/learning/profiles/:space_id`
- `POST /api/cortex/viewspecs/learning/profiles/:space_id/recompute`
- `POST /api/cortex/viewspecs/learning/profiles/:space_id/reset`
- `POST /api/cortex/viewspecs/:view_spec_id/confidence/recompute`
17. Learning signals MUST be persisted under `/cortex/ux/viewspecs/learning/signals/<YYYY-MM-DD>.jsonl`.
18. Learning signal index MUST be persisted at `/cortex/ux/viewspecs/learning/signals/index.json`.
19. Space profiles MUST be persisted under `/cortex/ux/viewspecs/learning/profiles/<space_id>.json`.
20. Replay artifacts MUST be persisted under `/cortex/ux/viewspecs/learning/replay/<space_id>/<run_id>.json`.
21. Stage/lock/fork/propose handlers MUST emit learning signals after successful ViewSpec persistence.
22. Learning signal ingestion MUST reject unresolved `space_id` and unsupported `event_type`.
23. Confidence recompute endpoint MUST return advisory confidence preview without implicit ViewSpec persistence.
24. System MUST expose proposal governance APIs:
- `GET /api/cortex/viewspecs/proposals`
- `GET /api/cortex/viewspecs/proposals/:proposal_id`
- `POST /api/cortex/viewspecs/proposals/:proposal_id/review`
- `POST /api/cortex/viewspecs/proposals/:proposal_id/ratify`
- `POST /api/cortex/viewspecs/proposals/:proposal_id/reject`
- `POST /api/cortex/viewspecs/proposals/:proposal_id/merge`
- `GET /api/cortex/viewspecs/active`
25. System MUST enforce proposal state transitions:
- `staged -> under_review -> approved -> ratified`
- `staged|under_review|approved -> rejected`
- `ratified -> superseded` (same scope when newer ratified proposal is adopted).
26. Ratify MUST persist active scope adoption under `/cortex/ux/viewspecs/active/<scope>.json`.
27. Proposal index MUST be persisted at `/cortex/ux/viewspecs/proposals/index.json`.
28. Proposal history MUST be persisted under `/cortex/ux/viewspecs/proposals/history/<scope>/<timestamp>_<proposal_id>.json`.
29. Proposal events MUST be persisted under `/cortex/ux/viewspecs/proposals/events/<YYYY-MM-DD>.jsonl`.
30. Governance decision evidence MUST be persisted under `/cortex/ux/viewspecs/governance/events/<YYYY-MM-DD>.jsonl`.
31. System MUST expose replay/digest APIs:
- `GET /api/cortex/viewspecs/proposals/:proposal_id/replay`
- `GET /api/cortex/viewspecs/proposals/:proposal_id/digest`
32. Replay artifacts MUST be persisted under `/cortex/ux/viewspecs/replay/<proposal_id>/<run_id>.json`.
33. Latest digest MUST be persisted under `/cortex/ux/viewspecs/replay/<proposal_id>/digest_latest.json`.
34. Proposal decision responses MUST include:
- `gate_level`
- `gate_status`
- `decision_gate_id`
- `replay_contract_ref`
- `source_of_truth` in `{canister, cache, fallback}`
- optional `degraded_reason`.

## Non-Functional Requirements
1. Compilation MUST be deterministic for identical inputs.
2. APIs MUST be additive and backward-compatible with existing gateway endpoints.
3. Persistence MUST use `cortex_ux_store_manager` for VFS primary + local fallback.
4. Proposal staging MUST not auto-ratify or auto-promote.
5. Candidate generation MUST support controlled modes (`deterministic_scaffold`, `template_hybrid`) with deterministic component topology for identical deterministic inputs.
6. Studio lane MUST provide HITL generation/reload/stage flow without autonomous promotion.
7. Learning replay MUST be deterministic for identical ordered signal input.
8. Learning arithmetic MUST clamp scores within configured min/max model bounds.
9. Learning mode MUST remain local and space-scoped (no global profile merge path in Phase 3).
10. Replay digest payloads MUST be deterministic for unchanged proposal lineage state.
11. Desktop Studio lane MUST expose proposal governance operator actions (queue, review, ratify/reject, merge, active indicator).
12. Cross-host response semantics MUST preserve proposal status and active scope metadata contract.

## Security and Governance Requirements
1. Human authority metadata (`locked_by`, approval fields) MUST be captured for structural lock operations.
2. Proposal submission MUST be blocked when validation fails.
3. Fork lineage MUST preserve `parent_view_spec_id` and `fork_reason`.
4. Candidate staging MUST remain recommendation/HITL mode and MUST NOT trigger lock/propose/ratify side effects.
5. Learning policies MUST enforce `auto_apply_enabled=false` and `global_merge_enabled=false` in Phase 3.
6. Recompute/reset operations MUST be keyed by explicit `space_id` and MUST NOT mutate other spaces.
7. Learning endpoints MUST NOT trigger autonomous lock/propose/ratify workflows.
8. Proposer MUST NOT be allowed to self-ratify.
9. Ratify and merge actions MUST enforce canonical governance authority (role binding + scope evaluation).
10. Ratify and merge MUST respect signed-intent policy mode (missing signature rejected when mode requires).
11. Read-only proposal APIs MUST remain available in degraded governance mode.
