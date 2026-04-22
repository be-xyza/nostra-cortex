# LiteParse Analysis

**Repository**: [run-llama/liteparse](https://github.com/run-llama/liteparse)
**Analyzed**: 2026-03-27
**License**: Apache-2.0
**Primary Language**: TypeScript / Node.js
**Steward**: Research Steward

## Placement
`research/reference/topics/data-knowledge/liteparse`

## Intent
Evaluate whether LiteParse closes the current parser gap between Cortex's extraction orchestration shell and a real document-understanding backend.

## Initiative Links
- `037-nostra-knowledge-engine`
- `042-vector-embedding-strategy`
- `051-rag-ingestion-pipeline`
- `078-knowledge-graphs`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- Local-first parser with spatial text extraction, bounding boxes, JSON/text output, and screenshots.
- Small OCR adapter contract: built-in Tesseract or a simple HTTP `/ocr` service.
- Multi-format ingestion path through Office/image-to-PDF conversion.
- Good fit for page-level provenance, screenshot review, and lightweight local parsing.

## Possible Links To Nostra Platform and Cortex Runtime
- A Cortex extraction adapter could ingest LiteParse page, text, bbox, and screenshot output without violating the 118 boundary, as long as LiteParse stays out-of-process.
- Bounding boxes could improve chunk provenance and human-review overlays in the knowledge-engine path.
- Screenshots are attractive for agent review loops and multimodal fallback.

## Adoption Decision
**Recommendation:** Reference and limited adapter candidate.

LiteParse does solve a real gap: the current local pipeline has no true PDF/layout parser at all. But it does **not** solve the full architecture gap by itself. Upstream explicitly says complex documents do better with LlamaParse, which is a strong signal that LiteParse is a lightweight parser, not a production-grade general document-intelligence backend.

Use it as:
- a local-first fallback,
- a bbox/screenshot pre-pass,
- or a narrow parser for text-heavy PDFs and simple office/image flows.

Do **not** make it the sole Cortex parser recommendation.

## Known Risks
- Limited fit for dense tables, complex layouts, charts, handwriting, and hard scanned PDFs.
- Node runtime does not align with the Rust/WASM purity boundary and must remain external.
- Parser output is geometric and textual, not schema-aware semantic extraction.

## Suggested Next Experiments
- Benchmark LiteParse against Docling and Marker on a governed corpus: text PDF, scanned PDF, slide deck, invoice/table-heavy PDF, and formula-rich paper.
- Define the smallest normalized Cortex parser contract that can carry bbox and screenshot provenance without importing LiteParse concepts into the platform model.
