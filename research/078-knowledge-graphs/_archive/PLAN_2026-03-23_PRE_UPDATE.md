---
id: 078
name: knowledge-graphs
title: 'Knowledge Graphs — Schema, Ontology & Graph Strategy'
type: general
project: nostra
status: active
authors:
- User
tags: [knowledge-graphs, ontology, schema, context-cores, graph-query]
created: '2026-02-05'
updated: '2026-03-23'
---

# Knowledge Graphs — Schema, Ontology & Graph Strategy

## Background

Phases A–C (M1–M20) evaluated `motoko-graph` as a graph runtime candidate, concluding with a **Hold Deferred / Watch-First** posture. Phase D shifts focus from runtime evaluation to **knowledge modeling strategy**: how Nostra/Cortex organizes schemas, ontologies, and graph query interfaces in the commons layer.

This phase was triggered by the [TrustGraph reference intake](../reference/analysis/trustgraph.md), which surfaced the **Context Core** pattern and a clear **schema vs ontology duality** architecture that directly addresses gaps in Nostra's current `shared/` commons.

---

## Phase D: Schema/Ontology Strategy & Knowledge Modeling (M21+)

### Problem Statement

Nostra currently has:
- **Constitutional Types** in Motoko/Candid (`shared/specs.md`, `.did` files) — well-defined
- **JSON Schemas** scattered across `shared/standards/` subdirectories — functional but unregistered
- **No ontology layer** — no formal entity/relationship vocabulary for knowledge modeling
- **No knowledge bundle format** — no way to package and transport graph + embeddings + provenance
- **No triple query interface** — agents cannot traverse `Contribution`/`Relation` data via S-P-O patterns

### Proposed Three-Tier Commons Model

| Tier | Purpose | Format | Location | Status |
|------|---------|--------|----------|--------|
| **Constitutional Types** | Invariant data model | Motoko/Candid | `shared/specs.md`, `.did` files | ✅ Exists |
| **JSON Schemas** | Operational contracts for artifacts | JSON Schema | `shared/standards/schemas/` (consolidated) | ⚠️ Exists but scattered |
| **Domain Ontology** | Knowledge modeling vocabulary | JSON-LD subset or custom | `shared/ontology/` (new) | ❌ Not yet created |

### Milestones

#### M21: Schema Registry Consolidation
- [ ] Audit all existing JSON schemas in `shared/standards/`
- [ ] Create `shared/standards/schemas/` with `schema_registry.toml` index
- [ ] Implement **Schema-Guided Extraction Context** (DEC-051-003) for mapping Artifacts to Schemas
- [ ] Verify all schema consumers still resolve correctly

#### M22: Minimal Domain Ontology
- [ ] Design minimal ontology format (JSON-LD subset vs custom)
- [ ] Create `shared/ontology/` with core vocabulary (Space, Contribution, Capability, Relation)
- [ ] Define entity types, relationship types, and property constraints
- [ ] Document ontology authoring guidelines

#### M23: Knowledge Bundle Specification (**Space Knowledge Bundle**)
- [ ] Design `KnowledgeBundle` type inspired by TrustGraph **Context Cores**
- [ ] Define bundle contents: ontology + graph data + embeddings manifest + event provenance
- [ ] Include **Retrieval Policy** definitions (traversal rules, authority weights, freshness)
- [ ] Define bundle versioning and MessagePack-based export/import protocols
- [ ] Add to `shared/specs.md`

#### M24: Triple Query Interface Prototype
- [ ] Design S-P-O query interface for `Contribution`/`Relation` model
- [ ] Support **Named Graph Scoping** (System vs Actor vs Agent graphs via `GlobalEvent`)
- [ ] Protocol: Use **MessagePack** for high-performance bulk triple transfer (from TrustGraph)
- [ ] Prototype in Cortex worker or agent context
- [ ] Evaluate integration with existing `GlobalEvent` traversal patterns

#### M25: Graph RAG Implementation
- [ ] Implement multi-hop semantic traversal pipeline (Embed -> Find -> Extract -> Reason)
- [ ] Integrate **Hybrid Similarity Scoring** (RRF + Schema Planning) from `042`
- [ ] Prototype in `cortex-eudaemon` for graph-grounded agent chat
- [ ] Benchmark against pure-vector RAG performance

### Verification

- Schema registry consolidation: all existing consumers pass with new paths
- Ontology format: validated against at least 2 Space domain vocabularies
- Bundle spec: round-trip export/import test with sample data
- Query interface: agent can answer structured knowledge questions via S-P-O

---

## Reference Material

- [TrustGraph Analysis](../reference/analysis/trustgraph.md) — Detailed architectural analysis and pattern extraction
- [TrustGraph Repository](../reference/topics/data-knowledge/trustgraph/) — Cloned reference
- [Shared Specs](../../shared/specs.md) — Current constitutional types
- [Knowledge Graph Schemas](../../shared/standards/knowledge_graphs/) — Existing JSON schemas
- [082-Graphiti Integration](../082-graphiti-integration-analysis/) — Related graph integration analysis
- [130-Space Capability Graph](../130-space-capability-graph-governance/) — Related capability graph governance

---

## Prior Decisions (Phases A–C)

See [DECISIONS.md](./DECISIONS.md) for DEC-001 through DEC-013 covering `motoko-graph` runtime evaluation (M1–M20).
