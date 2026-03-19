---
id: "127F"
name: "cortex-ui-ux-enrichment"
title: "Research: UI/UX Enrichment via Polymorphic Blocks"
type: "research"
project: "nostra"
status: active
authors:
  - "User"
  - "Antigravity"
tags: ["cortex", "ui", "ux", "polymorphic-blocks", "a2ui", "siq"]
created: "2026-02-25"
updated: "2026-02-25"
---

# Research: UI/UX Enrichment via Polymorphic Blocks

This research artifact documents UI/UX enrichment opportunities that arise from the intersection of the **Cortex Invariant Engine (127)**, the **Universal Polymorphic Block Adoption (124)**, and the **Cortex UI Substrate (074)**.

By shifting from ephemeral, chat-based logs to a normalized, workspace-level block paradigm, we unlock powerful new interaction models for Cortex users.

---

## 1. The Polymorphic Payload Foundation

As of 2026-02-25, the `EMIT_HEAP_BLOCK` contract standardizes five interoperable payload types:
1.  `a2ui` (Interactive widget trees)
2.  `rich_text` (ProseMirror docs)
3.  `media` (Images, Videos)
4.  `structured_data` (Arbitrary JSON, e.g., Credentials from Initiative 095)
5.  `pointer` (Cross-references)

Because **all** these payloads share a common envelope containing `relations` (tags/mentions) and a `crdt_projection` stream, the UI can orchestrate complex workflows seamlessly.

---

## 2. Enrichment Opportunities (Designer Perspective)

### A. The "SIQ Scorecard" as a First-Class Citizen (`a2ui`)
Instead of burying SIQ validation results in a CI/CD pipeline or terminal, the Invariant Engine emits the scorecard as a persistent `a2ui` Polymorphic Block into the user's workspace.

**UX Enrichment:**
*   **Locality of Remediation**: The block renders failing invariants alongside the code/blocks they reference. 
*   **Contextual Agent Actions**: Because the block is an `a2ui` tree, the agent can inject "Auto Fix" buttons directly into the validation error details. Clicking "Fix" dispatches a new `rich_text` or `a2ui` block containing the proposed diff for the user to accept.
*   **Motion**: Using the `074` theme framework, the resolution of an invariant gracefully animates the scorecard from an `error` layout to a `success_morph` layout.

### B. Interactive Mentions & Graph Queries (`pointer` & Relations)
When the Cortex Agent proposes a code change or a new component, it doesn't just output text. It outputs blocks with rich `relations` (tags and mentions).

**UX Enrichment:**
*   **Hover-to-Preview**: Mentions inside `rich_text` or `a2ui` blocks support deep hover previews. If the agent says "We should update `#LoginComponent`", hovering over the tag previews the `LoginComponent` source file without context-switching.
*   **Graph Filtering**: The user can filter their Heap View not just by "show me pull requests," but by graph relations: "Show me all SIQ failures affecting `#Authentication`".

### C. Personal Credentials & Verification (`structured_data` / `media`)
Following Initiative `095` (Personal Space Substrate), Cortex can serve as a portfolio tracker.

**UX Enrichment:**
*   **Live Verification Badges**: If a user uploads a certification as a `media` block, an Agent can evaluate its validity (using an external oracle or simple matching) and append a signed `structured_data` block (a `CredentialReference`). 
*   **Visual Authority**: The UI Substrate renders verified blocks with specific aesthetic treatments (e.g., subtle glow borders or authenticated badge icons) that are cryptographically backed by the CRDT sequence history.

### D. The "Diff Viewer" Block (`a2ui`)
When an Agent proposes a codebase refactor (e.g., solving an SIQ failure), the output is an `a2ui` block implementing a Split Diff Viewer.

**UX Enrichment:**
*   **Interactive Review**: The user doesn't just read the diff; they interact with it. They can highlight specific lines in the proposed diff and type a comment, creating a new threaded `rich_text` block attached to that specific line via a `pointer` relation.
*   **Atomic Acceptance**: The diff block includes "Accept Hunk" buttons, allowing the user to partially merge the Agent's proposal, driving the exact CRDT mutation stream required to alter the codebase.

---

## 3. Implementation Pathway

To bridge these UX concepts into reality, the implementation must focus on:
1.  **Strict Typing**: Ensuring the `siq_formatter` wraps the `a2ui` tree inside a compliant `EMIT_HEAP_BLOCK` envelope.
2.  **Relation Extraction**: Ensuring that when an SIQ rule fails on a specific file (e.g., `src/auth.rs`), the formatter adds a `mention` relation pointing to that file's block ID.
3.  **Renderer Agnosticism**: Writing these blocks such that both Lit (`cortex-web`) and Dioxus (`cortex-desktop`) can parse the polymorphism and apply the Native Theme tokens equivalently.
