---
id: sparql-1-1-overview
name: sparql-1-1-overview
title: SPARQL 1.1 Overview Standard Analysis
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: nostra
authority_mode: recommendation_only
reference_topics: [web-semantics, data-knowledge]
reference_assets:
  - "research/reference/knowledge/web-semantics/2013_W3C_SPARQL_1_1_Overview"
evidence_strength: strong
handoff_target:
  - "Research Steward"
authors:
  - "Codex"
tags: [sparql, rdf, query, graph-semantics]
stewardship:
  layer: "Architectural"
  primary_steward: "Research Steward"
  domain: "Knowledge Graphs"
created: "2026-04-02"
updated: "2026-04-02"
---

# SPARQL 1.1 Overview Standard Analysis

## Placement
- Destination: `research/reference/knowledge/web-semantics/2013_W3C_SPARQL_1_1_Overview`

## Intent
Evaluate SPARQL 1.1 as the baseline standards comparator for future graph query semantics in Nostra/Cortex.

## Possible Links To Nostra Platform and Cortex Runtime
- `078-knowledge-graphs` read-only graph query facade design.
- `118-cortex-runtime-extraction` graph-query interoperability boundaries.

## Initiative Links
- `078-knowledge-graphs`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- Standard graph query model over RDF data.
- Strong comparator for triple-pattern matching, filters, and named-graph scoping.
- Useful as a boundary marker between minimal Nostra-native querying and full RDF-store expectations.

## Adoption Decision
**Recommendation:** Adopt patterns, keep compatibility aspirations narrow.

SPARQL 1.1 should guide the design of any future graph-query surface, but the project should not promise full SPARQL support unless a later initiative proves that need. The useful move now is to measure a smaller read-only query facade against SPARQL concepts instead of copying the whole language.

## Known Risks
- Full SPARQL compatibility would create a much larger contract than current work requires.
- Query-language ambitions can distort storage and ontology decisions too early.

## Suggested Next Experiments
1. Draft a minimal read-only triple or relation-query facade and compare it against core SPARQL concepts.
2. Explicitly keep SPARQL update semantics out of scope for the near term.
