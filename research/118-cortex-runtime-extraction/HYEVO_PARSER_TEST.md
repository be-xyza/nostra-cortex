# HyEvo Parser Test Comparison

> Extracted from the HyEvo paper analysis to maintain separation of concerns between workflow research and document extraction benchmarks.

## Intent
Retain the HyEvo paper PDF as a real-world parser comparison specimen for the Cortex upload/extraction runtime. The paper is useful as a parser test specimen because a current arXiv PDF exercises the new artifact-first extraction path with realistic academic layout structure.

## Actual Parser Comparison
The comparison is now grounded in real parser execution against the same HyEvo PDF, not just the Cortex gateway fallback path.

### Docling run
- Executed via `scripts/docling_upstream_adapter.py` with the workspace Docling venv.
- Output summary:
  - `parser_backend: docling`
  - `parser_profile: docling`
  - `model_id: docling:python-api:2.82.0`
  - `page_count: 9`
  - `block_count: 228`
  - `first_page_blocks: 13`
- Strengths observed:
  - Clean title / author / affiliation extraction on page 1
  - Structured block output is compact and easy to consume downstream
  - Better fit for normalized extraction and artifact card summaries

### LiteParse run
- Executed via the real `lit parse` CLI from `@llamaindex/liteparse` v1.4.2.
- Output summary:
  - `page_count: 9`
  - `text_item_count: 7742`
  - `first_page_items: 656`
  - page text preserves the expected title / author / affiliation content
- Strengths observed:
  - Much denser spatial / text-item preservation
  - Stronger raw layout visibility for page-level inspection
  - Good fit for artifact-side provenance and visual QA

### Interpretation
- Docling and LiteParse are not interchangeable on this document.
- Docling is the more compact normalized extractor.
- LiteParse is the more layout-preserving parser, which makes it useful for QA, review, and fallback analysis.
- The earlier live Cortex gateway records showing `10555` pages / `34654` blocks were local fallback-shaped runs, not the canonical parser comparison result.
- LiteParse should remain surfaced on the upload / extraction path, while Providers stays focused on operator/runtime inventory.
