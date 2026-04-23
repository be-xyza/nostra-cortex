---
id: 078
name: knowledge-graphs
title: 'Knowledge Graphs — Schema, Ontology & Graph Strategy'
type: general
project: nostra
status: active
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Knowledge Modeling Strategy"
authors:
- User
tags: [knowledge-graphs, ontology, schema, context-cores, graph-query]
created: '2026-02-05'
updated: '2026-03-23'
---

# Knowledge Graphs — Schema, Ontology & Graph Strategy

## Background

Phases A–C (M1–M20) evaluated `motoko-graph` as a graph runtime candidate, concluding with a **Hold Deferred / Watch-First** posture. Phase D shifts focus from runtime evaluation to **knowledge modeling strategy**: how Nostra/Cortex organizes schemas, ontologies, and graph query interfaces in the commons layer.

This phase was triggered by the [TrustGraph reference intake](../reference/analysis/trustgraph.md), which surfaced the **Context Core** pattern and a clear **schema vs ontology duality** architecture that directly informs how Nostra should organize commons-layer graph contracts.

---

## Phase D: Schema/Ontology Strategy & Knowledge Modeling (M21+)

### Problem Statement

Nostra currently has:
- **Constitutional Types** in Motoko/Candid (`shared/specs.md`, `.did` files) — well-defined
- **Operational JSON Schemas** scattered across `shared/standards/` subdirectories — functional but not centrally registered
- **No explicit domain ontology layer** — no first-class entity/relationship vocabulary for knowledge modeling
- **No manifest-based knowledge bundle format** — no first-class way to package and transport graph references, embeddings manifests, provenance, and policy
- **No graph query facade** — agents cannot traverse `Contribution`/`Relation` data via an official S-P-O interface

### Proposed Three-Tier Commons Model

| Tier | Purpose | Format | Location | Status |
|------|---------|--------|----------|--------|
| **Constitutional Types** | Invariant data model | Motoko/Candid | `shared/specs.md`, `.did` files | ✅ Exists |
| **JSON Schemas** | Operational contracts for artifacts | JSON Schema | `shared/standards/` + registry index | ⚠️ Exists but scattered |
| **Domain Ontology** | Knowledge modeling vocabulary | Minimal JSON-LD subset or custom | `shared/ontology/` (new) | ❌ Not yet created |

### Milestones

#### M21: Schema Registry Consolidation
- [ ] Audit all existing JSON schemas in `shared/standards/`
- [ ] Create a lightweight `schema_registry.toml` index over existing paths
- [ ] Implement **Schema-Guided Extraction Context** (DEC-051-003) for mapping artifacts to schemas
- [ ] Keep existing schema file locations stable until consumers are migrated intentionally
- [ ] Verify all schema consumers still resolve correctly

#### M22: Minimal Domain Ontology
- [ ] Design a minimal ontology format (JSON-LD subset vs custom)
- [ ] Create `shared/ontology/` with core vocabulary (`Space`, `Contribution`, `Capability`, `Relation`, `ProvenanceScope`)
- [ ] Define entity types, relationship types, property constraints, and extension rules
- [ ] Document ontology authoring guidelines and compatibility rules

#### M23: Knowledge Bundle Specification (**Space Knowledge Bundle**)
- [ ] Design `KnowledgeBundle` type inspired by TrustGraph **Context Cores**
- [ ] Define bundle contents: ontology reference, graph snapshot reference, embeddings manifest, provenance root, and retrieval policy
- [ ] Include **Retrieval Policy** definitions (traversal rules, authority weights, freshness, allowed predicates)
- [ ] Prefer JSON manifest semantics for normal bundle contracts
- [ ] Reserve MessagePack for bulk export/import paths
- [ ] Add to `shared/specs.md`

#### M24: Triple Query Interface Prototype
- [ ] Design an S-P-O query facade for the `Contribution`/`Relation` model
- [ ] Support **Named Graph Scoping** (System vs Actor vs Agent scopes via `GlobalEvent`)
- [ ] Make JSON the default query protocol for interactive calls
- [ ] Use MessagePack only for bulk triple transfer and bundle export/import
- [ ] Prototype in Cortex worker or agent context
- [ ] Evaluate integration with existing `GlobalEvent` traversal patterns

#### M25: Graph RAG Implementation
- [ ] Gate implementation on successful M21-M24 validation
- [ ] Implement multi-hop semantic traversal pipeline (Embed -> Find -> Extract -> Reason)
- [ ] Integrate **Hybrid Similarity Scoring** (RRF + Schema Planning) from `042`
- [ ] Prototype in `cortex-eudaemon` for graph-grounded agent chat
- [ ] Benchmark against pure-vector RAG performance
- [ ] Start with curated corpora and controlled evaluation, not open-ended general retrieval

### Verification

- Schema registry consolidation: all existing consumers pass with the registry index in place
- Ontology format: validated against at least 2 Space domain vocabularies
- Bundle spec: round-trip manifest export/import test with sample data
- Query interface: agent can answer structured knowledge questions via S-P-O over the current graph substrate
- Graph RAG: only begins after ingestion and cross-index quality gates are green

---

## Reference Material

- [TrustGraph Analysis](../reference/analysis/trustgraph.md) — Detailed architectural analysis and pattern extraction
- [TrustGraph Repository](../reference/topics/data-knowledge/trustgraph/) — Cloned reference
- [037 Knowledge Engine](../037-nostra-knowledge-engine/DECISIONS.md) — Retrieval and ask pipeline already in motion
- [042 Embedding Strategy](../042-vector-embedding-strategy/DECISIONS.md) — Vector, hybrid scoring, and cross-indexing direction
- [051 RAG Ingestion Pipeline](../051-rag-ingestion-pipeline/RESEARCH.md) — Extraction and resolution upstream of RAG
- [130 Space Capability Governance](../130-space-capability-graph-governance/PLAN.md) — Governance pattern for overlays and deterministic compilation
- [136 Cortex Explore Graph](../136-cortex-explore-graph/PLAN.md) — UX projection target for graph semantics
- [Shared Specs](../../shared/specs.md) — Current constitutional types
- [Knowledge Graph Schemas](../../shared/standards/knowledge_graphs/) — Existing JSON schemas
- [082-Graphiti Integration](../082-graphiti-integration-analysis/) — Related graph integration analysis
- [130-Space Capability Graph](../130-space-capability-graph-governance/) — Related capability graph governance

---

## Prior Decisions (Phases A–C)

See [DECISIONS.md](./DECISIONS.md) for DEC-001 through DEC-015 covering the `motoko-graph` runtime evaluation and the new Phase D commons strategy (M1–M24).
