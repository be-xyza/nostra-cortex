---
id: '070'
name: a2ui-testing-ground
title: A2UI Testing Ground Research
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# A2UI Testing Ground Research

**Status**: PROPOSED
**Connected Initiative**: 028 A2UI Integration Feasibility
**Created**: 2026-01-26

## Objective
Enhance the A2UI Rendering Lab (`ui_studio_lab.rs`) to serve as a robust **Testing Ground** for agent-generated interfaces. This facility will allow users and agents to define scenarios with specific outcomes and constraints, generate A2UI workflows, and catalog successful patterns for future training and reuse.

## Context & Findings
*   **Existing Capabilities**:
    *   `a2ui.rs`: Defines the A2UIRenderer and Schema (Container, Row, Input, etc.).
    *   `a2ui_lab.rs`: A basic JSON editor and previewer.
    *   `ui_studio_lab.rs`: An advanced "UI Studio" that connects to an Agent to generate UIs from prompts and allows saving presets.
*   **The Gap**:
    *   Current generation is "Prompt -> Output". There is no formal "Scenario" definition (Constraints, Context, Expected Outcome).
    *   No automated validation (e.g., "Did the agent actually include the required 'email' field?").
    *   Focus is on single screens, not multi-step workflows.

## Proposed Solution: The "A2UI Validator"
Transform the "UI Studio" into a "Validator" or "Testing Ground" where:

1.  **Scenario Definition**: Users define a **Test Case**:
    *   *Goal*: "Create a signup form"
    *   *Context*: Mocked User Profile (e.g., "Admin", "New User") and System State.
    *   *Constraints*: ["Must collect email", "Must have Terms of Service checkbox", "Theme: Dark Mode"]
    *   *Data Model*: Initial state (optional).
    *   **Feature: AI Enhancement**: Users can click "Enhance" on any constraint or goal. The AI will expand it to reinforce Nostra standards (e.g., "Must collect email" -> "Must collect email and validate format per `040-schema-standards`").
2.  **Generation Loop**: The Agent generates the A2UI JSON.
3.  **Validation**:
    *   **Automated**: System checks for presence of required components/fields (by ID or Label).
    *   **Deep Integration (Real Scenarios)**:
        *   **Action dry-run**: When a button is configured to call `submit_outcome`, the validator checks if that payload matches the active `UserTask` schema.
        *   **State Simulation**: Renders the UI with *injected context* to verify conditional logic (e.g., "Logout" button only appears if `user.is_logged_in`).
    *   **Manual**: User reviews the rendered UI and marks it as "PASS" or "FAIL".
4.  **Cataloging (The Testing Index)**:
    *   Results are stored in a **Testing Index** (initially a persistent JSON store, later a Vector DB).
    *   This allows semantic search: "Show me all passed tests for 'Login Forms'".
    *   Serves as a regression suite to ensure new agent versions don't break known good patterns.

## Implementation Strategy
1.  **Enhance `ui_studio_lab.rs`**:
    *   Add a "Test Mode" toggle.
    *   Split the "Prompt" into "Goal" and "Constraints".
    *   Add a "Validation" panel showing passed/failed constraints.
2.  **Constraint Logic**:
    *   Simple heuristic checks (e.g., JSON traversal to find `Entry "type": "Input", "label": "Email"`).
3.  **Refined Storage**:
    *   Update `SavedUI` (or create new `SavedScenario` type) to include the metadata (Goal, Constraints, Pass/Fail status).

## Benefits
*   **Quality Assurance**: Ensures agents produce usable, correct UIs.
*   **Data Collection**: Builds a dataset of "Good UIs" for fine-tuning.
*   **Safety**:Verifies that critical elements (e.g., disclaimer text) are present.
