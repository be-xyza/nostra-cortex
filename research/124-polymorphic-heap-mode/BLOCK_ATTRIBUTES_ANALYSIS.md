---
id: "124B"
name: "polymorphic-heap-mode-attributes"
title: "Research: Enhancing Polymorphic Blocks with Attributes"
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

# Research: Enhancing Polymorphic Blocks with Attributes

This document evaluates the necessity and architectural impact of adding an explicit **Attributes** primitive to the Universal Polymorphic Block (`EmitHeapBlock`) schema.

## 1. The Current State & Gap Analysis

Currently, the `EmitHeapBlock` schema supports 5 `payload_type` variants (`a2ui`, `rich_text`, `media`, `structured_data`, `pointer`). It classifies blocks via two primarily visual/structural fields:
*   `block.type` (e.g., "note", "widget", "task")
*   `relations` (`tags`, `mentions`)

**The Gap:**
If a User or Agent creates a `rich_text` block representing a "Meeting Note", how do we classify the specific properties of that meeting (e.g., `location: "Zoom"`, `duration: "60m"`)?
Currently, it would require burying that data inside a `structured_data` block, preventing it from being attached to a block whose primary payload is `rich_text` or `a2ui`.

Furthermore, the core `cortex-domain::graph::Node` struct (used by the Invariant Engine) relies heavily on a flat key-value map for its policy selectors:
```rust
pub struct Node {
    pub id: String,
    pub node_type: String,
    pub attributes: BTreeMap<String, String>, // <-- Currently unpopulated by Heap Blocks
}
```

## 2. The Proposed Solution: `block.attributes`

Adding a flat key-value `attributes` primitive to the `HeapBlockMeta` envelope solves this gap elegantly.

```json
"block": {
  "id": "...",
  "type": "meeting",
  "title": "Weekly Sync",
  "attributes": {
    "location": "Zoom",
    "behavior": "recurring",
    "preference": "recorded"
  }
}
```

### Why not just use `structured_data`?
`structured_data` is a **content payload execution type**. If a block is primarily an `a2ui` widget (like our SIQ Scorecard), its `payload_type` is `a2ui`. The `attributes` map operates orthogonally to the payload, acting as a universal indexing and classification primitive for the *wrapper* block.

## 3. UI/UX & Functional Benefits

1.  **Semantic Enrichment & Filtering**: Users can filter their workspace not just by tags, but by attributes. "Show me all `type: media` blocks where `attribute.media_type = video` and `attribute.person = UserA`."
2.  **A2UI Theming & Layout**: The UI Substrate (074) can use attributes to dynamically apply visual tokens. `attributes: { priority: "high", status: "blocked" }` could drive a red border around a `rich_text` block in the workspace.
3.  **Governance & Invariants**: The Cortex Invariant Engine can execute OPA/Rego policies directly against block attributes. For example, a policy could mandate: *"Any block with `type: process` MUST possess an `attribute.owner`."*

## 4. Optimal Path Forward

This addition is strictly beneficial and highly recommended. The implementation acts as **Phase 5** of the overall Polymorphic Block rollout:

1.  **Schema Update**: Append `"attributes": { "type": "object", "additionalProperties": { "type": "string" } }` to `EMIT_HEAP_BLOCK.schema.json` under the `block` property.
2.  **Domain Mapping**: In `cortex-desktop`, when mapping an `EmitHeapBlock` into the CRDT stream, pipe the `block.attributes` directly into the `Node.attributes` of the `block_graph`.
3.  **Runtime**: Ensure `format_siq_as_heap_block` (and other agent formatters) can populate this map.
