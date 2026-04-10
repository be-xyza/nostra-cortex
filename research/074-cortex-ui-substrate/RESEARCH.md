---
id: '074'
name: cortex-ui-substrate
title: 'Research: Cortex UI Substrate & Governance'
type: general
project: nostra
status: active
authors:
- User
tags: []
created: '2026-02-17'
updated: '2026-02-25'
---

# Research: Cortex UI Substrate & Governance

**Status**: Active
**Owner**: Architect (Winston)
**Previous Title**: Themes System
**Consolidated Initiatives**: 028, 045, 070
**Last Updated**: 2026-02-25

## 1. Executive Summary

This initiative establishes the **Cortex UI Substrate**—a governed, versioned, and schema-driven UI runtime for the Nostra ecosystem.
We move beyond simple "theming" (colors/fonts) to controlling the **structure**, **behavior**, and **motion** of the user interface through governance.

**Core Philosophy**:
1.  **UI is a Governed Artifact**: The layout of a Space, the components available in a Lab, and the navigation of the Desktop are defined by versioned schemas (Artifacts), not hardcoded binaries.
2.  **4-Layer Architecture**:
    *   **Layer 1 (Governance)**: Spaces/Labs define the specs.
    *   **Layer 2 (Runtime Schema)**: A2UI-based payloads define the layout and content.
    *   **Layer 3 (Visual Primitives)**: Dioxus + Shoelace render the pixels.
    *   **Layer 4 (Motion Semantics)**: Governed tokens define transitions and easing.
3.  **Dioxus Native**: We target Dioxus (Rust/WASM) as the unified renderer for Cortex Desktop and Web, enabling "OS-grade" performance and shared logic with the backend.

---

## 2. Architecture Layers

### Layer 1: Governance & Versioning (The "Why")
*   **Spaces as UI Domains**: Each Nostra Space (e.g., "DAO Governance", "Personal Finance") publishes a `UI_MANIFEST` artifact.
*   **Labs as Experimental Forks**: A Lab can fork a Space's UI manifest, inject a new experimental component (e.g., "WebGPU Graph View" from `039`), and propose it back.
*   **User Overrides**: Users can override specific layers (e.g., "Always use Dark Mode", "Hide Sidebar") but cannot break the invariant structure defined by the Space.

### Layer 2: The Runtime Schema (The "What")
*   **Standard**: **A2UI Protocol (v0.8+)**.
*   **Extension**: We define a "Shell Protocol" for Cortex Desktop:
    ```json
    {
      "type": "defineLayout",
      "layout": "sidebar_right",
      "regions": {
        "main": { "component": "nostra:activity_stream" },
        "sidebar": { "component": "nostra:context_panel" }
      }
    }
    ```
*   **Data Model Updates**: We separate *Layout* (Schema) from *Content* (State). The Schema is static and cached; the Content is streamed via `dataModelUpdate` (Signal-based).

### Layer 3: Visual Primitives (The "How")
*   **Runtime Host**: **Dioxus** (Rust).
*   **Component Strategy**:
    *   **Primitives**: **Shoelace** (Web Components) wrapped in Dioxus. Handles standard UI (Buttons, Inputs, Dialogs).
    *   **Systems**: **Dioxus Native** (wgpu). Handles specialized views (Graphs, Atlases - see `039`).
    *   **Theming**: **CSS Variables** (per Phase 1 research) driven by the governed Theme Artifact.

### Layer 4: Motion Semantics (The "Feel")
*   **Philosophy**: Govern animation intent, not implementation.
*   **Tokens**: Semantic tokens (e.g., `standard_enter`, `fade_out`) defined in the Theme.
*   **Policy**: Governance controls motion reduction (`prefers-reduced-motion`) and performance tiers.

---

## 3. Phase 1: Visual Primitives & Theming (Completed)
*Previously "074-themes-system"*

### 3.1. Token Taxonomy
We adopted the "3-Tier Loudness" system from Web Awesome/Shoelace:
*   **Tiers**: `Quiet`, `Normal`, `Loud`.
*   **Semantic Roles**: `Brand`, `Success`, `Warning`, `Error`, `Info`.
*   **State Mixing**: `color-mix` for hover/active states.

### 3.2. A2UI Integration
Theme tokens are injected via the `styles` property in A2UI `beginRendering`.
*   **Constraint**: A2UI v0.9 Catalog must be expanded to support the full token set (Actioned).

---

## 4. Phase 2: Governed Layouts (Active)

### 4.1. The "Renderer" Gap
We currently have `renderers/lit` (Web) and `renderers/angular`.
**We must build `renderers/dioxus`.**

**Specification**:
1.  **Stream Parser**: Consumes A2UI JSONL.
2.  **Component Registry**:
    *   Maps `Button` -> `rsx! { sl_button { ... } }`
    *   Maps `Card` -> `rsx! { sl_card { ... } }`
3.  **State Machine**: Manages the Component Tree (Virtual DOM) and patches Dioxus Signals on `surfaceUpdate`.

### 4.2. Safe Mode
Accessing a verified Space loads the custom UI Schema.
Accessing an untrusted/new Space loads **Safe Mode**:
*   Ignores custom Layouts.
*   Renders data as a raw "Inspector View" or "Standard Document".
*   Prevents UI-redressing attacks.

---

## 5. Phase 3: Component Governance (Planned)

### 5.1. Component Registry Artifacts
Components themselves are governed artifacts.
*   `nostra:proposal_card` (v1.0)
*   `nostra:proposal_card` (v1.1 - adds "Impact Score")

Spaces declare dependencies: `requires: ["nostra:proposal_card@^1.0"]`.

---

## 6. Phase 4: Motion Governance (Implemented — 2026-02-25)

### 6.1. Motion Tokens
*   **Goal**: Eliminate ad-hoc CSS animations.
*   **Implementation**: Motion primitives implemented in `tailwind_fallback.css` (10 keyframe animations, 6 easing/duration tokens, 10 utility classes, 5 stagger delays). Policy helpers (`current_motion_policy_enum()`, `is_motion_allowed()`) added to `theme_policy.rs`. All animations respect `prefers-reduced-motion` via the existing media query suppression block.
*   **Reference**: Component 1 of Phase A Heap UX.

---

## 7. Integration Actions

1.  **Rename Initiative**: `074-themes-system` -> `074-cortex-ui-substrate`. (Done)
2.  **Consolidate**:
    *   `028-a2ui-integration-feasibility` -> Merged here.
    *   `045-component-library-labs` -> Merged here.
    *   `070-a2ui-testing-ground` -> Merged here.
3.  **Implementation**:
    *   Scaffold `renderers/dioxus`.
    *   Port "Theme Engine" logic to Rust/Dioxus.
    *   ~~**Integrate Motion Layer**~~ ✅ Done (Phase A Component 1, 2026-02-25).

---

## 8. Constitutional Alignment

| Principle | Constitution | Alignment |
| :--- | :--- | :--- |
| **History is Sacred** | Knowledge Integrity §17 | UI Schemas are immutable history. We can render the UI *exactly as it was* during a past proposal interaction. |
| **Agents First** | Agent Charter §6 | Simple JSON schemas allow Agents to read, verify, and generate UIs. |
| **Everything is a Contribution** | Knowledge Integrity §17 | Themes, Layouts, Components, and **Motion** are Contributions. |
| **Day-0 UI Standard** | UI/UX Manifesto §15 | All primitives comply with 5 Design Principles and Forbidden Elements list. |
| **Capability Containment** | Agent Charter §19 | Renderer capabilities cannot collapse provenance or bypass fork flows. |
| **Environment Profiles** | UI/UX Manifesto §15.8 | Desktop profile enables Pure Minimalism + power panels. |
| **Constitutional LSP** | UI/UX Manifesto §13 | Cortex Desktop exposes which clause blocks actions. |

---

## 9. Strategic Enrichments (Cortex Desktop)

### 9.1 UI Capability Envelope
To prevent authority leakage, we define a explicit "Envelope" for what the UI Substrate can do:
1.  **Declarative Only**: No imperative side effects (e.g., `fetch()`) in components.
2.  **No Cross-Space Write**: Components cannot write to other Spaces without explicit user Intent/Drag-and-Drop.
3.  **No Hidden State**: All UI state must be inspectable in the `surfaceUpdate` stream.

### 9.2 VFS Integration ("UI as File")
The UI Substrate maps directly to the Virtual File System:
*   `UI_MANIFEST` -> `/space/.config/ui.json` (Versioned File)
*   `Component` -> `/lib/components/my-card.wasm` (Blob)
*   `Theme` -> `/space/.config/theme.css` (File)

**Implication**: UI Diffs are File Diffs. Forking a Space forks its UI dependency graph.

### 9.3 Schema Downgrade Path (Safe Mode v2)
For untrusted Spaces, we support graceful degradation:
*   **Unknown Components** -> Render as `<SafePlaceholder />` (Generic Card)
*   **Unknown Motion** -> Reduced to `Transition: None`
*   **Layout** -> Forces standard "Document View"

This ensures content is always accessible even if the "Application Layer" is rejected.

---

## 10. Bootstrap Implementation Note (2026-02-06): A2UI Theme & Policy Layer (v1)

To reduce UI hardcoding and make “design direction” evolve via principles/policy (instead of per-view styling), we implemented a small **Theme & Policy Layer** for A2UI:

- **Tokens**: Global default token sets live in `shared/a2ui/themes/` as JSON (`cortex.json`, `nostra.json`).
- **Schema semantics**: A2UI supports `meta` fields (`theme`, `tone`, `context`, `density`, `priority`, `intent`, `severity`). For v1 compatibility, renderers also accept these as component `props`.
- **Renderer responsibility**: Web + Desktop renderers treat A2UI as *intent*, then apply policy defaults and tokens to produce consistent visuals.
- **Default assignment**: Nostra UI surfaces should use `nostra` theme; Cortex UI surfaces should use `cortex`. Surfaces can override via `meta.theme`.

This is intentionally **bootstrap**: it does not replace governed theme/layout artifacts (the broader UI Substrate), but provides a stable, policy-driven baseline that Unified Inbox and conflict/task surfaces can rely on immediately.

Reference: `docs/architecture/a2ui-theme-policy.md`.

## 11. Unified Inbox Prerequisite Fit (2026-02-06)

The Unified Inbox implementation now depends on this substrate through a strict prerequisite:

1. **Policy before layout work**  
   Inbox routing, filtering, and urgency styling are driven by A2UI metadata, not ad-hoc screen styling.

2. **Host defaults, schema overrides**  
   Nostra hosts default to `nostra` theme and Cortex hosts default to `cortex`; `meta.theme` can override when needed.

3. **One inbox, semantic labels**  
   The runtime keeps one inbox stream and derives labels from metadata/id/action contracts.

4. **Cross-host parity contract**  
   Web and desktop renderers consume the same metadata semantics for consistent triage behavior.

### Clarifications (Resolved 2026-02-25)
- ~~Safe-mode component allowlist for untrusted surfaces.~~ ✅ Implemented via Workstream C (074 PLAN.md).
- ~~Theme token version handshake enforcement.~~ ✅ `token_version` field implemented in theme policy.
- ~~Motion token governance for inbox transitions/acknowledgements.~~ ✅ Motion primitives and `MotionPolicy` helpers implemented (Phase A Component 1).


### Cross-References

- [Day-0 Primitives](file:///Users/xaoj/ICP/research/034-nostra-labs/NOSTRA_KNOWLEDGE_INTEGRITY_MEMORY_DOCTRINE.md) (§17)
- [Day-0 UI Standard](file:///Users/xaoj/ICP/research/034-nostra-labs/NOSTRA_UI_UX_MANIFESTO.md) (§15)
- [Capability Containment](file:///Users/xaoj/ICP/research/034-nostra-labs/NOSTRA_AGENT_BEHAVIOR_AUTHORITY_CHARTER.md) (§19)

## 12. 2026-02-09 Implementation Closeout: Default Theme Formalization + Desktop Conformance

### Decision Summary
- Default theme strategy is formalized as **evolve existing canonical Cortex theme**, not replace.
- Added additive shared token compatibility field `text_on_accent` to both canonical themes.
- Completed a full Cortex Desktop component conformance sweep for banned structural styling patterns and ASCII-first iconography.

### Implemented Changes
1. Shared theme contract updates
- `shared/a2ui/themes/cortex.json`
- `shared/a2ui/themes/nostra.json`

2. Desktop token plumbing
- `cortex/apps/cortex-desktop/src/components/a2ui/theme.rs`
- `cortex/apps/cortex-desktop/src/services/theme.rs`
- `cortex/apps/cortex-desktop/assets/tailwind_fallback.css`

3. Full desktop component sweep
- Tokenized structural classes (`bg-[#...]`, `border-white/*`, `dark:` removed from components tree).
- Replaced non-ASCII production symbols with ASCII-safe labels.

4. Blocking conformance gate
- Added `scripts/check_cortex_ui_theme_conformance.sh`.
- Integrated into `scripts/cortex-desktop-closeout-check.sh` (`cortex_desktop_closeout:ui_theme_conformance`).
- Registered as release blocker in `scripts/generate_test_catalog.sh`.

### Evidence
- Conformance report: `logs/testing/cortex_ui_theme_conformance_latest.json`.
- Catalog registration: `logs/testing/test_catalog_latest.json` includes `mixed:scripts:check-cortex-ui-theme-conformance.sh`.
- Gate summary refreshed in blocking mode: `logs/testing/test_gate_summary_latest.json`.

### Verification Snapshot
- `cargo check --manifest-path cortex/apps/cortex-desktop/Cargo.toml` passed.
- `cargo test --manifest-path cortex/apps/cortex-desktop/Cargo.toml --tests` passed.
- `bash scripts/check_cortex_ui_theme_conformance.sh` passed with zero violations.
- `bash scripts/check_test_catalog_consistency.sh --mode blocking --allow-synthetic-latest` passed (legacy run artifacts emit warnings only).

## 13. 2026-02-25: Phase A Heap UX Interaction Layer

Phase A of the "World-Class Heap UX" initiative implemented 8 components that directly extend the UI Substrate with interactive capabilities:

| Component | Capability | Files Modified |
|:---|:---|:---|
| Motion Design System | 10 keyframes, 6 tokens, 10 animation classes, `MotionPolicy` helpers | `tailwind_fallback.css`, `theme_policy.rs` |
| Block Selection + Agent Context | Multi-select with floating action bar, `POST /blocks/context` endpoint | `heap_workspace_view.rs`, `heap_block_wrapper.rs`, `server.rs` |
| Full-Text Search | `⌘K` overlay, client-side search across title/tags/mentions | `heap_workspace_view.rs` |
| View Modes | All / Unlinked / Pinned tabs with contextual empty states | `heap_workspace_view.rs` |
| Advanced Filter Logic | OR/AND/NOT compound tag filtering with colored pills | `heap_workspace_view.rs` |
| Block Export | Markdown (YAML frontmatter + wikilinks) and JSON export endpoints | `server.rs`, `heap_block_wrapper.rs` |
| Keyboard Shortcuts | `⌘K` search, `Escape` dismiss, `⌘A` select-all | `heap_workspace_view.rs` |
| Block Versioning | `GET /blocks/:id/history` reads audit JSONL for version timeline | `server.rs` |

### Architectural Impact
- **Layer 4 (Motion Semantics)**: Fully bootstrapped with governed animation primitives.
- **Layer 3 (Visual Primitives)**: Block interaction chrome extended with selection, export, and search affordances.
- **Layer 2 (Runtime Schema)**: 3 new gateway endpoints (`/context`, `/export`, `/history`) extend the heap contract.

### Cross-Initiative Resolution
- **124 (Polymorphic Heap Mode)**: Phase A capabilities registered as new decision entry.
- **095 (Personal Space Substrate)**: No changes needed (already current via Polymorphic Block resolution).
