---
id: "103-agent-client-protocol-alignment-pilot-gate-report"
name: "acp-pilot-gate-report"
title: "ACP Pilot Gate Report"
type: "report"
project: "nostra"
status: in_review
authors:
  - "User"
  - "Codex"
created: "2026-02-07"
updated: "2026-02-08"
---

# ACP Pilot Gate Report

## Scope
Gate assessment for ACP pilot hardening and operationalization in Cortex Desktop against:
- `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/ACP_NOSTRA_EVENT_MAPPING_MATRIX.md`
- `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PROMOTION_CRITERIA.md`

## Gate Results

| Gate | Result | Evidence |
|---|---|---|
| 1. Policy-enforced filesystem and terminal adapter wrappers | PASS | `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_adapter.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/terminal_service.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/gateway/server.rs` |
| 2. ACP-to-Nostra event projection with idempotency and ordering | PASS | `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_event_projector.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_protocol.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_session_store.rs` |
| 3. `_meta` namespace and validation profile | PASS | `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_meta_policy.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_protocol.rs` |
| 4. Session replay durability beyond optional `session/load` | PASS | `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_session_store.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_protocol.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_event_sink.rs` |
| 5. Pilot operational controls (`CORTEX_ACP_PILOT`, outbox flush, runtime limits) | PASS | `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_protocol.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/main.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/terminal_service.rs` |
| 6. Pilot observability metrics and CI execution lane | PASS | `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_metrics.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/gateway/server.rs`, `/Users/xaoj/ICP/.github/workflows/test-suite.yml` |
| 7. Outage-to-recovery observability fallback drill | PASS | `/Users/xaoj/ICP/cortex/apps/cortex-desktop/tests/acp_staging_operationalization.rs` |
| 8. Cortex Desktop native workflow visibility and control for ACP automation | PASS | `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/gateway/server.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/components/views/workflows_view.rs`, `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/workflow_service.rs` |

## Verification Evidence
1. ACP module test set:
   - `RUSTFLAGS='-Awarnings' cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml acp_`
   - Result: `23 passed, 0 failed`
2. Gateway integration lifecycle:
   - `RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_gateway_integration -- --ignored`
   - Result: `1 passed, 0 failed`
3. Staging operationalization drill:
   - `RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_staging_operationalization -- --ignored`
   - Result: `1 passed, 0 failed`
4. Pilot operations runbook:
   - `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_RUNBOOK.md`
5. Log-registry contract and promotion criteria:
   - `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/LOG_REGISTRY_PILOT_CONTRACT.md`
   - `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PROMOTION_CRITERIA.md`
6. Contract-compliant test gate refresh (local IDE):
   - `bash /Users/xaoj/ICP/scripts/knowledge-phase-next-run.sh --environment local_ide --agent-id codex-local --scope release_blocker --mode advisory`
   - Result: run artifact `/Users/xaoj/ICP/logs/testing/runs/local_ide_phase_next_20260208T034051Z.json`
7. Blocking gate summary and consistency check:
   - `NOSTRA_TEST_GATE_MODE=blocking bash /Users/xaoj/ICP/scripts/generate_test_gate_summary.sh --mode blocking`
   - `NOSTRA_TEST_GATE_MODE=blocking bash /Users/xaoj/ICP/scripts/check_test_catalog_consistency.sh --mode blocking`
   - Result: `/Users/xaoj/ICP/logs/testing/test_gate_summary_latest.json` with `overall_verdict=ready`
8. Cortex Desktop gateway workflow catalog tests:
   - `cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests`
   - Result: `8 passed, 0 failed` (includes ACP native workflow visibility/status tests).

## Remaining Risks
1. 14-day staging SLO evidence remains incomplete; this blocks `pilot -> adopt` promotion.
2. Log-registry endpoint auth model is still environment-scoped (pilot-acceptable, not production-final).

## Gate Verdict
Implementation, operationalization, and local staging drill gates are complete.
Current recommendation remains **pilot continuation** until 14-day staging SLO evidence is completed and steward-reviewed.
