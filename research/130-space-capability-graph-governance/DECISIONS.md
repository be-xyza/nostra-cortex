# Decisions

## DEC-130-001: Catalog + Space Overlay Split
**Date**: 2026-03-03

**Decision**: Keep a single platform capability catalog and introduce a steward-governed `SpaceCapabilityGraph` overlay per space.

**Rationale**:
1. Preserves global semantic consistency while enabling local subsidiarity.
2. Supports space-level UX evolution without host-specific hardcoding.
3. Keeps compilation deterministic and parity-safe across hosts.

**Implications**:
1. Structural graph edits become steward-gated operations.
2. Space registry stores linkage metadata to capability graph artifacts.
3. Runtime exposes additive APIs for catalog, overlay, and compiled plans.
