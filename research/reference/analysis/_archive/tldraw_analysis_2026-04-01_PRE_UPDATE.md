---
id: tldraw
name: tldraw
title: tldraw SDK
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: nostra
authority_mode: recommendation_only
reference_topics: [ui-substrate, visualization, frontend, agent-systems]
reference_assets:
  - "research/reference/topics/ui-substrate/tldraw"
evidence_strength: strong
handoff_target:
  - "UX Steward"
  - "Systems Steward"
authors:
  - "User"
  - "Codex"
tags: [ui, infinite-canvas, agents, frontend, crdt]
stewardship:
  layer: "Capabilities"
  primary_steward: "UX Steward"
  domain: "Interfaces & Extensibility"
created: "2026-02-22"
updated: "2026-02-22"
---

# tldraw Reference Analysis

## Overview
tldraw is a production-grade React-based infinite canvas framework designed for multiplayer whiteboarding, diagramming, and increasingly, AI agent interactions. It provides a highly extensible `Editor` API, CRDT-based sync architecture, and built-in shape and tool primitives.

## Why Intake?
The Cortex and Nostra vision requires extensible and capable UI substrates for agent interaction. Current standard interfaces are primarily chat-based or linear document streams. `tldraw` provides proven patterns for integrating AI agents with a 2D spatial context (an infinite canvas), which directly impacts our `A2UI` (Agent-to-UI) rendering strategy. We need to evaluate whether injecting spatial capabilities into Cortex Web Architecture (`123-cortex-web-architecture`) provides the necessary leverage for complex agent orchestration.

## Placement
`research/reference/topics/ui-substrate/tldraw`

## Intent
Analyze the `tldraw` infinite canvas SDK to assess how its state management, extensibility, and "AI Agent primitive" patterns can benefit our A2UI/AGUI streaming components, enabling multi-modal agent interaction.

## Initiative Links
- `123-cortex-web-architecture`: Frontend component architecture.
- `115-cortex-viewspec-governed-ui-synthesis`: Dynamically generated UI components.
- `074-cortex-ui-substrate`: Core definitions of the UI.
- `014-ai-agents-llms-on-icp`: Multi-agent capabilities.

## Pattern Extraction
- **Spatial Agent UX:** Allows agents to output geometric shapes, arrows, flowcharts, and annotations onto a 2D plane instead of standard linear chat streams.
- **AI Canvas Primitives:** Built-in hooks and abstractions for LLMs to read, interpret, and modify the canvas state (`open-ag-ui-canvas` precedents).
- **CRDT State Sync:** Local-first, real-time collaboration (`@tldraw/sync`), deeply aligned with Nostra's local-first principles.
- **Runtime API:** The `Editor` API permits direct programmatic manipulation of the canvas, making it a perfect target for an A2UI payload interpreter.

## Possible Links To Nostra Platform and Cortex Runtime
- **A2UI Streaming Target:** An agent could stream `SurfaceUpdate` commands that map directly to `tldraw` shapes (e.g., `createShape`, `updateShape`).
- **Data Model Updates:** Integrating the canvas CRDT within our decentralized state architecture.

## Adoption Decision
**Recommendation:** Strong consideration for `A2UI` expansion.
- Integrate the concept of a "Canvas" or "SpatialPlane" into the A2UI widget registry.
- Study the `tldraw` state format to ensure our `viewspec` and `DataModelUpdate` payloads can map efficiently to spatial graphs.

## Known Risks
- Heavy dependency size and specific React bindings might conflict with framework-agnostic rendering if not carefully isolated.
- The state sync mechanism might overlap or conflict with Nostra's built-in synchronization models if we try to use `@tldraw/sync` directly instead of just the rendering SDK.
- Licensing/commercial constraints for production usage can change adoption economics and release policy for host products.

## Suggested Next Experiments
- Prototype an `A2UI` canvas widget using `tldraw` in `cortex-web`, enabling a test agent to emit simple topological maps or state diagrams.

## Validation: Quality and Usefulness
### Quality Verdict
**Validated with conditions.** The analysis is directionally strong and useful, but it was initially weighted toward technical capability and underweighted on product governance and operating constraints.

### Why It Is Useful To Our System
- It identifies a concrete leverage point for active initiatives (`074`, `115`, `123`): a spatial interaction substrate that can increase agent legibility versus linear chat streams.
- It supports `A2UI` evolution without requiring immediate protocol replacement: a new widget class can remain additive to existing contracts.
- It aligns with recommendation-only governance posture: we can evaluate in bounded experiments before promoting to default UI patterns.

### Gaps Found In The Original Analysis
- No explicit license/go-to-production decision gate, despite upstream production licensing requirements in `research/reference/topics/ui-substrate/tldraw/README.md`.
- No explicit host-parity strategy: `tldraw` is React-first, while Cortex requires cross-host semantics parity (`cortex-web` and `cortex-desktop`).
- No boundary between render-layer adoption and sync-layer adoption (risk of accidental duplication with Nostra/Cortex state and governance lineage).
- No kill criteria or escalation criteria if integration adds complexity without measurable operator value.

## Approach Validation
- **Recommended approach:** adopt `tldraw` as a **replaceable render adapter**, not as a canonical state authority.
- **Canonical authority remains Nostra/Cortex artifacts** (`A2UI`/`ViewSpec`/governance events). `tldraw` should consume compiled surface instructions, not define governance truth.
- **Sync policy:** do not adopt `@tldraw/sync` in phase 1. Start single-user/local render mode; evaluate sync only after governance and lineage requirements are satisfied.
- **Parity policy:** define a `Canvas`/`SpatialPlane` contract in `A2UI` first, then implement host-specific renderers so desktop/web remain semantically aligned.

## Implementation Plan (Next Steps)
### Phase 0: Decision Gate (1-2 days)
- Confirm legal/operational constraints for production usage and record decision in initiative artifacts (`123` and/or `DECISIONS.md`).
- Define success metric baseline for spatial UI experiment:
  - operator task completion quality
  - decision-surface legibility
  - added complexity cost (bundle/runtime)

### Phase 1: Contract-First Design (2-3 days)
- Add `Canvas` (or `SpatialPlane`) as an additive `A2UI` component contract aligned with initiative `115` validation rules.
- Define minimal command mapping:
  - `create_shape`
  - `update_shape`
  - `delete_shape`
  - `focus_bounds`
- Keep payload schema renderer-agnostic; avoid direct `tldraw` type leakage into canonical contracts.

### Phase 2: Web Adapter Prototype (3-5 days)
- Implement a gated widget in `cortex/apps/cortex-web/src/components/a2ui/WidgetRegistry.ts`.
- Add a dedicated adapter component (e.g., `cortex/apps/cortex-web/src/components/a2ui/TldrawCanvas.tsx`) to map `A2UI` commands to tldraw runtime calls.
- Integrate behind feature flag and recommendation-only route to avoid default-path regression.

### Phase 3: Governance + Integrity Hooks (2-3 days)
- Ensure canvas-originated actions emit existing governance/event metadata and do not bypass approval flows.
- Enforce surface classification boundaries so constitutional/governance controls are never spoofed by canvas visuals.
- Add deterministic replay fixture in `shared/a2ui/fixtures/` for one spatial scenario.

### Phase 4: Evaluation and Go/No-Go (1-2 days)
- Run a side-by-side operator exercise (linear surface vs spatial surface) for one initiative flow.
- Decide:
  - **Go:** measurable legibility/workflow improvement with bounded complexity.
  - **No-Go:** preserve reference-only status and keep concept as optional labs path.

### Exit Criteria
- `A2UI` schema remains backward compatible and renderer-agnostic.
- Spatial renderer does not become source of truth for governance state.
- Evidence shows improved operator usefulness, not just visual novelty.
- A clear rollback path exists (feature flag off, no schema breakage).
