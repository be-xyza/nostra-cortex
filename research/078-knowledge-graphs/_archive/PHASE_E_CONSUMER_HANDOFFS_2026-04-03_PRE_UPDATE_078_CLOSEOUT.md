# Phase E/F Consumer Handoffs

Date: 2026-04-03

This note links the ratified Phase E outputs to the adjacent initiatives that should consume them.

## 037 Knowledge Engine

- Continue using worker search/ask surfaces as the primary execution path.
- Treat the Phase E graph retrieval harness as a controlled comparator and citation-enrichment lane, not a replacement API.
- Use `logs/knowledge/graph_pilot_comparison_latest.json` as the current shared comparison artifact against the existing `037` retrieval benchmark baseline.
- The next shared evaluation run can deepen this into runtime-path parity, but the Phase F baseline already records graph-only, vector-only, and hybrid comparisons in one machine-readable artifact.

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
- Use the Phase F topology emission lane (`logs/knowledge/graph_pilot_topology_latest.json`) as the deterministic handoff artifact until a broader internal runtime entrypoint is needed.
- Any future network surface for Explore should remain a derived projection over these canonical contracts, not a bypass around them.

## Phase F Internal Operational Surfaces

- `cortex/apps/cortex-eudaemon/src/services/knowledge_graph_service.rs` is now the daemon-internal entrypoint for read-only graph execution.
- `scripts/run_knowledge_graph_pilot.sh` is the reproducible evidence runner for benchmark, comparison, and topology artifacts.
- No new public network contract was added in Phase F; all new surfaces remain internal execution or derived artifact paths.
