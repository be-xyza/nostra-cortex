---
id: "124C"
name: "polymorphic-heap-mode-behaviors"
title: "Research: Enhancing Blocks with Contextual Behaviors"
type: "research"
project: "nostra"
status: active
authors:
  - "User"
  - "Antigravity"
tags: ["cortex", "ui", "ux", "polymorphic-blocks", "architecture"]
created: "2026-02-25"
updated: "2026-02-25"
---

# Research: Enhancing Blocks with Contextual Behaviors

Following the adoption of the `attributes` primitive for static classification (e.g., `location: Zoom`, `media_type: podcast`), a secondary gap emerges in the Universal Polymorphic Block schema regarding **contextual UX state**.

## 1. The Gap Analysis: Attributes vs. Behaviors

`attributes` are excellent for querying and Policy Evaluation ("What is this block?"). However, they are distinct from contextual UI toggles ("How should this block act right now?").

Consider common workspace actions:
*   A user "Pins" a note to the top of their workspace.
*   A user "Mutes" an annoying SIQ score violation block temporarily.
*   A user "Collapses" a large UI widget.
*   A block is marked "Read-Only" by a governance policy.

Currently, the schema has no standard place to store these boolean-like states. Storing them in `structured_data` hides them from the UI renderer, and mixing them into `attributes` pollutes the semantic classification taxonomy with ephemeral UI state.

## 2. The Proposed Solution: `block.behaviors`

We recommend introducing a `behaviors` array of strings to the `HeapBlockMeta` envelope.

```json
"block": {
  "id": "...",
  "type": "note",
  "title": "Weekly Sync",
  "attributes": { "location": "Zoom" },
  "behaviors": ["pinned", "collapsed", "read-only"]
}
```

### UX Capabilities Unlocked:

1.  **Thematic Priority**: The UI Substrate (074) can elevate blocks with the `pinned` behavior without deeply parsing their content.
2.  **Notification Control**: Blocks with the `muted` behavior can suppress `GlobalEvent` emission or visual flashing.
3.  **Rendering Hints**: `collapsed` or `expanded` behaviors instruct the renderer on default mount state.
4.  **Temporal/Ephemeral State**: Behaviors are easily added or removed via quick A2UI or CRDT mutations without disrupting the block's core semantic `attributes`.

## 3. Mapping the Ontology

To ensure consistency, we recommend a controlled vocabulary for default behaviors (though extensible by agents):

*   **Visibility**: `pinned`, `hidden`, `collapsed`, `expanded`
*   **Interaction**: `read-only`, `disabled`, `locked`
*   **Attention**: `muted`, `urgent`, `unread`

## 4. Optimal Path Forward

This addition completes the Block envelope's capability to drive the Cortex UI Substrate seamlessly.

1.  **Schema Update**: Append `"behaviors": { "type": "array", "items": { "type": "string" } }` to `EMIT_HEAP_BLOCK.schema.json` under the `block` properties.
2.  **Implementation Plan**: Include this update in **Phase 5** alongside the `attributes` primitive rollout. The UI mappers and formatters should be updated to handle both fields simultaneously.
