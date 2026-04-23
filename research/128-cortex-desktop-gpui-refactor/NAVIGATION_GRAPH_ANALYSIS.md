---
id: "128-nav-matrix-analysis"
title: "Analysis: Capability Navigation Graph vs. Traditional Routing"
type: "research"
project: "nostra"
status: active
authors: ["Antigravity", "User"]
tags: ["cortex", "desktop", "gpui", "ux-system", "navigation-graph"]
created: "2026-02-25"
---

# Analysis: Capability Navigation Graph Architecture (Deprecated context)

> **Note**: As of 2026-03-01, while the *Navigation Graph* concepts remain architecturally valid, their implementation via the GPUI pivot has been deprecated (see `DEC-128-001`). The unified UI strategy now standardizes on the React-based `cortex-web` shell to resolve the Dioxus traditional routing issues described below.

**Context**: As part of the greenfield pivot to GPUI (Initiative 128 - *Deprecated*), we must determine the core navigation and layout architecture of the new native client. Traditional desktop applications use static Sidebar/Menu routing (e.g., Dioxus `Router`). The user proposed a "navigational and view matrix or graph" to visualize, configure, and navigate available app capabilities based on intent/view level.

## 1. The Critique of Traditional Routing

The legacy `cortex-desktop` Dioxus implementation utilized `dioxus_router` with 22 hardcoded routes (`/spaces`, `/sandbox`, `/kg/motoko-graph`).

**Why this is suboptimal for Cortex:**
1. **Dark Capabilities:** Traditional routing hides the bounds of the system. Users only see what is currently explicitly linked in a sidebar. They cannot see "what is available" or what capabilities exist slightly outside their current role/intent.
2. **Flat Hierarchy:** Routes are 1-dimensional strings. They lack contextual depth—they don't understand the relationship between a "Testing Sandbox" and a "Production Release Matrix," even if the internal components are identical.
3. **Hardcoded Structure:** Changing the layout requires a rust recompilation of the Router enum.

## 2. The Greenfield Optimal Path: The Capability Navigation Graph

The user's assumption is entirely correct and aligns perfectly with the goals of **Initiative 109 (Cortex Desktop UX System)**, which attempted (but struggled) to retrofit `NavigationGraphSpec` and `ViewCapabilityManifest` onto the Dioxus router.

By purging Dioxus and adopting a GPUI greenfield approach, we have the opportunity to build the **Capability Navigation Graph** as the foundational UX paradigm from Day 1.

### How it Works (The Matrix Paradigm)

Instead of a `Router<Route>`, the GPUI application holds a central state model: the **`PlatformCapabilityGraph`**.

1. **Nodes are Capabilities, not Pages:** Every capability in Cortex (e.g., "Knowledge Graph", "Heap Workspace", "Console") is a node in the graph.
2. **Edges are Contextual Navigations:** Nodes are connected by edges that define relationships (e.g., `Requires_Approval`, `Follows_Workflow`, `Drill_Down`).
3. **Intent-Based Projection (A2UI):** When a user navigates to a node, the node doesn't load a hardcoded Rust component. It sends its `CapabilitySpec` to the local Gateway, which returns an **A2UI RenderSurface** uniquely generated for the user's current `intent`, `role`, and `density` preferences.
4. **The "Macro-Map" View:** Because navigation is a literal mathematical graph in memory, we can render a "System Map" UI. The user can zoom out and see the entire capability matrix of the Cortex platform—identifying exactly what tools they have access to, what workflows are available, and the systemic relationship between them.

## 3. Structural Advantages of the Matrix Path

| Feature | Traditional Routing | Capability Navigation Graph |
| :--- | :--- | :--- |
| **Discovery** | Hidden behind menus and dropdowns. | Visualized as a map; users can see adjacent, unexplored nodes. |
| **Access Control** | Imperative `if !has_role() { return PageNotFound; }` | Declarative. Nodes the user cannot access are rendered differently (e.g., "locked" or "shadow" nodes) in the matrix visualization but remain visible for systemic context. |
| **Dynamic Reconfiguration** | Requires code changes and recompilation. | The Gateway can push a CRDT mutation to the `NavigationGraphSpec`, instantly re-wiring the desktop client's available paths without the client updating. |
| **Contextual A2UI** | A component looks the same everywhere. | Selecting a node requests an A2UI payload. If the user's intent is "Audit", the gateway returns the audit-optimized A2UI tree for that node. |

## 4. Implementation Strategy (Phase B Hook)

To implement this on the GPUI pivot, we modify **Phase B (GPUI Greenfield Initialization)**:

Instead of just rendering the `HeapWorkspaceView`, the root GPUI component is a **`NavigationMatrixShell`**.

1. **The Core Entity:** `Entity<NavigationGraphSpec>`
2. **The Default View:** A spatial map or hierarchical matrix rendering the nodes of the graph.
3. **The Interaction:** Clicking a node executes a `GatewayClient::request_capability(node.id, user_context)`.
4. **The Resolution:** The Gateway responds with an abstract `A2UI Surface`. The GPUI `A2UIRenderer` renders the payload over the central frame.

## 5. Conclusion

**Verdict: Highly Optimal.** The user's proposal solves the "feature discovery" and "contextual capability" problems inherent in monolithic desktop apps.

By building the navigation as a dynamic graph loaded from the Gateway, the Rust GPUI client remains an extraordinarily "dumb" but lightning-fast terminal. It knows *nothing* about the business logic of a "Motoko Graph" - it only knows how to display the Navigation Graph and render the abstract A2UI blocks it lands on. This is the ultimate realization of the Minimal Viable Kernel (MVK) principle.
