---
status: "active"
owner: "Architect"
tags: ["ndl", "a2ui", "governance", "ui", "verified-projection"]
created: "2026-02-19"
updated: "2026-04-27"
stewardship:
  layer: "protocol"
  primary_steward: "Design Systems Steward"
  domain: "Interface Constitution"
---

# Plan: Nostra Design Language (NDL)

## 1. Objective
Establish the **Nostra Design Language (NDL)** as a "Constitutional Interface Layer" for the Nostra Platform. This initiative defines the visual grammar, temporal motion semantics, and anti-spoofing protocols required to render governance artifacts securely.

## 2. Rationale
While Initiative `074-cortex-ui-substrate` handles the mechanical UI rendering (Dioxus, A2UI), it does not define **what** is rendered or **how** truth is visually guaranteed.
A malicious AI agent could spoof a "Ratified" badge via an A2UI payload if the renderer blindly trusts the input. NDL solves this by defining an immutable visual grammar and a "Verified Projection Architecture" that forces the runtime to query the graph directly for constitutional components, completely decoupling UI declaration from truth state.

## 3. Scope
**In Scope:**
*   NDL Specification Document (Philosophy, Grammar, AI Contracts).
*   NDL JSON Schema for machine-enforceable rules.
*   NDL Component Registry mapping (Tier 1 vs Tier 2 vs Tier 3).
*   Verified Projection Architecture design (A2UI `*Ref` components).
*   Cortex Runtime `NdlComponentResolver` strategy.

**Out of Scope:**
*   Building the `cortex-desktop` Dioxus UI Substrate (handled by `074`).
*   Building standard, non-constitutional components like Buttons or TextFields (handled by `045-component-library-labs`).

## 4. Key Deliverables
- [x] `DESIGN_LANGUAGE.md` (Authoritative Specification)
- [x] `NDL_JSON_SCHEMA_v0.1.json` (Machine-Validation Schema)
- [x] `NDL_COMPONENT_REGISTRY_v0.1.md` (Tier Classification)
- [x] `A2UI_NDL_POINTERS_0_1.json` (Extension Catalog for Verified Projection)
- [x] Integrate Surface Boundary Protocol into Research 014, 018, 008

## 5. Timeline & Dependencies
*   **Dependencies**: Requires `074-cortex-ui-substrate` (for Dioxus rendering) and `103-agent-client-protocol-alignment` (for A2UI spec updates).
*   **Status**: Phase 1 (Scaffolding), Phase 2 (Verified Projection Architecture), and Phase 3 (Surface Boundary Doctrine) are complete. NDL is now architecturally aligned with Cortex Runtime.

## 6. Space Design Meta-Scope Addendum

### Locked Design Reality

Current design and theme contracts are treated as the locked state of truth for all Space-level design work. `SPACE_DESIGN.md`, `NdlDesignProfileV1`, imported design elements, and template packs are subordinate to these authority surfaces:

1. NDL verified projection and surface-boundary doctrine from this initiative.
2. A2UI theme/render metadata, safe-mode fallback, token-version handling, motion policy, and contrast preference under `074-cortex-ui-substrate`.
3. Cortex Web as the current interactive Workbench host under `123-cortex-web-architecture`.
4. ViewSpec governance for generated UI artifacts under `115-cortex-viewspec-governed-ui-synthesis`.
5. Space capability overlays and navigation plans under `130-space-capability-graph-governance`.
6. Branding policy contracts under `shared/standards/branding/`.
7. Accessibility and authority containment under `shared/standards/ACCESSIBILITY.md`.
8. Hermes advisory cognition boundaries under `132-eudaemon-alpha-initiative`.

This addendum does not authorize a new design framework, a runtime theme switcher, or direct import of external component systems.

### Correct Scope

The correct scope is a meta-cognition and standards-hardening layer for Space-level visual identity:

1. Capture the locked design reality before evaluating a Space profile.
2. Evaluate proposed Space design profiles against NDL, A2UI, branding, accessibility, ViewSpec, and Space capability rules.
3. Allow design templates and imports only as steward-reviewed candidate materials.
4. Emit recommendations, findings, and promotion gates before any runtime adoption.
5. Preserve creative differentiation between Spaces without allowing visual style to imply governance authority.

### New Planning Primitives

- `DesignRealitySnapshotV1`: a bounded manifest of current design authority sources, expected checks, and source hashes or refs used for one analysis pass.
- `SpaceDesignProfileV1`: the Nostra-owned profile wrapper currently prototyped as `NdlDesignProfileV1`.
- `DesignElementImportV1`: a candidate import record for established design elements such as palettes, typography systems, component recipes, icon sets, layout templates, or token packs.
- `SpaceTemplatePackV1`: a reusable bundle of Space profile defaults for known Space archetypes.
- `DesignAuditUnitV1`: a source packet for local/Hermes analysis of one design profile, template pack, or import candidate.
- `DesignRecommendationV1`: structured advisory output with `adopt`, `adapt`, `reject`, or `needs_steward_review`.
- `DesignPromotionGateV1`: required evidence before a profile can move from `recommendation_only` to `steward_approved`.

### Alignment Resolution

- `005-nostra-design` remains legacy doctrine; only timeless principles carry forward.
- `028-a2ui-integration-feasibility`, `045-component-library-labs`, and `070-a2ui-testing-ground` are consolidated under `074-cortex-ui-substrate`.
- `088-accessibility-strategy` remains a hardening dependency for contrast, focus, keyboard, motion, semantic roles, and cognitive-load checks.
- `106`, `107`, and `108` constrain decision-surface legibility and governance-action styling.
- `115` owns generated UI artifact governance; Space profiles may inform ViewSpec candidates but cannot ratify them.
- `123` owns web-host consumption and fixture validation.
- `130` owns Space capability activation and navigation context.
- `132` owns advisory meta-cognition boundaries; Hermes may analyze but not approve, mutate, import, or enforce design profiles.

### Drift Note

`074-cortex-ui-substrate` references `docs/architecture/a2ui-theme-policy.md`, which is absent in this checkout. Until that document is restored or replaced, the effective theme-policy truth is the combination of `shared/a2ui/themes/*`, `shared/a2ui/fixtures/*`, `cortex/libraries/cortex-domain/src/theme/policy.rs`, and `cortex/apps/cortex-eudaemon/src/services/theme_policy.rs`.
