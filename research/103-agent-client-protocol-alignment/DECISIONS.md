---
id: "103-agent-client-protocol-alignment-decisions"
name: "agent-client-protocol-alignment-decisions"
title: "Decision Log: Agent Client Protocol Alignment"
type: "decision"
project: "nostra"
status: in_review
authors:
  - "User"
  - "Codex"
tags: [protocol, interoperability, governance]
created: "2026-02-07"
updated: "2026-02-08"
---

# Decision Log: Agent Client Protocol Alignment

Track architectural and governance decisions for ACP evaluation against Nostra and Cortex.

---

## DEC-001: Create dedicated ACP alignment initiative
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Track ACP only as a loose reference note
2. Create a dedicated research initiative with formal artifacts

**Decision**: Create initiative `103-agent-client-protocol-alignment` with formal PLAN/RESEARCH/REQUIREMENTS/DECISIONS/FEEDBACK artifacts.

**Rationale**: ACP affects protocol boundaries, authority controls, and execution semantics across multiple active Nostra/Cortex initiatives.

**Implications**: ACP evaluation is now explicit, traceable, and reviewable for future implementation decisions.

---

## DEC-002: Keep ACP intake in recommendation-only mode pending steward approval for structural placement
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Move ACP repo from `research/reference/inbox` immediately
2. Keep ACP in inbox and complete analysis/index linkage first

**Decision**: Keep ACP in `research/reference/inbox/agent-client-protocol` for now and record placement recommendation only.

**Rationale**: Structural move is treated as a sensitive root action under current governance rules.

**Implications**: Study proceeds without structural mutation; steward can approve final placement after review.

---

## DEC-003: Approve ACP placement under agent-systems and publish event mapping matrix
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Keep ACP in `research/reference/inbox`
2. Place ACP in `research/reference/topics/agent-systems` after approval and update all linkage artifacts

**Decision**: Move ACP to `research/reference/topics/agent-systems/agent-client-protocol` and publish `ACP_NOSTRA_EVENT_MAPPING_MATRIX.md`.

**Rationale**: User approval was provided, topic fit is high, and initiative 103 now has a concrete protocol-to-event mapping artifact for pilot gating.

**Implications**: Catalog and analysis paths must remain synchronized with the new location; future work proceeds from topic placement instead of intake state.

---

## DEC-004: Implement minimal Rust ACP adapter spike in Cortex Desktop
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Keep the study documentation-only with no executable adapter code
2. Build a minimal adapter module with explicit filesystem/terminal policy wrappers and tests

**Decision**: Implement a minimal ACP adapter module at `cortex/apps/cortex-desktop/src/services/acp_adapter.rs` and validate with focused unit tests.

**Rationale**: Converts key policy assumptions into executable checks and provides a concrete baseline for a future pilot.

**Implications**: Future ACP pilot work should build on this module, preserving strict path/cwd boundaries, command allowlists, env allowlists, and output limits.

---

## DEC-005: Approve ACP pilot hardening track with hybrid event durability
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Keep ACP at spike-only level and defer runtime/session hardening
2. Implement pilot hardening with local durability only
3. Implement pilot hardening with hybrid local durability + log-registry emission path

**Decision**: Implement option 3 for `cortex-desktop`, retaining recommendation-only governance for production promotion decisions.

**Rationale**: Hybrid durability closes the replay and observability gaps identified in the mapping matrix while staying within pilot authority boundaries.

**Implications**: ACP session and permission state are now durable locally, events are projected deterministically, and failed observability emits are queued to local outbox fallback.

---

## DEC-006: Publish steward-ready adoption package with pilot recommendation
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Keep initiative in draft without gate verdict
2. Publish gate report and explicit recommendation (`watch` / `pilot` / `adopt`)

**Decision**: Publish gate report and adoption recommendation with outcome **pilot** and explicit residual risks.

**Rationale**: Phase 4 requires an actionable steward handoff rather than an open-ended findings summary.

**Implications**: Next authority action is steward review for pilot continuation, not automatic production promotion.

---

## DEC-007: Operationalize ACP pilot with explicit feature gate and durability controls
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Keep pilot always-on once runtime is present
2. Add explicit pilot gating + operational controls before steward checkpoint

**Decision**: Implement option 2 with `CORTEX_ACP_PILOT`, terminal-route gate enforcement, observability retries with idempotency header, and startup outbox flush hook.

**Rationale**: Pilot runs need deterministic enable/disable behavior and measurable reliability controls before any adoption decision.

**Implications**: ACP can now be safely toggled for staging experiments and rolled back without removing code paths.

---

## DEC-008: Treat integration socket test as environment-gated evidence
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Require integration socket tests to run in all environments
2. Keep integration test coverage but allow environment-gated execution where loopback sockets are restricted

**Decision**: Keep the integration test in `tests/acp_gateway_integration.rs` and mark it ignored by default in restricted environments.

**Rationale**: The sandbox used for current validation denies loopback socket binding; the test remains valuable and executable in staging/CI environments that permit networking.

**Implications**: Steward review includes both current passing ACP unit coverage and a staged execution requirement for full integration evidence.

---

## DEC-009: Steward approves ACP pilot continuation
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Hold implementation in `in_review` state pending additional evidence
2. Approve pilot continuation with existing operational controls and runbook

**Decision**: Approve option 2. ACP proceeds in pilot mode with `CORTEX_ACP_PILOT=1` under `recommendation_only` governance.

**Rationale**: Required pilot controls are implemented, core ACP test suites and integration lifecycle test evidence are available, and rollback is explicit via feature gating.

**Implications**: Initiative advances to staged pilot operation; next checkpoint is production-promotion criteria definition (`pilot -> adopt/watch`) after staged SLO evidence.

---

## DEC-010: Lock ACP pilot observability contract and metrics evidence surfaces
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Keep log-registry pilot expectations implicit and environment-defined
2. Publish explicit endpoint contract, SLO targets, and metrics evidence surfaces

**Decision**: Adopt option 2 with:
- `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/LOG_REGISTRY_PILOT_CONTRACT.md`
- `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_metrics.rs`
- `GET /api/metrics/acp` in `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/gateway/server.rs`

**Rationale**: Pilot continuation requires measurable reliability targets and consistent evidence collection, not implicit expectations.

**Implications**: ACP emit/fallback performance is now observable with stable counters and can be evaluated against steward-defined SLO thresholds.

---

## DEC-011: Define promotion criteria and hold post-staging outcome as steward decision
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Defer promotion criteria until after ad hoc staging observations
2. Define explicit promotion criteria now and record post-staging outcome later

**Decision**: Adopt option 2 via `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PROMOTION_CRITERIA.md`, with evidence tracked in `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_STAGING_EVIDENCE.md`.

**Rationale**: Defining the criteria before staging prevents retrospective gate-tuning and keeps authority/accountability clear.

**Implications**: Current adoption state remains `pilot`; steward must record final `pilot -> adopt/watch` outcome after staging window evidence is complete.

---

## DEC-012: Execute staging operationalization drill and keep promotion state at pilot
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Promote immediately after local operational drill
2. Keep state at `pilot` until 14-day staging SLO evidence is complete

**Decision**: Adopt option 2.

Executed evidence commands:
- `RUSTFLAGS='-Awarnings' cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml acp_` (`23 passed`)
- `RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_gateway_integration -- --ignored` (`1 passed`)
- `RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_staging_operationalization -- --ignored` (`1 passed`)

**Rationale**: The local staging drill validates outage/recovery controls and metrics behavior, but promotion criteria explicitly require a 14-day staging SLO window before `adopt` can be selected.

**Implications**: ACP remains in approved pilot mode; steward checkpoint for `pilot -> adopt/watch` is deferred until the SLO window evidence is attached.

---

## DEC-013: Start the 14-day SLO evidence collection window with scripted capture/evaluation
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Continue manual ad hoc metrics collection
2. Standardize collection and evaluation with scripts + tracked JSONL window log

**Decision**: Adopt option 2 with:
- `/Users/xaoj/ICP/scripts/acp_collect_metrics.sh`
- `/Users/xaoj/ICP/scripts/acp_evaluate_slo.sh`
- window log at `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl`

**Rationale**: Promotion gating depends on reproducible evidence, not one-off snapshots. Scripted collection/evaluation reduces operator variance and improves steward auditability.

**Implications**: The promotion checkpoint remains blocked until the window log reaches the required duration/sample count and meets SLO thresholds.

---

## DEC-014: Adopt Cortex worker native automation as the modular orchestration path for ACP pilot ops
**Date**: 2026-02-07
**Status**: Decided

**Options Considered**:
1. Keep ACP pilot operation sequencing script-driven and app-local
2. Move to worker-native workflow orchestration with reusable `SystemOp` adapters and scheduler controls

**Decision**: Adopt option 2 with:
1. `OperationAdapter` registry at `/Users/xaoj/ICP/nostra/worker/src/workflows/op_registry.rs`
2. native `acp_pilot_ops` workflow at `/Users/xaoj/ICP/nostra/worker/src/workflows/acp_pilot_ops.rs`
3. worker scheduler and control surface at:
   - `/Users/xaoj/ICP/nostra/worker/src/workflows/scheduler.rs`
   - `/Users/xaoj/ICP/nostra/worker/src/api.rs`
4. runtime wiring in:
   - `/Users/xaoj/ICP/nostra/worker/src/workflows/engine_runner.rs`
   - `/Users/xaoj/ICP/nostra/worker/src/main.rs`

**Rationale**: This preserves pilot governance constraints while making operations declarative, composable, and reusable for future automation domains beyond ACP.

**Implications**:
1. ACP orchestration is now Cortex-native with role-gated controls (`run-now`, `pause`, `resume`).
2. Workflow `SystemOp` execution is modular and adapter-dispatched by namespaced op type.
3. Production portability remains intact by keeping workflow/op semantics compatible with future canonical workflow-engine authority.

---

## DEC-015: Expose ACP native automation visibility in Cortex Desktop and refresh steward evidence contract
**Date**: 2026-02-08
**Status**: Decided

**Options Considered**:
1. Keep ACP automation operational but visible only through worker API endpoints and logs
2. Expose ACP native workflow in Cortex Desktop Workflows surface and refresh test-catalog/gate evidence artifacts

**Decision**: Adopt option 2.

Implemented evidence:
1. Desktop workflow catalog and ACP native entry/status mapping:
   - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/gateway/server.rs`
2. Workflow UI actions:
   - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/components/views/workflows_view.rs`
   - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/workflow_service.rs`
3. Contract-compliant test evidence refresh:
   - `/Users/xaoj/ICP/logs/testing/runs/local_ide_phase_next_20260208T034051Z.json`
   - `/Users/xaoj/ICP/logs/testing/test_gate_summary_latest.json` (`mode=blocking`, `overall_verdict=ready`)

**Rationale**: Native visibility is required for operator confidence, controlled pilot operations, and steward-auditable evidence without depending on implicit API knowledge.

**Implications**:
1. ACP orchestration is both executable and observable in Cortex Desktop.
2. Steward package now includes fresh contract-valid testing evidence tied to current implementation state.
3. Promotion state remains `pilot` until 14-day staging SLO window evidence is complete.

---

## DEC-016: Standardize Cortex workflow surfacing metadata and enforce test-catalog refresh consistency check
**Date**: 2026-02-08
**Status**: Decided

**Options Considered**:
1. Keep workflow surfacing limited to coarse status labels and rely on operator interpretation.
2. Add explicit automation health/control metadata to workflow catalog and enforce test-catalog refresh contract checks in the main run script.

**Decision**: Adopt option 2.

Implemented changes:
1. Workflow catalog metadata enrichment in Cortex Desktop gateway:
   - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/gateway/server.rs`
   - Added automation descriptor fields (`interval_secs`, `last_run_at`, `last_status`, `can_run_now`, `can_pause`, `can_resume`, `pause_reason`).
2. Workflow UI health surfacing in Cortex Desktop:
   - `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/components/views/workflows_view.rs`
   - Native workflow cards now display automation health context and derive control enablement from surfaced metadata.
3. Test-catalog refresh contract enforcement:
   - `/Users/xaoj/ICP/scripts/knowledge-phase-next-run.sh`
   - Ensures `scripts/check_test_catalog_consistency.sh` is executed in requested mode as part of the run pipeline.

**Rationale**: This reduces ambiguity in pilot operations, aligns UI behavior with scheduler authority state, and guarantees that generated test evidence artifacts satisfy the v1 contract before closeout.

**Implications**:
1. Cortex workflow surfacing now follows a reusable “status + automation descriptor + action capabilities” pattern.
2. Test evidence generation remains contract-compliant by default in the primary execution path.
3. Operator and steward confidence improves through clearer runtime visibility and deterministic artifact validation.
