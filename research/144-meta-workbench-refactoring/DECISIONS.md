---
id: "144-meta-workbench-refactoring"
name: "meta-workbench-refactoring"
title: "Decisions & Context: Meta Workbench Refactoring"
type: "decisions"
project: "cortex"
status: draft
authors:
  - "Antigravity Agent"
tags: ["workbench", "space", "meta", "ui"]
created: "2026-03-13"
updated: "2026-03-13"
---

# Architecural Decisions & Context

## Overview
This document logs the structural decisions made during the design and execution of the Meta Workbench concept.

---

## Decisions Log

### DEC-1: Leveraging Omitting `space_id` for Global Queries
**Context:** When the user enters the "Meta Workbench", we need to load blocks across all spaces.
**Decision:** We will use `get_cortex_heap_blocks(None)` (omitting `space_id`) rather than passing an explicit `meta` string to the query payload.
**Rationale:** The backend `server.rs` already features exactly this logic. By omitting the `space_id`, the block projection skips the `workspace_id` filter loop, aggregating naturally.
**Alternatives Considered:** Passing `space_id: "meta"` or modifying `get_cortex_heap_blocks` to treat the `"meta"` string natively. This was rejected because the `space_id` type in some structs is `Option<String>`, meaning `None` is semantically correct for "no specific space constraint".

### DEC-2: Intercepting `"meta"` for Navigation Plans
**Context:** The `get_space_navigation_plan` requires determining what global views to show when a user isn't in a specific space.
**Decision:** The frontend will explicitly pass `"meta"` as the `space_id` for navigation plan requests when in the Meta Workbench view.
**Rationale:** Unlike block querying, navigation plans *must* be concrete sets of UI targets. If `space_id` is null, it's an error. `"meta"` acts as a concrete trigger to return the static global graph, bypassing the space capabilities logic cleanly.

### DEC-3: Global State `activeSpaceId` Optionality
**Context:** `uiStore.ts` tracks the active space.
**Decision:** We will allow `activeSpaceId` to hold the literal string value `"meta"`.
**Rationale:** This minimizes type signature disruption across the frontend and forces components (like `WorkbenchSurfaceView`) to explicitly handle the `"meta"` context check.

---

## Unresolved Questions
1. How do we cleanly handle "creation context" within the Meta Workbench? If a user creates a block while standing in the "meta" view, what space does it default to?
2. How should multi-space visual differentiation be applied (e.g., small colored context chips) to ensure users know where a block lives?

---

## References

- [PLAN.md](./PLAN.md) - Implementation phases
- [REQUIREMENTS.md](./REQUIREMENTS.md) - Architectural decisions
