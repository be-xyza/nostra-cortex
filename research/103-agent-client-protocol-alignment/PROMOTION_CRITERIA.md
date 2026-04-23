---
id: "103-agent-client-protocol-alignment-promotion-criteria"
name: "acp-promotion-criteria"
title: "ACP Pilot Promotion Criteria"
type: "criteria"
project: "nostra"
status: in_review
authors:
  - "User"
  - "Codex"
created: "2026-02-07"
updated: "2026-02-07"
---

# ACP Pilot Promotion Criteria

## Goal
Define objective gates for the steward decision path from `pilot` to either `adopt` or `watch` under `recommendation_only` governance.

## Hard Gates (Must Pass)
1. ACP unit/integration suite passes:
   - `RUSTFLAGS='-Awarnings' cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml acp_`
   - `RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_gateway_integration -- --ignored`
2. Pilot gate control verified (`CORTEX_ACP_PILOT` on/off contract).
3. No confirmed policy-boundary bypass for path, command, or env controls.
4. Log-registry pilot contract and SLO targets met per `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/LOG_REGISTRY_PILOT_CONTRACT.md`.
5. Pilot runbook completed with evidence capture (`/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_RUNBOOK.md`).
6. 14-day metrics window captured at `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl` with at least 14 daily samples.

## Decision Matrix
1. `adopt`:
   - All hard gates pass.
   - 14-day staging window meets SLOs.
   - No unresolved high-severity operational issues.
2. `pilot` (continue):
   - Core tests pass, but SLO window is incomplete or risk mitigations are still in progress.
3. `watch`:
   - Hard-gate failure, repeat boundary breaches, or unstable observability durability.

## Required Evidence Package
1. `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_STAGING_EVIDENCE.md`
2. `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_GATE_REPORT.md`
3. `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/ADOPTION_RECOMMENDATION.md`
4. `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/DECISIONS.md` decision entry for post-staging outcome.
5. SLO evaluation output from:
   - `/Users/xaoj/ICP/scripts/acp_evaluate_slo.sh /Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl`

## Steward Decision Protocol
1. Technical recommendation remains non-binding (`recommendation_only`).
2. Steward records final outcome in `DECISIONS.md` with rationale and residual risks.
3. If outcome is `adopt`, promotion is staged and reversible through `CORTEX_ACP_PILOT` rollback path until full production policy ratification.
