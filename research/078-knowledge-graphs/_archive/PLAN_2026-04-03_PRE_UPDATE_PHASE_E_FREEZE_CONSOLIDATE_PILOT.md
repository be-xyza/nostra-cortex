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
updated: '2026-04-03'
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
- **Operational JSON Schemas** scattered across `shared/standards/` subdirectories — a knowledge-graphs registry skeleton now exists, but broader discovery and consumer coverage are still incomplete
- **A draft explicit domain ontology layer** — `shared/ontology/core_ontology_v1.json` and its schema exist, but interoperability format decisions and example vocabularies are still open
- **A draft knowledge bundle format** — `knowledge_bundle.schema.json` exists and now has validated portability, negative-fixture, and round-trip coverage, but broader discovery/index closure is still open
- **A draft graph query facade contract** — triple query request/response schemas now have an internal `cortex-eudaemon` adapter spike and executable fixture matrix, but public surfacing remains intentionally deferred

### Proposed Three-Tier Commons Model

| Tier | Purpose | Format | Location | Status |
|------|---------|--------|----------|--------|
| **Constitutional Types** | Invariant data model | Motoko/Candid | `shared/specs.md`, `.did` files | ✅ Exists |
| **JSON Schemas** | Operational contracts for artifacts | JSON Schema | `shared/standards/` + registry index | ⚠️ Partial registry exists; broader inventory still open |
| **Domain Ontology** | Knowledge modeling vocabulary | Minimal JSON-LD subset or custom | `shared/ontology/` | ⚠️ Draft core manifest exists; examples and interop still open |

### Current Artifact Snapshot

Phase D already has draft commons artifacts on disk:
- `shared/standards/knowledge_graphs/schema_registry.toml`
- `shared/standards/knowledge_graphs/ontology_manifest.schema.json`
- `shared/standards/knowledge_graphs/knowledge_bundle.schema.json`
- `shared/standards/knowledge_graphs/triple_query_request.schema.json`
- `shared/standards/knowledge_graphs/triple_query_response.schema.json`
- `shared/ontology/core_ontology_v1.json`

The remaining Phase D work is therefore not "create from zero," but "validate, narrow, connect, and govern the draft contracts that now exist."

Phase D also now has an explicit earned-freeze validation lane:
- `shared/ontology/authoring_guidelines.md`
- `shared/ontology/earned_freeze_validation.md`
- `shared/ontology/reference_alignment_matrix.json`
- `shared/ontology/shacl_core_validation_checklist.json`
- `shared/standards/knowledge_graphs/sparql_query_facade_matrix.json`
- `scripts/validate_knowledge_graph_contracts.py`

### Milestones

#### M21: Schema Registry Consolidation
- [ ] Audit all existing JSON schemas in `shared/standards/`
- [ ] Expand the current `schema_registry.toml` beyond the knowledge-graphs slice into a reliable cross-standards discovery index
- [ ] Implement **Schema-Guided Extraction Context** (DEC-051-003) for mapping artifacts to schemas
- [ ] Keep existing schema file locations stable until consumers are migrated intentionally
- [ ] Verify all schema consumers still resolve correctly

#### M22: Minimal Domain Ontology
- [x] Confirm the current custom ontology manifest is the active baseline, and decide how JSON-LD relates to it as an interoperability layer rather than a replacement-by-default
- [x] Refine the existing `shared/ontology/` core vocabulary (`Space`, `Contribution`, `Capability`, `Relation`, `ProvenanceScope`)
- [x] Define entity types, relationship types, property constraints, and extension rules
- [x] Document ontology authoring guidelines and compatibility rules

#### M23: Knowledge Bundle Specification (**Space Knowledge Bundle**)
- [x] Refine the draft `KnowledgeBundle` contract inspired by TrustGraph **Context Cores**
- [x] Define bundle contents: ontology reference, graph snapshot reference, embeddings manifest, provenance root, and retrieval policy
- [x] Include **Retrieval Policy** definitions (traversal rules, authority weights, freshness, allowed predicates)
- [x] Prefer JSON manifest semantics for normal bundle contracts
- [x] Reserve MessagePack for bulk export/import paths
- [x] Add to `shared/specs.md`

#### M24: Triple Query Interface Prototype
- [x] Refine the draft S-P-O query facade contracts for the `Contribution`/`Relation` model
- [x] Support **Named Graph Scoping** (System vs Actor vs Agent scopes via `GlobalEvent`)
- [x] Make JSON the default query protocol for interactive calls
- [x] Use MessagePack only for bulk triple transfer and bundle export/import
- [x] Prototype in Cortex worker or agent context
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
- Ontology format: validated against research, operations, and adversarial cross-space additive examples
- Bundle spec: round-trip manifest normalization and negative portability/compatibility tests pass
- Query interface: an internal adapter can answer structured knowledge questions via S-P-O over a fixture-backed representation of the current graph substrate; public surfacing remains deferred
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
- [Core Ontology](../../shared/ontology/core_ontology_v1.json) — Current draft ontology manifest
- [Ontology Home](../../shared/ontology/README.md) — Ontology contract and compatibility rules
- [Ontology Authoring Guidelines](../../shared/ontology/authoring_guidelines.md) — Core vs Space-local ontology authoring rules
- [Earned Freeze Validation](../../shared/ontology/earned_freeze_validation.md) — Reference-backed v1 freeze criteria
- [Knowledge Graph Contract Validator](../../scripts/validate_knowledge_graph_contracts.py) — Semantic validation and fixture matrix gate
- [082-Graphiti Integration](../082-graphiti-integration-analysis/) — Related graph integration analysis
- [130-Space Capability Graph](../130-space-capability-graph-governance/) — Related capability graph governance

---

## Prior Decisions (Phases A–C)

See [DECISIONS.md](./DECISIONS.md) for DEC-001 through DEC-015 covering the `motoko-graph` runtime evaluation and the new Phase D commons strategy (M1–M24).
