---
id: 038
name: real-time-chat-streaming
title: 'Decisions: Real-Time Chat Streaming'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions: Real-Time Chat Streaming

**Initiative**: 038-real-time-chat-streaming

## Decision Log

| ID | Date | Decision | Rationale |
|:---|:-----|:---------|:----------|
| DEC-038-001 | 2026-01-19 | Use **IC-WebSocket** (Option B) over Polling (Option A) | Push-based streaming provides ~27ms token intervals vs polling overhead. Verified in Echo Prototype. |
| DEC-038-002 | 2026-01-19 | Use **Rust** backend over Motoko | Motoko persistence constraints (`M0219`, `M0220`) made WebSocket state management impractical. |
| DEC-038-003 | 2026-01-19 | Use `ic-websocket-cdk@0.4.2` (Rust) | Stable API with timer support. Compatible with `ic-cdk@0.18`. |
| DEC-038-004 | 2026-01-19 | Use `ic-websocket-js@0.5.0` frontend | v0.2.2 had critical Node.js polyfill issues with Vite. v0.5.0 is browser-compatible. |
| DEC-038-005 | 2026-01-19 | Gateway port: **8081** (local dev) | Port 8080 conflicts with Dioxus dev server. |


## Errors Encountered

| Error | Cause | Resolution |
|:------|:------|:-----------|
| `M0219: implicitly transient` | Motoko persistence strictness for actor class fields | Switched to Rust backend |
| `M0220: actor should be persistent` | Motoko singleton actors require stable state | Switched to Rust backend |
| `readable-stream.slice undefined` | `ic-websocket-js@0.2.2` Node.js polyfill failures | Upgraded to v0.5.0 |
| `Identity is required` | `ic-websocket-js@0.5.0` API change | Added `generateRandomIdentity()` |
| `Invalid record {text:text}` | Frontend sent string, backend expected `AppMessage` record | Changed `send("START")` → `send({ text: "START" })` |
| Gateway connection failed | Port 8080 occupied by Dioxus | Restarted gateway on 8081 |
