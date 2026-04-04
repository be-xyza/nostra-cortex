# Initiative 078 Implementation Checklist

This checklist turns Phase D into an execution order that stays Nostra-native, keeps the existing graph substrate intact, and defers heavier GraphRAG work until the contract and ingestion layers are stable.

## Current State Snapshot

- [x] `shared/standards/knowledge_graphs/schema_registry.toml` exists as a draft registry for the knowledge-graphs contract family.
- [x] `shared/standards/knowledge_graphs/ontology_manifest.schema.json` exists.
- [x] `shared/ontology/core_ontology_v1.json` exists with the current core vocabulary.
- [x] `shared/standards/knowledge_graphs/knowledge_bundle.schema.json` exists as a draft bundle contract.
- [x] `shared/standards/knowledge_graphs/triple_query_request.schema.json` and `triple_query_response.schema.json` exist as draft query contracts.
- [x] These artifacts now have validation, examples, runtime hookup, and governance hardening sufficient for ratified-candidate status.

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

- [x] Inventory every JSON schema under `shared/standards/`.
- [x] Record schema purpose, owner, current path, and known consumers.
- [x] Classify schemas into operational, governance, testing, and graph-related groups.
- [x] Add `schema_registry.toml` as an index over existing paths.
- [x] Define a stable lookup convention for schema IDs and aliases.
- [x] Keep current file locations unchanged in the first pass.
- [x] Document any schema that should remain legacy-only.
- [x] Add a resolver test that confirms every known consumer still finds its schema.
- [x] Add a drift check that fails if the registry and file tree disagree.
- [x] Add `schema_guided_extraction_context.json` linking extraction-oriented artifact families to schema ids.

### Exit Criteria

- [x] Every existing schema is discoverable from the registry.
- [x] No consumer breaks because of path movement.
- [x] The registry explains, but does not yet reorganize, the tree.

## M22. Minimal Domain Ontology

### Goal
Define a small, Nostra-native ontology layer for graph semantics and extension control.

### Checklist

- [x] Choose the ontology encoding approach: current baseline is a custom JSON manifest, with JSON-LD still under evaluation as an interoperability layer.
- [x] Define core vocabulary for `Space`, `Contribution`, `Relation`, `Capability`, and `ProvenanceScope`.
- [x] Define allowed relationship types between those core concepts.
- [x] Define property constraints for identity, labels, provenance, and scope.
- [x] Define versioning and compatibility rules for ontology changes.
- [x] Define extension rules for Space-local vocabulary additions.
- [x] Define what counts as a breaking ontology change.
- [x] Document how ontology terms map to existing constitutional types.
- [x] Add examples for at least two Space vocabularies.
- [x] Add an adversarial cross-space additive extension example.
- [x] Add semantic validation for invalid endpoints, core-term redefinition, and provenance-scope closure.

### Exit Criteria

- [x] The ontology can express the core graph vocabulary needed by `078`.
- [x] Space-local extensions are possible without breaking global terms.
- [x] The format is minimal enough to ship in the current stage.

## M23. Space Knowledge Bundle

### Goal
Specify a portable bundle manifest for graph state, provenance, and retrieval policy.

### Checklist

- [x] Define the `KnowledgeBundle` manifest structure.
- [x] Include references for ontology, graph snapshot, embeddings manifest, provenance root, and retrieval policy.
- [x] Specify bundle identity, version, and compatibility metadata.
- [x] Decide which fields are required versus optional.
- [x] Specify how the bundle references existing artifacts instead of embedding everything inline.
- [x] Keep JSON as the default manifest format.
- [x] Reserve MessagePack for bulk export/import and snapshot transport.
- [x] Define import/export round-trip behavior.
- [x] Define a sample bundle for one real Space.
- [x] Add export-grade bundle fixtures with immutable portable refs.
- [x] Add graph-only bundle coverage proving embeddings remain conditional.
- [x] Add bundle validation rules for missing references and incompatible versions.

### Exit Criteria

- [x] A bundle can be exported and re-imported without losing meaning.
- [x] The bundle format is readable enough for governance review.
- [x] The manifest is portable across environments.

## M24. Triple Query Facade

### Goal
Expose a read-only S-P-O query surface over `Contribution`/`Relation` data with named graph scopes.

### Checklist

- [x] Define request and response contracts for triple-style queries.
- [x] Support subject, predicate, object, and graph-scope filtering.
- [x] Support named graph scopes for `System`, `Actor`, and `Agent`.
- [x] Make JSON the default interactive query format.
- [x] Reserve MessagePack for bulk transfer paths only.
- [x] Define a read-only translation layer from current graph substrate to triple-shaped results.
- [x] Define deterministic ordering rules for query results.
- [x] Include provenance fields in query responses where available.
- [x] Add tests for empty, exact-match, and multi-hop-style queries.
- [x] Add tests for scope filtering and provenance isolation.
- [x] Add `any` scope and provenance-disabled fixture coverage.
- [x] Add a runtime-backed `GlobalEvent` projection path that reproduces the same triple semantics as the fixture adapter.
- [x] Add a derived topology read-model contract for Explore consumers.

### Exit Criteria

- [x] Agents can traverse graph data through a stable facade.
- [x] Query results are deterministic for the same input.
- [x] The interface does not require a TrustGraph-style runtime port.
- [x] Runtime-backed traversal preserves the `system` / `actor` / `agent` provenance partition.

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

### Phase E Pilot Status

- [x] Add a bounded internal graph-retrieval harness in `cortex-eudaemon`.
- [x] Support `graph_only`, `vector_only`, and `hybrid_graph_embedding` pilot modes.
- [x] Emit citation-bearing retrieval responses.
- [x] Add a curated benchmark case where hybrid retrieval outperforms a single-mode baseline.
- [ ] Compare the pilot against the current `037` ask/search runtime in a shared evaluation run.

### Phase F Operationalization Status

- [x] Restore a green default-feature `cortex-eudaemon` build gate separate from the slim graph-contract harness.
- [x] Add a daemon-internal graph service that wraps runtime projection, triple query, retrieval, and topology derivation.
- [x] Add a runner that emits benchmark, comparison, and topology artifacts under `logs/knowledge/`.
- [x] Record a shared comparison artifact against the current `037` benchmark baseline.
- [x] Keep the Explore topology surface derived from canonical triples rather than introducing a new graph authority.

### Exit Criteria

- [ ] Graph RAG improves answer quality on multi-hop questions.
- [ ] Retrieval remains explainable and lineage-aware.
- [ ] The pipeline is still compatible with Nostra-native contracts.

## Cross-Cutting Enrichments

- [x] Add provenance scopes to all new graph contracts.
- [x] Add compatibility notes for every breaking graph-related contract.
- [x] Keep `shared/specs.md` as the constitutional source of truth.
- [x] Keep ontology and query additions additive until a steward approves a breaking change.
- [x] Add a decision log entry whenever a new graph contract is finalized.
- [x] Link new Phase E discovery artifacts and handoff surfaces to the relevant initiative references.

## Suggested First Sprint

- [ ] Complete the schema inventory and expand the registry beyond the current knowledge-graphs slice.
- [x] Validate the current ontology vocabulary and extension rules against at least two realistic Space examples.
- [x] Draft the bundle manifest shape with required fields.
- [x] Decide the query facade request/response shape.
