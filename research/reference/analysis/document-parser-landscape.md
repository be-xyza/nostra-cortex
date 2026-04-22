# Document Parser Landscape — Cortex Intake Resolution

**Analyzed**: 2026-03-27
**Steward**: Research Steward

## Scope
This memo resolves the document-parsing intake across:
- `research/037-nostra-knowledge-engine`
- `research/042-vector-embedding-strategy`
- `research/051-rag-ingestion-pipeline`
- `research/078-knowledge-graphs`
- `research/118-cortex-runtime-extraction`

## Current Architectural Reality
Cortex/Nostra already has an extraction orchestration shell, fallback policy, and indexing path, but it does **not** yet have a serious parser behind that shell. The present implementation still relies on:
- parser hints instead of real document parsing,
- string content instead of binary/page/layout inputs,
- heuristic entity extraction instead of document-grade structure recovery,
- simulated cloud/external adapters.

## Decision
Do **not** adopt `liteparse` blindly as the answer to the parser gap.

The parser gap is real, but the right resolution is a ranked reference set:

1. **Docling**: strongest primary parser reference candidate
2. **LiteParse**: lightweight local fallback / bbox + screenshot pre-pass candidate
3. **Marker**: hard-case benchmark oracle, not default integration target
4. **MarkItDown**: low-fidelity normalization lane for office-style ingestion
5. **Unstructured**: preprocessing / partitioning comparator for future adjacent scope
6. **MinerU**: specialist scientific-document benchmark
7. **Unstract**: higher-level extraction-service comparator after parser normalization exists

## Why This Ranking Fits The Portfolio
- `051` needs a real parser before chunk -> extract -> resolve -> index becomes credible.
- `042` needs structured, provenance-safe parser output before schema-guided extraction and hybrid retrieval can be meaningfully evaluated.
- `078` is blocked on ingestion quality and cross-index integrity, not on adding more tool brands.
- `118` requires any parser to remain outside the pure runtime boundary behind a narrow adapter contract.

## Recommended Next Move
Design a governed parser output contract first, then benchmark Docling, LiteParse, and Marker against it.

The contract should carry only what Cortex actually needs:
- page identifiers
- reading order
- block text
- bounding boxes where available
- tables/images/formulas as typed blocks
- parser confidence and provenance
- optional screenshots for review

Once that contract exists, Cortex can decide whether it needs:
- one default parser,
- a lightweight fallback,
- and/or a separate low-fidelity normalization lane.
