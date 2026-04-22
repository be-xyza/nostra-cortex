---
id: tldraw
name: tldraw
title: tldraw SDK and tldraw computer
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: cortex
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
tags: [ui, infinite-canvas, agents, workflow, frontend, crdt, viewspec]
stewardship:
  layer: "Capabilities"
  primary_steward: "UX Steward"
  domain: "Interfaces & Extensibility"
created: "2026-02-22"
updated: "2026-04-01"
---

# tldraw Reference Analysis

## Overview
tldraw is the strongest current reference in our corpus for infinite-canvas AI interaction. The official docs and starter kits now cover three patterns directly relevant to Nostra Cortex:

- canvas as output
- visual workflows
- agent-driven canvas manipulation

The `computer` use case is not just "AI on a whiteboard". It combines canvas-native interaction, live multimodal reasoning, and workflow-like state transformation in a way that is materially closer to Cortex Workbench than to a conventional chat UI.

## Source-Validated Signals
### Official product and ecosystem signal
- tldraw positions AI integration as a first-class SDK capability, with official docs for canvas output, visual workflows, and agent manipulation.
- tldraw ships official starter kits for `agent`, `workflow`, `chat`, and `branching-chat`, which is a strong signal that these are not one-off demos but reusable product patterns.
- The public tldraw site highlights production adoption, including ClickUp replacing legacy whiteboard infrastructure with the tldraw SDK, and shows broad community traction.

### `computer` signal
- Google AI's tldraw showcase, published on December 11, 2024, describes `computer` as a "natural language computing" experience built with Gemini and the tldraw canvas SDK.
- The case study explicitly frames `computer` as a new interaction model, not just a drawing app with chat bolted on.
- The official tldraw AI docs match that framing: the same repo and docs expose workflow and agent starter kits that cover the primitives `computer` depends on.

## What `computer` Actually Demonstrates
### Pattern layer
- Spatial-first interaction instead of linear thread-first interaction.
- Nodes, bindings, and proximity as part of meaning, not just decoration.
- Multimodal context gathering from both screenshot-like visual context and structured canvas state.
- Live, typed canvas actions instead of unstructured DOM mutation.

### Runtime layer
- Agent reads current viewport, selection, shape structure, and session history.
- Agent applies validated actions to a live editor.
- Workflow graphs can represent operations, data sources, and transformations with ports and connections.
- Execution can be incremental and reactive rather than full-screen request/response.

### UX layer
- Users can branch, compare, rearrange, and annotate work directly on the same surface.
- The canvas acts as working memory.
- The interface supports a mixed mode: authored structure, direct manipulation, and AI assistance.

## Alignment With Nostra and Cortex
### Strong alignment
- `074-cortex-ui-substrate`: validates spatial interaction as a real substrate concern, not a novelty widget.
- `115-cortex-viewspec-governed-ui-synthesis`: maps cleanly to a governed `ViewSpec -> SpatialPlane` compile target.
- `123-cortex-web-architecture`: fits the React host and adapter strategy already in place.
- `136-cortex-explore-graph`: reinforces semantic zoom, persistent spatial memory, and action-capable graph views.

### Conditional alignment
- `112-cortex-runtime-convergence-and-live-collab`: supports durable execution and deterministic replay, but `computer`-style editing needs richer canvas state and collaboration semantics than the current experiment.
- `113-cortex-crdt-collaboration-governance`: relevant if we ever want shared spatial editing, but tldraw's sync layer should not become canonical authority by default.

### Principle alignment
- Boundary-first: Nostra should define the canonical artifacts, permissions, lineage, and space-scoped meaning; Cortex should execute and render them.
- Host neutrality: tldraw is acceptable only as a replaceable render adapter, not as protocol truth.
- Deterministic projections: the same governed input must compile to the same spatial result.
- Recommendation-only default: AI-generated spatial changes should remain proposal/staging flows until explicitly promoted.

## Current Repo Validation
### What exists today
- `cortex-web` already has a gated `SpatialPlane` widget routed through the A2UI registry.
- The web host has a `TldrawCanvas` adapter that dynamically imports `tldraw` at runtime and falls back to SVG if the package is unavailable.
- The replay contract is renderer-agnostic and parity-scoped through the desktop parity spec.
- The evaluation harness already measures linear vs spatial interaction outcomes and persists experiment events.

### What is explicitly missing today
- `tldraw` is not an installed dependency in `cortex-web`.
- The canonical `SpatialPlane` contract only supports `note` and `arrow` shapes.
- The command model only supports `create_shape`, `update_shape`, `delete_shape`, and `focus_bounds`.
- There is no native concept of node ports, bindings, connection handles, region grouping, execution graph resolution, or canvas-side agent loop.
- There are no tests covering actual `tldraw` runtime mounting, custom shape registration, or workflow-node behavior.

## Visual and Functional Parity Assessment
### Where Cortex/Nostra already has leverage
- Better authority model than the demo: governed artifacts, approval flows, lineage, and space scoping.
- Better execution posture than the demo: Cortex can route actions into durable workflows and runtime services.
- Better host-governed contract discipline: `SpatialPlane` is already renderer-agnostic instead of vendor-shaped.

### Where parity is not present yet
- No node-and-wire interaction model.
- No custom shape system for operator tools, prompts, models, procedures, or output ports.
- No binding layer that keeps connections attached as nodes move.
- No canvas-native execution engine or dependency resolver comparable to the workflow starter kit.
- No agent action loop that reads viewport/selection and streams multi-step canvas edits.
- No persistent spatial memory contract for user-authored layouts.
- No multi-user spatial collaboration semantics.

### Bottom line
We do not currently have visual or functional parity with `computer`. We have an early substrate experiment with the correct architectural direction but only a thin slice of the required primitives.

## Native Nostra/Cortex Comparison
### Where a native integration can be better
- Governance and lineage can be first-class instead of bolted on.
- Canvas changes can compile from governed `ViewSpec` and workflow artifacts instead of living only in client-local editor state.
- Spatial work can be attached to Spaces, Contributions, proposals, approvals, and durable workflow traces.
- Operator-only and constitutional surfaces can keep existing authority boundaries.

### Where a native integration will initially be worse
- Interaction smoothness and breadth of built-in canvas tooling will lag tldraw unless we either adopt it as a renderer or invest heavily in a custom canvas substrate.
- Node editing, snapping, selection UX, and custom binding behavior are already solved by tldraw and not yet solved in Cortex.

## Adoption Decision
### Decision
Adopt tldraw as a pattern source immediately and as an optional web render adapter in labs. Do not adopt tldraw state or sync as canonical authority.

### Why
- This preserves boundary correctness.
- It keeps the existing `SpatialPlane` contract valid.
- It allows us to pursue high-value parity in the web host without committing Nostra or Cortex authority to a vendor-specific state model.

## Recommended Path
### Phase 1: Contract enrichment
- Extend `SpatialPlane` above simple shapes to support nodes, ports, bindings, regions, and execution status.
- Keep the canonical schema renderer-agnostic.

### Phase 2: Web adapter experiment
- Install and gate `tldraw` in `cortex-web`.
- Implement custom node and binding shapes behind feature flags.
- Compare SVG fallback, tldraw adapter, and existing graph views on one operator workflow.

### Phase 3: Native governance integration
- Compile governed `ViewSpec` and workflow artifacts into spatial projections.
- Keep human approval and lineage metadata on promotion paths.
- Treat AI-generated spatial edits as staged proposals, not automatic truth.

### Phase 4: Collaboration and persistence
- Add persisted layout memory and spatial replay.
- Evaluate whether collaboration needs CRDT semantics or whether deterministic op-log remains sufficient for the first production path.

## Kill Criteria
- If tldraw remains web-only and blocks host parity.
- If license or production key constraints are unacceptable.
- If richer spatial UI does not materially improve operator task quality over linear surfaces.
- If adapter complexity starts leaking tldraw-specific concepts into canonical contracts.

## Validated Conclusion
The draft analysis was directionally right on strategic fit and wrong on current parity. The optimal path is not "replace D3 with tldraw" and not "copy the demo". The optimal path is:

1. keep Nostra artifacts and Cortex workflow/runtime as the authority model
2. enrich the canonical spatial contract until it can express node/workflow semantics
3. use tldraw as the fastest credible web adapter for labs and parity exploration
4. promote only the parts that survive governance, parity, and usefulness checks
