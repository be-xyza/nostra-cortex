---
id: "128"
name: "cortex-desktop-gpui-refactor"
title: "Research: Cortex Desktop GPUI Refactor Strategy"
type: "research"
project: "nostra"
status: active
authors:
  - "User"
  - "Antigravity"
tags: ["cortex", "desktop", "gpui", "native", "mvk"]
created: "2026-02-25"
updated: "2026-02-25"
---

# Cortex Desktop GPUI Refactor Strategy (Deprecated)

> **Note**: As of 2026-03-01, the GPUI refactor strategy has been formally deprecated (see `DEC-128-001`). The Cortex visualization layer has standardized on the React-based `cortex-web` shell, which provides better ecosystem alignment and velocity than pursuing a greenfield native GPUI application. The `cortex-desktop` layer remains active solely as a headless daemon/gateway.

## 1. Problem Statement

`cortex-desktop` currently renders via **Dioxus 0.7** (`dioxus-desktop`), which wraps a system webview (WKWebView on macOS). This introduces:

- **Webview overhead**: Every render cycle serializes a virtual DOM diff across the Rust ↔ WebView bridge.
- **JavaScript injection**: D3, xterm.js, and Tailwind CSS are injected at runtime via `eval()`, bypassing compile-time guarantees.
- **Styling inconsistency**: Tailwind utility classes are embedded as a fallback CSS blob, creating drift between the desktop and web clients.

## 2. Target Architecture: GPUI

GPUI is a native GPU-accelerated UI framework that eliminates the webview entirely. It provides:

| Concept | Dioxus (Current) | GPUI (Target) |
|---------|-------------------|---------------|
| Render target | WebView (HTML/CSS) | GPU (Metal/Vulkan) |
| Layout | CSS Flexbox via browser | Taffy flexbox in Rust |
| State | `Signal<T>` + `use_context` | `Entity<T>` + `Context<T>` |
| Concurrency | `tokio::spawn` + `spawn` | `cx.spawn` / `cx.background_spawn` |
| Styling | Tailwind classes (string) | Fluent Rust builder methods |

### Key Mappings

- `Signal<GlobalState>` → `Entity<WorkspaceState>` with `cx.notify()` on mutation.
- `Router<Route>` → GPUI `Window` with manual view switching via `Entity` updates.
- `dioxus::document::eval()` (JS injection) → Eliminated entirely. D3 graphs become native GPUI `Element` trees.
- `rsx! { div { class: "..." } }` → `div().bg(cx.theme().bg_surface).border_1().child(...)`.

## 3. The "Clean Break" Strategy (Optimal Greenfield Path)

Given the greenfield nature of this initiative, carrying the technical debt of a Dioxus/GPUI hybrid binary is an anti-pattern. The optimal path is a **Clean Break**, utilizing `cortex-web` as the capability bridge while the native desktop client is rebuilt from scratch.

### Phase A: The Great Purge & Gateway Extraction
Instead of untangling Dioxus from business logic, we isolate the backend and start fresh.
- Preserve the `cortex-desktop` gateway and service layers as a headless daemon (`cortex-gateway`).
- **Delete** all Dioxus UI components, router, and webview initialization logic.
- Remove `dioxus` and `dioxus-desktop` from the dependency tree entirely.

### Phase B: GPUI Greenfield Initialization
Boot a pristine GPUI application built strictly on Zed's architecture and MVK principles.
- Initialize native windowing, raw Metal/Vulkan GPU rendering context, and Taffy layout.
- Implement a rigid, native A2UI Renderer (`a2ui-gpui`).
- Rebuild the `HeapWorkspaceView` as the *only* initial native surface.

### Phase C: Dual-Host Capability Delegation
Rather than porting complex visual views (D3 graphs, xterm.js terminals) to GPUI immediately:
- GPUI strictly renders A2UI Polymorphic Blocks natively.
- For legacy, highly-visual, DOM-dependent tasks, the natively running `cortex-gateway` serves the Vite-built `cortex-web` application to the user's browser at `http://localhost:X`.
- The web browser acts as the complex visualization engine; the desktop app acts as the secure, high-performance execution kernel.

## 4. Principles Alignment

| Principle | How the Clean Break Aligns |
|-----------|-----------------|
| **MVK** | Immediate elimination of the massive webview runtime footprint. Zero HTML/JS injection in the kernel. |
| **Greenfield Pivot** | Prevents "strangler fig" technical debt. Forces strict adherence to the new native paradigms without legacy crutches. |
| **Governance** | Dual-host alignment (DEC-2026-02-22-014) is enforced. Web does web things; Desktop does native things. |

## 5. Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Immediate loss of native Desktop visual features (Graphs, Terminals) | Medium | Fully delegated to `cortex-web` backed by the local Desktop Gateway. Operators open a browser tab for complex views. |
| GPUI A2UI Renderer Parity | High | Focus 100% of desktop UI engineering on the `a2ui-gpui` renderer. If A2UI renders perfectly, the UI is instantly restored without manual component rewrites. |
