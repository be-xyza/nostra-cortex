# Validation: 116 Cortex Realtime GA Trust Hardening

## Validation Targets
1. Governance nonce replay is rejected.
2. Realtime fixture parity includes Phase 7 endpoints.
3. Runtime SLO endpoints return structured payloads.
4. Realtime integrity endpoint returns ack/degraded telemetry.
5. Realtime resync endpoint executes replay/resync flow safely.

## Task Evidence Map
| task_id | Scope | Primary evidence |
|---|---|---|
| `P7-001-day1-baseline-validation` | Day 1 command matrix | `research/116-cortex-realtime-ga-trust-hardening/VALIDATION.md` |
| `P7-002-ci-phase7-gates` | CI hardening and parity gates | `.github/workflows/test-suite.yml` |
| `P7-003-canary-d2` | D2 canary metrics | `research/116-cortex-realtime-ga-trust-hardening/VALIDATION.md` |
| `P7-004-canary-d3` | D3 canary metrics | `research/116-cortex-realtime-ga-trust-hardening/VALIDATION.md` |
| `P7-005-canary-d4` | D4 canary metrics | `research/116-cortex-realtime-ga-trust-hardening/VALIDATION.md` |
| `P7-006-canary-d5` | D5 canary metrics | `research/116-cortex-realtime-ga-trust-hardening/VALIDATION.md` |
| `P7-007-canary-d6` | D6 canary metrics | `research/116-cortex-realtime-ga-trust-hardening/VALIDATION.md` |
| `P7-008-canary-d7` | D7 canary metrics | `research/116-cortex-realtime-ga-trust-hardening/VALIDATION.md` |
| `P7-009-drill-stream-outage` | Outage drill | `research/116-cortex-realtime-ga-trust-hardening/VALIDATION.md` |
| `P7-010-drill-gap-resync` | Gap/resync drill | `research/116-cortex-realtime-ga-trust-hardening/VALIDATION.md` |
| `P7-011-drill-replay-drain` | Replay drain drill | `logs/cortex/ux/_runtime/streaming_replay_queue.json` |
| `P7-012-governance-reject-paths` | Governance reject-path signoff | `logs/cortex/ux/artifact_audit_events.jsonl` |
| `P7-013-governance-allow-path` | Governance allow-path signoff | `logs/cortex/ux/promotion_decisions.jsonl` |
| `P7-014-integrity-resync-ack-evidence` | Integrity/resync/ack evidence | `logs/cortex/ux/_runtime/streaming_ack_cursors.json` |
| `P7-015-day8-go-no-go` | Day 8 decision | `research/116-cortex-realtime-ga-trust-hardening/DECISIONS.md` |
| `P7-016-status-transition` | Initiative status transition | `research/RESEARCH_INITIATIVES_STATUS.md` |

## Day 1 Command Matrix (Executed 2026-02-09 UTC)
| task_id | Check | Command | Start (UTC) | End (UTC) | Result | Evidence |
|---|---|---|---|---|---|---|
| `P7-001-day1-baseline-validation` | Desktop compile gate | `cargo check -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml` | 2026-02-09T17:12:03Z | 2026-02-09T17:12:04Z | PASS | `/tmp/cortex_phase7_day1_logs_2026-02-09/cargo_check_desktop.log` |
| `P7-001-day1-baseline-validation` | Governance replay rejection | `cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml artifact_governance_nonce_replay_is_rejected` | 2026-02-09T17:12:04Z | 2026-02-09T17:12:05Z | PASS | `/tmp/cortex_phase7_day1_logs_2026-02-09/test_governance_nonce_replay.log` |
| `P7-001-day1-baseline-validation` | Runtime SLO API payloads | `cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml cortex_runtime_slo_endpoints_return_payloads` | 2026-02-09T17:12:05Z | 2026-02-09T17:12:05Z | PASS | `/tmp/cortex_phase7_day1_logs_2026-02-09/test_slo_endpoints.log` |
| `P7-001-day1-baseline-validation` | Desktop fixture parity | `cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml shared_contract_fixture_declares_phase7_realtime_endpoints` | 2026-02-09T17:12:05Z | 2026-02-09T17:12:06Z | PASS | `/tmp/cortex_phase7_day1_logs_2026-02-09/test_fixture_phase7_desktop.log` |
| `P7-001-day1-baseline-validation` | Frontend fixture parity | `cargo test -q --manifest-path /Users/xaoj/ICP/nostra/frontend/Cargo.toml shared_contract_fixture_declares_phase7_realtime_collaboration_endpoints` | 2026-02-09T17:12:06Z | 2026-02-09T17:12:06Z | PASS | `/tmp/cortex_phase7_day1_logs_2026-02-09/test_fixture_phase7_frontend.log` |
| `P7-001-day1-baseline-validation` | Realtime transport ordering smoke | `cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml loopback_publish_and_poll_are_ordered` | 2026-02-09T17:13:17Z | 2026-02-09T17:13:18Z | PASS | terminal run on 2026-02-09 |

## CI Gate Hardening Verification (Executed 2026-02-09 UTC)
| task_id | Check | Command | Start (UTC) | End (UTC) | Result | Evidence |
|---|---|---|---|---|---|---|
| `P7-002-ci-phase7-gates` | Desktop Phase 7 fixture endpoints | `cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml shared_contract_fixture_declares_phase7_realtime_endpoints` | 2026-02-09T17:56:15Z | 2026-02-09T17:56:45Z | PASS | `/tmp/cortex_phase7_closeout_gate_logs_2026-02-09/test_phase7_fixture_endpoints_desktop.log` |
| `P7-002-ci-phase7-gates` | Desktop Phase 7 governance fixture fields | `cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml shared_contract_fixture_declares_phase7_governance_metadata_fields` | 2026-02-09T17:56:45Z | 2026-02-09T17:56:46Z | PASS | `/tmp/cortex_phase7_closeout_gate_logs_2026-02-09/test_phase7_fixture_governance_desktop.log` |
| `P7-002-ci-phase7-gates` | Desktop realtime default-on/kill-switch | `cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml realtime_feature_flag_defaults_enabled_and_supports_kill_switch` | 2026-02-09T17:56:46Z | 2026-02-09T17:56:46Z | PASS | `/tmp/cortex_phase7_closeout_gate_logs_2026-02-09/test_phase7_featureflag_desktop.log` |
| `P7-002-ci-phase7-gates` | Desktop runtime SLO payload contract | `cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml cortex_runtime_slo_endpoints_return_payloads` | 2026-02-09T17:56:46Z | 2026-02-09T17:56:47Z | PASS | `/tmp/cortex_phase7_closeout_gate_logs_2026-02-09/test_phase7_slo_endpoints_desktop.log` |
| `P7-002-ci-phase7-gates` | Frontend Phase 7 realtime endpoint parity | `cargo test -q --manifest-path /Users/xaoj/ICP/nostra/frontend/Cargo.toml shared_contract_fixture_declares_phase7_realtime_collaboration_endpoints` | 2026-02-09T17:56:47Z | 2026-02-09T17:56:47Z | PASS | `/tmp/cortex_phase7_closeout_gate_logs_2026-02-09/test_phase7_fixture_endpoints_frontend.log` |
| `P7-002-ci-phase7-gates` | Frontend Phase 7 governance metadata parity | `cargo test -q --manifest-path /Users/xaoj/ICP/nostra/frontend/Cargo.toml shared_contract_fixture_declares_phase7_realtime_governance_metadata` | 2026-02-09T17:56:47Z | 2026-02-09T17:56:49Z | PASS | `/tmp/cortex_phase7_closeout_gate_logs_2026-02-09/test_phase7_fixture_governance_frontend.log` |

## Day 1 Evidence Path Status (Observed 2026-02-09 UTC)
| task_id | Path | Status | Notes |
|---|---|---|---|
| `P7-014-integrity-resync-ack-evidence` | `/Users/xaoj/ICP/logs/cortex/ux/_runtime/streaming_ack_cursors.json` | Exists | Created by realtime transport test; mtime 2026-02-09T11:13:17Z |
| `P7-011-drill-replay-drain` | `/Users/xaoj/ICP/logs/cortex/ux/_runtime/streaming_replay_queue.json` | Missing | Expected until backlog event occurs during outage/replay drill |
| `P7-011-drill-replay-drain` | `/Users/xaoj/ICP/logs/cortex/ux/_runtime/streaming_slo_alerts.jsonl` | Missing | Expected until SLO breach alert is emitted |
| `P7-012-governance-reject-paths` | `/Users/xaoj/ICP/logs/cortex/ux/artifact_audit_events.jsonl` | Missing | Pending privileged mutation drill evidence |
| `P7-013-governance-allow-path` | `/Users/xaoj/ICP/logs/cortex/ux/promotion_decisions.jsonl` | Missing | Pending privileged governance drill evidence |

## 7-Day Canary Tracker (Realtime default-on)
SLO thresholds:
- `convergence_latency_p95 <= 3000ms`
- `replay_backlog_drain_time_p95 <= 60s`
- `duplicate_drop_rate <= 0.5%`
- `daily_degraded_duration <= 15m`

| task_id | Day | Date (UTC) | convergence_latency_p95_ms | replay_backlog_drain_time_p95_s | duplicate_drop_rate | daily_degraded_duration_s | Result | Evidence |
|---|---|---|---:|---:|---:|---:|---|---|
| `P7-001-day1-baseline-validation` | D1 | 2026-02-09 | 0 | 0 | 0.000000 | 0 | PASS (initial baseline) | Day 1 command matrix + runtime checks |
| `P7-003-canary-d2` | D2 | 2026-02-10 | pending | pending | pending | pending | PENDING | Pending canary snapshot |
| `P7-004-canary-d3` | D3 | 2026-02-11 | pending | pending | pending | pending | PENDING | Pending canary snapshot + outage drill |
| `P7-005-canary-d4` | D4 | 2026-02-12 | pending | pending | pending | pending | PENDING | Pending canary snapshot |
| `P7-006-canary-d5` | D5 | 2026-02-13 | pending | pending | pending | pending | PENDING | Pending canary snapshot + resync drill |
| `P7-007-canary-d6` | D6 | 2026-02-14 | pending | pending | pending | pending | PENDING | Pending canary snapshot |
| `P7-008-canary-d7` | D7 | 2026-02-15 | pending | pending | pending | pending | PENDING | Pending final canary day |

## Incident Drill Execution Log (Days 3-5)
| task_id | Drill | Trigger time (UTC) | Recovery time (UTC) | MTTR | Commands used | Degraded transition | Outcome |
|---|---|---|---|---|---|---|---|
| `P7-009-drill-stream-outage` | Stream outage simulation | pending | pending | pending | pending | pending | PENDING |
| `P7-010-drill-gap-resync` | Gap detection + resync | pending | pending | pending | pending | pending | PENDING |
| `P7-011-drill-replay-drain` | Replay backlog drain | pending | pending | pending | pending | pending | PENDING |

## Governance and Identity Signoff (Days 4-6)
| task_id | Scenario | Expected | Status | Evidence |
|---|---|---|---|---|
| `P7-012-governance-reject-paths` | Invalid signature rejected | HTTP deny + audit entry | PENDING | Pending drill |
| `P7-012-governance-reject-paths` | Expired envelope rejected | HTTP deny + audit entry | PENDING | Pending drill |
| `P7-012-governance-reject-paths` | Replayed nonce rejected | HTTP deny + audit entry | PASS | `artifact_governance_nonce_replay_is_rejected` |
| `P7-012-governance-reject-paths` | HTTP/WS actor mismatch rejected | HTTP deny + audit entry | PENDING | Pending drill |
| `P7-013-governance-allow-path` | Valid steward action accepted | Mutation + audit + ledger | PENDING | Pending drill |

## CI and Release Gate Checklist (Days 6-7)
| task_id | Gate | Requirement | Status | Evidence |
|---|---|---|---|---|
| `P7-002-ci-phase7-gates` | Desktop fixture parity | Required and passing | PASS | `shared_contract_fixture_declares_phase7_realtime_endpoints` |
| `P7-002-ci-phase7-gates` | Frontend fixture parity | Required and passing | PASS | `shared_contract_fixture_declares_phase7_realtime_collaboration_endpoints` |
| `P7-002-ci-phase7-gates` | Desktop governance fixture fields | Required and passing | PASS | `shared_contract_fixture_declares_phase7_governance_metadata_fields` |
| `P7-002-ci-phase7-gates` | Frontend governance fixture fields | Required and passing | PASS | `shared_contract_fixture_declares_phase7_realtime_governance_metadata` |
| `P7-002-ci-phase7-gates` | Realtime feature-flag guard | Required and passing | PASS | `realtime_feature_flag_defaults_enabled_and_supports_kill_switch` |
| `P7-002-ci-phase7-gates` | Governance drift metadata | CI rejects drift without approval metadata | PASS (repo-configured) | `/Users/xaoj/ICP/.github/workflows/test-suite.yml` + `shared_contract_fixture_drift_requires_approval_metadata` |
| `P7-002-ci-phase7-gates` | Runtime SLO APIs | Required and passing | PASS | `cortex_runtime_slo_endpoints_return_payloads` |
| `P7-014-integrity-resync-ack-evidence` | Realtime integrity/resync/ack paths | API validation evidence recorded | PENDING | Pending drill execution |

## Exit Criteria
1. Seven consecutive canary days complete with SLO compliance and evidence links.
2. Governance reject/allow paths complete with audit and ledger traces.
3. Runbook drills complete with MTTR and degraded transition evidence.
4. CI parity/drift gates confirmed mandatory on mainline.
5. Closeout decision recorded in `DECISIONS.md` and status transition handled per day-8 gate (`P7-015-day8-go-no-go`, `P7-016-status-transition`).

## Current Closeout Status (As of 2026-02-10)
- Initiative status: `active`.
- Day 1 execution: complete.
- CI parity/governance gate updates: implemented and locally validated.
- Remaining blockers: time-bound canary window and scheduled drill execution.
