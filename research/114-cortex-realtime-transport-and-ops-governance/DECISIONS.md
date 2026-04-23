---
id: '114'
name: cortex-realtime-transport-and-ops-governance
title: 'Decisions: 114 Cortex Realtime Transport and Ops Governance'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-09'
updated: '2026-02-09'
---

# Decisions: 114 Cortex Realtime Transport and Ops Governance

## DEC-001: Streaming Canister Transport is Canonical Realtime Plane
- Date: 2026-02-09
- Status: Accepted
- Decision: Use `nostra/streaming/streaming.did` lifecycle methods as canonical realtime transport.
- Rationale: Aligns collaboration transport with existing IC realtime architecture without introducing new canisters.
- Consequence: Gateway must bridge certified message polling and local desktop fanout.

## DEC-002: Persist-First Write Contract with Replay Fallback
- Date: 2026-02-09
- Status: Accepted
- Decision: Persist CRDT mutations first, publish realtime envelope second; queue replay on transport failure.
- Rationale: Preserves durable correctness even under transport outage.
- Consequence: Degraded mode telemetry and replay queue management are mandatory.

## DEC-003: Privileged Collaboration Actions Require Decision Proof
- Date: 2026-02-09
- Status: Accepted
- Decision: Publish and force-resolve require steward role plus governance envelope (`approved_by`, `rationale`, `approved_at`, `decision_proof`).
- Rationale: Maintains constitutional HITL governance and explicit accountability.
- Consequence: Existing payloads must include additive governance fields and test fixtures must verify metadata presence.
