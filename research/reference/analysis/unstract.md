# Unstract Analysis

**Repository**: [Zipstack/unstract](https://github.com/Zipstack/unstract)
**Analyzed**: 2026-03-27
**License**: AGPL-3.0
**Primary Language**: Python
**Steward**: Research Steward

## Placement
`research/reference/repos/unstract`

## Intent
Re-evaluate the already-intaken Unstract reference as part of the document-parsing and extraction landscape, especially because local code already contains a simulated Unstract adapter path.

## Initiative Links
- `037-nostra-knowledge-engine`
- `042-vector-embedding-strategy`
- `051-rag-ingestion-pipeline`
- `118-cortex-runtime-extraction`

## Pattern Extraction
- Prompt-defined schema extraction to structured JSON.
- API deployment, ETL pipeline mode, MCP exposure, and human-review flows.
- More platform-like than parser-like: it sits higher in the stack than LiteParse/Docling/Marker.

## Possible Links To Nostra Platform and Cortex Runtime
- Relevant for an external-adapter contract after a parser layer exists.
- Helpful for human-review and document-to-JSON workflow ideas.
- Relevant because `nostra/worker/src/skills/extraction.rs` already sketches an Unstract adapter, though only in simulated form today.

## Adoption Decision
**Recommendation:** External extraction-service reference only.

Unstract is not the missing parser core for Cortex. It is a higher-level extraction platform that could become relevant after the parser and normalization boundary are defined. Treat it as a governed external service comparator, not as the answer to the current parser gap.

## Known Risks
- AGPL-3.0 licensing.
- Heavy Docker/platform footprint.
- Risk of jumping too early from parser gap to product-platform adoption.

## Suggested Next Experiments
- After the parser benchmark is complete, compare parser-normalized inputs against Unstract's prompt-defined JSON extraction on a small steward-approved corpus.
- Decide whether the future external-adapter contract should support parser services, extraction services, or both as separate lanes.
