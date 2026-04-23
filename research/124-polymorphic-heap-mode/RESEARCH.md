# Research Initiative 124: Polymorphic Heap Mode Methodology

## 1. Executive Summary

This research investigates the adoption of the "Heaper methodology" (tag-centric, bottom-up block organization via CRDTs) as a primary polymorphic heap mode for A2UI streaming interfaces and derived view surfaces, specifically tailored for Personal Knowledge Management (PKM).

The goal is to transition A2UI from an ephemeral, chat-centric paradigm to a persistent, organically developing ecosystem of components. By translating Heaper's "Block" and "Heap" concepts into Nostra/Cortex architectures, emitted interfaces become durable, interconnected artifacts within the user's workspace.

**Recommendation:** **Strongly Recommended.** Adopting the Heap methodology aligns perfectly with Nostra's SpaceGraph architecture, the Minimal Viable Kernel (MVK) doctrine, and the Surface Boundary Doctrine. It provides a structured ontology for persisting A2UI components.

> **Reference Material:** For a complete breakdown of Heaper's UI taxonomy and feature set, refer to the [Heaper Knowledge Collection](../reference/knowledge/ui-substrate/heaper/).

### 1.1 Implementation Status (2026-02-24)

Initiative 124 is now in productionization execution with desktop-first authority:

1. Desktop canonical heap API and mapper flow implemented in `cortex/apps/cortex-desktop`.
2. Agent compatibility adapter routes legacy `RenderSurface` outputs through canonical heap emit when heap mode is enabled.
3. Desktop heap board is data-backed with live query filters and live actions.
4. Web parity consumer is implemented behind feature flag `VITE_HEAP_PARITY_ENABLED`.
5. Web parity request-contract tests are active in `cortex/apps/cortex-web/tests/heapApiContract.test.ts`.
6. Block selection with multi-action floating bar (Send to Agent / Delete / Clear) and `POST /blocks/context` endpoint for Agent Harness.
7. Client-side full-text search with `⌘K` toggle, filtering across title, tags, mentions, and block_type.
8. View modes (All / Unlinked / Pinned) with contextual empty states and tab-style selector.
9. Advanced filter logic (OR/AND/NOT compound tag filtering) with colored tag pills and expandable filter panel.
10. Block export (`GET /blocks/:id/export?format=markdown|json`) and version history (`GET /blocks/:id/history`) endpoints.

---

## 2. Methodology Analysis: Heaper vs. Heap Views

### 2.1 The Heaper Paradigm
- **Universal Unit:** Everything is a "Block" (a file, a note, a tag).
- **Organization:** Bottom-up. No rigid folders. Organized via Mentions (`RELATES TO`) and Tags (`IS A`).
- **Data Layer:** Local-first SQLite with Yjs (CRDT) for conflict-free multi-device sync, augmented by Content-Addressable Storage (SHA256) for file deduplication.
- **UI UX:** A chronological or filtered "Heap" of cards, rather than rigid documents.

### 2.2 The A2UI Challenge
Currently, A2UI streams dynamic UI components (charts, forms, interactive widgets) into a linear chat feed.
- **The Problem:** Once the chat scrolls up, the UI artifact is effectively lost. It is transient and isolated. It cannot easily be referenced, tagged, or remixed by the user.
- **The Solution:** Treat every emitted A2UI component as a **Block**.

### 2.3 The "Heap Mode" for A2UI
When the Cortex Agent emits an A2UI component, it is automatically persisted as a discrete block.
- **Agent Tags & Mentions:** The agent can automatically attach tags (e.g., `#data-viz`, `#q3-analysis`) and mentions relating it to previous context (e.g., `@Project-Alpha-Brief`).
- **User Curation:** The user can interact with the A2UI component, annotate it with rich text (just like Heaper blocks), and retag it. The A2UI component becomes a living artifact in the user's PKM graph.

---

## 3. Translation to Cortex & Nostra

### 3.1 Nostra Architecture Mapping
Heaper concepts map directly to Nostra primitives:

| Heaper Concept | Nostra Primitive | A2UI/View Integration |
|----------------|------------------|-----------------------|
| Block | `Contribution` / `Locus` | An A2UI component payload wrapped inside a Contribution envelope. |
| Tag | Node Property / Entity Type | SpaceGraph metadata defining what the A2UI component *is*. |
| Mention | `SpaceGraph` Edge | A directional link connecting the A2UI widget to other nodes. |
| Heap | `Space` / View | A filtered query over the SpaceGraph rendering a grid/list of A2UI blocks. |
| Yjs / CRDT | Nostra Sync Protocol | Decentralized, real-time sync of A2UI component state. |

### 3.2 Cortex Agent Runtime Mapping
For Cortex agents:
1. **A2UI Emission:** Instead of just sending JSON A2UI to the UI shell, the agent utilizes a tool or protocol to commit a `Contribution` to the local graph.
2. **Context Injection:** When opening a "Heap View", the agent is injected with the Sub-Graph of relevant blocks, identical to how it would read a file directory, but highly semantic.
3. **Organic Development:** The agent can read existing A2UI blocks, modify them (via CRDT operations), or generate new ones that reference the old ones.
4. **Pluralistic Isolation:** Heaper's capability of maintaining separate top-level workspaces (e.g., "Work" vs "Personal") maps perfectly to Nostra `Spaces` acting as boundaries for distinct graph domains, utilizing shared Content-Addressable Storage for deduplication across these spaces.

---

## 4. Docker Container Analysis

The user referenced the Heaper Docker container (`ghcr.io/JanLunge/heaper`).

### Overview of Heaper Server Container
- **Purpose:** Acts as a centralized WebSockets relay for Yjs CRDTs, a thumbnail generation service for heavy media, and an authentication gateway (public/private key mapping).
- **Feasibility:** Running this exact container alongside Nostra is *technically feasible* via Docker Compose.

### Architectural Recommendation (MVK Doctrine)
**Do not adopt the literal Heaper Docker container.**
- **Why:** Nostra is building a Rust-based, WASM-first `cortex-gateway` for offline-first peer-to-peer sync. Adopting a black-box Node/Go-based Docker container for a core data path violates the WASM-First portability principle and fragments the data layer.
- **What to Adopt:** Adopt the **Pattern**. The pattern is a concurrent multi-writer CRDT syncing relayer. We must implement this by **leveraging Initiative 113 (Cortex Real-Time CRDT Collaboration)**, which already provides a deterministic CRDT engine within `cortex-domain`. We should extend this existing engine to support A2UI component state, rather than introducing redundant engines like Yjs or Automerge.

---

## 5. Implementation Path

How to accomplish "Polymorphic Heap Mode":

### Phase 1: Substrate Readiness (UI Layer)
1. **Block Wrapper Component:** Extend the React/Lit A2UI renderers (`@renderers/lit` or Nostra frontend) to wrap every incoming A2UI root in a "Block Card".
2. **Block Metadata UI:** Add UI controls to the block card for Tags, Mentions, and basic markdown annotations (replicating the `user_34-40` Heaper screenshots flow).

### Phase 2: Domain Layer Translation (Nostra)
1. **A2UI Contribution Schema:** Define a new `Contribution` payload type specifically for polymorphic heap state. This schema stores the initial A2UI rendering JSON and a serialized CRDT blob for ongoing state changes.
2. **Graph Edges (Mentions):** Ensure the frontend translates `mentions` from the A2UI payload into actual `ContributionRef` or SpaceGraph edges. The `mentions` array in the UI payload acts as the semantic intent, which Nostra then solidifies as structural graph links.

### Phase 3: Agent Integration (Cortex)
1. **Agent Output Formatting:** Instruct the LLM to output A2UI components with suggested initial tags and mentions, providing the necessary routing data for Phase 2's graph edge creation.
2. **Heap ViewSpec:** Create a `ViewSpec` in Cortex that renders a Space not as a document tree, but as a reverse-chronological, filterable masonry grid of polymorphic heap blocks.

---

## 6. Resolving Relevant Initiatives

This adoption resolves and aligns with several existing initiatives:

- **Initiative 008 (Nostra Contribution Types):** Validates the need for an `A2UI` specific contribution type that acts as a standalone block.
- **Initiative 074 (Cortex UI Substrate) & 115 (ViewSpec):** The "Heap" becomes a first-class ViewSpec. The UI Substrate must support masonry/grid layouts of isolated, self-contained interactive widgets.
- **Initiative 080 (DPub Standard) & VFS:** DPubs (composite artifacts) and Heap Blocks (atomic interactive artifacts) are complementary. The VFS layer designed for DPub bundles will natively support any arbitrary binary data (images, PDFs) required by A2UI Heap blocks, eliminating the need for external file-handling libraries.
- **Initiative 113 (Cortex CRDT Collaboration):** Provides the deterministic CRDT engine that will back the persistent A2UI blocks, enabling multi-device sync without third-party opaque silos.
- **Initiative 119/123 (Cortex Web Architecture):** The web shell natively supports displaying the "Heap" surface boundary next to the "Chat" surface boundary.
- **Modular Plurality:** We aren't forcing PKM on all uses. "Heap Mode" is an opt-in View within a Space. Other spaces might use "Canvas Mode" (tldraw) or "Document Mode" (ProseMirror).

## Conclusion

Translating Heaper's methodology to a polymorphic heap mode transforms ephemeral LLM outputs into a durable, graph-backed Personal Knowledge Management system. By wrapping A2UI streams in CRDT-backed "Blocks" connected via Mentions and Tags, we create an organic, bottom-up workspace without compromising the architectural purity of Nostra's Minimal Viable Kernel.
