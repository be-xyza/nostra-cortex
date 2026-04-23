# Initiative 078 Implementation Checklist

This checklist turns Phase D into an execution order that stays Nostra-native, keeps the existing graph substrate intact, and defers heavier GraphRAG work until the contract and ingestion layers are stable.

## Current State Snapshot

- [x] `shared/standards/knowledge_graphs/schema_registry.toml` exists as a draft registry for the knowledge-graphs contract family.
- [x] `shared/standards/knowledge_graphs/ontology_manifest.schema.json` exists.
- [x] `shared/ontology/core_ontology_v1.json` exists with the current core vocabulary.
- [x] `shared/standards/knowledge_graphs/knowledge_bundle.schema.json` exists as a draft bundle contract.
- [x] `shared/standards/knowledge_graphs/triple_query_request.schema.json` and `triple_query_response.schema.json` exist as draft query contracts.
- [ ] These draft artifacts are not yet enough to call M21-M24 complete; they still need validation, examples, runtime hookup, and governance hardening.

## Working Order

1. `M21` establishes schema discovery without moving files.
2. `M22` defines the minimal ontology and extension rules.
3. `M23` defines the portable bundle manifest.
4. `M24` exposes read-only graph query semantics.
5. `M25` layers graph RAG on top of the stable contract and ingestion path.

## M21. Schema Registry Consolidation

### Goal
Create a schema registry index over the existing `shared/standards/` layout without changing consumer-visible paths.

### Checklist

- [ ] Inventory every JSON schema under `shared/standards/`.
- [ ] Record schema purpose, owner, current path, and known consumers.
- [ ] Classify schemas into operational, governance, testing, and graph-related groups.
- [x] Add `schema_registry.toml` as an index over existing paths.
- [ ] Define a stable lookup convention for schema IDs and aliases.
- [ ] Keep current file locations unchanged in the first pass.
- [ ] Document any schema that should remain legacy-only.
- [ ] Add a resolver test that confirms every known consumer still finds its schema.
- [ ] Add a drift check that fails if the registry and file tree disagree.

### Exit Criteria

- [ ] Every existing schema is discoverable from the registry.
- [ ] No consumer breaks because of path movement.
- [ ] The registry explains, but does not yet reorganize, the tree.

## M22. Minimal Domain Ontology

### Goal
Define a small, Nostra-native ontology layer for graph semantics and extension control.

### Checklist

- [x] Choose the ontology encoding approach: current baseline is a custom JSON manifest, with JSON-LD still under evaluation as an interoperability layer.
- [x] Define core vocabulary for `Space`, `Contribution`, `Relation`, `Capability`, and `ProvenanceScope`.
- [ ] Define allowed relationship types between those core concepts.
- [ ] Define property constraints for identity, labels, provenance, and scope.
- [x] Define versioning and compatibility rules for ontology changes.
- [x] Define extension rules for Space-local vocabulary additions.
- [ ] Define what counts as a breaking ontology change.
- [x] Document how ontology terms map to existing constitutional types.
- [x] Add examples for at least two Space vocabularies.

### Exit Criteria

- [ ] The ontology can express the core graph vocabulary needed by `078`.
- [ ] Space-local extensions are possible without breaking global terms.
- [ ] The format is minimal enough to ship in the current stage.

## M23. Space Knowledge Bundle

### Goal
Specify a portable bundle manifest for graph state, provenance, and retrieval policy.

### Checklist

- [x] Define the `KnowledgeBundle` manifest structure.
- [ ] Include references for ontology, graph snapshot, embeddings manifest, provenance root, and retrieval policy.
- [ ] Specify bundle identity, version, and compatibility metadata.
- [ ] Decide which fields are required versus optional.
- [ ] Specify how the bundle references existing artifacts instead of embedding everything inline.
- [ ] Keep JSON as the default manifest format.
- [ ] Reserve MessagePack for bulk export/import and snapshot transport.
- [ ] Define import/export round-trip behavior.
- [x] Define a sample bundle for one real Space.
- [ ] Add bundle validation rules for missing references and incompatible versions.

### Exit Criteria

- [ ] A bundle can be exported and re-imported without losing meaning.
- [ ] The bundle format is readable enough for governance review.
- [ ] The manifest is portable across environments.

## M24. Triple Query Facade

### Goal
Expose a read-only S-P-O query surface over `Contribution`/`Relation` data with named graph scopes.

### Checklist

- [x] Define request and response contracts for triple-style queries.
- [ ] Support subject, predicate, object, and graph-scope filtering.
- [ ] Support named graph scopes for `System`, `Actor`, and `Agent`.
- [ ] Make JSON the default interactive query format.
- [ ] Reserve MessagePack for bulk transfer paths only.
- [ ] Define a read-only translation layer from current graph substrate to triple-shaped results.
- [ ] Define deterministic ordering rules for query results.
- [ ] Include provenance fields in query responses where available.
- [ ] Add tests for empty, exact-match, and multi-hop-style queries.
- [ ] Add tests for scope filtering and provenance isolation.

### Exit Criteria

- [ ] Agents can traverse graph data through a stable facade.
- [ ] Query results are deterministic for the same input.
- [ ] The interface does not require a TrustGraph-style runtime port.

## M25. Graph RAG

### Goal
Build graph-grounded retrieval only after the ontology, bundle, and query layers are stable.

### Preconditions

- [ ] M21 is complete.
- [ ] M22 is complete.
- [ ] M23 is complete.
- [ ] M24 is complete.
- [ ] Ingestion quality gates from `037`, `042`, and `051` are green enough for controlled evaluation.

### Checklist

- [ ] Define the retrieval pipeline stages: embed, find, extract, reason.
- [ ] Define which stages consume graph triples versus embeddings versus provenance.
- [ ] Reuse hybrid scoring concepts from `042` instead of inventing a separate ranking stack.
- [ ] Add retrieval-policy inputs from `M23` to the pipeline.
- [ ] Add citation output so answers can show source lineage.
- [ ] Start with a curated corpus and known-answer evaluation set.
- [ ] Benchmark against pure vector retrieval.
- [ ] Benchmark against the current local knowledge-engine path from `037`.
- [ ] Add failure modes for low-confidence or incomplete graph coverage.
- [ ] Keep the first version bounded to controlled corpora and operator review.

### Exit Criteria

- [ ] Graph RAG improves answer quality on multi-hop questions.
- [ ] Retrieval remains explainable and lineage-aware.
- [ ] The pipeline is still compatible with Nostra-native contracts.

## Cross-Cutting Enrichments

- [ ] Add provenance scopes to all new graph contracts.
- [ ] Add compatibility notes for every breaking graph-related contract.
- [ ] Keep `shared/specs.md` as the constitutional source of truth.
- [ ] Keep ontology and query additions additive until a steward approves a breaking change.
- [ ] Add a decision log entry whenever a new graph contract is finalized.
- [ ] Link each new artifact to the relevant initiative reference in `078`, `037`, `042`, `051`, `130`, or `136`.

## Suggested First Sprint

- [ ] Complete the schema inventory and expand the registry beyond the current knowledge-graphs slice.
- [ ] Validate the current ontology vocabulary and extension rules against at least two realistic Space examples.
- [ ] Draft the bundle manifest shape with required fields.
- [ ] Decide the query facade request/response shape.
