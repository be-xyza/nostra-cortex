---
id: "103-agent-client-protocol-alignment-log-registry-pilot-contract"
name: "acp-log-registry-pilot-contract"
title: "ACP Log Registry Pilot Contract"
type: "contract"
project: "nostra"
status: in_review
authors:
  - "User"
  - "Codex"
created: "2026-02-07"
updated: "2026-02-07"
---

# ACP Log Registry Pilot Contract

## Purpose
Define the operational contract between `cortex-desktop` ACP observability emission and the staging log-registry boundary during pilot mode.

## Endpoint Contract
1. Endpoint source: `CORTEX_ACP_LOG_REGISTRY_URL`.
2. Method: `POST`.
3. Headers:
   - `Content-Type: application/json`
   - `X-Idempotency-Key: <event_id>` (required)
4. Body: CloudEvent JSON envelope produced from `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_event_projector.rs` via `nostra-cloudevents`.

## Response Semantics
1. `2xx`: emit success.
2. `429` or `5xx`: transient failure (retry and/or outbox fallback).
3. Other `4xx`: rejected payload (drop from outbox, warn in logs).

## Delivery Semantics
1. Direct emit retries: 3 retries with exponential delays `250ms`, `500ms`, `1000ms`.
2. On terminal emit failure: enqueue local outbox item `kind=acp_observability_event`.
3. Outbox drain: FIFO order, stop-on-first-transient-failure.
4. Local durability is always canonical fallback (`~/.cortex/acp_events.jsonl`, local gateway queue).

## Pilot Security Boundary
1. Current pilot path uses network boundary controls (no additional app-layer auth header in emitter).
2. Staging endpoint must be restricted to approved pilot environments.
3. Any requirement for bearer/mTLS auth is a steward-controlled post-pilot enhancement.

## Pilot SLO Targets
1. `rolling_5m_success_rate >= 0.95` from `/api/metrics/acp`.
2. Outbox drain `p95 <= 5000ms` measured by `drain_latency_ms_p95`.
3. Outbox depth returns to `0` within `10 minutes` of endpoint recovery in fault-injection test.
4. No event loss across induced outage replay test (`acp_events.jsonl` count >= remote accepted count + queued remainder).

## Evidence Sources
1. `GET /api/metrics/acp`.
2. `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_STAGING_EVIDENCE.md`.
3. `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_GATE_REPORT.md`.

## Steward Checkpoint
Promotion beyond pilot requires steward confirmation that this contract and SLO targets were met for the agreed staging window.
