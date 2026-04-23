---
id: '011'
name: tech-stack-video-streaming
title: 'Decisions Log: Video & Audio'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions Log: Video & Audio

**Context**: Strategic technical decisions for media handling on ICP.

## DEC-001: Hybrid Storage/Transport
*   **Context**: Storage vs Live Communication.
*   **Decision**:
    *   **Archival/Artifacts**: Use **Canister-per-Video** (Sovereign Storage) on ICP.
    *   **Live Meetings**: Use **WebRTC** (P2P) with ICP Signaling.
*   **Status**: DECIDED

## DEC-002: Wasm Witness for Attribution
*   **Context**: How to track "minutes watched" without frequent expensive on-chain updates?
*   **Decision**: Use **Client-Side Wasm Reporting** ("Smart Witness").
    *   Client aggregates stats in secure memory.
    *   Submits one signed "Batch Report" at session end.
*   **Status**: PROPOSED
*   **See Also**: [RESEARCH.md](./RESEARCH.md#5-usage-attribution--fee-capabilities)

## DEC-003: Graph-Based Royalty (KIP Compliance)
*   **Context**: How to reward content creators?
*   **Decision**: Use **Citation-Based Flow** tracked on the Knowledge Graph.
    *   **Constraint**: The `Link` (Citation) and `Node` (Video) must be **KIP Compliant** (See `014`) to allow automatic royalty calculation agents to traverse the graph.
    *   If Node B links to Node A, Node A shares in Node B's value.
*   **Status**: DECIDED
