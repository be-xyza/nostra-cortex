---
id: ''
name: nostra-on-nostra
title: 'Requirements: Enabling the Research Process'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Enabling the Research Process

## 1. Technical Requirements

### A. Graph Linkage Store
*   **Capability**: Must support linking "ephemeral" data (Poll responses, Chat messages) to "structural" data (Feature nodes, Code artifacts).
*   **Schema**:
    *   `SentimentNode`: { score: Float, context: Text, source: Link }
    *   `Relation`: `MOTIVATES` (Feedback -> Feature), `BLOCKS` (Feedback -> Release).

### B. Workflow State Machine
*   **Capability**: A canister-based engine to track the state of a `ResearchInitiative`.
*   **Data Structure**:
    *   `WorkflowInstance`: { id: ID, current_step: StepID, history: [Log] }
    *   `ContributionPath`: A prospective set of steps available to users.

### C. Multi-Agent Interfaces
*   **Capability**: Standardized APIs for agents to:
    1.  Read the current state of a workflow.
    2.  Post artifacts (Summaries, Plans).
    3.  Trigger state transitions.

## 2. UI/UX Requirements

### A. The "Pathfinder" View
*   A visual interface showing the "Innovation Loop" as a map.
*   Users can see where different initiatives are stuck (e.g., "Blocked on Design").

### B. Integrated Feedback Tools
*   **In-App Polls**: Drag-and-drop components developers can place in the UI.
*   **Sentinel Mode**: A mode where users can highlight any UI element and leave context-aware feedback.

## 3. Practical Outputs (Deliverables)
1.  **Github Integration**: A workflow step that automatically creates a GitHub Issue/PR when a Research Initiative reaches the "Materialization" phase.
2.  **Feedback Canister**: A standardized canister for collecting and serving sentiment data.
