---
id: '005'
name: nostra-design
title: 'Research Initiative 005: Cortex Visual Orchestration Surface (CVOS)'
type: general
project: nostra
status: archived
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-31'
---

# Research Initiative 005: Cortex Visual Orchestration Surface (CVOS)

## 1. Core Objective
To define the implementation of the **Cortex Visual Orchestration Surface (CVOS)**. This is not a standard web UI, but a high-performance **Visual Operating System** for inspecting and governing the Nostra Contribution Graph.

## 2. Hypothesis: The "Rust Sandwich"
We hypothesize that standard DOM/SVG renderers cannot handle the scale (10k+ nodes) or distinct visual effects (SDF, Time Halos) required for a "Sovereign Personal OS". We propose a "Rust Sandwich" architecture that separates the Surface Engine from the Chrome.

## 3. The Stack Defintion

### Plane 1: The Surface Engine (CVOS Core)
*   **Objective**: Rendering, World Model, Spatial Indexing.
*   **Tech Stack**:
    *   **Renderer**: `wgpu` (WebGPU) + `SDF` (Signed Distance Fields).
    *   **Logic**: `bevy_ecs` (Entity Component System).
    *   **State**: `Loro` (CRDT) for Draft/Collaboration layers.
*   **Validation**: Infinite zooming, shader interactions, independent logic systems (Physics, Layout).

### Plane 2: The Chrome (Controls)
*   **Objective**: Buttons, Forms, Chat, Governance Overlays.
*   **Tech Stack**: `Dioxus` (App Shell) + `Shoelace` (Web Components).
*   **Interaction**: The Chrome talks to the Surface via an Event Bridge; it does not render the graph itself.

## 4. Implementation Study
*   **Goal**: Prove the stack with a "Vertical Slice" prototype.
*   **Scope**: Render 10,000 "Agent Nodes" with:
    1.  SDF-based shapes (Hexagons).
    2.  Zoom-dependent Level of Detail (LOD).
    3.  Loro-backed "Draft Move" interactions.
*   **Success Metric**: 60fps at 10k nodes with sub-16ms layout updates.

## 5. Layout Strategy
*   **Local**: Use `Yoga` (Flexbox) for "Text-to-Diagram" automated layouts.
*   **Global**: Use Force-Directed or Layered DAG algorithms for the world view.

## 6. Resources
*   See `infinite-canvas-tutorial` for WebGPU/ECS implementation details.
*   See `wgpu-rs` examples for Compute Shader integration.

## 7. Governance Physics
*   **Protocol Definition**: See `ACTOR_LEVELS.md` for the immutable "Dharma" protocols that govern the relationship between Cool Mind (Execution) and Warm Heart (Experience). This defined the "Physics" of the graph nodes.
