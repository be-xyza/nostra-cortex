# Marker Analysis

**Repository**: [datalab-to/marker](https://github.com/datalab-to/marker)
**Analyzed**: 2026-03-27
**License**: GPL-3.0 code license with additional model-license constraints
**Primary Language**: Python
**Steward**: Research Steward

## Placement
`research/reference/topics/data-knowledge/marker`

## Intent
Evaluate Marker as the high-accuracy comparator for hard documents and schema-aware extraction workloads.

## Initiative Links
- `037-nostra-knowledge-engine`
- `042-vector-embedding-strategy`
- `051-rag-ingestion-pipeline`
- `078-knowledge-graphs`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- High-quality Markdown, JSON, chunk, and HTML conversion.
- Strong handling for tables, forms, equations, OCR, and reading order.
- Beta JSON-schema extraction and optional LLM-assisted hybrid mode.
- Benchmark culture is valuable because it compares itself against Docling and LlamaParse.

## Possible Links To Nostra Platform and Cortex Runtime
- Useful as a benchmark oracle for the corpus that will eventually govern parser selection.
- Relevant to schema-guided extraction experiments where raw parser output needs to be pushed toward structured fields.
- Less attractive as a direct integration because the licensing story is materially riskier than Docling or LiteParse.

## Adoption Decision
**Recommendation:** Benchmark and comparator reference, not default integration target.

Marker is technically compelling, but its licensing and hybrid-LLM posture make it hard to recommend as the primary parser path for Cortex. It is more valuable as a hard-case benchmark that keeps the evaluation honest.

## Known Risks
- GPL-3.0 plus model-license constraints.
- LLM mode introduces extra nondeterminism and ops complexity.
- Some forms and very complex layouts remain known limitations.

## Suggested Next Experiments
- Use Marker in the evaluation harness for the hardest PDFs.
- Compare its output against Docling and LiteParse before any adapter investment.
- Treat any direct integration as steward-gated due to licensing.
