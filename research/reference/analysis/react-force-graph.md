---
id: react-force-graph
name: react-force-graph
title: React Force Graph
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: cortex
authority_mode: recommendation_only
reference_topics: [visualization, ui-substrate, frontend]
reference_assets:
  - "research/reference/topics/visualization/react-force-graph"
evidence_strength: strong
handoff_target:
  - "UX Steward"
  - "Systems Steward"
authors:
  - "User"
  - "Agent"
tags: [graph, visualization, react, force-directed, d3, canvas, webgl, background]
stewardship:
  layer: "Capabilities"
  primary_steward: "UX Steward"
  domain: "Interfaces & Extensibility"
created: "2026-03-14"
updated: "2026-03-14"
---

# React Force Graph — Reference Analysis

## Overview

[react-force-graph](https://github.com/vasturiano/react-force-graph) provides React bindings for 4 stand-alone force-directed graph components:

| Package | Renderer | Engine |
|---------|----------|--------|
| `react-force-graph-2d` | HTML Canvas | d3-force-3d |
| `react-force-graph-3d` | WebGL / ThreeJS | d3-force-3d |
| `react-force-graph-vr` | A-Frame (VR) | d3-force-3d |
| `react-force-graph-ar` | AR.js (AR) | d3-force-3d |

**Key stats:** ~2.9k dependents, MIT license, actively maintained by vasturiano.
All packages share an identical API surface covering data input, node/link styling, custom rendering (canvas/ThreeJS), force engine tuning, interaction events (click, hover, drag, zoom), DAG modes, and render lifecycle hooks.

## Why Intake?

The user is exploring `react-force-graph-2d` as a **background graph renderer** for the Cortex Workbench — a soft, ambient visualization of the Space/Workbench contribution graph that renders behind primary content. This is a distinct use case from the existing `ForceGraph.tsx` (which uses raw d3 SVG for the Capability Map interactive view).

### Current State in cortex-web

| Existing | Technology | Notes |
|----------|-----------|-------|
| `ForceGraph.tsx` | d3 v7 + SVG | Interactive contribution graph (foreground focus view) |
| d3@^7.9.0 | Direct dependency | Already in `package.json` |
| `/api/kg/spaces/.../contribution-graph/graph?mode=d3-force` | API endpoint | Serves `{ nodes, links }` data in d3-force format |

## Placement

`research/reference/topics/visualization/react-force-graph`

Topic fit: `visualization` topic already contains `cytoscape.js`, `d3`, `excalidraw`, and `infinite-canvas-tutorial`.

## Intent

Evaluate `react-force-graph-2d` as an ambient background graph renderer for Cortex Workbench, and as a potential replacement/complement to raw d3 SVG for interactive graph views.

## Initiative Links

- `123-cortex-web-architecture`: Frontend component architecture
- `124-polymorphic-heap-mode`: Heap block persistence and graph visualization
- `074-cortex-ui-substrate`: Core UI substrate definitions

## Feasibility Assessment: Background Graph Renderer

### ✅ Strongly Feasible

| Dimension | Assessment | Rationale |
|-----------|-----------|-----------|
| **React compatibility** | ✅ Native | First-class React component, drop-in to cortex-web (React/Vite) |
| **Data compatibility** | ✅ Direct | Expects `{ nodes: [], links: [] }` — identical to existing API output |
| **Canvas rendering** | ✅ Excellent for background | 2D variant renders to HTML Canvas, which layers naturally behind DOM content via CSS z-index and `pointer-events: none` |
| **Performance (2D)** | ✅ Strong | Canvas-based rendering is significantly faster than SVG for large graphs; d3-force handles 1000+ nodes fluidly |
| **Opacity / styling** | ✅ Built-in | `nodeCanvasObject`, `linkCanvasObject`, `backgroundColor`, and alpha channel control enable soft/ambient rendering |
| **Interaction disable** | ✅ Props | `enableNodeDrag={false}`, `enableZoomInteraction={false}`, `enablePanInteraction={false}`, `enablePointerInteraction={false}` — all background-friendly |
| **Animation control** | ✅ `cooldownTicks` / `warmupTicks` | Can pre-compute layout then freeze, or run soft continuous animation |
| **Bundle size (2D)** | ⚠️ ~60KB gzipped | Adds `force-graph` + `d3-force-3d` (though d3-force already present) |

### Background Renderer Architecture Sketch

```
┌─────────────────────────────────────────┐
│  Cortex Workbench Container             │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │  z-index: 0  │  ForceGraph2D     │  │
│  │  opacity: 0.08-0.15              │  │
│  │  pointer-events: none            │  │
│  │  position: absolute, inset: 0    │  │
│  │  (ambient graph background)      │  │
│  └───────────────────────────────────┘  │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │  z-index: 1  │  Primary Content  │  │
│  │  (Heap blocks, Canvas, etc.)     │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

Key props for background mode:
```tsx
<ForceGraph2D
  graphData={spaceGraph}
  width={containerWidth}
  height={containerHeight}
  backgroundColor="transparent"
  nodeColor={() => 'rgba(148, 163, 184, 0.12)'}
  linkColor={() => 'rgba(148, 163, 184, 0.06)'}
  nodeLabel={null}
  enableNodeDrag={false}
  enableZoomInteraction={false}
  enablePanInteraction={false}
  enablePointerInteraction={false}
  cooldownTicks={100}     // freeze after layout converges
  onEngineStop={() => {}} // no-op after convergence
/>
```

### Performance Considerations

| Factor | Impact | Mitigation |
|--------|--------|------------|
| Canvas redraw frequency | Medium | `cooldownTicks={100}` freezes simulation after convergence; `pauseAnimation()` after stop |
| Node count > 500 | Low | Canvas handles thousands; use `nodeVal` for size-based filtering |
| Memory | Low | Single canvas element vs. hundreds of SVG DOM nodes |
| CPU (background animation) | Negligible once frozen | Simulation stops after `cooldownTicks`; only redraws on resize |

### 3D Variant Considerations

The `react-force-graph-3d` (WebGL/ThreeJS) variant could create a stunning depth effect but:
- Significantly heavier bundle (~300KB+ gzipped with ThreeJS)
- GPU resource competition with primary content
- Overkill for ambient background; 2D canvas is the principled choice

## Possible Links To Nostra Platform and Cortex Runtime

- **Background Graph Layer**: Ambient visualization of Space contribution graph topology
- **Heap View Enhancement**: Soft graph context behind masonry block layout
- **Contribution graph navigation**: Interactive mode could replace current SVG-based `ForceGraph.tsx` with better performance at scale
- **Agent topology visualization**: Workflow/agent graphs rendered via the same component

## Pattern Extraction

1. **Canvas-over-SVG for graph rendering** — Canvas provides better performance for large graphs and natural layering for background use
2. **Declarative interaction toggling** — Boolean props to disable all interaction make background mode trivial
3. **d3-force compatibility** — Same physics engine, direct data format compatibility with existing API
4. **Custom node/link rendering** — `nodeCanvasObject` callback enables full visual customization without forking

## Adoption Decision

**Recommendation: Accept intake as reference. Strong candidate for background graph renderer.**

The library directly addresses the user's design intent with minimal integration friction:
- Native React component into existing React/Vite stack
- Identical data format to existing API (`{ nodes, links }`)
- Canvas rendering is architecturally correct for background layering
- All interaction can be declaratively disabled for ambient mode
- d3-force physics engine already in the dependency tree

### vs. Current ForceGraph.tsx (Raw D3 SVG)

| Dimension | Current (d3 SVG) | react-force-graph-2d |
|-----------|-----------------|---------------------|
| Renderer | SVG (DOM nodes per element) | HTML Canvas (single element) |
| Performance at 500+ nodes | Degrades | Stable |
| React integration | Manual `useEffect` lifecycle | Native React component |
| Background layering | Requires complex CSS | Natural (single canvas + transparent bg) |
| Interaction control | Manual event handling | Declarative props |
| Bundle impact | Already present (d3) | +~60KB (force-graph wrapper) |
| Custom rendering | SVG manipulation | Canvas API callbacks |

## Scorecard

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| ecosystem_fit | 4 | React-native, d3-force based — aligns with existing stack |
| adapter_value | 4 | Drop-in React component with identical data format to existing API |
| component_value | 5 | Directly usable as background renderer and potential graph view upgrade |
| pattern_value | 4 | Canvas rendering, declarative interaction control, lifecycle hooks |
| ux_value | 5 | Ambient graph background directly addresses user's design vision |
| future_optionality | 4 | 3D/VR/AR variants provide future experiential upgrades |
| topic_fit | 5 | Fits squarely in existing `visualization` topic |

**Total value score: 26/30** (well above 12 threshold)

## Known Risks

- **Bundle size**: +~60KB for 2D variant on top of existing d3; manageable but not zero
- **Rendering overlap**: Must ensure background canvas doesn't cause unnecessary repaints when content above changes
- **d3 version alignment**: Uses `d3-force-3d` (vasturiano's fork) vs. standard `d3-force`; packages are compatible but worth monitoring
- **Upstream dependency**: Single maintainer (vasturiano); high quality but single point of failure

## Suggested Next Experiments

1. **Prototype ambient background**: Add `react-force-graph-2d` to cortex-web, wire to existing contribution graph API, render at ~10% opacity behind Heap view
2. **Performance benchmark**: Compare canvas-based ForceGraph2D vs. current SVG-based ForceGraph.tsx with 200, 500, 1000 node graphs
3. **Interaction hybrid mode**: Evaluate using react-force-graph-2d for both background (interactions disabled) and foreground interactive view (replacing current ForceGraph.tsx)
4. **Theme integration**: Map node/link colors to Cortex design tokens for coherent ambient aesthetics
