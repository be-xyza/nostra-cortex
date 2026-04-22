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

## Suggested Next Experiments
- Prototype an `A2UI` canvas widget using `tldraw` in `cortex-web`, enabling a test agent to emit simple topological maps or state diagrams.
