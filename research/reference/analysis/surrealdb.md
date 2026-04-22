---
id: surrealdb
name: surrealdb
title: SurrealDB
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: nostra
authority_mode: recommendation_only
reference_topics: [data-knowledge, database, graph, memory]
reference_assets:
  - "research/reference/topics/data-knowledge/surrealdb"
evidence_strength: strong
handoff_target:
  - "Systems Steward"
authors:
  - "User"
  - "Codex"
tags: [database, graph, kv, embedded]
stewardship:
  layer: "Architectural"
  primary_steward: "Systems Steward"
  domain: "Data & Knowledge"
created: "2026-02-20"
updated: "2026-02-20"
---

# SurrealDB Reference Analysis

## Overview
SurrealDB is a highly scalable, multi-model database (document, graph, relational, time-series) built in Rust. It functions by layering an advanced execution model and querying surface over fully abstracted Key-Value (KV) storage engines (RocksDB, TiKV, IndexDB).

## Why Intake?
The user requested an architectural analysis of SurrealDB to determine if its approach to memory, graph storage, and temporal versioning could benefit the Nostra ecosystem, specifically in addressing the gaps solved by the newly proposed Git-backed Cortex Memory FS.

## Key Architectural Patterns Identified

### 1. KV Storage Abstraction (`surrealdb-core/src/kvs/`)
- **Pattern:** SurrealDB uses a `Transactable` trait (`api.rs`) that abstracts the implementation details of the underlying storage engine. This permits SurrealDB to run on an embedded memory engine, RocksDB, distributed TiKV, or even WebAssembly IndexDB utilizing the exact same upper-layer database logic.
- **Nostra Value:** Validates the `StorageAdapter` pattern designed in Initiative `118-cortex-runtime-extraction`. This confirms that pushing persistence interfaces to byte-level KV buffers is the correct path for WASM and Internet Computer compatibility in Nostra APIs.

### 2. Graph Edges as Documents (`surrealdb-core/src/doc/edges.rs`)
- **Pattern:** SurrealDB does not treat graph edges as magical links stored in a separate paradigm. Edges are first-class documents marked with a `RecordType::Edge` metadata tag. These edge documents hold explicit `IN` and `OUT` pointers.
- **Nostra Value:** Applies directly to Nostra's Knowledge Graph and `dpub.mo`. By treating relational links as independent semantic artifacts (capable of holding confidence scores, temporal bounds, and authorship), Nostra can natively query edge provenance without polluting the primary nodes.

### 3. Temporal MVCC vs. Agent Episodic Context (`surrealdb-core/src/kvs/version/`)
- **Pattern:** SurrealDB utilizes Multi-Version Concurrency Control (MVCC) by threading a `version: Option<u64>` parameter through every KV interaction. It records historical row versions efficiently.
- **Nostra Value (The Gap):** While technically enabling "time-travel", a raw `u64` MVCC implementation is purely *mechanical*. It answers *when* data changed, but completely lacks the structural semantics needed for Agent Memory (i.e. *intent*, *summarization*, *parallel reasoning branches*). This validates our decision that standard DB temporal capabilities are insufficient for complex cognition.
- **Conclusion:** The Git-backed Cortex Memory FS remains the superior, necessary solution for active Agent Episodic Memory because it natively affords semantic `commitメッセージ` logging and isolated `branches` for experimental simulation.

## Placement
`research/reference/topics/data-knowledge/surrealdb`

## Intent
Analyze KV abstractions and graph relational data structures for potential architectural adoption.

## Initiative Links
- `118-cortex-runtime-extraction` (StorageAdapter validation)

## Pattern Extraction
- **KV Storage Abstraction** (`Transactable` api)
- **Graph Edges as Documents** (`RecordType::Edge`)
- **Temporal MVCC vs Epiodic Context**

## Adoption Decision
**Recommendation:** Extract and emulate isolated patterns.
- Do **not** import or re-write the entirety of SurrealDB into Nostra. It is a massive monolith.
- **Adopt** the `Transactable` abstraction paradigm for Cortex's internal storage layers.
- **Adopt** the Graph `RecordType::Edge` pattern within Nostra Graph (`000-contribution-graph`).
- **Reject** SurrealDB as a replacement for the Cortex Memory FS; continue building the natively Git-backed filesystem for agent context.

## Known Risks
Large monolithic ecosystem and differing language models mean direct dependency is dangerous.

## Suggested Next Experiments
Attempt creating temporal graph links in `dpub.mo`.

## Possible Links To Nostra Platform and Cortex Runtime
- Cortex StorageAdapter
- Nostra Knowledge Graph (`dpub.mo`)

---

## Verification Plan

### Automated Checks
```bash
# Ensure analysis file exists
ls -la research/reference/analysis/surrealdb.md

# Validate index metadata
python3 scripts/check_reference_metadata_v2.py --strict
```
