---
id: shacl
name: shacl
title: SHACL Standard Analysis
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: nostra
authority_mode: recommendation_only
reference_topics: [web-semantics, data-knowledge]
reference_assets:
  - "research/reference/knowledge/web-semantics/2017_W3C_SHACL"
evidence_strength: strong
handoff_target:
  - "Research Steward"
authors:
  - "Codex"
tags: [shacl, rdf, validation, constraints, ontology]
stewardship:
  layer: "Architectural"
  primary_steward: "Research Steward"
  domain: "Knowledge Graphs"
created: "2026-04-02"
updated: "2026-04-02"
---

# SHACL Standard Analysis

## Placement
- Destination: `research/reference/knowledge/web-semantics/2017_W3C_SHACL`

## Intent
Evaluate SHACL as the primary standards comparator for graph validation and constraint reporting around a future Nostra ontology layer.

## Possible Links To Nostra Platform and Cortex Runtime
- `051-rag-ingestion-pipeline` graph validation before indexing.
- `078-knowledge-graphs` ontology constraints and extension governance.
- `118-cortex-runtime-extraction` sidecar validation and compatibility checks.

## Initiative Links
- `051-rag-ingestion-pipeline`
- `078-knowledge-graphs`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- Separate shapes graph for describing and validating data graphs.
- Strong model for constraint components, validation reports, and conformance checks.
- Useful benchmark for deciding how much validation expressivity Nostra actually needs.

## Adoption Decision
**Recommendation:** Adopt patterns, not default implementation.

SHACL is the right reference point for graph constraints, but it is still larger than the minimum viable validation surface. Start by comparing Nostra-native rules against SHACL Core, and keep SHACL-SPARQL out of scope unless the simpler surface proves insufficient.

## Known Risks
- Direct adoption could overshoot the project's minimal validation needs.
- SPARQL-powered constraints can add substantial operational and conceptual complexity.

## Suggested Next Experiments
1. Express a tiny Nostra ontology vocabulary in SHACL Core only.
2. Compare SHACL validation output against the project’s desired operator-facing diagnostics.
