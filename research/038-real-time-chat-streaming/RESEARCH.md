---
id: 038
name: real-time-chat-streaming
title: 'Research: Real-Time Chat Streaming'
type: general
project: nostra
status: completed
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-25'
---

# Research: Real-Time Chat Streaming

**Initiative**: 038-real-time-chat-streaming
**Status**: ✅ COMPLETE (Prototype Validated 2026-01-19)
**Owner**: DevOps Agent
**Dependencies**: `014-ai-agents-llms-on-icp`, `028-a2ui-integration-feasibility`

## Validated Next Steps

1. **014 Nostra Chat** — Immediate (no blockers)
2. **028 A2UI Streaming** — After basic chat pipes work
3. **Production Gateway** — Deferred until 031 Phase 3 + QA gaps complete

## 2. Problem Statement

The Internet Computer's native architecture is request/response based (`update` calls take 2-5s, `query` calls are fast but read-only). Large Language Models (LLMs) generate text sequentially.

- **Current State**: Users would send a message, wait 5-10s for the full generation, then see the whole block of text.
- **Desired State**: Users see the first token immediately, and the rest stream in real-time.

## 3. Architecture Analysis

### Option A: Polling (The "Naive" Approach)
The client repeatedly polls the canister for "new tokens".
- **Pros**: Simple to implement using standard `ic-agent`.
- **Cons**: High polling rate = High Ingress cost (Cycles). Jittery UI.

### Option B: IC-WebSockets (The "Real-Time" Approach)
Uses `ic-websocket-cdk` (Rust/Motoko) and `ic-websocket-js/rs` (Client).
- **Pros**: Push-based, low latency, efficient.
- **Cons**: Requires dedicated Gateway infrastructure (or 3rd party service). Adds architectural complexity.

### Option C: HTTP Streaming (Chunked Transfer)
Using native HTTP outcalls or custom gateways to stream data.
- **Status**: ICP HTTP support is generally for *outcalls*, not *streaming capability* from canister to client directly without a gateway.

## 4. Integration with A2UI

This research must align with `028-a2ui`. The stream isn't just text; it's a stream of **A2UI Partial Updates**.

- **Protocol**:
  - Stream `surfaceUpdate` events.
  - Render components progressively (e.g., a "Thinking" component that resolves to a "Graph").

## 5. Plan

1.  **Prototype**: Build a minimal "Echo Streaming" canister test.
2.  **Evaluate**: Test Option B (IC-WebSockets) for viability on strictly decentralized infra.
3.  **Implement**: creating a `StreamingChat` capability in `nostra_backend`.

## 6. References

- [VALIDATED_REPORT](./VALIDATED_REPORT.md)
- [014 AI Agents](./../014-ai-agents-llms-on-icp/RESEARCH.md)
