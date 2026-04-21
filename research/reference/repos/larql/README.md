# larql

- Upstream repository: `https://github.com/chrishayuk/larql`
- Intake date: `2026-04-14`
- Placement: `research/reference/repos/larql/`
- Authority mode: `recommendation_only`

## Why This Placement Exists

This folder records the governed intake placement for `larql` as a cross-topic repository reference. The repo is relevant to Nostra/Cortex primarily for query-language design, immutable patch overlays, and Rust-native CLI/server separation.

It is not vendored into the checkout as a managed dependency. The live source was inspected from a temporary clone during intake, and ongoing governed ingestion work should eventually flow through initiative `127-cortex-native-repo-ingestion`.

## Current Judgment

`larql` is useful as a reference for semantic query and mutation interface design. It is not a direct dependency candidate, not a SPARQL parser, and not a replacement for Nostra authority surfaces.

## Analysis

See `research/reference/analysis/larql.md` for the formal scorecard, corrected assumptions, risks, and suggested experiments.
