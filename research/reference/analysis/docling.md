# Docling Analysis

**Repository**: [docling-project/docling](https://github.com/docling-project/docling)
**Analyzed**: 2026-03-27
**License**: MIT
**Primary Language**: Python
**Steward**: Research Steward

## Placement
`research/reference/topics/data-knowledge/docling`

## Intent
Evaluate Docling as the strongest open-source candidate for the parser layer missing from the current Nostra/Cortex ingestion path.

## Initiative Links
- `037-nostra-knowledge-engine`
- `042-vector-embedding-strategy`
- `051-rag-ingestion-pipeline`
- `078-knowledge-graphs`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- Advanced PDF understanding: page layout, reading order, table structure, formulas, code, image classification.
- Unified `DoclingDocument` representation plus Markdown, HTML, DocTags, and lossless JSON export.
- Local execution, OCR support, broad format coverage, and structured information extraction in beta.
- Natural fit for a document-to-normalized-blocks adapter feeding chunking, extraction, and indexing.

## Possible Links To Nostra Platform and Cortex Runtime
- Best candidate to populate the missing `IngestStage` reality behind the current `docling+ocrmypdf` parser hint.
- Can serve the 051 ingestion tube before schema-guided extraction and entity resolution.
- Should live behind a sidecar/service boundary so 118 runtime purity remains intact.

## Adoption Decision
**Recommendation:** Strongest primary parser reference candidate.

Docling is the best match to the current portfolio gap because the gap is not "OCR exists somewhere"; it is "Cortex lacks a serious, structured, provenance-friendly parser for real documents." Docling addresses that directly and cleanly.

Recommendation posture:
- Prefer Docling as the leading parser reference for future adapter work.
- Keep the integration minimal: parser service in, normalized block/page/table/image contract out.
- Do not import Docling framework concepts directly into Nostra authority types.

## Known Risks
- Heavy Python/ML stack compared with the Cortex runtime target.
- Broad feature surface may tempt oversized integration.
- Requires careful contract narrowing so parser output does not become de facto platform authority.

## Suggested Next Experiments
- Design a governed parser output contract with page, block, bbox, reading-order, table, image, and confidence fields.
- Prototype a Docling sidecar that emits only that contract.
- Run it through the current extraction + indexing path to see what contract pressure appears before larger adoption.
