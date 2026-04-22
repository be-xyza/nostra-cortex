---
id: tldraw-computer
name: tldraw-computer
title: tldraw Computer Pattern Analysis
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: cortex
authority_mode: recommendation_only
reference_topics: [ui-substrate, visualization, frontend, agent-systems, workflow-orchestration]
reference_assets:
  - "research/reference/topics/ui-substrate/tldraw"
  - "research/reference/analysis/tldraw_analysis.md"
evidence_strength: strong
handoff_target:
  - "UX Steward"
  - "Systems Steward"
authors:
  - "User"
  - "Codex"
tags: [ui, infinite-canvas, agents, spatial-computing, workbench, workflow]
stewardship:
  layer: "Capabilities"
  primary_steward: "UX Steward"
  domain: "Interfaces & Extensibility"
created: "2026-04-01"
updated: "2026-04-01"
---

# tldraw Computer Pattern Analysis

## Scope
This note expands the existing `tldraw` reference intake with a focused evaluation of the `computer` demos and the December 11, 2024 Google AI Studio case study. The goal is to validate whether the `computer` pattern should influence Nostra platform visualization and Cortex Workbench interaction design.

## Sources Reviewed
- `https://computer.tldraw.com`
- Shared demo URLs under `https://computer.tldraw.com/t/*`
- `https://aistudio.google.com/case-studies/tldraw`
- `research/reference/topics/ui-substrate/tldraw/README.md`
- `cortex/apps/cortex-web/src/components/a2ui/TldrawCanvas.tsx`
- `cortex/apps/cortex-web/src/components/a2ui/spatialMapper.ts`
- `research/074-cortex-ui-substrate/PLAN.md`
- `research/115-cortex-viewspec-governed-ui-synthesis/PLAN.md`
- `research/123-cortex-web-architecture/PLAN.md`
- `research/120-nostra-design-language/RESEARCH.md`
- `research/130-space-capability-graph-governance/PLAN.md`
- `research/136-cortex-explore-graph/PLAN.md`
- `research/097-nostra-cortex-alignment/IMPLEMENTATION_PLAN.md`
- `shared/standards/LOCAL_FIRST.md`
- `nostra/spec.md`

## What The `computer` Pattern Actually Is
The validated pattern is not "chat with a canvas skin". It is a spatial execution surface where language and multimodal inputs produce, modify, and connect visual program state directly on an infinite canvas.

From the official sources:
- tldraw positions the SDK as an infinite canvas engine with runtime editor APIs, custom shapes/tools/bindings, AI integrations, and optional multiplayer sync.
- tldraw ships starter kits for `agent`, `workflow`, `chat`, and `branching chat`, which confirms that "computer" is part of a broader product direction rather than a one-off demo.
- Google published an official case study on December 11, 2024 describing `computer` as "natural language computing" built on the tldraw canvas with Gemini, initially with Gemini 1.5 Flash and future iterations prototyped on Gemini 2.0 Flash.

## Adoption And Pattern Strength
Adoption is strong enough to treat this as a credible reference, not novelty:
- The pattern is backed by an official tldraw starter-kit ecosystem, not only a marketing video.
- Google chose it for an AI Studio case study, which is meaningful external validation of the interaction model.
- tldraw itself is already a major upstream UI substrate in the repo's reference set and in the broader React canvas ecosystem.

This is sufficient evidence for pattern intake. It is not sufficient evidence to make tldraw the canonical authority layer.

## Validation Of The Draft Analysis

### Supported
- The draft is correct that `computer` shifts interaction from linear chat toward spatial execution.
- The draft is correct that this aligns more naturally with Cortex Workbench than with purely document-style UI.
- The draft is correct that Nostra can add durability, lineage, and governance that the demo does not visibly emphasize.
- The draft is correct that `ViewSpec` / `A2UI` compilation is the right place to keep canonical control if we adopt this pattern.

### Corrections
- "Nodes are Cortex Workers/Agents and arrows mirror the Cortex Workflow Engine" is too literal. This is a strong analogy, not an existing one-to-one mapping in our contracts.
- "A2UI already perfectly supports real-time, block-level canvas modifications" is overstated. The current contract only supports a narrow `SpatialPlane` with `note` and `arrow` shapes plus `create/update/delete/focus`.
- "We have the basic rendering primitives" is only partially true. We have a good starting point, but not parity with `computer` interaction density.
- "Functionally superior primitives" is too strong. We have stronger governance and durability primitives, but not current visual or interaction parity.

## Current Repo Reality

### What We Already Have
- `cortex-web` already has an additive `SpatialPlane` widget path in `/Users/xaoj/ICP/cortex/apps/cortex-web/src/components/a2ui/WidgetRegistry.ts`.
- The web host already has a renderer-agnostic parity spec for `SpatialPlane` in `/Users/xaoj/ICP/research/123-cortex-web-architecture/SPATIAL_PLANE_DESKTOP_PARITY_SPEC.md`.
- The current web adapter can replay deterministic canvas commands into tldraw when available, with SVG fallback when unavailable.
- Contract tests already cover deterministic replay, mapper validation, and locked event names.

### What We Do Not Yet Have
- No `tldraw` dependency is declared in `/Users/xaoj/ICP/cortex/apps/cortex-web/package.json`, so the runtime import path in `TldrawCanvas.tsx` currently resolves to fallback mode unless that dependency is added.
- No node/port/procedure/group/branch/media/embed primitives exist in the current `SpatialPlane` contract.
- No persisted canvas layout model exists for this feature.
- No collaborative sync model exists for this feature.
- No governed bridge exists yet from canvas mutations to durable Nostra contributions or Cortex workflow artifacts.

## Visual And Functional Parity Assessment

### Visual Parity
Not yet.

Our present stack can render graphs and a basic spatial plane, but `computer` demonstrates a much richer canvas grammar:
- direct manipulation of spatial objects
- high-fidelity panning/zooming/selection
- visually legible branching structures
- mixed media and multimodal context
- canvas-native composition rather than graph-only projection

`react-force-graph-2d` and D3 remain useful for Explore/topology projections, but they are not by themselves a drop-in match for a tool-like spatial computing surface.

### Functional Parity
Partial substrate only.

We can already support:
- deterministic replay
- execution-surface gating
- event instrumentation
- host-parity discipline

We cannot yet match:
- canvas-native procedure authoring
- direct language-to-node graph transformation
- visual branching chat / branching workflows
- live multimodal manipulation on the canvas
- durable layout/history semantics for the spatial surface

## Alignment With Nostra And Cortex Principles

### Strong Alignment
- `/Users/xaoj/ICP/nostra/spec.md`: Nostra already treats time, simulation, and interactive processes as first-class, and explicitly names React/Vite, A2UI, Lit, and D3 as part of the host/protocol/visualization stack.
- `/Users/xaoj/ICP/research/097-nostra-cortex-alignment/IMPLEMENTATION_PLAN.md`: Flow graph derivation, lineage edges, and persisted layout are already on-record as Workbench goals.
- `/Users/xaoj/ICP/research/136-cortex-explore-graph/PLAN.md`: semantic zooming, persistent spatial memory, and intent-driven projections strongly reinforce this direction.

### Required Constraints
- `/Users/xaoj/ICP/research/120-nostra-design-language/RESEARCH.md`: the canvas must remain an `execution` surface. It cannot draw constitutional truth into existence.
- `/Users/xaoj/ICP/shared/standards/LOCAL_FIRST.md`: local-first is a capability, not authority. Local canvas edits must be replayable intent, not source-of-truth by themselves.
- `/Users/xaoj/ICP/research/130-space-capability-graph-governance/PLAN.md`: Space-specific surfacing still needs deterministic compilation and steward-gated structural mutation.

## Relevant Initiative Resolution

### Primary
- `/Users/xaoj/ICP/research/074-cortex-ui-substrate/PLAN.md`
  - Owns substrate hardening and host parity. This is the right place for renderer boundaries and feature-flag policy.
- `/Users/xaoj/ICP/research/115-cortex-viewspec-governed-ui-synthesis/PLAN.md`
  - Owns the governed artifact contract. This is where spatial surface primitives should become canonical, renderer-agnostic `ViewSpec` output.
- `/Users/xaoj/ICP/research/123-cortex-web-architecture/PLAN.md`
  - Owns the React/Vite Workbench host. This is where a tldraw adapter can exist as a web execution-host concern.

### Also Relevant Beyond The Usual Set
- `/Users/xaoj/ICP/research/120-nostra-design-language/PLAN.md`
  - Prevents governance spoofing. Any `computer`-like canvas must stay in execution mode.
- `/Users/xaoj/ICP/research/136-cortex-explore-graph/PLAN.md`
  - Covers semantic zoom and persistent spatial memory for graph reading and navigation.
- `/Users/xaoj/ICP/research/097-nostra-cortex-alignment/IMPLEMENTATION_PLAN.md`
  - Covers flow graph derivation, lineage edges, and layout persistence, which are direct building blocks for a native variant.
- `/Users/xaoj/ICP/research/130-space-capability-graph-governance/PLAN.md`
  - Ensures Space-level projection and placement remain deterministic and governed.
- `/Users/xaoj/ICP/research/112-cortex-runtime-convergence-and-live-collab/PLAN.md`
  - Relevant if the canvas begins as deterministic op-log collaboration.
- `/Users/xaoj/ICP/research/113-cortex-crdt-collaboration-governance/PLAN.md`
  - Relevant only if the canvas becomes a true concurrent editing surface that justifies CRDT cost.

## Recommended Native Strategy

### 1. Keep tldraw as an adapter, not authority
Use tldraw as an optional web renderer for an A2UI/ViewSpec-defined `SpatialPlane`, never as the canonical state or governance engine.

### 2. Expand the contract before expanding the dependency
The next missing layer is not "install tldraw". It is a richer renderer-agnostic contract:
- `procedure_node`
- `tool_node`
- `input_node`
- `output_node`
- `port`
- `edge`
- `group`
- `annotation`
- `media_embed`
- `branch`

### 3. Separate two families of spatial UI
- `Explore` graph:
  - read/navigation heavy
  - topology and semantic zoom
  - fits Initiative 136
- `Computer` / execution canvas:
  - authoring and manipulation heavy
  - fits Workbench execution surfaces
  - should not be collapsed into the Explore graph

### 4. Persist meaning, not raw UI diffs
Follow `LOCAL_FIRST.md`:
- persist layout and user intent as replayable commands or contribution/workflow artifacts
- do not make opaque tldraw document state the only durable representation

### 5. Delay sync-layer adoption
Do not adopt `@tldraw/sync` as a default path in phase 1. First prove value with single-user or deterministic op-log behavior. Evaluate CRDT only if multi-user concurrent editing becomes core to the workflow.

## Go / No-Go Recommendation
Go for a native Nostra/Cortex interpretation of the pattern.

Specifically:
- yes to the `computer` interaction model as a Cortex Workbench execution-surface reference
- yes to tldraw as a replaceable web adapter candidate
- no to making upstream tldraw state or sync the primary authority layer
- no to claiming parity until the contract grows beyond `note` and `arrow`

## Suggested Immediate Next Steps
1. Promote `SpatialPlane` from a two-shape experiment to a renderer-agnostic execution-canvas contract under Initiative 115.
2. Add persisted spatial layout and lineage bindings using the Workbench/flow-graph direction in Initiative 097.
3. Prototype a web-only adapter path in Initiative 123 with `tldraw` explicitly added as an optional dependency.
4. Keep Explore graph work separate under Initiative 136 so topology navigation and execution authoring do not collapse into one overloaded surface.
