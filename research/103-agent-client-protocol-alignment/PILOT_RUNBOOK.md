---
id: "103-agent-client-protocol-alignment-pilot-runbook"
name: "acp-pilot-runbook"
title: "ACP Pilot Runbook"
type: "runbook"
project: "nostra"
status: in_review
authors:
  - "User"
  - "Codex"
created: "2026-02-07"
updated: "2026-02-07"
---

# ACP Pilot Runbook

## Purpose
Operational checklist for running ACP pilot mode in `cortex-desktop`, validating gate behavior, and collecting steward decision evidence.

## Required Environment
1. `CORTEX_ACP_PILOT=1`
2. `CORTEX_ACP_LOG_REGISTRY_URL=<staging-log-registry-endpoint>` (staging/validated environments only)

## Startup Checks
1. Launch `cortex-desktop` and confirm gateway health:
   - `GET /api/health` -> `200` and `{ "status": "ok" }`
2. Confirm ACP gate-on response:
   - `POST /api/acp/rpc` with `initialize` returns `result.protocolVersion = "0.1-pilot"`.
3. Confirm startup outbox flush behavior:
   - logs include `ACP pilot startup flush completed` or explicit warning.
4. Confirm metrics surface is available:
   - `GET /api/metrics/acp` returns counters including `emit_attempts_total` and `rolling_5m_success_rate`.

## Smoke Tests
1. Gate-off behavior:
   - unset `CORTEX_ACP_PILOT`
   - `POST /api/acp/rpc` returns JSON-RPC error `code=-32030`, `data.errorCode=ACP_PILOT_DISABLED`
   - `POST /api/acp/terminal/create` returns HTTP `503`, `errorCode=ACP_PILOT_DISABLED`
2. Lifecycle behavior (gate on):
   - `initialize -> session/new -> session/prompt -> session/load`
   - verify session load includes ordered updates.
3. Policy behavior:
   - disallowed terminal command (`rm`) returns policy denial.
   - filesystem relative path read returns policy denial.
4. Reliability behavior:
   - with invalid `CORTEX_ACP_LOG_REGISTRY_URL`, confirm local JSONL durability and outbox queue fallback.
   - restore valid endpoint and confirm queue drain via startup flush or explicit flush trigger.

## Validation Commands
```bash
RUSTFLAGS='-Awarnings' cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml acp_
RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_gateway_integration -- --ignored
RUSTFLAGS='-Awarnings' cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml --test acp_staging_operationalization -- --ignored
```

## CI Evidence
1. Verify `acp-gateway-integration` job passes in:
   - `/Users/xaoj/ICP/.github/workflows/test-suite.yml`
   - job includes both ignored suites:
     - `acp_gateway_integration`
     - `acp_staging_operationalization`
2. Capture workflow run URL or job artifact for steward packet.

## 14-Day SLO Window Operation
1. Start pilot with `CORTEX_ACP_PILOT=1` and staging log-registry endpoint.
2. Capture metrics snapshots (recommended hourly or daily at minimum):
```bash
/Users/xaoj/ICP/scripts/acp_collect_metrics.sh http://127.0.0.1:3000 /Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl
```
3. Evaluate current window against SLO thresholds:
```bash
/Users/xaoj/ICP/scripts/acp_evaluate_slo.sh /Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl
```
4. Record evaluation outputs in:
   - `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_STAGING_EVIDENCE.md`

## Rollback
1. Disable pilot mode: unset `CORTEX_ACP_PILOT`.
2. Restart `cortex-desktop`.
3. Verify ACP endpoints return disabled responses (RPC error / terminal REST `503`).
4. Preserve local durability artifacts for forensics:
   - `~/.cortex/acp_sessions.json`
   - `~/.cortex/acp_permission_ledger.json`
   - `~/.cortex/acp_events.jsonl`
   - `~/.nostra/cortex/local_gateway_queue.json`
   - `~/.cortex/acp_metrics.json`

## Evidence to Capture for Steward Review
1. ACP test command output summary.
2. Gate-off and gate-on smoke request/response samples.
3. Outbox fallback and flush evidence.
4. Metrics snapshots from `/api/metrics/acp` during fault/recovery window.
5. Updated gate report, promotion criteria, and adoption recommendation links.
