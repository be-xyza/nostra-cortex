---
id: '041'
name: nostra-vector-store
title: 'Decisions: Nostra Vector Store (041)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-07'
---

# Decisions: Nostra Vector Store (041)

## DEC-001: Vector Provider Selection
- **Decision**: Adopt `elna-vector-db` as the primary on-chain vector storage solution.
- **Rationale**: Best alignment with ICP's decentralization and stable memory capabilities.
- **Status**: Research complete; integration in progress.

## DEC-002: Vector Service Architecture
- **Decision**: Implement a `VectorService` in Rust that abstracts the embedding generation and storage calls.
- **Rationale**: Decouples the application logic from specific providers (Ollama vs OpenAI) and vector backends (Elna vs Mock).
- **Status**: Implemented in `nostra-worker`.

## DEC-003: Hybrid Search Strategy
- **Decision**: Use a fusion of keyword-based (Graph) and semantic-based (Vector) search scores.
- **Rationale**: Combines the precision of exact matches with the recall of semantic similarity.
- **Status**: Conceptualized in 041/042.

## DEC-004: Embedding Lineage Metadata & Perspective Filters
- **Decision**: Persist embedding lineage metadata (`source_version_id`, `model_id`, `perspective_scope`) and support query filters by `perspective_scope` and `produced_by_agent`.
- **Rationale**: Enables auditability, re-embedding control, and perspective-aware search behavior.
- **Status**: APPROVED

## DEC-005: ELNA-Gated Backend with Local Shadow Compare
- **Date**: 2026-02-07
- **Decision**: Introduce runtime backend switch (`VECTOR_BACKEND=mock|elna`) with local shadow parity logging (`NOSTRA_VECTOR_SHADOW_COMPARE`).
- **Rationale**: Supports safe cutover to ELNA while keeping deterministic local fallback and retrieval-parity observability.
- **Status**: Implemented in `nostra/worker/src/vector_service.rs`.

## DEC-006: Metadata-Carrying Vector Index Records
- **Date**: 2026-02-07
- **Decision**: Enrich indexed records with `space_id`, `source_ref`, `source_type`, `tags`, `timestamp_ms`, and `cei_metadata` to support filterable retrieval and provenance-forward responses.
- **Rationale**: Enables lineage-aware search and grounded UX without waiting for full canister-side metadata indexing.
- **Status**: Implemented in worker retrieval/index contracts.
