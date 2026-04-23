---
id: '021'
name: kip-integration
title: 'Feedback: KIP Integration'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Feedback: KIP Integration

This document tracks open questions, feedback, and resolutions for the KIP Integration research initiative.

> [!NOTE]
> See [STUDY_LIVING_LIBRARIES_INTEGRATION.md](./STUDY_LIVING_LIBRARIES_INTEGRATION.md) for the detailed analysis of Living Libraries × KIP alignment.

---

## Open Questions

### Q1: Full execute_kip vs Wrapped APIs?

**Question**: Should we expose a raw `execute_kip(command: text)` endpoint, or wrap KIP in Nostra-specific APIs like `createEntity()`, `queryGraph()`?

**Options**:
- A) Raw KIP only — Maximum flexibility for AI agents
- B) Wrapped only — Simpler for frontend, hide KIP complexity
- C) Both — Raw for agents, wrapped for frontend

**Status**: OPEN

---

### Q3: Gardener Scheduling?

**Question**: How should the Gardener agent be scheduled?

**Options**:
- A) On-chain heartbeat (canister timer)
- B) Off-chain cron job (external worker)
- C) Event-triggered (after N mutations)

**Status**: OPEN

---

### Q4: Backward Compatibility Strategy?

**Question**: How do we handle existing entities that lack KIP metadata fields?

**Options**:
- A) Migration script to add default metadata
- B) Lazy migration (add metadata on first access)
- C) Dual-schema support (old and new types)

**Status**: OPEN

---

---

## Resolved Questions

### Q2: Library Framework vs Agent Memory Priority? ✓

**Question**: Which should we implement first?

**Resolution**: **Neither first — validate both with Phase 0 (Nostra Core Library)**.

Phase 0 creates the Nostra Core Library as a test harness that exercises:
- Library framework (Domain + Capsules)
- Chronicle events (Agent memory primitive)
- Library archetypes (Personal/Curated/Shared)

This provides an integrated test before committing to either full phase.

**Status**: RESOLVED — Phase 0 added to PLAN.md

---

### Q5: ChronicleEvent Type Implementation ✓

**Question**: Should ChronicleEvent use KIP's built-in `Event` type or be a distinct `$ConceptType`?

**Resolution**: **Option A — Use KIP's built-in `Event` type**.

**Validation**: KIP's `Event` type (from `Event.kip`) is well-suited for Chronicle events:

| KIP Event Field | Chronicle Use |
|:---|:---|
| `event_class` | Maps to Chronicle category: "EntityCreated", "WorkflowCompleted", etc. |
| `start_time` | Event timestamp |
| `participants` | Actor principal(s) involved |
| `content_summary` | Brief description of what happened |
| `key_concepts` | Links to affected entities (via names) |
| `outcome` | Result of the action |
| `context` | Library ID, Space ID, workflow context |

**Benefits**:
- Leverages `$self`/`$system` memory patterns (consolidation, decay)
- Compatible with KIP MCP server tools
- No duplicate type definitions

**Implementation**: Use `event_class` values like:
- `"chronicle:EntityCreated"`
- `"chronicle:RelationshipFormed"`
- `"chronicle:LibraryForked"`

**Status**: RESOLVED — Use KIP `Event` with `event_class: "chronicle:*"` prefix

---

### Q6: Fork Semantics ✓

**Question**: When a user forks a Curated Library into a Personal one, what carries over?

**Resolution**: **Composable forks with role-based permissions**.

**Fork Configuration Schema**:
```json
{
  "fork_policy": {
    "allowed_roles": ["member", "contributor", "steward"],
    "include": {
      "schema": true,       // $ConceptType definitions
      "seed_data": true,    // Initial concepts
      "workflows": false,   // Workflow definitions
      "chronicle": false    // Event history
    },
    "exclude_tags": ["private", "internal"],
    "attribution": {
      "source_metadata": true,
      "forked_from_link": true
    }
  }
}
```

**Fork Process**:
1. Check caller's role against `fork_policy.allowed_roles`
2. Create new Domain with `metadata.forked_from: "OriginalLibrary"`
3. Copy items based on `include` flags
4. Apply `exclude_tags` filter
5. Tag copied items with `metadata.source: "forked:OriginalLibrary:v1.0"`

**Owner Controls**: Library steward defines `fork_policy` in library manifest (`n-lib.json`).

**Status**: RESOLVED — Composable fork policy stored in library manifest

---

### Q7: High-Frequency Event Handling ✓

**Question**: How to handle high-frequency events like `EntityViewed`?

**Resolution**: **Don't store individual view events — aggregate to ViewSummary**.

**Optimal Path for Efficiency**:

| Event Type | Storage Strategy |
|:---|:---|
| `EntityCreated`, `EntityUpdated` | **Full storage** — Critical for graph history |
| `RelationshipFormed`, `RelationshipRemoved` | **Full storage** — Critical for graph history |
| `WorkflowCompleted` | **Full storage** — Important milestones |
| `EntityViewed` | **Aggregate only** — See below |
| `QueryExecuted` | **Discard** — Ephemeral, logs only |

**ViewSummary Aggregation**:
```prolog
// Instead of: 1000 EntityViewed events per entity per day
// Store: 1 ViewSummary per entity per week

UPSERT {
  CONCEPT ?view_summary {
    {type: "ViewSummary", name: "views:icp:2026-W03"}
    SET ATTRIBUTES {
      entity_id: "icp",
      period: "2026-W03",
      view_count: 47,
      unique_viewers: 12,
      peak_day: "2026-01-16"
    }
  }
}
WITH METADATA { source: "aggregation:gardener", author: "$system" }
```

**Gardener Workflow**: `aggregate-views` runs weekly to compact view data.

**Status**: RESOLVED — Aggregate views, full storage for mutations

---

### Q8: Visual Storytelling Scope ✓

**Question**: Is "Timeline Scrubber" and "Growth Animations" in scope for KIP integration, or deferred to design research?

**Resolution**: **Already implemented in Cortex — document integration only**.

The "temporal play feature" in Cortex graph view provides:
- Timeline scrubbing
- Animated replay of graph evolution
- Event markers

**Scope for 021-kip-integration**:
- Backend: Expose KQL temporal queries (`getGraphAtTime`)
- Frontend: Connect existing Cortex feature to KQL backend

**No new UI work needed** — leverage existing implementation.

**Status**: RESOLVED — Backend queries in 021, UI already exists in Cortex

---

## Feedback Received

### 2026-01-16: User Feedback on Implementation Plan

1. **motoko-maps-kg → nostra-cortex**: All references updated
2. **$system ↔ Gardener relationship**: Clarified as integration, not replacement
3. **Living Libraries integration requested**: Created `STUDY_LIVING_LIBRARIES_INTEGRATION.md`

### 2026-01-16: User Answers to Open Questions

1. **Q5**: Use KIP's `Event` type — validated as optimal
2. **Q6**: Composable forks with role-based permissions
3. **Q7**: Aggregate views for efficiency, full storage for mutations
4. **Q8**: Already implemented in Cortex temporal play feature

---

## References

- [RESEARCH.md](./RESEARCH.md) - Research analysis
- [PLAN.md](./PLAN.md) - Implementation phases
- [DECISIONS.md](./DECISIONS.md) - Architectural decisions
- [STUDY_LIVING_LIBRARIES_INTEGRATION.md](./STUDY_LIVING_LIBRARIES_INTEGRATION.md) - Living Libraries mapping
