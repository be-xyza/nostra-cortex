---
id: 028
name: a2ui-integration-feasibility
title: A2UI Integration Feasibility
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# A2UI Integration Feasibility

**Status**: APPROVED (Critical Path for 013)
**Owner**: Nostra Architecture Team
**Created**: 2026-01-17

## Objective
Evaluate the feasibility and benefits of integrating **Google's A2UI (Agent-to-User Interface)** protocol into the Nostra ecosystem. A2UI provides a standard JSON format for AI agents to generate rich, interactive user interfaces that are rendered by specific client-side libraries.

## Executive Summary
A2UI represents a paradigm shift for Nostra's "Agentic" vision. Instead of hardcoding UIs for every possible workflow or agent interaction, we can implement a generic A2UI Renderer. Agents (including the Workflow Builder and specialized domain agents) can then "speak UI," generating wizards, forms, and dashboards dynamically.

This aligns perfectly with:
1.  **027 Workflow Builder**: Unexpected org configurations need dynamic setup wizards.
2.  **013 Workflow Engine**: `UserTask` steps require flexible form definitions.
3.  **017 AI Agents**: Agents need a way to present complex results beyond text.

## Core Concepts
1.  **JSONL Protocol**: A stream of `surfaceUpdate` and `dataModelUpdate` messages.
2.  **Adjacency List**: Flat component list (LLM-friendly) rather than deep nesting.
3.  **Widget Registry**: Client-side mapping of abstract types (`Row`, `Card`) to native components.
4.  **Separation of Concerns**: Structure (Components) vs. State (Data Model).

## Strategic Fit for ICP & Nostra

### Benefits
*   **Dynamic UX**: The "Workflow Builder" can generate a bespoke setup wizard for a "Dental Practice DAO" on the fly, without us writing a specific "Dental Practice" component.
*   **Decoupled Architecture**: Backend canisters/agents define the UI. Updating the UI logic doesn't require redeploying the frontend asset canister.
*   **Safety**: We execute a JSON description of a UI, not arbitrary JavaScript from an agent. This is crucial for a decentralized, trust-minimized platform.
*   **Standardization**: Adopting an open standard (Google's) saves us from reinventing a "Nostra UI Protocol."

### Drawbacks & Challenges
*   **Frontend Complexity**: We must build a robust **A2UI Renderer** in Dioxus/Rust (or React if we switch). This is a non-trivial engineering lift.
*   **State Management**: Syncing client-side state with the agent's data model across the IC boundary (latency) requires careful design.
*   **Component Catalog**: We need to define and maintain a "Nostra Catalog" of components (`GraphView`, `ProposalCard`) and ensure agents stick to it.

## Cross-Initiative Resolution

How A2UI integrates with existing research tracks:

| Initiative | A2UI Application |
|:-----------|:-----------------|
| **013 Workflow Engine** | The **`form_definition`** for a `UserTask` can be an A2UI payload. The engine sends the JSON, the client renders the form, and the user submits data back as a `completeStep` action. |
| **027 Workflow Builder** | The **"Wizard"** itself is an A2UI session. An AI agent generates the wizard steps based on the user's prompt ("I want a non-profit for cat rescue"). |
| **007 Nostra Spaces** | Spaces can have a "Dashboard" surface. Agents can push widgets (charts, lists) to this dashboard via A2UI `surfaceUpdate` messages. |
| **018 Library Registry** | Libraries can export **Custom A2UI Components** (e.g., a specific "Vote Slider" or "Treasury Chart") added to the client's registry. |
| **018 Agent Tools** | Tools can return **Interactive Surfaces** (Charts, Previews) instead of text. See [A2UI Tool Response](../018-nostra-library-registry/A2UI_TOOL_RESPONSE_STD.md). |
| **025 OneKE Search** | Search results can be returned as A2UI cards, allowing rich, interactive answers (e.g., a mini-calculator for a finance query). |

## Recommendation
**Proceed with Integration.** The value for the Workflow Builder alone justifies the investment. It solves the "long tail" of organization types by allowing AI to construct the configuration UI dynamically.
