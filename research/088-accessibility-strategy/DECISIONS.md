---
id: "088"
name: "accessibility-strategy"
title: "Accessibility Strategy Decisions"
type: "decisions"
project: "nostra"
status: approved
authors:
  - "X"
  - "Winston (Architect)"
tags: [accessibility, decisions]
created: "2026-01-28"
updated: "2026-02-06"
---

# Accessibility Strategy Decisions

## Decision Log

### D1: Target Conformance Level
**Decision**: WCAG 2.2 Level AA + WAI-ARIA 1.2
**Date**: 2026-01-28
**Rationale**: Industry standard baseline, globally recognized, legally defensible

### D2: Scope of Application
**Decision**: Applies to Nostra & Cortex at appropriate levels
**Date**: 2026-01-28
**Details**:
- Core platform interfaces must meet Level AA
- User-generated themes and content have extended creative expression rights
- Guidelines provide direction for accessibility feature planning, not rigid enforcement on user content

**Creative Expression Exemption**:
> User-generated themes, content, and creative works are not required to meet accessibility guidelines. However, the platform should provide tools and guidance to help creators make accessible content when they choose.

### D3: A2UI ARIA Mappings
**Decision**: Yes - Add ARIA role mappings to A2UI standard catalog
**Date**: 2026-01-28
**Rationale**: Essential for assistive technology compatibility with streaming UI protocol

### D4: Canonical Accessibility Catalog Location
**Decision**: Store accessibility catalogs under `nostra/A2UI/specification/v0_9/json/`
**Date**: 2026-02-05
**Rationale**: Keeps accessibility requirements co-located with A2UI specs and tools for versioned enforcement.

### D5: A2UI `a11y` Metadata Requirement
**Decision**: Require `a11y` metadata on interactive A2UI components (top-level field in V1, with legacy `props.a11y` accepted)
**Date**: 2026-02-05
**Rationale**: Enables consistent enforcement, linting, and renderer behavior aligned with WCAG 2.2 AA.

### D6: Linter Compatibility for Legacy A2UI Message Shapes
**Decision**: Extend `lint-a2ui.ts` to parse both canonical and legacy payload shapes (`surfaceUpdate`, `updateComponents`, and `component`) during enforcement.
**Date**: 2026-02-06
**Rationale**: Real sample payloads in the repository use mixed schema shapes; compatibility removes false negatives and keeps accessibility linting effective across catalog and legacy examples.

---

## Implementation Implications

1. **Platform vs. User Content Distinction**
   - Platform UI (navigation, controls, system messages): **Must comply**
   - User-generated themes: **Encouraged, tools provided**
   - User creative content: **Expression freedom preserved**

2. **Theme System Integration**
   - Default themes must pass WCAG AA contrast requirements
   - Custom themes can override (user assumes responsibility)
   - Theme editor should include accessibility warnings/suggestions

3. **A2UI Catalog Updates**
   - Add `ariaRole` property to component definitions
   - Add `ariaStates` and `ariaProperties` where applicable
   - Document keyboard interaction patterns per component
