---
id: '040'
name: nostra-schema-standards
title: 'Research Findings: Nostra Schema Standards'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-30'
---

# Research Findings: Nostra Schema Standards

## 1. Evolution Strategy
(Updated via 052-openspg-analysis)

### 1.1 The "SPG" Shift
We are moving from a purely static Entity model to a **Semantic-enhanced Programmable Graph (SPG)** model.
-   **Old View**: Schema = Struct definitions.
-   **New View**: Schema = `Entity` (Static) + `Event` (Temporal) + `Concept` (Taxonomy).

### 1.2 Key Primitive: `EventType`
We will adopt OpenSPG's `EventType` to model high-volume transactional data (e.g., "User X read Book Y").
-   **Subject**: The Actor (User/Agent).
-   **Object**: The Target (Book/Asset).
-   **Time**: `occur_time` (Bi-temporal: Valid time vs Record time).

## 2. Recommendation
Implement `NostraSchema` in Rust using an Enum approach:
```rust
enum NostraType {
    Entity(EntityDef),
    Event(EventDef),
    Concept(ConceptDef)
}
```
This allows the `NostraGraph` canister to efficiently partition storage (Events -> Append Only Log, Entities -> State Map).

### 1.3 Agent Schemas (Elasticsearch Analysis)
(Updated via 053-elasticsearch-analysis)
We introduce standard schemas for the **Agent Capabilities** layer, modeled after Elasticsearch's declarative APIs.
*   **`AgentSwarmState`**: The versioned topology of the fleet.
*   **`AgentAction`**: Standard definition for `search`, `think` (produces reasoning), and `act`.
*   Reference: `research/053-elasticsearch-analysis/specs/`.

## 3. Reference Versioning Standard

> [!NOTE]
> Version pinning for schema references is defined in **080-dpub-standard**.

All schema references support three versioning modes (see [REFERENCE_VERSIONING.md](../080-dpub-standard/REFERENCE_VERSIONING.md)):

| Mode | Example | Behavior |
|:---|:---|:---|
| Floating | `@latest` | Always resolves to newest |
| Pinned | `@v1.0` or `@7f2a9c...` | Immutable hash-verified |
| Compatible | `@^1.0` | Semver range (≥1.0, <2.0) |

This standard applies to all URN references: `urn:nostra:contribution:...`, `urn:nostra:book:...`, `urn:nostra:asset:...`.
