---
id: '030'
name: artifacts-editor
title: 'Decisions Log: Artifacts Editor'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-25'
---

# Decisions Log: Artifacts Editor

## 001: Rich Text Storage Format
**Date**: 2026-01-17
**Status**: ACCEPTED

### Context
We need a storage format for rich text (Reports, Essays, Descriptions) that supports:
1.  **Graph Linking**: Strong references to other Contributions/Entities.
2.  **Versioning**: Granular tracking of changes.
3.  **AI Reasoning**: Structure that Agents can parse reliably.
4.  **Deterministic Execution**: Canister safety.

### Options Considered
1.  **Raw Markdown String**: Store `String`, parse on render.
    *   *Pros*: Standard, easy import/export.
    *   *Cons*: Opaque to the graph. "Mentions" are just text. parsing is expensive on the canister.
2.  **HTML Blobs**: Store sanitized HTML.
    *   *Pros*: Browser native.
    *   *Cons*: Hard to version, hard for AI to reason about structure, security risk on-chain.
3.  **Structured Blocks (NostraBlock)**: Custom AST (Abstract Syntax Tree).
    *   *Pros*: Semantically rich, graph-native, deterministic, efficient.
    *   *Cons*: Requires custom editor logic, conversion step.

### Decision
We will use **Option 3: Structured Blocks (NostraBlock)** as the canonical storage format.

*   **Logic**: "Everything is a Contribution" implies that even a paragraph or a link should be addressable in the Graph. Raw text hides this structure.
*   **Implementation**: We will use `pulldown-cmark` (WASM/Rust) to lower Markdown inputs into this AST.
*   **Consequences**: The Artifacts Editor must support a "View Source" mode that regenerates Markdown from the AST, but the AST is the source of truth.

## 002: Markdown Parser Selection
**Date**: 2026-01-17
**Status**: ACCEPTED

### Context
We need a robust Markdown parser that runs in WASM (Frontend) and Rust (Backend Canister) deterministically.

### Options Considered
1.  **pulldown-cmark**: Rust-native, stream-based, CommonMark compliant.
2.  **comrak**: GitHub flavored, but heavier AST allocation.
3.  **markdown-it**: JS only (not safe for backend).

### Decision
We will use **pulldown-cmark**.
*   **Reason**: It is "WASM-safe" and "Deterministic", critical for canister consensus. It avoids the heavy DOM allocation of Comrak, fitting better within canister memory limits.

## 003: A2UI Rendering Strategy (Efficiency)
**Date**: 2026-01-17
**Status**: ACCEPTED

### Context
Users expressed concern that rendering complex Rich Text via A2UI (Agent-to-User Interface) might be inefficient if it requires decomposing documents into thousands of primitive widgets (Rows, Columns, Text).

### Options Considered
1.  **Primitive Decomposition**: Convert `NostraBlock` AST -> Tree of A2UI `Text/Layout` widgets.
    *   *Pros*: Pure A2UI.
    *   *Cons*: Massive JSON payload size, serialization overhead, slow rendering.
2.  **Native Widget Wrapper**: Define a single `NostraRichText` widget in the A2UI Registry.
    *   *Pros*: Payload contains the compact AST. Rendering is done by a native, optimized compiled component.
    *   *Cons*: Requires the client to have the `NostraRichText` component pre-installed (it is not "zero-shot").

### Decision
We will use **Option 2: Native Widget Wrapper**.
*   **Rationale**: Efficiency is paramount for large artifacts. The `NostraRichText` renderer is a core system component, so it is acceptable to bake it into the standard client registry. Agents will emit a single component call, passing the content as data, rather than constructing the UI tree manually.

## 004: CRDT Strategy for Collaborative Editing
**Date**: 2026-01-29
**Status**: ACCEPTED

### Context
For real-time collaborative editing (see `STUDY_COLLABORATIVE_EDITING.md`), we need a CRDT implementation. Research into 2025 production patterns revealed trade-offs between Yjs and Automerge 2.0.

### Research Findings
| Factor | Yjs | Automerge 2.0 |
|:---|:---|:---|
| **Text Performance** | Superior (optimized for rich text) | Good |
| **JSON Model** | Manual mapping required | Native JSON-like |
| **Memory Efficiency** | Lower (default garbage collection) | Higher (retains full history) |
| **WASM Support** | Yes | Yes (Rust/WASM rewrite) |
| **Awareness Protocol** | Built-in cursor presence | Requires custom implementation |

### Options Considered
1.  **Yjs Only**: Use Yjs for all collaborative data.
    *   *Pros*: Best text performance, mature ecosystem, built-in awareness.
    *   *Cons*: Mapping NostraBlock structure to `Y.Map/Y.Array` is complex.
2.  **Automerge Only**: Use Automerge for all collaborative data.
    *   *Pros*: Native JSON model fits NostraBlock structure.
    *   *Cons*: Text performance inferior for large documents.
3.  **Hybrid Approach**: Yjs for inline text, Automerge for block structure.
    *   *Pros*: Best of both worlds—fast text editing + clean block operations.
    *   *Cons*: Two libraries to maintain, synchronization complexity.

### Decision
We will use **Option 3: Hybrid Approach**:
*   **Yjs `Y.Text`** for paragraph content (`InlineText` spans within `NostraBlock::Paragraph`)
*   **Automerge** for block-level operations (reordering, inserting, deleting blocks)
*   **Yjs Awareness Protocol** for cursor presence and user indicators

**Rationale**:
1.  Yjs excels at character-level text editing where users type rapidly.
2.  Automerge's JSON model elegantly represents `Vec<NostraBlock>` for structural changes (which occur less frequently).
3.  This separation aligns with our existing block vs. inline distinction in the data model.

### Consequences
1.  **Two CRDT States**: Each document has a Yjs doc (for text) and an Automerge doc (for structure).
2.  **Sync Coordination**: On save, both states must be atomically committed.
3.  **Performance Target**: <100ms sync latency for typical editing operations.

## 005: Block-Level vs Document-Level Sync Resolution
**Date**: 2026-01-29
**Status**: ACCEPTED

### Context
Open Question #1 in `STUDY_COLLABORATIVE_EDITING.md`: Should we sync the *entire document* or *individual blocks*?

### Decision
**Threshold-Based Hybrid Sync**:
*   **Document-Level Sync** for documents with fewer than 20 blocks (typical notes, comments)
*   **Block-Level Sync** for documents with 20+ blocks (long-form content, books)

**Implementation**:
1.  Each `NostraBlock` has a stable `block_id: UUID` (per existing proposal in Study).
2.  Large documents partition into sync "shards" based on block boundaries.
3.  Clients subscribe to visible blocks + neighboring blocks (predictive loading).

**Rationale**: This balances simplicity (small docs) with efficiency (large docs), avoiding premature optimization while enabling scalability.

### Consequences
1.  Block IDs must be stable across all operations (insert, delete, move).
2.  VFS metadata must track document size to determine sync mode.
3.  UI must handle "loading..." states for out-of-viewport blocks in large docs.

## 006: Polymorphic Block Alignment
**Date**: 2026-02-25
**Status**: ACCEPTED

### Context
Initiative 124 introduced the Universal Polymorphic Block with discriminated payload types (`a2ui`, `rich_text`, `media`, `structured_data`, `pointer`). The existing `NostraBlock` AST and the document composition model need formal alignment.

### Decision
**The document AST is a subset of the Polymorphic Block array.** A long-form document is an ordered sequence of Polymorphic Blocks where:
- Paragraphs and headings are `rich_text` blocks (synced via Yjs `Y.Text`).
- Embedded media, charts, and widgets are `media` or `a2ui` blocks (synced via Automerge structure).
- Citations and cross-references are `pointer` blocks linking to external entities.

### Consequences
1. The `NostraBlock` type in `pulldown-cmark` output maps directly to `rich_text` payload content.
2. Hybrid sync (DEC-004/005) operates cleanly: Yjs handles inline text within `rich_text` blocks, Automerge handles block-level structural mutations across all types.
3. DPub chapter composition (080) and Heap Board rendering (124) share the identical block array format.
