# Unstructured Analysis

**Repository**: [Unstructured-IO/unstructured](https://github.com/Unstructured-IO/unstructured)
**Analyzed**: 2026-03-27
**License**: Apache-2.0
**Primary Language**: Python
**Steward**: Research Steward

## Placement
`research/reference/topics/data-knowledge/unstructured`

## Intent
Evaluate Unstructured as the preprocessing and partitioning comparator around the parser gap, especially for broad connectors and element-based document decomposition.

## Initiative Links
- `037-nostra-knowledge-engine`
- `042-vector-embedding-strategy`
- `051-rag-ingestion-pipeline`
- `078-knowledge-graphs`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- Broad document coverage with partition-style outputs.
- Strong ETL and preprocessing orientation for downstream chunking and enrichment.
- Helpful for evaluating whether Cortex needs connector/partition capabilities in addition to parsing.

## Possible Links To Nostra Platform and Cortex Runtime
- Relevant to ingestion stages before chunking and indexing.
- Potentially useful for office, HTML, and email-style content where parser geometry is less important than stable element segmentation.

## Adoption Decision
**Recommendation:** Preprocessing comparator, not primary parser recommendation.

Unstructured is valuable, but it is solving a broader ETL problem than the immediate Cortex gap. The current gap is the absence of a serious document parser behind the extraction API. Unstructured helps answer the adjacent question: do we also need a partition/connectors layer once the parser exists?

## Known Risks
- Large Python dependency surface.
- Platform tilt can expand scope quickly.
- Element outputs still need normalization to fit Cortex provenance expectations.

## Suggested Next Experiments
- Compare Unstructured partition outputs with Docling and MarkItDown on HTML, Office docs, and email-like inputs.
- Decide whether partition-first preprocessing deserves its own adapter lane.
