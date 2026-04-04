---
id: '047'
name: temporal-architecture
title: 'Decisions: Temporal Architecture Adoption (047)'
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions: Temporal Architecture Adoption (047)

## DEC-001: Durable Execution Model
- **Decision**: Adopt the state-machine driven durable execution model for all Agentic logic (Cortex).
- **Rationale**: Provides fault-tolerance, determinism, and long-running state management essential for autonomous systems on the IC.
- **Status**: Implemented in `nostra-workflow-core` patterns.

## DEC-002: Visibility Decoupling
- **Decision**: Separate the execution history (Truth) from the visibility index (Query).
- **Rationale**: Prevents bottlenecks in the kernel and enables scalable hybrid search (keyword + vector).
- **Status**: Implemented in standardization of 041/042.

## DEC-003: Interoperability (Nexus RPC)
- **Decision**: Standardize on Nexus RPC for all cross-boundary (canister, cluster, region) service linking.
- **Rationale**: Replaces ad-hoc canister calls with a unified, version-aware, and auditable protocol.
- **Status**: Ratified.

## DEC-004: Eventual Consistency for Logs
- **Decision**: Use a "Visibility Ingest Protocol" where canisters emit async events to an indexer.
- **Rationale**: Offloads non-critical compute (indexing/embedding) from the hot path of execution.
- **Status**: Implemented in 019 Log Registry and 041 Vector Store.

## DEC-005: Temporal SDK Integration Strategy
- **Date**: 2026-01-24
- **Decision**: Adopt **Pure-Rust pattern extraction** over direct SDK dependency.
- **Rationale**:
  1. `temporal-sdk-core` has tokio/gRPC/network dependencies incompatible with ICP WASM (`wasm32-unknown-unknown`).
  2. 013 prototype (503 lines) proves FSM patterns viable in Rust.
  3. `nostra/worker/workflows/benchmark.rs` demonstrates production workflow execution.
  4. 57 documented patterns from 047 RESEARCH.md can be surgically adopted.
- **Implementation Path**:
  1. Extract `DurableActivity`, `WorkflowExecutor` traits into `nostra-workflow-core`.
  2. Use `StableBTreeMap` for ICP-native persistence.
  3. Implement Timer Queue via canister heartbeat.
  4. Implement Transfer Queue (Outbox Pattern) for reliable messaging.
- **Alternatives Rejected**: Waiting for native SDK WASM support (unknown timeline, external dependency risk).
- **Status**: DECIDED
- **References**: 
  - [013 Prototype](../013-nostra-workflow-engine/prototypes/engine-rust/src/lib.rs)
  - [047 RESEARCH.md](./RESEARCH.md) (57 patterns)
  - [047 UI_UX_PATTERNS.md](./UI_UX_PATTERNS.md)

