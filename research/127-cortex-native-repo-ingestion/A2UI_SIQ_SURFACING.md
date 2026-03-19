---
id: "127E"
name: "cortex-a2ui-siq-surfacing"
title: "Research: A2UI Surfacing for System Integrity Quality"
type: "research"
project: "nostra"
status: active
authors:
  - "User"
  - "Antigravity"
tags: ["cortex", "memory-fs", "validation", "a2ui", "ui-substrate", "siq", "heap-mode"]
created: "2026-02-25"
updated: "2026-02-25"
---

# A2UI Surfacing for System Integrity Quality (SIQ)

This document extends Phase 3 of the **Cortex Invariant Governance Engine (127B)** implementation plan. It defines the UI/UX architecture for moving structural invariant violations out of hidden terminal logs and surfacing them as first-class, interactive **A2UI Blocks** within the Cortex Desktop and Web shells.

By adopting the **Heaper Methodology (Initiative 124)** and the **Cortex UI Substrate (Initiative 074)**, we treat compliance as a living, remediable artifact in the user's workspace.

---

## 1. The UX Paradigm: Compliance as a "Heap Block"

Historically, CI/CD systems report failures linearly: a build runs, output streams block the pipeline, and the user reads the stdout text. 

In Cortex, **Execution is First-Class**. When the Invariant Engine computes a `SystemIntegrityQuality` (SIQ) score from a `RepoProjection`, it does not "throw an error." It emits a structured `GlobalEvent::InvariantViolation`. 

The A2UI frontend catches these events and renders them as interactive **Heap Blocks** within the user's Unified Inbox or Workspace View. 

### 1.1 The Widget Hierarchy (A2UI V0.8+)

To surface the SIQ metrics, we will compose existing A2UI `Standard Catalog` components:

*   **L0: The SIQ Scorecard Block (`Container`, `Card`)**
    *   Acts as the parent wrapper for the workspace's health.
    *   **Visuals**: A large hero metric (e.g., "SIQ: 85/100") using the `Heading` component with `intent: error` or `intent: success` derived from the `074` theme tokens.
*   **L1: The Policy Roster (`Tabs`, `List`)**
    *   Breaks down the `GovernanceProfile`.
    *   **Tabs**: "Failing Invariants" vs "Passing Invariants".
*   **L2: The Violation Detail (`Expander`, `Text`, `Badge`)**
    *   Each failing `InvariantViolation` is a `List Item` or an `Expander`.
    *   Contains the `rule_id`, affected graph nodes (`affected_nodes`), and human-readable explanation.
*   **L3: Remediation Actions (`Button`, `ActionGroup`)**
    *   Unlike a static log, the SIQ Block is actionable. 

---

## 2. The Remediation Workflow

When a structural invariant fails (e.g., "Initiative 034 is missing physical directory"), the A2UI block provides immediate remediation pathways.

### 2.1 Agentic Proposal (The "Auto-Fix")
Because the policy failure specifically identifies the exact node in the `RepoProjection` (e.g., the directory path or dependency name), the UI can provide a "Generate Fix Proposal" button. 

*   **Action**: `dispatch_action("generate_remediation")`
*   **Flow**: The Cortex Agent is passed the specific `InvariantViolation`. The agent generates a new `Contribution` (e.g., creating the missing folder or bumping the unpinned dependency) and streams a *new* A2UI "DiffViewer" block to the user for approval. 

### 2.2 Manual Fix & Verification 
If the user prefers to fix the codebase manually in their IDE:

*   **Action**: `dispatch_action("re-evaluate_projection")`
*   **Flow**: The user clicks "Verify" on the SIQ Block. The Cortex Runtime re-ingests the `Memory FS` sandbox, incrementally recalculates the `RepoProjection` (via Nx-style caching), and runs the WASM policy again. If it passes, the A2UI block dynamically updates its state to "Passing" via a `dataModelUpdate` message.

---

## 3. Motion Semantics & Theming (074 Alignment)

To ensure the SIQ UX feels premium and "Day-0 Standard":

1.  **Motion (Layer 4)**: The transition from a "Failing" SIQ state to a "Passing" state uses the governed `success_morph` semantic token. The red error badges smoothly collapse and transition into green checks, rewarding the user without jarring layout shifts.
2.  **Tokens (Layer 3)**: Violations must strictly adhere to the `error` and `warning` semantic roles defined in `shared/a2ui/themes/cortex.json`. Hardcoded hex colors (e.g., `text-red-500`) are banned.

---

## 4. Phase 3 Expansion Timeline

To implement this UI/UX extension, the following steps are added to the Web/Desktop integration phase:

1.  **Draft A2UI Formatter**: Build a Rust adapter in `cortex-web`/`cortex-desktop` that translates the `SystemIntegrityQuality` domain struct into a valid A2UI V0.8 JSON JSONL stream.
2.  **Integrate with Heaper Layout**: Ensure the resulting SIQ Block renders correctly within the masonry grid constraints of the Heap View (`HeapBlockWrapper`).
3.  **Bind Actions**: Connect the A2UI `action` handlers back to the `cortex-runtime` to trigger the `re-evaluate_projection` workflow.
