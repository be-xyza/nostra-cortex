---
id: '065'
name: code-editor-analysis
title: Decisions
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-05'
updated: '2026-02-05'
---

# Decisions

## DEC-001: Rename to VS Code Analysis (2026-02-06)
**Status**: ✅ Decided

**Decision**: Rename initiative from `vscode-integration` to `vscode-analysis` and limit scope to analysis and pattern extraction. No direct VS Code integration is planned at this stage.

**Rationale**: Align with analysis initiatives and avoid premature integration commitments.

**Implications**: Future integration or implementation requires a separate decision and updated scope.

## DEC-002: Expand to Code Editor Analysis (2026-02-06)
**Status**: ✅ Decided

**Decision**: Expand scope to `code-editor-analysis`, covering both VS Code and Zed pattern analysis (local repositories) and focusing on reusable components rather than direct integration.

**Rationale**: The broader scope better reflects the objective of extracting editor patterns and capabilities that can be mapped to Cortex Desktop and Nostra workflows.

**Implications**: Rename initiative files/paths accordingly; any integration work remains gated by separate decisions.

## DEC-003: GPUI Feasibility Study (2026-02-06)
**Status**: ✅ Decided

**Decision**: Add a focused feasibility study on Zed's GPUI framework as part of the code-editor-analysis initiative.

**Rationale**: GPUI may offer performance and collaboration architecture advantages, but must be evaluated for compatibility with Cortex Desktop's Dioxus/webview stack and Nostra's portability constraints.

**Implications**: Study is analysis-only; any adoption requires a separate decision and scope update.

## DEC-004: Split Artifacts + Full Deep-Dive Scan (2026-02-06)
**Status**: ✅ Decided

**Decision**: Produce split analysis artifacts (`CAPABILITY_MATRIX.md`, `COMPONENT_INVENTORY.md`, `GPUI_FEASIBILITY.md`, `PERFORMANCE_PARITY.md`) and perform a full deep-dive scan of Zed crates and VS Code source paths.

**Rationale**: Split artifacts improve reuse and traceability; a full scan yields stronger component mapping for Cortex Desktop and Nostra governance.

**Implications**: `RESEARCH.md` will reference the artifact index; verification snapshot must log the scan and artifact creation.
