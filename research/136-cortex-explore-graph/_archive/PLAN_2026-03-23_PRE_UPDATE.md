# Plan: Cortex Explore Graph (136)

## Initiative Overview

**Mission:** Graduate the generic "Canvas" concept into a robust, navigable "Explore" graph representation. This visualization must move beyond passive "dots and lines" and become a primary interface for reading, navigating, and acting upon the Nostra knowledge graph.

**Core Insight:** The "Explore" page cannot be a one-size-fits-all visualization. The graph must be determined by the **intent, configuration, and type of the Space** (e.g., Intro Space, Research Space, Family Space, Governance Space, Geographic Space).

---

## 1. Architectural Pillars

### A. Intent-Driven Projections
As identified by the stewardship team, an "Introductory Space" needs to tell a story (Temporal awareness, Decisions, Roadmaps). A "Research Space" needs high-density conceptual linking. A "Governance Space" needs to visualize proposal lineage and voting state. A "Geographic Space" needs location-based awareness.
- **Implementation:** The Space's `SpaceCapabilityGraph` and `compiled_navigation_plan` will dictate the *default projection mode* of the Explore graph.
- **Concepts:**
  - `Story Mode`: Guided, chronological, or narrative-driven graph paths (ideal for Intro spaces).
  - `Density Mode`: Tag-clustered, highly linked views (ideal for Research).
  - `Lineage Mode`: Strict DAG representation of decisions and forks (ideal for Governance).
  - `Geographic Mode`: A projection that anchors nodes to real-world or virtual coordinates, allowing spatial exploration based on physics and topology rather than abstract concepts.

### B. Semantic Zooming (Not Optical)
The graph must maintain legibility at any scale.
- **Macro (World level):** Clusters and domain galaxies. Low detail, high structural awareness.
- **Meso (City level):** Nodes with distinct recognizable icons and titles. Edges showing typed relationships (`clarifies`, `evolves`).
- **Micro (Street level):** Nodes expand seamlessly into readable A2UI content cards or interactable components directly within the spatial canvas.

### C. Persistent Spatial Memory
Users must be able to shape their environment. If a user drags a cluster of blocks to the top right corner, that `[x,y]` coordinate data must be persisted to the backend (or local storage/preferences) so the spatial arrangement remains stable across sessions.

---

## 2. Technology Stack & Evaluation

- **Renderer:** `react-force-graph-2d` (HTML5 Canvas).
  - *Rationale:* We already use it for ambient backgrounds. It handles thousands of nodes flawlessly. It supports custom drawing callbacks (`nodeCanvasObject`) required for Semantic Zooming.
- **Data Source:** A dedicated `/api/kg/spaces/:id/topology` endpoint.
  - *Rationale:* The Heap API is optimized for paginated lists. The Explore graph requires the *entire* topology to simulate force physics properly.

---

## 3. Phased Execution Plan

### Phase 1: The Semantic Shift (Immediate)
- Rename current UI paths from "Canvas" to "Explore".
- Re-align the internal data models to support `Inbox`, `Drafts`, `Activity`, and `Pinned` as lists, while `Explore` becomes the true spatial view.

### Phase 2: Technical Prototype (Canvas Capabilities)
- Integrate `react-force-graph-2d` into the Explore view as the primary interactive element (replacing the masonry grid for the "All/Explore" filter).
- Implement basic Semantic Zooming: render dots when zoomed out, render text/icons when zoomed in.
- Implement click-to-focus: clicking a node dims the graph and highlights 1-hop connections.

### Phase 3: Intent-Driven Projections
- Connect the Explore graph to the Space's configuration profile.
- Implement the first unique projection: **"Story Mode"** for the Nostra Intro space (highlighting history, documentation, and pathfinding).

### Phase 4: Spatial Persistence
- Introduce coordinates to the node payload.
- Allow users to "pin" node positions.
- Persist pinned layouts to a remote `Layout Preferences` or User Profile capability.

---

## 4. Open Questions for Research

1. **Rendering HTML inside Canvas:** How effectively can we render complex A2UI blocks inside the HTML5 Canvas when zoomed in to the Micro level? (We may need to overlay absolute-positioned React DOM elements tracking the canvas coordinates).
2. **Data Hydration Scale:** At what node count does the initial topology payload become too large for a single network request?
3. **Multiplayer Spatial Sync:** If multiple users are in the Explore view sorting nodes, do we sync the spatial coordinates over WebSockets in real-time, or is the spatial layout strictly personal preference?
