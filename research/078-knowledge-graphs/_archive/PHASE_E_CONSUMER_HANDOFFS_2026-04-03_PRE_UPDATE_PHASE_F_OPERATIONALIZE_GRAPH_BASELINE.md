# Phase E Consumer Handoffs

Date: 2026-04-03

This note links the ratified Phase E outputs to the adjacent initiatives that should consume them.

## 037 Knowledge Engine

- Continue using worker search/ask surfaces as the primary execution path.
- Treat the Phase E graph retrieval harness as a controlled comparator and citation-enrichment lane, not a replacement API.
- The next shared evaluation run should compare the current `037` search/ask path against graph-only and hybrid graph retrieval for curated multi-hop questions.

## 042 Embedding Strategy

- The Phase E retrieval pilot reuses hybrid reasoning direction rather than inventing a new ranking stack.
- `hybrid_graph_embedding` is now an explicit retrieval mode in the bundle contract and the internal retrieval harness.
- Phase E benchmark cases should remain compatible with future `042` ranking improvements without changing the commons contracts.

## 051 RAG Ingestion Pipeline

- `schema_guided_extraction_context.json` is the initial contract bridge from extraction-oriented artifacts to their governing schema ids.
- The next ingestion step should emit artifacts that can be validated directly against the ontology, bundle, and topology contracts added in `078`.
- Full ingestion modernization remains separate from the Phase E retrieval pilot.

## 136 Cortex Explore Graph

- Consume `explore_topology_view.schema.json` as the internal topology/read-model contract.
- Keep Explore projections derived from the canonical triple facade rather than defining a new graph authority surface.
- Any future network surface for Explore should remain a derived projection over these canonical contracts, not a bypass around them.
