# MarkItDown Analysis

**Repository**: [microsoft/markitdown](https://github.com/microsoft/markitdown)
**Analyzed**: 2026-03-27
**License**: MIT
**Primary Language**: Python
**Steward**: Research Steward

## Placement
`research/reference/topics/data-knowledge/markitdown`

## Intent
Evaluate MarkItDown as a low-friction normalization tool for broad office/document inputs rather than as a high-fidelity parser.

## Initiative Links
- `037-nostra-knowledge-engine`
- `042-vector-embedding-strategy`
- `051-rag-ingestion-pipeline`
- `078-knowledge-graphs`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- Broad file coverage and simple Markdown-first outputs.
- Plugin support, MCP exposure, and optional Azure Document Intelligence path.
- Strong candidate for cheap normalization of office, HTML, text, and mixed-media inputs.

## Possible Links To Nostra Platform and Cortex Runtime
- Useful where the goal is normalized text for indexing, not high-fidelity geometry.
- Could coexist with a higher-fidelity parser lane as a convenience path for office/email/media ingestion.

## Adoption Decision
**Recommendation:** Companion normalizer reference, not parser-core recommendation.

MarkItDown does not solve the same problem Docling or Marker solve. It is intentionally lighter and more Markdown-centric. That makes it useful, but in a different slot: preprocessing and normalization, not authoritative document parsing.

## Known Risks
- High-fidelity layout and bbox provenance are not the goal.
- OCR may depend on plugins or Azure Document Intelligence.
- Easy to overestimate because of breadth of input types.

## Suggested Next Experiments
- Test MarkItDown on Office-heavy and email-like corpora.
- Decide whether Cortex should explicitly support a low-fidelity markdown ingress lane beside the higher-fidelity parser lane.
