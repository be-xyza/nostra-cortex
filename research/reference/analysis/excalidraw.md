---
id: excalidraw
name: excalidraw
title: Excalidraw
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: nostra
authority_mode: recommendation_only
reference_topics: [ui-substrate, visualization, frontend, agent-systems]
reference_assets:
  - "research/reference/topics/visualization/excalidraw"
evidence_strength: strong
handoff_target:
  - "UX Steward"
  - "Systems Steward"
authors:
  - "User"
  - "Agent"
tags: [ui, infinite-canvas, agents, frontend, collaboration]
stewardship:
  layer: "Capabilities"
  primary_steward: "UX Steward"
  domain: "Interfaces & Extensibility"
created: "2026-03-05"
updated: "2026-03-05"
---

# Excalidraw Reference Analysis

## Overview
Excalidraw is a virtual whiteboard for sketching hand-drawn like diagrams. It provides an infinite, canvas-based collaborative whiteboard with support for end-to-end encryption, local-first autosaving, and an open `.excalidraw` JSON format. It is highly customizable and accessible via an npm package.

## Why Intake?
The Cortex and Nostra vision requires extensible UI substrates for multi-modal agent interaction. Like `tldraw` (`tldraw_analysis.md`), `Excalidraw` provides an infinite canvas and spatial context for agents. Its `.excalidraw` JSON format is already prevalent in the ecosystem (used for architectural diagrams in `research/reference/topics/icp-core/portal/docs/`). Evaluating Excalidraw helps determine the best `A2UI` (Agent-to-UI) rendering strategy for spatial, diagrammatic outputs.

## Placement
`research/reference/topics/visualization/excalidraw`

## Intent
Analyze the `Excalidraw` npm package to assess its viability as an `A2UI` streaming component adapter. Evaluate its `.excalidraw` JSON state format for compatibility with Cortex's CRDT-based Decentralized State Architecture and agent payloads.

## Initiative Links
- `123-cortex-web-architecture`: Frontend component architecture.
- `115-cortex-viewspec-governed-ui-synthesis`: Dynamically generated UI components.
- `074-cortex-ui-substrate`: Core definitions of the UI.
- `014-ai-agents-llms-on-icp`: Multi-agent capabilities.

## Pattern Extraction
- **Spatial Agent UX:** Allows agents to output architectural diagrams, flowcharts, and sketches.
- **Open Format:** The `.excalidraw` JSON format is highly structured and widely supported, providing a clear target for LLM generation.
- **Collaborative Engine:** Built-in real-time collaboration features that align with Nostra's local-first and collaborative principles.

## Possible Links To Nostra Platform and Cortex Runtime
- **A2UI Streaming Target:** An agent could stream `SurfaceUpdate` commands that map to Excalidraw elements.
- **Data Model Updates:** Integrating Excalidraw's state into the Cortex CRDT mapping.

## Adoption Decision
**Recommendation:** Strong consideration for `A2UI` expansion, side-by-side with `tldraw`.
- The `.excalidraw` JSON format is already used by Nostra engineers for architectural diagrams; enabling agents to natively read/write these diagrams provides immediate utility.

## Known Risks
- Heavy dependency size.
- Potential overlap with `tldraw` evaluations. We must choose one primary infinite canvas substrate for the core `A2UI` widget registry, or support both via a generalized `SpatialPlane` contract.

## Suggested Next Experiments
- Prototype an `A2UI` widget using the `@excalidraw/excalidraw` npm package.
- Feed an existing `.excalidraw` architecture diagram to an agent and have it propose modifications visually.
