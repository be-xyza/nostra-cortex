---
id: '023'
name: cortex-flashcards-use-case
title: 'Requirements: Cortex Flashcards & Recursive Libraries'
type: use-case
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Cortex Flashcards & Recursive Libraries

## 1. Data Structures (Schema)

### 1.1. The Source (Any Library)
The app must be able to ingest *any* library.
*   **Requirement**: A standard interface for extracting "Learnable Units" from a library.
*   **Structure**: `Entity (Concept)` -> `Attribute (Definition)` or `Relationship (Connection)`.

### 1.2. The App Logic (Flashcard Generation)
*   **Flashcard Object**:
    *   `front`: Text / Image (Prompt)
    *   `back`: Text / Image (Answer)
    *   `source_entity_id`: Reference to the original node.
    *   `difficulty`: Estimated difficulty (1-10).
*   **Generator**: A service (likely AI Agent) that transforms Library Nodes -> Flashcards.

### 1.3. The Output (Knowledge Library)
A valid Nostra Library representing the user's state.
*   **Library Archetype**: `Personal`
*   **Domain**: `UserKnowledge`
*   **Entity Structure**:
    *   Mirror the Source Library's IDs (or reference them).
    *   **Attributes**:
        *   `repetition_count`: Int
        *   `last_recalled`: Timestamp
        *   `next_due`: Timestamp (Spaced Repetition algorithm)
        *   `mastery_score`: Float (0.0 - 1.0)
    *   **Relationships**:
        *   `mastered` -> Reference to Source Entity.

## 2. Workflow Engine Integration

The "Game" is a Workflow.
*   **Workflow Type**: `InteractiveSession`
*   **Steps**:
    1.  **Initialize**: Load "Knowledge Library" state (or create if new).
    2.  **Select**: Query "Knowledge Library" for items `where next_due < now()` OR new items.
    3.  **Generate**: If utilizing AI, generate card content for selected items.
    4.  **Interact**: User Loop (Present Card -> User Input -> Feedback).
    5.  **calc**: Update Spaced Repetition metadata (SM-2 or similar algorithm).
    6.  **Persist**: Batch update the "Knowledge Library".

## 3. Cortex User Interface

### 3.1. Library Context Menu
*   **Location**: Library Card / Header.
*   **Trigger**: "Run App" or "Play".
*   **Discovery**: The UI must query a registry to find Apps compatible with `type: Library`.

### 3.2. App View
*   **Mode**: Fullscreen / Focused.
*   **State Awareness**: The app must show "Session Progress" and "Overall Mastery".

## 4. Gap Analysis (The Test)

This architecture must verify:
*   **Cross-Library Linking**: Can Library B (Knowledge) robustly reference entities in Library A (Source) even if Library A changes?
*   **Write Permissions**: The App is a 3rd party tool. How does it get permission to write to the User's "Personal Knowledge Library"?
*   **Agent Latency**: Can we generate flashcards on the fly, or must they be pre-materialized?
