---
id: '035'
name: auto-claude-analysis
title: 'Research Initiative: Auto Claude Analysis'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research Initiative: Auto Claude Analysis

**Status**: DRAFT
**Owner**: Nostra Architecture Team
**Created**: 2026-01-18
**Related**: 013-nostra-workflow-engine, 028-a2ui-integration-feasibility, 007-nostra-spaces-concept

## 1. Objective
To analyze the "Auto Claude" autonomous coding framework and identify features, patterns, and UI concepts that should be ported or adapted for the **Nostra Orchestration System**. This initiative aims to accelerate our agentic capabilities by learning from a leading open-source implementation.

## 2. Auto Claude Overview
Auto Claude is an autonomous multi-agent coding framework that:
-   **Plans**: Breaks down tasks into steps.
-   **Builds**: Executes code changes in isolated environments.
-   **Validates**: Runs tests and QA loops.
-   **Visualizes**: Provides a Kanban board for tracking agent progress.

## 3. Feature Analysis & Porting Decisions

We analyze key Auto Claude features and decide their fate in the Nostra ecosystem.

| **Feature** | **Auto Claude Implementation** | **Nostra Equivalent / Adaptation** | **Decision** |
| :--- | :--- | :--- | :--- |
| **Kanban Board** | React-based drag-and-drop board. Columns: **Planning, In Progress, AI Review, Human Review, Done**. | **A2UI Workflow Dashboard**. A reusable A2UI component. We will adopt the exact state names as a starting point for the "Dev Loop" workflow. | **ADOPT** |
| **Agent Terminals** | Grid of xterm.js instances (e.g., 6 terminals) showing raw shell output. | **A2UI Log Grid**. A "Grid" component containing "LogViewer" widgets. This allows watching multiple agents (canisters) in parallel. | **ADAPT** |
| **Autonomous Loop** | Hardcoded Plan -> Code -> Test loop. | **Standard "Dev" Workflow**. A declarative Serverless Workflow definition (JSON). | **ADOPT** |
| **Roadmap** | "Must Have", "Should Have" priority buckets. | **Contribution Prioritization**. We will add `Priority` fields to our Contribution Objects and visualize them similarly. | **ADOPT** |
| **Context Injection** | One-click context (files, errors) injection. | **Context Graph**. Agents query the Knowledge Graph (OneKE) for relevant context automatically. | **ENHANCE** |

## 4. Deep Dive: The "Nostra Kanban" (A2UI)

Auto Claude's most visible feature is the Kanban board. In Nostra, this should not be a hardcoded React page, but a **dynamic A2UI surface**.

### Concept
*   **Source**: The Workflow Engine (013) queries all active `ProcessInstances`.
*   **Transformation**: An "Orchestrator Agent" converts this list into an A2UI `Board` component.
*   **Interactivity**: Dragging a card triggers a `completeStep` or `transition` action in the Workflow Engine.

### Implementation
1.  **Schema Definition**: The `KanbanBoard` A2UI component must support:
    *   `columns`: Array of state keys (e.g., `["planning", "in_progress", "ai_review", "human_review", "done"]`).
    *   `cards`: Array of objects with `id`, `title`, `status`, `summary`.
2.  **State Mapping**:
    *   `Planning` -> Workflow State: `defining_requirements`
    *   `In Progress` -> Workflow State: `executing_step`
    *   `AI Review` -> Workflow State: `self_verification`
    *   `Human Review` -> Workflow State: `awaiting_approval` (UserTask)
    *   `Done` -> Workflow State: `completed`

## 5. Deep Dive: "Agent Terminals" (Visual Analysis)
The reference image shows a **Grid View** (e.g., 2x3) of active terminals.
*   **Adaptation**: We need a `Grid` layout component in A2UI (currently we have Row/Column).
*   **Widget**: A `LogStream` widget that takes a `canister_id` and `topic`.
*   **Interaction**: The visual shows "Try 'fix lint errors'" input prompts in the terminal. This maps to **A2UI Input Actions** (sending a signal to the running workflow).

## 6. Strategic value for Nostra
Adopting Auto Claude's "Visual Management" style solves a key UX problem for Nostra: **invisibility of agent work**.
*   **Problem**: Background canisters are invisible. Users don't know if they are working or stuck.
*   **Solution**: The "Nostra Dashboard" (inspired by Auto Claude) makes invisible background processes visible and tangible.

## 9. Enforcement & Standards (Governance)
User Question: *How do we ensure consistent experience?*

We use a "Constraint-based" approach rather than a "Permission-based" one.

### 1. The Component Registry (Hard Constraints)
Agents cannot write arbitrary HTML or CSS. They can only instantiate components from the **Nostra A2UI Registry**.
*   **Concept**: A "Lego Kit". If there is no "Blinking Marquee" block in the kit, the agent cannot build one.
*   **Enforcement**: The A2UI Renderer (Client) simply ignores unknown component types or properties.

### 2. The Theme Engine (Visual Consistency)
All components (Cards, Buttons, Inputs) are **Unstyled logic wrappers** that inherit styles from the Nostra Design System (tokens).
*   **Result**: Even if a "Bad Agent" tries to make a red button, if the `Button` component only accepts `variant="primary"` (which is blue), it will be blue.

### 3. The UI Linter Agent (Soft Guidance)
A specialized "UX Reviewer" agent runs before user display.
*   **Role**: Analyzes the A2UI JSON payload.
*   **Checks**: "Too many items in a row?", "Is the text contrast accessible?", "Is the 'Delete' action hidden sufficiently?".
*   **Action**: Rejects the payload or wraps it in a "Warning" block if it violates guidelines.

## 10. Labs Strategy (Safe Testing Ground)
User Question: *Do we need a Labs project to safely tweak/test?*

**Yes.** We will leverage **Nostra Labs (034)** as the "Storybook" for A2UI.

### The "A2UI Workbench" Lab
A dedicated section in Labs (`labs:a2ui`) where developers (and Agents!) can:
1.  **Component Playground**: Render isolated A2UI components (e.g., input a JSON snippet for a `KanbanBoard` and see it render live).
2.  **Theme Tester**: Apply different "Design Tokens" (Light/Dark/High Contrast) to see how they affect all components instantly.
3.  **Linter Dry-Run**: Paste a full Agent payload and run the "UI Linter" to see what warnings it generates without executing the agent.
4.  **Latency Simulator**: A toggle to simulate 2s-5s network delay. This forces developers to verify that **Optimistic Updates** (e.g., dragging a card) work smoothly before the server confirms.
5.  **State Time Travel**: record the stream of `surfaceUpdate` messages and "scrub" back and forth to see how the UI evolved during an agent's execution.

This allows us to **evolve the standards** without breaking the production "Dev Space".

## 11. Next Steps
1.  **Design the Kanban Schema**: Define the A2UI JSON structure for a board, columns, and cards.
2.  **Define the "Dev Loop" Workflow**: Create a `dev_loop.sw.json` compatible with 013-nostra-workflow-engine.
3.  **Prototype**: Implement a simple "Mock Agent" that updates a Kanban card state to demonstrate the UI.
