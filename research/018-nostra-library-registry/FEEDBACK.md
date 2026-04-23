---
id: 018
name: nostra-library-registry
title: 'Feedback & Open Questions: Living Libraries'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Feedback & Open Questions: Living Libraries

Track open questions, user feedback, and areas needing clarification.

---

## Open Questions

### OQ-1: Privacy Boundaries for Personal Libraries

**Question**: For Personal Libraries, what events should be logged vs. kept transient?

**Context**:
- Logging every "entity viewed" event creates a detailed activity trail
- This could be useful for "interest" detection but raises privacy concerns
- Users may not want a permanent record of everything they explored

**Options**:
1. Log all events (maximum data, privacy concern)
2. Log only mutations, not reads (safer, less data)
3. User-configurable privacy settings per library

**Status**: 🔶 Open

---

### OQ-2: Chronicle Storage Costs

**Question**: Chronicle can grow large. What's the retention/pruning policy?

**Context**:
- ICP storage is relatively cheap but not free
- A highly active library could generate thousands of events/month
- Historical events have diminishing value over time

**Options**:
1. Keep all events forever (simple, expensive)
2. Prune events older than N days (data loss)
3. Archive old events to cheaper storage (complex)
4. Charge users for Chronicle storage (economic model)

**Status**: 🔶 Open

---

### OQ-3: Fork Semantics

**Question**: When a user "forks" a Curated Library into a Personal one, what carries over?

**Considerations**:
- Entities and relationships: Yes (that's the point)
- Workflows: Should personal copies of workflows be editable?
- Dependencies: Does the fork still depend on parent's dependencies?
- Updates: Can the fork "pull" updates from the parent?

**Status**: 🔶 Open

---

### OQ-4: Shared Library Merge Conflicts

**Question**: How do Shared Libraries handle conflicting contributions?

**Scenario**:
- User A and User B both modify the same entity concurrently
- Or: User A deletes an entity that User B is linking to

**Options**:
1. Last-write-wins (simple, data loss risk)
2. Governance-mediated (proposals, votes)
3. Branch-and-merge (Git-like, complex)

**Status**: 🔶 Open

---

### OQ-5: AI Agent Autonomous Growth

**Question**: Can AI agents autonomously grow a library based on detected patterns?

**Scenario**:
- AI detects a cluster forming around "Authentication"
- AI proactively creates a "Synthesis" contribution summarizing the cluster
- AI suggests new entities or relationships to fill gaps

**Considerations**:
- Autonomy level: Suggest vs. Draft vs. Publish
- Attribution: Who owns AI-created contributions?
- Governance: Does AI action require approval?

**Status**: 🔶 Open

---

## User Feedback

*(To be populated as feedback is collected)*

---

## Research Gaps

### RG-1: Graph Algorithm Selection

**Gap**: Which graph algorithms are optimal for pattern detection on ICP?

**Needed Research**:
- Benchmark Louvain vs. Label Propagation for community detection
- Evaluate Rust graph libraries compatible with IC
- Determine if algorithms can run on-chain or need off-chain workers

---

### RG-2: Snapshot Optimization

**Gap**: What's the optimal snapshot frequency for temporal queries?

**Needed Research**:
- Profile query latency vs. snapshot frequency
- Determine storage cost vs. latency tradeoff
- Consider adaptive snapshotting (more frequent during high activity)

---

### RG-3: Visual Storytelling UX

**Gap**: How do users want to experience their library's growth?

**Needed Research**:
- User interviews on timeline/replay expectations
- Competitive analysis (how do other tools visualize growth?)
- A/B testing of different visual metaphors (timeline vs. rings vs. layers)
