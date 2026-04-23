---
id: '065'
name: code-editor-analysis
title: 'Research: Code Editor Analysis for Cortex Desktop'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-05'
---

**Status**: ACTIVE (Analysis)
**Date**: 2026-01-25

> [!NOTE]
> **Rename**: This initiative was renamed from `vscode-integration` to `vscode-analysis` on 2026-02-06, and expanded to `code-editor-analysis` on 2026-02-06.
> **Resolution**: This is a **Control Layer** component.
> Implementation work is deferred until **Phase 1 (Data Layer)** of [Research 066](../066-temporal-boundary/PLAN.md) is complete.
>
> **Reasoning**: We should analyze and document patterns now, but avoid direct integration until *Contribution Types* (Data) are defined.

---

# Research: Code Editor Analysis for Cortex Desktop

**Date**: 2026-01-25
**Status**: ACTIVE
**Context**: Cortex Desktop operates as a headless daemon (previously a Rust/Dioxus application - *Deprecated via DEC-123-004*). We are analyzing modern code editor patterns and components (local sources: `ICP/vscode`, `ICP/zed`) to inform the React/A2UI experience in `cortex-web`. This is **analysis-only**, not direct integration.

## 1. Executive Summary

Cortex Desktop needs a high-performance, collaboration-friendly editor surface. VS Code and Zed provide complementary patterns. The goal is to **extract reusable patterns, capability requirements, and component boundaries** that can be implemented natively in Cortex and aligned with Nostra governance.

**Boundary note**: Nostra defines the platform-level data model, governance, and permissions. Cortex Desktop is the execution/UI layer that should consume those contracts and render collaborative editor experiences.

**Recommendation**: Adopt a **Code Editor Pattern Inventory** approach.
1. **VS Code**: Treat as a UX and layout reference (command palette, workbench structure, theming tokens).
2. **Zed**: Treat as an architecture and collaboration reference (panes, docks, multibuffers, remote execution, trust model).
3. **Output**: A capability matrix + component inventory mapped to Nostra/Cortex principles and roadmaps.

## Artifact Index
- `CAPABILITY_MATRIX.md`
- `COMPONENT_INVENTORY.md`
- `GPUI_FEASIBILITY.md`
- `PERFORMANCE_PARITY.md`


## 2. Scope & Inputs

### 2.1 In Scope
- Editor UX patterns (command palette, panels, tabs, layouts).
- Collaboration models (channels, notes, following, guest access).
- Trust and capability gating.
- Remote execution split (local UI vs remote compute).
- Tasks, terminal integration, and context variables.
- Multibuffer/multi-file editing patterns.
- Agent/MCP integration patterns for tools.

### 2.2 Out of Scope
- Direct integration of VS Code or Zed workbenches.
- Embedding Electron or running the VS Code extension host.
- UI porting from Zed's GPUI (Deprecated via DEC-128-001) into React (`cortex-web`).

### 2.3 Sources (Local)
- `ICP/vscode` (source layout and design tokens).
- `ICP/zed/docs/src/*` (product and development docs).
- `ICP/zed/crates/*` (component inventory signals).

## 3. Zed Inspection (Local Repository)

See `COMPONENT_INVENTORY.md` for the full deep-dive inventory and crosswalk.

### 3.1 UI Composition & Editor Model
From the Zed glossary, the UI and editor are organized around a **Workspace** containing **Pane** and **Dock** regions, with **Panels** (left/right/bottom) and **Editors** in panes. Zed distinguishes **Worktrees** (local or remote file roots), **Buffers** (in-memory files), and **Multibuffers** (multi-file excerpts). These structures map cleanly to Cortex Desktop's Workbench aspirations and suggest a stable internal vocabulary for layout state and navigation.

**Cortex mapping**:
- `Workspace` -> Cortex Desktop root surface
- `Pane` -> editor/workflow tabs or multiview
- `Dock`/`Panel` -> persistent side/bottom panels (project tree, agent panel, tasks)
- `Worktree` -> Nostra space- or project-scoped file roots
- `Buffer` -> resource-backed editable view
- `Multibuffer` -> multi-resource editing/preview surface

### 3.2 Collaboration Model
Zed uses **channels** for ongoing collaboration, with a shared room, channel notes, and ambient awareness. Sharing a project grants access to the host's file system with read/write or guest read-only modes. Following a collaborator is pane-scoped, and screen sharing is integrated. Zed explicitly warns about trust and advises only sharing with trusted collaborators.

**Cortex mapping**:
- Channels -> Nostra Spaces or Project Rooms
- Channel notes -> shared Idea/Spec note with provenance
- Follow behavior -> UI-level "focus sync" for pair-review
- Guest access -> read-only contributor role

### 3.3 Trust & Capability Gating
Zed starts worktrees in **Restricted mode** and requires explicit trust before enabling project settings that can install or run language servers or MCP servers. Trust can be granted at single-file, directory, or parent-directory levels. Extension capabilities are also gated (e.g., `process:exec`, `download_file`, `npm:install`) and can be narrowly scoped.

**Cortex mapping**:
- Worktree trust -> Space-scoped permission gates
- Capability grants -> Steward-approved tool policies
- Safe default -> aligns with Nostra's recommendation-only mode

### 3.4 Remote Execution Split

See `PERFORMANCE_PARITY.md` for the full performance roadmap and targets.
Zed's remote development runs **UI locally** while language servers, tasks, and terminals run on a remote server over SSH. This separation preserves responsiveness and isolates compute-heavy workloads. This is primarily a **Cortex Desktop** concern; Nostra sets the governance and data contracts that the runtime must honor.

**Cortex mapping**:
- Local UI + remote execution -> React UI (`cortex-web`) with Rust worker/agent backends
- Remote task contexts -> execution sandboxes with explicit resource refs

**Performance parity steps for Cortex Desktop**:
- Maintain a strict UI-thread budget and move parsing, indexing, and LSP work to background worker processes.
- Use incremental rendering and fine-grained UI diffing to reduce layout and paint costs.
- Adopt rope-based text buffers and incremental syntax trees for large-file edits.
- Stream file content on demand and avoid full-file loads in the UI process.
- Keep diagnostics and search results in separate, asynchronously refreshed views.
- Implement batched file watching and debounced filesystem events to reduce churn.
- Use GPU-accelerated rendering paths where the webview allows it; avoid heavy DOM reflow.
- Prioritize input latency: handle keystrokes locally and sync to background processes asynchronously.
- Cache LSP results and symbol indices with explicit invalidation rules.
- Use remote execution for heavy tasks, but keep low-latency editor features local.
- Apply worktree trust and capability gating before enabling background agents.

### 3.5 Multibuffers
Zed uses multibuffers to edit multiple files simultaneously (e.g., search results, diagnostics, multi-definition). Edits in a multibuffer propagate to source buffers, and navigation can jump from excerpts to source locations.

**Cortex mapping**:
- Multibuffers -> multi-resource editing for Nostra graph entities
- Search/diagnostics -> unified "evidence" buffer or knowledge diff view

### 3.6 Tasks & Terminal Integration
Zed tasks can spawn commands in an integrated terminal, reuse terminals, and read editor context via environment variables (`ZED_FILE`, `ZED_SYMBOL`, `ZED_SELECTED_TEXT`, etc). Terminals can be docked or opened in the center pane, and shell/working-directory behavior is configurable.

**Cortex mapping**:
- Tasks -> workflow steps with context variables
- Terminal -> execution log panel for agent runs
- Context vars -> ResourceRef and Event metadata mapping

### 3.7 Agent + MCP Integration
Zed supports MCP **Tools** and **Prompts**, can reload tools dynamically when servers change, and provides an agent panel with per-profile tool approval and MCP server selection. External agents can be connected via ACP, and tool approval is configurable.

**Cortex mapping**:
- MCP -> standardized tool access for Nostra/Cortex agents
- Tool approval -> governance and steward gating
- Agent panel -> A2UI-compatible agent surface

### 3.8 Component Inventory Signals (Zed crates)
The Zed repo includes dedicated crates for collaboration, multi-buffers, remote connections, extension host, tasks, terminal, command palette, and agent tooling. This implies these components are treated as first-class subsystems rather than ad-hoc features.

**Cortex mapping**:
- Treat these subsystems as explicit modules with interface contracts and event streams.

## 3.9 GPUI Feasibility Study (Zed UI Framework)

See `GPUI_FEASIBILITY.md` for the canonical feasibility assessment.

**What it is**: GPUI is Zed's native Rust UI framework. *(Note: GPUI adoption abandoned via DEC-128-001)* It is optimized for low-latency editor interactions and collaborative features. It is not a web/WASM framework, and it is tightly coupled to Zed's architecture.

**Why it is relevant**:
- Strong editor-specific UI primitives (panes, docks, panels, pickers, actions).
- Concurrency and task execution patterns built into the UI runtime.
- A proven architecture for collaboration-driven UX.

**Feasibility questions for Cortex Desktop**:
- Can GPUI be embedded or bridged into a Dioxus-based shell without a full rewrite?
- Which GPUI concepts are portable as patterns rather than code?
- What portability constraints exist for Nostra's platform goals (web/WASM safety)?

**Possible outcomes**:
- **Pattern-only adoption**: Extract architecture patterns, keep React/A2UI (`cortex-web`).
- **Hybrid shell**: *(Deprecated strategy: attempting to mix Dioxus/GPUI)*.
- **Defer adoption**: Maintain React and implement only GPUI-inspired concepts.

**Minimum study steps**:
- Map GPUI core concepts to Cortex Desktop architecture boundaries.
- Identify what would need to be re-platformed for webview compatibility.
- Estimate engineering effort for a hybrid or full swap approach.
- Identify any licensing or dependency constraints.

**Decision gate**:
- Only consider GPUI adoption if it materially improves performance or collaboration capability beyond what Dioxus can deliver within planned timelines.

## 4. VS Code Reference Patterns (High-Level)

VS Code remains a strong reference for **workbench layout**, **command palette**, **themable tokens**, and the editor UX bar for common developer actions. The Monaco editor model is still a good reference for capability parity, but should be treated as a benchmark rather than a dependency.

## 5. Capability Matrix (Draft)

See `CAPABILITY_MATRIX.md` for the canonical version.

| Capability | VS Code Signal | Zed Signal | Cortex/Nostra Mapping |
| --- | --- | --- | --- |
| Command Palette | Strong | Strong | Unified action routing; agent commands |
| Multibuffer / Multi-file editing | Moderate | Strong | Multi-resource editing surface |
| Collaboration | Medium (Live Share) | Strong (channels, follow, notes) | Space rooms + shared notes |
| Remote execution | Medium (SSH/WSL) | Strong (remote server) | Local UI + remote agents |
| Trust model | Extension permissions | Worktree trust + capability gating | Steward-governed access |
| Tasks | Tasks.json | Tasks w/ context vars | Workflow steps w/ ResourceRef |
| Terminal | Integrated | Integrated | Execution output panel |
| AI / MCP | Extension ecosystem | MCP + agent profiles | Agent workflows via MCP |

## 6. Reusable Component Patterns Across Cortex/Nostra

- **Workbench Layout**: Pane/Dock/Panel taxonomy for all apps.
- **Command Palette**: Unified action interface across Cortex Desktop.
- **Multibuffer**: Multi-entity editing and refactor views.
- **Channel Notes**: Collaborative spec/decision pads in Spaces.
- **Trust Model**: Explicit permission gates for project or space settings.
- **Capability System**: Steward-approved tool and agent access.
- **Remote Execution**: Split UI/compute for heavy workflows.
- **Task Context Variables**: Standardized workflow context in execution runs.

## 7. Alignment with Nostra Principles

- **Stewardship & Roles**: Capability gating maps to steward-controlled permissions.
- **Safe Default**: Restricted/trust-first behavior aligns with recommendation-only defaults.
- **Knowledge Integrity**: Channel notes + multibuffers can surface provenance and context.
- **Contribution Lifecycle**: Explicit scope control avoids premature integration.

### 7.1 Enrichment Needed for Nostra Alignment
- **Space scoping**: Collaboration channels should map to Spaces with explicit permission boundaries.
- **Provenance**: Channel notes and multibuffer edits should emit `Event` metadata and retain lineage.
- **Governance**: Capability grants must be auditable and tied to steward approvals.
- **Execution safety**: Tasks/terminal actions should be mediated by Cortex workflows and cycles policies.
- **Data confidence**: Search/diagnostics outputs should carry confidence metadata when surfaced to users.

## 8. Open Questions

- Should Cortex adopt a Zed-like **worktree trust** UI for project-level settings?
- Which multibuffer workflows align best with Nostra graph editing?
- Do we need a unified action registry to power the command palette and agent tools?

## 9. Resolution

**Plan**: Proceed with analysis and pattern extraction only.
**Specs**: If editor decisions are made, record them before updating downstream plans.
**Principles**: Aligns with "Premium Design" and "Developer Ergonomics" without premature integration.
