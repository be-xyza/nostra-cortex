# 115 Validation Notes

## Validation Targets
1. Schema rejects invalid policy values and missing required fields.
2. Validator rejects missing `a11y.label` on interactive components when hard mode is enabled.
3. Validator rejects non-catalog component references.
4. Compiler output is deterministic for identical ViewSpec inputs.
5. Lock API rejects structural lock requests without HITL metadata.
6. Fork API persists lineage pointers.
7. Proposal API blocks invalid ViewSpecs.
8. Candidate generation returns explicit per-candidate validation and preview envelopes.
9. Candidate set reload endpoint returns persisted candidate sets.
10. Stage API rejects hash mismatch (`expected_input_hash`).
11. Stage API persists valid candidates and leaves `lock` unset.
12. Deterministic mode preserves component topology for identical inputs.
13. Learning signal ingestion rejects missing/unresolved `space_id`.
14. Learning replay recompute persists profile + replay artifact.
15. Confidence recompute returns deterministic advisory preview (`persisted=false`).
16. Stage/lock/propose operations emit learning signals in space scope.
17. Learning replay and score updates are deterministic for stable signal ordering.
18. Ratify endpoint rejects proposals that are not `approved`.
19. Proposer self-ratify is rejected.
20. Ratify writes active scope adoption pointer and proposal/governance events.
21. Reject writes `viewspec_proposal_rejected` and `proposal_rejected` learning signal.
22. Merge requires rationale and a distinct target proposal ID.
23. Ratify/merge enforce canonical governance role binding and scope checks.
24. Ratify/merge responses expose `source_of_truth` and `degraded_reason` semantics.
25. Replay and digest endpoints return deterministic artifacts for unchanged proposal state.

## Evidence Commands
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

## Current Evidence (2026-02-09)
1. `cargo check --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml` passed.
2. `cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml viewspec` passed.
3. `cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml viewspec_learning` passed.
4. `cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests::viewspec_candidate_stage` passed.
5. `cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests::viewspec_learning` passed.
6. `cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml gateway::server` remains noisy from unrelated legacy runtime tests; targeted gateway suites are the release gate for this phase.
7. Phase 4-6 targeted suites and closeout scripts were added to the evidence command contract for governance promotion and replay/digest hardening.
