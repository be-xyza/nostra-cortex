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
- **A ratified candidate explicit domain ontology layer** — `shared/ontology/core_ontology_v1.json` now has earned-freeze evidence, JSON-LD parity, and comparator outcomes recorded
- **A ratified candidate knowledge bundle format** — `knowledge_bundle.schema.json` now has validated portability, negative-fixture, round-trip, and freeze-readiness coverage
- **A ratified candidate graph query facade contract** — triple query request/response schemas now have both fixture-backed and runtime-backed internal `cortex-eudaemon` validation, while public surfacing remains intentionally deferred

### Proposed Three-Tier Commons Model

| Tier | Purpose | Format | Location | Status |
|------|---------|--------|----------|--------|
| **Constitutional Types** | Invariant data model | Motoko/Candid | `shared/specs.md`, `.did` files | ✅ Exists |
| **JSON Schemas** | Operational contracts for artifacts | JSON Schema | `shared/standards/` + registry index | ✅ Root registry + delegated package registry now exist |
| **Domain Ontology** | Knowledge modeling vocabulary | Minimal JSON-LD subset or custom | `shared/ontology/` | ✅ Ratified v1 candidate with earned-freeze evidence |

### Current Artifact Snapshot

Phase D already has draft commons artifacts on disk:
- `shared/standards/standards_registry.toml`
- `shared/standards/schema_guided_extraction_context.json`
- `shared/standards/knowledge_graphs/schema_registry.toml`
- `shared/standards/knowledge_graphs/ontology_manifest.schema.json`
- `shared/standards/knowledge_graphs/knowledge_bundle.schema.json`
- `shared/standards/knowledge_graphs/triple_query_request.schema.json`
- `shared/standards/knowledge_graphs/triple_query_response.schema.json`
- `shared/standards/knowledge_graphs/explore_topology_view.schema.json`
- `shared/ontology/core_ontology_v1.json`
- `shared/ontology/freeze_readiness_report.json`

The remaining Phase D work is therefore not "create from zero," but "validate, narrow, connect, and govern the draft contracts that now exist."

Phase D also now has an explicit earned-freeze validation lane:
- `shared/ontology/authoring_guidelines.md`
- `shared/ontology/earned_freeze_validation.md`
- `shared/ontology/freeze_readiness_report.json`
- `shared/ontology/reference_alignment_matrix.json`
- `shared/ontology/shacl_core_validation_checklist.json`
- `shared/standards/knowledge_graphs/sparql_query_facade_matrix.json`
- `scripts/validate_knowledge_graph_contracts.py`
- `scripts/validate_standards_registry.py`

### Phase E: Freeze, Consolidate, and Pilot Graph Retrieval

Phase E converts the Phase D candidate contracts into a ratified baseline and consumes them in one controlled internal pilot:

1. `M21` is completed through a root standards registry plus delegated knowledge-graphs registry coverage.
2. `M22`–`M24` are frozen through an explicit freeze-readiness report rather than draft incumbency.
3. `M24` is extended from fixture-only semantics to a runtime-backed `GlobalEvent` projection path in `cortex-eudaemon`.
4. `M25` begins as a bounded graph-retrieval pilot with graph-only, vector-only, and hybrid evaluation.
5. `136` receives a derived topology read-model contract rather than a new graph source of truth.

### Phase F: Operationalize the Ratified Graph Baseline

Phase F turns the ratified commons contracts and bounded pilot into an operational internal capability without widening the public surface:

1. Restore a separate default-feature `cortex-eudaemon` build gate alongside the slim `knowledge-graph-tests` contract lane.
2. Promote the graph pilot from test-only helpers to an internal service layer in `cortex-eudaemon`:
   - `knowledge_graph_service.rs`
   - `knowledge_graph_query.rs`
   - `knowledge_graph_retrieval.rs`
   - `knowledge_graph_topology.rs`
3. Add an internal runner and evidence path:
   - `scripts/run_knowledge_graph_pilot.sh`
   - `logs/knowledge/graph_pilot_benchmark_latest.json`
   - `logs/knowledge/graph_pilot_comparison_latest.json`
   - `logs/knowledge/graph_pilot_topology_latest.json`
4. Keep all graph, topology, and retrieval outputs internal and derived:
   - no public HTTP/WebSocket graph API
   - no new graph source of truth for `136`
   - no mutation semantics
5. Use the Phase F evidence lane to compare graph-only, vector-only, and hybrid retrieval against the current `037` benchmark baseline before any broader rollout decision.

### Milestones

#### M21: Schema Registry Consolidation
- [x] Audit all existing JSON schemas in `shared/standards/`
- [x] Expand the current `schema_registry.toml` beyond the knowledge-graphs slice into a reliable cross-standards discovery index
- [x] Implement **Schema-Guided Extraction Context** (DEC-051-003) for mapping artifacts to schemas
- [x] Keep existing schema file locations stable until consumers are migrated intentionally
- [x] Verify all schema consumers still resolve correctly

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
- [x] Evaluate integration with existing `GlobalEvent` traversal patterns

#### M25: Graph RAG Implementation
- [x] Gate implementation on successful M21-M24 validation
- [ ] Implement multi-hop semantic traversal pipeline (Embed -> Find -> Extract -> Reason)
- [x] Integrate **Hybrid Similarity Scoring** (RRF + Schema Planning) from `042` in the bounded pilot
- [x] Prototype in `cortex-eudaemon` for graph-grounded agent chat
- [x] Benchmark against pure-vector RAG performance
- [x] Start with curated corpora and controlled evaluation, not open-ended general retrieval

### Verification

- Schema registry consolidation: all existing consumers pass with the root registry and delegated package registry in place
- Ontology format: validated against research, operations, and adversarial cross-space additive examples; freeze report returns `freeze`
- Bundle spec: round-trip manifest normalization and negative portability/compatibility tests pass
- Query interface: an internal adapter can answer structured knowledge questions via S-P-O over both fixture-backed and runtime-backed graph projections; public surfacing remains deferred
- Graph retrieval pilot: graph-only, vector-only, and hybrid evaluation exists with citation-bearing output and curated benchmark coverage
- Graph operationalization: an internal runner emits benchmark, comparison, and topology artifacts under `logs/knowledge/` without widening the network surface
- Explore handoff: a derived topology read-model exists for `136` without introducing a new graph source of truth

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
- [Standards Registry Validator](../../scripts/validate_standards_registry.py) — Cross-standards registry and delegated discovery gate
- [Freeze Readiness Report](./FREEZE_READINESS.md) — Human-readable ratification summary for M22-M24
- [Phase E Consumer Handoffs](./PHASE_E_CONSUMER_HANDOFFS.md) — Implementation note for `037`, `042`, `051`, and `136`
- [082-Graphiti Integration](../082-graphiti-integration-analysis/) — Related graph integration analysis
- [130-Space Capability Graph](../130-space-capability-graph-governance/) — Related capability graph governance

---

## Prior Decisions (Phases A–C)

See [DECISIONS.md](./DECISIONS.md) for DEC-001 through DEC-015 covering the `motoko-graph` runtime evaluation and the new Phase D commons strategy (M1–M24).
