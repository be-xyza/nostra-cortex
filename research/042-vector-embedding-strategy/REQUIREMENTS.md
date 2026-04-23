---
id: '042'
name: vector-embedding-strategy
title: 'Requirements: Vector Database & Embedding Model Strategy'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Vector Database & Embedding Model Strategy

This document specifies the technical and functional requirements for the vector database and embedding model strategy.

---

## 1. Functional Requirements

### FR-042-01: Vector Storage
- **MUST** support storage of vectors with configurable dimensions (default: 384).
- **MUST** persist vectors across canister upgrades (stable memory).
- **MUST** support metadata association with each vector (entity ID, type, timestamp).
- **SHOULD** support batch insert operations (up to 100 vectors).

### FR-042-02: Similarity Search
- **MUST** support k-Nearest Neighbor (k-NN) queries.
- **MUST** support configurable `k` value (1-100).
- **MUST** use cosine similarity as the default distance metric.
- **SHOULD** return similarity score with each result.

### FR-042-03: Access Control
- **MUST** integrate with Nostra Space permissions.
- **MUST** support SuperUser and Admin roles (ELNA RBAC).
- **SHOULD** support per-Space vector namespaces.

### FR-042-04: Embedding Generation
- **MUST** abstract embedding generation behind a pluggable interface.
- **MUST** support OpenAI `text-embedding-3-small` as initial provider.
- **SHOULD** support local model (`all-MiniLM-L6-v2`) for offline use.
- **SHOULD** support ICP-native models when available (Ignition).

### FR-042-05: Hybrid Search
- **MUST** support combined keyword + semantic search.
- **MUST** implement Reciprocal Rank Fusion (RRF) for result merging.
- **SHOULD** allow users to disable semantic search (keyword-only mode).

### FR-042-06: Synchronization
- **MUST** keep vector index synchronized with Knowledge Graph entities.
- **MUST** update vectors when entities are modified.
- **MUST** delete vectors when entities are deleted.

---

## 2. Non-Functional Requirements

### NFR-042-01: Performance
| Metric | Requirement |
|:-------|:------------|
| Insert Latency | < 100ms per vector |
| Query Latency (k=10) | < 200ms |
| Throughput | ≥ 10 inserts/sec |
| Index Size | Support up to 100K vectors per canister |

### NFR-042-02: Scalability
- **MUST** define clear scaling threshold (alert at 80K vectors).
- **MUST** support hybrid architecture fallback (off-chain index).
- **SHOULD** support multi-canister sharding (future consideration).

### NFR-042-03: Cost Efficiency
| Operation | Target Cycles |
|:----------|:--------------|
| Vector Insert (384-dim) | < 2M cycles |
| k-NN Query (k=10) | < 10M cycles |
| Monthly (100K vectors, 1K queries/day) | < 500B cycles |

### NFR-042-04: Reliability
- **MUST** persist vectors through canister upgrades without data loss.
- **MUST** handle embedding provider failures gracefully (retry, queue).
- **SHOULD** implement health check endpoint for vector service.

### NFR-042-05: Security
- **MUST** validate caller permissions before vector operations.
- **MUST** sanitize input text before embedding generation.
- **SHOULD NOT** log raw text content (only vector IDs).

### NFR-042-06: Interoperability
- **MUST** define embedding model version in stored metadata.
- **MUST** support vector migration when standard model changes.
- **SHOULD** export vectors in standard format (JSON, NDJSON).

---

## 3. Data Requirements

### DR-042-01: Vector Schema
```candid
type Vector = record {
    id: text;               // Unique identifier (Entity ID)
    embedding: vec float32; // The vector (384 dimensions standard)
    dimension: nat32;       // Vector dimension
    model_id: text;         // Embedding model identifier
    created_at: nat64;      // Timestamp (nanoseconds)
    metadata: opt text;     // Optional JSON metadata
};
```

### DR-042-02: Search Query Schema
```candid
type SearchQuery = record {
    query_vector: vec float32; // Query embedding
    k: nat32;                  // Number of results
    namespace: opt text;       // Optional Space/namespace filter
    min_similarity: opt float32; // Optional threshold
};
```

### DR-042-03: Search Result Schema
```candid
type SearchResult = record {
    id: text;           // Matched entity ID
    similarity: float32; // Cosine similarity score
    metadata: opt text;  // Vector metadata
};
```

---

## 4. Integration Requirements

### IR-042-01: Backend Integration
- Vector service callable from `nostra-worker` via inter-canister calls.
- CRUD operations exposed: `insert_vector`, `query_similar`, `delete_vector`, `update_vector`.

### IR-042-02: Frontend Integration
- Cortex search bar triggers hybrid search.
- Results display both keyword and semantic matches.
- UI indicates search mode (keyword/semantic/hybrid).

### IR-042-03: Knowledge Engine Integration
- [037-nostra-knowledge-engine](../037-nostra-knowledge-engine/RESEARCH.md) generates embeddings during ingestion.
- Embedding generation is optional step in "The Loom" pipeline.

### IR-042-04: Schema Standards Integration
- `nostra.vector` domain defined in [040-nostra-schema-standards](../040-nostra-schema-standards/RESEARCH.md).
- Standard model, dimension, and distance metric codified.

---

## 5. Compatibility Requirements

### CR-042-01: ICP Constraints
- Vectors stored in stable memory (500GB max per canister).
- HNSW index rebuilt from stable memory on canister upgrade.
- Wasm memory limit respected (heap used only for hot queries).

### CR-042-02: ELNA-Vector-DB Compatibility
- Use ELNA's `insert_embedding`, `search`, `delete` APIs.
- Respect ELNA's RBAC for admin/user operations.

### CR-042-03: Future Compatibility
- Design for Ignition (on-chain embeddings) integration.
- Support model versioning and migration.

---

## 6. Constraints

| Constraint | Description |
|:-----------|:------------|
| **C-01** | Embedding generation requires external API (until Ignition). |
| **C-02** | Vector dimension must be consistent across all vectors in namespace. |
| **C-03** | ELNA canister is single-tenant; multi-Space requires multiple canisters or namespacing. |
| **C-04** | Cross-canister queries add ~50-100ms latency. |
