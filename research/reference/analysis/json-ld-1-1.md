---
id: json-ld-1-1
name: json-ld-1-1
title: JSON-LD 1.1 Standard Analysis
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: nostra
authority_mode: recommendation_only
reference_topics: [web-semantics, data-knowledge]
reference_assets:
  - "research/reference/knowledge/web-semantics/2020_W3C_JSON_LD_1_1"
evidence_strength: strong
handoff_target:
  - "Research Steward"
authors:
  - "Codex"
tags: [json-ld, rdf, ontology, graph-semantics]
stewardship:
  layer: "Architectural"
  primary_steward: "Research Steward"
  domain: "Knowledge Graphs"
created: "2026-04-02"
updated: "2026-04-02"
---

# JSON-LD 1.1 Standard Analysis

## Placement
- Destination: `research/reference/knowledge/web-semantics/2020_W3C_JSON_LD_1_1`

## Intent
Evaluate JSON-LD 1.1 as the most plausible standards baseline for a minimal, JSON-first ontology exchange layer inside Nostra/Cortex.

## Possible Links To Nostra Platform and Cortex Runtime
- `078-knowledge-graphs` minimal ontology format and extension rules.
- `042-vector-embedding-strategy` ontology-guided extraction and controlled vocabulary transport.
- `051-rag-ingestion-pipeline` graph-ready interchange between extraction and indexing.
- `118-cortex-runtime-extraction` off-canister ontology conversion boundaries.

## Initiative Links
- `042-vector-embedding-strategy`
- `051-rag-ingestion-pipeline`
- `078-knowledge-graphs`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- JSON-native linked-data representation.
- Practical bridge between local JSON contracts and RDF-compatible graph semantics.
- Strong candidate for a constrained ontology manifest that remains understandable to Rust-first workflows.

## Adoption Decision
**Recommendation:** Adopt patterns, and treat JSON-LD 1.1 as the leading exchange-format candidate.

JSON-LD is the least disruptive way to gain standards alignment without abandoning the project's JSON-first operational posture. The right move is not "become an RDF stack," but "use JSON-LD selectively where it improves ontology clarity and interoperability."

## Known Risks
- It can smuggle in broader RDF complexity if the allowed subset is not explicitly governed.
- JSON familiarity may create false confidence about semantic simplicity.

## Suggested Next Experiments
1. Define a minimal Nostra ontology manifest as a constrained JSON-LD subset.
2. Round-trip that manifest through Owlish and Horned OWL to check semantic drift.
