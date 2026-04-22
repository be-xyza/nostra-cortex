# MinerU Analysis

**Repository**: [opendatalab/MinerU](https://github.com/opendatalab/MinerU)
**Analyzed**: 2026-03-27
**License**: Copyleft license family; review required before any direct integration
**Primary Language**: Python
**Steward**: Research Steward

## Placement
`research/reference/topics/data-knowledge/MinerU`

## Intent
Evaluate MinerU as a specialist parser for scientific, scanned, formula-rich, and table-heavy documents.

## Initiative Links
- `037-nostra-knowledge-engine`
- `042-vector-embedding-strategy`
- `051-rag-ingestion-pipeline`
- `078-knowledge-graphs`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- Converts PDFs to Markdown and JSON with strong reading-order normalization.
- Strong table, image, formula, and OCR coverage, including large multilingual OCR support.
- Offers visualization outputs that can help human review and corpus debugging.
- Especially useful for scientific-document edge cases the current pipeline cannot touch.

## Possible Links To Nostra Platform and Cortex Runtime
- Valuable as a specialist comparator for research papers, reports, and scan-heavy knowledge artifacts.
- Could justify a niche fallback path if scientific-document quality materially beats general parsers.

## Adoption Decision
**Recommendation:** Watch and benchmark, not primary path.

MinerU clearly addresses real document-parsing pain, but it looks better as a specialist benchmark than as the default Cortex recommendation. The portfolio should first prove whether a science-document specialist is actually needed after Docling-class parsing is available.

## Known Risks
- Heavy runtime and operational surface.
- Younger project posture than some alternatives.
- Licensing and governance implications need explicit review before any direct integration.

## Suggested Next Experiments
- Include MinerU in the benchmark corpus only for scientific and formula-heavy inputs.
- Compare not just text quality, but table/formula fidelity and reviewability.
