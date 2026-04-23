---
id: "115-cortex-viewspec-governed-ui-synthesis"
name: "cortex-viewspec-governed-ui-synthesis"
title: "Phase 1-6: ViewSpec Governed UI Synthesis, Governance Promotion, and GA Hardening"
type: "plan"
project: "nostra"
status: active
portfolio_role: anchor
authors:
  - "User"
  - "Codex"
tags: ["cortex", "viewspec", "a2ui", "governance", "ui-substrate", "biscuit"]
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "UI Substrate"
created: "2026-02-09"
updated: "2026-02-09"
---

# 115 Plan: ViewSpec Governed UI Synthesis + Governance Promotion + GA Hardening

## Objective
Introduce `ViewSpecV1` as a first-class, governed UI artifact contract for Cortex/Nostra, with hard validation and deterministic A2UI compilation, while preserving HITL authority, lineage, and space-scoped evolution.

## Scope
### In Scope
- `ViewSpecV1` schema contract and validation gates.
- Gateway APIs for candidate generation, validation, compile, lock, get, fork, and proposal staging.
- Store integration via Cortex UX store manager under `/cortex/ux/viewspecs/*`.
- Reference intake and governance grounding for BISCUIT as a pattern source.

### Out of Scope
- Direct Jupyter extension implementation.
- Autonomous promotion/ratification.
- Global learning policy rollout.

## Cross-Initiative Alignment
- Depends on: `research/074-cortex-ui-substrate/PLAN.md`
- Extends: `research/109-cortex-desktop-ux-system/PLAN.md`
- Integrates with: `research/110-cortex-studio-artifacts-production-loop/PLAN.md`

## Locked Decisions
1. BISCUIT is treated as pattern inspiration, not implementation import.
2. `ViewSpecV1` is artifact-class in this phase; core contribution-type promotion is deferred.
3. Structural lock operations require HITL metadata.
4. Compilation target is existing A2UI v1 contract only.

## Phases
### Phase 0: Reference + Governance Grounding
- Intake BISCUIT package from inbox into `reference/topics/ui-substrate/biscuit`.
- Publish analysis with explicit split: paper-evidenced vs Nostra-native extension.
- Record architectural decision in root `DECISIONS.md`.

### Phase 1: Declarative View Constitution
- Implement `ViewSpecV1` contract and validation rules:
  - a11y hard requirements for interactive components
  - A2UI catalog allowlist
  - policy enum checks (`motion_policy`, `contrast_preference`)
- Implement deterministic compile path `ViewSpec -> RenderSurface`.
- Add human lock/fork/propose endpoints with lineage/event persistence.

### Phase 2: Assisted Candidate Generation (Controlled)
- Add controlled candidate-set lifecycle:
  - `POST /api/cortex/viewspecs/candidates` with generation mode and actor metadata.
  - `GET /api/cortex/viewspecs/candidates/:candidate_set_id` for operator continuity.
  - `POST /api/cortex/viewspecs/candidates/:candidate_set_id/stage` for HITL staging.
- Persist candidate sets under `/cortex/ux/viewspecs/candidates/<scope>/<candidate_set_id>.json`.
- Emit events:
  - `viewspec_candidates_generated`
  - `viewspec_candidate_staged`
- Require hash-checked staging (`expected_input_hash`) and re-validation before persistence.
- Keep recommendation mode boundaries:
  - no renderer mutation
  - no auto-lock
  - no auto-propose/ratify
  - no learning writes

### Phase 3: Temporal Learning (Space-Scoped)
- Add deterministic learning signal and replay service (`viewspec_learning`).
- Persist immutable learning signals under `/cortex/ux/viewspecs/learning/signals/<YYYY-MM-DD>.jsonl`.
- Persist profiles under `/cortex/ux/viewspecs/learning/profiles/<space_id>.json`.
- Persist replay artifacts under `/cortex/ux/viewspecs/learning/replay/<space_id>/<run_id>.json`.
- Add learning APIs:
  - `POST /api/cortex/viewspecs/learning/signals`
  - `GET /api/cortex/viewspecs/learning/profiles/:space_id`
  - `POST /api/cortex/viewspecs/learning/profiles/:space_id/recompute`
  - `POST /api/cortex/viewspecs/learning/profiles/:space_id/reset`
  - `POST /api/cortex/viewspecs/:view_spec_id/confidence/recompute`
- Emit learning signals from staged/locked/forked/proposed ViewSpec outcomes.
- Keep learning policy bounded:
  - `auto_apply_enabled = false`
  - `global_merge_enabled = false`
  - no auto lock/propose/ratify side effects.

### Phase 4: Governance-Backed Promotion (Gateway/Storage)
- Add proposal lifecycle APIs:
  - `GET /api/cortex/viewspecs/proposals`
  - `GET /api/cortex/viewspecs/proposals/:proposal_id`
  - `POST /api/cortex/viewspecs/proposals/:proposal_id/review`
  - `POST /api/cortex/viewspecs/proposals/:proposal_id/ratify`
  - `POST /api/cortex/viewspecs/proposals/:proposal_id/reject`
  - `POST /api/cortex/viewspecs/proposals/:proposal_id/merge`
  - `GET /api/cortex/viewspecs/active`
- Enforce state machine:
  - `staged -> under_review -> approved -> ratified`
  - `staged|under_review|approved -> rejected`
  - `ratified -> superseded` when a newer proposal in the same scope is ratified.
- Persist additive artifacts:
  - proposal index/history/events
  - active scope adoption pointers
  - governance evidence event stream
- Emit learning signals for governance outcomes:
  - `proposal_ratified`
  - `proposal_rejected`

### Phase 5: Canonical Governance Authority Integration
- Route privileged decisions through canonical governance checks:
  - actor principal/role binding validation
  - `evaluate_action_scope_with_actor` policy gate evaluation
- Action targets are fixed:
  - `governance:viewspec:review`
  - `governance:viewspec:ratify`
  - `governance:viewspec:reject`
  - `governance:viewspec:merge`
- Apply signed-intent policy for ratify/merge via existing signed mode contract.
- Return decision metadata on proposal decision responses:
  - `gate_level`, `gate_status`, `decision_gate_id`, `replay_contract_ref`
  - `source_of_truth` in `{canister, cache, fallback}`
  - `degraded_reason` when non-canonical/degraded.
- Degraded policy:
  - read operations remain available
  - ratify/merge are blocked when canonical authority is unavailable.

### Phase 6: Replay/Digest Hardening + Graduation Gate
- Add replay/digest APIs:
  - `GET /api/cortex/viewspecs/proposals/:proposal_id/replay`
  - `GET /api/cortex/viewspecs/proposals/:proposal_id/digest`
- Persist deterministic replay artifacts and latest digest per proposal.
- Include deterministic lineage payload:
  - proposal lineage
  - decision/gate metadata
  - active adoption pointer
  - learning signal counts
- Add release-gate verification for:
  - proposal transition integrity
  - replay/digest determinism
  - source-of-truth enum contract
  - cross-host governance semantics parity.

## Acceptance Criteria
1. `ViewSpecV1` rejects missing interactive `a11y.label` when `a11y_hard=true`.
2. Non-catalog component types are rejected.
3. Invalid policy values are rejected.
4. Same `ViewSpec` input compiles to identical A2UI output.
5. Structural lock requests without HITL metadata are rejected.
6. Fork operation persists parent lineage and resets lock state.
7. Proposal endpoint blocks invalid ViewSpecs and stages valid proposals.
8. Candidate generation returns validation and preview for each candidate envelope.
9. Stage endpoint rejects hash mismatches and invalid candidates.
10. Staged candidates persist without implicit lock state.
11. Studio lane exposes generate/reload/stage operator flow.
12. Learning signal ingestion rejects unresolved space identity.
13. Recompute persists profile and replay artifacts for the requested space.
14. Confidence recompute returns advisory preview without persisting ViewSpec mutation.
15. Stage/lock/fork/propose operations emit learning signals after successful persistence.
16. Learning replay is deterministic for stable `(timestamp, signal_id)` ordering.
17. Reset restores baseline policy and keeps autonomy flags disabled.
18. Proposal ratify is blocked unless proposal status is `approved`.
19. Proposer self-ratify is rejected.
20. Ratify writes active scope adoption pointer and governance/proposal events.
21. Reject writes rejection event and `proposal_rejected` learning signal.
22. Merge requires rationale and distinct valid proposal IDs in the same scope.
23. Ratify/merge responses include gate metadata and source-of-truth contract fields.
24. Ratify/merge are blocked when canonical governance authority is unavailable.
25. Replay and digest endpoints return deterministic payload/hash for unchanged proposal state.

## Verification
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml viewspec`
- `cargo check --manifest-path cortex/apps/cortex-desktop/Cargo.toml`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml gateway::server`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests::viewspec_candidate_stage`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml viewspec_learning`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests::viewspec_learning`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests::viewspec_proposal`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests::viewspec_governance`
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests::viewspec_replay_digest`
- `bash /Users/xaoj/ICP/scripts/cortex-desktop-closeout-check.sh`
- `bash /Users/xaoj/ICP/scripts/check_test_catalog_consistency.sh advisory`

## Alignment Addendum (Constitution + System Standards)
- Labs Constitution: default to production-safe contracts; experimental generation remains explicitly gated.
- Knowledge Integrity & Memory: preserve lineage and event history for all ViewSpec mutations.
- Spaces Constitution: scope is explicit and space-local by default.
- Stewardship & Roles: structural locks require accountable human metadata.
- Contribution Lifecycle: forks/proposals are additive and auditable.
- Agent Behavior & Authority: agent outputs remain recommendational until human lock.
- UI/UX Manifesto: provenance, actionability, and accessibility are explicit in surface metadata.
- Modularity & Composability: A2UI-compatible contract, additive API evolution.
- Data Confidence & Integrity: confidence and rationale persisted on ViewSpec artifacts.
- Portability & Durability: store-manager persistence with fallback/replay semantics.

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
