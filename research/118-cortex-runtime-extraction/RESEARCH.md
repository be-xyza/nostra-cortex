---
id: '118'
name: cortex-runtime-extraction
title: "Initiative 118 \u2014 Cortex Runtime Extraction"
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-11'
updated: '2026-02-14'
---

# Initiative 118 — Cortex Runtime Extraction

| Field | Value |
|---|---|
| **Status** | active |
| **Layer** | Architectural |
| **Primary Steward** | Systems Steward |
| **Domain** | Agents & Execution |
| **Created** | 2026-02-11 |

## Summary

Extract the implicit sovereign execution runtime from Cortex Desktop into a
host-agnostic `cortex-runtime` library crate. The desktop application becomes a
thin host adapter consuming the runtime via trait interfaces.

This is a **sovereignty migration**, not a refactor.

## Problem Statement

Cortex Desktop currently embeds the entire execution runtime — events, policy,
governance, workflows, agents, CRDT collaboration, streaming, and UX evaluation —
inside a single desktop process. Evidence of host coupling:

- `gateway/server.rs` is a 15,975-line monolith
- `std::env` used in 24 of 41 service files (58%)
- `std::fs` used in 16 of 41 service files (39%)
- `std::process::Command` used in 10 files
- `ic-agent` directly imported in 3 services
- `Utc::now()` / `SystemTime::now()` / `Instant::now()` across 36 call sites in 18 services (plus additional calls in `gateway/server.rs`)
- Global `OnceLock<LocalGateway>` with 22+ mutex lock sites

This violates constitutional standards:
- §1.1 Hexagonal Architecture
- §1.1 WASM-First
- §1.2 Event-Driven
- §1.4 Portability
- §1.5 Durable Execution

## Approach

**Constrained Extraction** — declare runtime invariants before extraction begins,
then extract toward those constraints incrementally.

The Runtime Purity Contract is the constitutional firewall. The CI dependency
firewall enforces it mechanically.

## Key Architectural Decisions

See [DECISIONS.md](./DECISIONS.md).

## Dependencies

| Direction | Initiative | Relationship |
|---|---|---|
| Consumes | 013 (Workflow Engine) | Workflow logic extraction |
| Consumes | 067 (Unified Protocol) | Trinity Stack architecture |
| Consumes | 074 (UI Substrate) | Renderer separation |
| Affects | 106–116 | All active Cortex initiatives |
| Enables | Future Web Host | Shared runtime for PWA/browser target |
| Enables | Future Server Host | Headless runtime for DAO/server nodes |

### Cross-Initiative Dependencies (Phase-Gated)

| Initiative | Dependency Type | Phase | Notes |
|---|---|---|---|
| 096 — Offline Sync | Decision input | 3 | If offline governance needed → forces local pre-validation |
| 013 — Workflow Engine | Type surface | 3 | Workflow type extraction must sync with 013 state |
| 085 — File Infrastructure | Adapter evolution | 2+ | StorageAdapter v1 will expand when 085 reaches implementation |
| 097 — Alignment Remediation | Complementary | — | Completed; provides governance infrastructure for 118 |
| 119 — Nostra Commons | Gated on 118 | L1+ | Commons enforcement requires SIQS (118 Layer 1) |
| 091 — Agent Systems | Gated on 118 | L3 | Benchmarking integration requires GSMS (118 Layer 3) |

## Cross-Initiative Note

097 operates at the governance alignment layer. Research 118 (Cortex Runtime Extraction)
operates at the architectural extraction layer. These are complementary, not overlapping.
097's completed stewardship metadata and resolution matrix infrastructure supports 118's
constitutional crate boundaries.
