---
id: "128-capability-loss"
title: "Trade-off Analysis: Dioxus to GPUI Capability Loss"
type: "research"
project: "nostra"
status: active
authors: ["Antigravity"]
tags: ["cortex", "desktop", "gpui", "native", "mvk", "analysis"]
created: "2026-02-25"
---

# GPUI Pivot Capability & Trade-off Analysis

**To:** Engineering Management
**Subject:** Capability Loss and Mitigation Strategy for Dioxus → GPUI Migration (Initiative 128)

Based on a structural review of `cortex-desktop`, current specifications (A2UI, 109 UX System), and our governance policies (DECISIONS.md), moving from a WebView-based framework (Dioxus) to a GPU-accelerated native framework (GPUI) introduces significant performance and MVK-alignment benefits, but trades off several web-ecosystem capabilities.

This document details exactly what we lose, the architectural implications, and strategic mitigations.

## 1. WebView-Dependent Capability Losses

### A. Dynamic JavaScript Injection & Interop (`dioxus::document::eval`)
- **Loss:** `cortex-desktop` currently injects standalone JavaScript libraries at runtime to bypass Rust/WASM compilation constraints.
- **Affected Features:**
  - **Terminal (`src/components/terminal.rs`):** Completely relies on `xterm.js` and `FitAddon` injected via `eval()`.
  - **Graph Visualizations:** Any usage of D3.js or browser-based canvas rendering for things like `MotokoGraphView` or `SpaceGraphExplorerView`.
- **Mitigation:**
  - *Terminal:* Adopt a native Rust text-grid renderer. Zed (built on GPUI) utilizes Alacritty's `vte` crate combined with GPUI text rendering. We will need to port this integration.
  - *Graphs:* Rewrite visualization layers using native Rust graph libraries (e.g., `petgraph`) and render them using GPUI's explicit 2D unclipped drawing primitives (`cx.paint_quad`, `cx.paint_path`).

### B. Cascading Style Sheets (CSS) and Tailwind Runtime
- **Loss:** GPUI does not parse or render CSS strings or Tailwind classes (e.g., `class: "w-full h-full bg-[var(--bg-surface)]"`). It uses a fluent Rust builder paradigm layered over `Taffy` (a flexbox implementation) (e.g., `div().w_full().h_full().bg(cx.theme().bg_surface)`).
- **Affected Features:**
  - **A2UI Renderer (`a2ui/renderer.rs`):** A2UI compiles JSON-defined components directly into HTML primitives mapped to Tailwind classes. This translation layer will break completely.
  - **Theming Extensibility:** External theme modifications currently rely on CSS variable injection (DEC-2026-02-09-01). GPUI requires themes to be defined as typed Rust structs.
- **Mitigation:**
  - *A2UI Native Renderer:* We must build a new `a2ui-gpui` renderer that maps A2UI JSON components to GPUI fluent builders. Since A2UI is abstract, this is structurally sound but requires a 1:1 parity rewrite of `renderer.rs`.
  - *Dual-Host Strategy:* Rely on DEC-2026-02-22-014 (Dual-Host Workbench Alignment) to maintain `cortex-web` as the canonical web-renderer. If A2UI fails to render perfectly in GPUI initially, operators can fall back to Cortex Web.

### C. DOM-Based rich formatting and Browser Extensions
- **Loss:** "Hackability" of the client via browser inspect tools, arbitrary HTML injection (e.g., `dangerouslySetInnerHTML`), and standard web accessibility (a11y) trees managed by the OS WebKit/Blink engine.
- **Affected Features:**
  - **Rich Text Parsing:** The A2UI playground and Heap Workspace rely on browser-native Markdown-to-HTML rendering.
- **Mitigation:**
  - GPUI has its own text shaping and rendering engine (using `core-text` / `rustybuzz`). We will need to leverage or build a native Markdown parser that compiles to GPUI `Text` spans.

## 2. Architectural Paradigm Shifts (Dioxus vs. GPUI)

### State Management & Concurrency
- **Dioxus:** Uses React-like `use_signal`, `use_resource`, and `use_coroutine`. Component re-renders are triggered by signal invalidations propagating down a Virtual DOM tree.
- **GPUI:** Uses strictly lifetime-bound `Entity<T>`, `Context<T>`, and manual `cx.notify()`. Async work requires `cx.spawn()`, capturing `WeakEntity` pointers to prevent memory leaks, and pushing updates back to the UI thread.
- **Mitigation Challenge:** The entire view logic (`motoko_graph_view.rs`, `space_graph_explorer_view.rs`) is deeply coupled to `use_signal` patterns.
- **Strategic Mitigation:** **Phase A (Kernel Extraction)** documented in `128/STRATEGY.md` is critical. We must rip business logic out of Dioxus components into generic state machines (`cortex-desktop-core`) *before* writing GPUI views. The UI should only bind `Entity` observers to these core models.

## 3. Recommended Mitigation Execution Strategy (The "Clean Break")

Given the greenfield nature of this initiative and the strict adherence to the Minimal Viable Kernel (MVK) principles, an incremental "Strangler Fig" migration (running Dioxus and GPUI side-by-side) is an anti-pattern. It carries dead weight, fragments engineering focus, and dilutes the architectural pivot.

**The optimal path is a Clean Break.**

1. **The Great Purge:**
   - Rip Dioxus completely out of `cortex-desktop`.
   - Delete all JS-injected views, the virtual DOM router, and the webview dependencies (`dioxus-desktop`).
   - The desktop binary is stripped down to its pure Rust kernel: the Gateway daemon and background services.
2. **Greenfield GPUI Instance:**
   - Initialize a pristine GPUI window on macOS (Metal).
   - Build exactly *one* UI target: The `A2UI GPUI Renderer` and the `HeapWorkspaceView`.
   - We do not port static views. We only build the engine that interprets A2UI JSON from the Gateway.
3. **Dual-Host Capability Delegation (The Escape Hatch):**
   - For all legacy capabilities that we strategically chose to lose in GPUI (D3 graphs, xterm.js terminals, markdown playgrounds), we fully delegate to `cortex-web`.
   - The native `cortex-gateway` serves the local Vite application. If an operator needs to see the `MotokoGraphView` or execute a heavily visual terminal command, they open `http://localhost:X` in their browser.
   - *The web handles web problems; the native kernel handles kernel problems.*

## 4. Conclusion & Decision Recommendation

You are absolutely correct to challenge the incremental migration. Carrying a webview framework into a native-rendered architecture defeats the purpose of a greenfield pivot.

**Recommendation:** Execute the **Clean Break**. Drop Dioxus completely. Build the new GPUI interface focused exclusively on core A2UI rendering, and delegate all complex/legacy visualization capabilities to the local `cortex-web` client. This is the fastest, cleanest, and most structurally sound path forward.
