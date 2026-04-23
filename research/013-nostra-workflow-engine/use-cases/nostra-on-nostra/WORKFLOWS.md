---
id: ''
name: nostra-on-nostra
title: 'Workflows: The Nostra Engine'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Workflows: The Nostra Engine

To support the "Nostra on Nostra" initiative, we need a flexible Workflow Engine that guides users and agents through contribution paths.

## 1. The Workflow Engine Concept
Unlike linear project management tools, Nostra's workflow engine is **exploratory and branching**.

*   **Nodes**: Steps in a process (e.g., "Define Problem", "Write Spec", "Code MVP").
*   **Edges**: Transitions triggered by events or completions.
*   **Agents**: Automated participants that can perform specific steps (e.g., "Summarize Feedback").

## 2. Core Workflow: The Innovation Loop
This is the standard process for turning an idea into reality within Nostra.

### Phase 1: Inception (The "What?")
*   **Trigger**: New `Insight` or `Sentiment` cluster identified in the Graph.
*   **Step 1.1**: **Contextualize**. Link the insight to existing nodes (Features, Goals).
*   **Step 1.2**: **Question**. Formulate the core research question (e.g., "How do we enable private voting?").
*   **Output**: A new `ResearchInitiative` node.

### Phase 2: Exploration (The "How?")
*   **Trigger**: `ResearchInitiative` created.
*   **Step 2.1**: **Pathfinding**. The system generates potential exploration paths.
    *   *Path A (Social)*: "Create a poll to gauge interest."
    *   *Path B (Technical)*: "Search GitHub for existing libraries."
*   **Step 2.2**: **Contribution**. Users/Agents select a path and execute.
*   **Output**: `Plan` or `Spec` artifacts.

### Phase 3: Materialization (The "Build")
*   **Trigger**: Approved `Plan`.
*   **Step 3.1**: **Task Generation**. Break down plan into atomic `Contributions`.
*   **Step 3.2**: **Assignment/Claiming**.
*   **Step 3.3**: **Verification**.
*   **Output**: Code merged, Feature deployed.

## 3. Example Workflow Definition (YAML/Pseudocode)

```yaml
workflow: "Feature Evolution"
trigger: "High Negative Sentiment on Feature"
steps:
  - id: synthesis
    role: "Architect Agent"
    action: "Summarize feedback into Problem Statement"
    next: [ideation]

  - id: ideation
    type: "Branch"
    options:
      - label: "Propose UI Fix"
        workflow: "Design-Sprint"
      - label: "Propose Backend Fix"
        workflow: "Technical-Spike"

  - id: selection
    role: "Community"
    action: "Vote on approach"
```

## 4. Practical Implementation
*   **Frontend**: A "Quest Log" or "Research Map" UI that visualizes where an initiative is in the workflow.
*   **Backend**: A state machine stored in the `Space` canister tracking the progress of `Initiatives`.
