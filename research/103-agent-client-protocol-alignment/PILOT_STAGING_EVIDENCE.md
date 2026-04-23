---
id: "103-agent-client-protocol-alignment-pilot-staging-evidence"
name: "acp-pilot-staging-evidence"
title: "ACP Pilot Staging Evidence"
type: "evidence"
project: "nostra"
status: in_review
authors:
  - "User"
  - "Codex"
created: "2026-02-07"
updated: "2026-02-08"
---

# ACP Pilot Staging Evidence

## Executed Validation Snapshot (2026-02-07)
1. ACP module test suite:
   - Command: `RUSTFLAGS='-Awarnings' cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml acp_`
   - Result: `23 passed, 0 failed`
2. ACP gateway integration lifecycle:
   - Command: `RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_gateway_integration -- --ignored`
   - Result: `1 passed, 0 failed`
3. ACP staging operationalization drill (outage -> fallback -> recovery):
   - Command: `RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_staging_operationalization -- --ignored`
   - Result: `1 passed, 0 failed`
4. CI lane coverage:
   - File: `/Users/xaoj/ICP/.github/workflows/test-suite.yml`
   - Includes `acp_gateway_integration` and `acp_staging_operationalization` ignored-suite runs.
5. Cortex worker native automation modules:
   - Command: `cargo test --manifest-path /Users/xaoj/ICP/nostra/worker/Cargo.toml workflows::acp_pilot_ops::tests`
   - Result: `2 passed, 0 failed`
6. Cortex worker registry/scheduler controls:
   - Command: `cargo test --manifest-path /Users/xaoj/ICP/nostra/worker/Cargo.toml workflows::engine_runner::tests`
   - Result: `2 passed, 0 failed`
   - Command: `cargo test --manifest-path /Users/xaoj/ICP/nostra/worker/Cargo.toml workflows::scheduler::tests`
   - Result: `1 passed, 0 failed`
7. Cortex worker automation API controls:
   - Command: `cargo test --manifest-path /Users/xaoj/ICP/nostra/worker/Cargo.toml api::tests::test_run_acp_now`
   - Result: `2 passed, 0 failed`

## Executed Validation Snapshot (2026-02-08)
1. Test catalog refresh run (release blocker scope):
   - Command: `bash /Users/xaoj/ICP/scripts/knowledge-phase-next-run.sh --environment local_ide --agent-id codex-local --scope release_blocker --mode advisory`
   - Result: `test_catalog_latest.json` refreshed, run artifact created at `/Users/xaoj/ICP/logs/testing/runs/local_ide_phase_next_20260208T034051Z.json`
2. Blocking gate summary and consistency validation:
   - Command: `NOSTRA_TEST_GATE_MODE=blocking bash /Users/xaoj/ICP/scripts/generate_test_gate_summary.sh --mode blocking`
   - Result: `overall_verdict=ready`, `required_blockers_pass=true`
   - Command: `NOSTRA_TEST_GATE_MODE=blocking bash /Users/xaoj/ICP/scripts/check_test_catalog_consistency.sh --mode blocking`
   - Result: `passed`
3. Cortex Desktop workflow visibility test surface:
   - Command: `cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml gateway::server::tests`
   - Result: `8 passed, 0 failed` including ACP native visibility and status mapping tests.

## Native Automation Event Contract (Worker)
ACP pilot automation emits the following operation-level events through workflow context and gateway event stream:
1. `AcpSloWindowSampleCaptured`
2. `AcpSloEvaluationComputed`
3. `AcpPromotionGateDecisionRequested`

Required event payload fields:
1. `event_type`
2. `workflow_id`
3. `timestamp`
4. `acp.slo.result` (where available)
5. `acp.slo.reason` (where available)
6. `confidence_score` (evidence publication)
7. `source_reliability` (evidence publication)
8. `validation_proof` (script/command reference)

Global semantic alignment reference:
1. `/Users/xaoj/ICP/shared/specs.md` section `4.3 ACP Automation Event Extension`.

## Staging Evidence Matrix
| Evidence Item | Status | Notes |
|---|---|---|
| 14-day SLO window data from `/api/metrics/acp` | In Progress | Collection file initialized at `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl` |
| Fault injection: remote emit outage -> local outbox queue -> recovery drain | Complete (local drill) | Validated in `acp_staging_operationalization` integration test |
| Policy denial behavior for disallowed command/path | Complete | Validated in `acp_gateway_integration` |
| Replay ordering stability by `(turn_seq, update_seq)` | Complete | Validated in ACP protocol test suite (`session_prompt_and_load_preserve_ordering`) |
| Contract-compliant local IDE test gate artifact refresh | Complete | Latest run id: `local_ide_phase_next_20260208T034051Z`; blocking gate verdict `ready` |

## Required Metric Fields
1. `emit_attempts_total`
2. `emit_success_total`
3. `emit_failure_total`
4. `fallback_queue_total`
5. `fallback_flush_success_total`
6. `fallback_flush_failure_total`
7. `rolling_5m_success_rate`
8. `drain_latency_ms_p95`

## Current Assessment
1. Operational controls and fault recovery path are test-verified.
2. Promotion to `adopt` remains blocked by the 14-day staging SLO window requirement in `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PROMOTION_CRITERIA.md`.
3. Recommended current state remains `pilot`.

## SLO Window Log (2026-02-07)
1. Initialized metrics collection file:
   - `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl`
2. Initial local drill evaluation command:
   - `/Users/xaoj/ICP/scripts/acp_evaluate_slo.sh /Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl`
3. Initial result:
   - `FAIL` (expected during outage-focused fault drill baseline; not representative of steady-state staging SLO window).

## SLO Window Log (2026-02-08)
1. Confirmed current readiness baseline with contract-compliant test artifact refresh:
   - `/Users/xaoj/ICP/logs/testing/runs/local_ide_phase_next_20260208T034051Z.json`
   - `/Users/xaoj/ICP/logs/testing/test_gate_summary_latest.json`
2. Promotion state remains blocked pending full 14-day staging SLO evidence collection window.

## Linked Criteria
1. `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/LOG_REGISTRY_PILOT_CONTRACT.md`
2. `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PROMOTION_CRITERIA.md`
