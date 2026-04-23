---
id: '021'
name: kip-integration
title: 'Decisions: KIP Integration'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions: KIP Integration

This document tracks architectural decisions for the KIP Integration research initiative.

---

## DEC-001: Adopt KIP as Standard Interface

**Status**: PROPOSED
**Date**: 2026-01-16

**Context**:
Nostra needs a standard protocol for knowledge graph operations that supports AI agent memory, library organization, and monetization.

**Decision**:
Adopt LDC Labs KIP (Knowledge Interaction Protocol) as the standard interface for nostra-cortex.

**Rationale**:
1. KIP is the only ICP-specific standard for Neuro-Symbolic AI
2. Provides schema bootstrapping (self-describing graph)
3. Includes KnowledgeFi primitives for monetization
4. Reference implementations exist (Anda Cognitive Nexus)

**Alternatives Considered**:
- Custom naive graph (current): No interoperability, no standard
- RDF/JSON-LD: Too verbose, expensive on-chain
- Vector databases: Good for RAG, bad for structured reasoning

**Consequences**:
- Must implement KQL/KML parser in Motoko
- Existing API must remain backward compatible
- AI agents can use standard KIP tools

---

## DEC-002: Library = KIP Domain + Capsules

**Status**: PROPOSED
**Date**: 2026-01-16

**Context**:
Need to define how Nostra Libraries are structured and distributed.

**Decision**:
Package each Nostra Library as:
- KIP `Domain` concept for organizational container
- KIP Knowledge Capsules (`.kip` files) for content
- JSON manifest (`n-lib.json`) for metadata

**Rationale**:
1. Aligns with KIP specification
2. Capsules are idempotent (safe to re-apply)
3. Domains provide clear organizational boundaries
4. Manifest enables versioning and dependency tracking

**Consequences**:
- Must define N-Lib manifest schema
- Library registry stores Domain references
- Libraries can depend on other libraries

---

## DEC-003: Gardener Integrates with $system

**Status**: PROPOSED
**Date**: 2026-01-16

**Context**:
KIP defines `$system` as a maintenance agent. Nostra has "Gardener" agent role for graph maintenance.

**Decision**:
The Gardener agent IS the Nostra-specific implementation of KIP's `$system` pattern. They integrate, not replace each other.

**Rationale**:
1. Same responsibilities: orphan detection, merging, pruning
2. `$system` provides protocol patterns (SleepTask, consolidation)
3. Gardener provides Nostra domain knowledge (Contribution types, Spaces)

**Consequences**:
- Gardener must process `SleepTask` nodes
- Gardener workflows expressed as KIP operations
- Allows standard $system tools to work with Nostra graph

---

## DEC-004: KIP Adapter as Separate Module

**Status**: PROPOSED
**Date**: 2026-01-16

**Context**:
How to structure the KIP implementation in the codebase.

**Decision**:
Create a `kip/` module folder in nostra backend containing:
- `parser.mo` - KQL/KML parser
- `executor.mo` - Command execution
- `types.mo` - KIP-specific types
- `adapter.mo` - Mapping between Nostra types and KIP

**Rationale**:
1. Clear separation of concerns
2. Easier to test in isolation
3. Could potentially be extracted as shared library

**Consequences**:
- Existing graph.mo remains unchanged
- KIP adapter calls into graph.mo for storage
- Easier to swap storage implementation later
---

## DEC-005: Chronicle Events Use KIP Event Type

**Status**: ACCEPTED
**Date**: 2026-01-16

**Context**:
Living Libraries requires a Chronicle event system for temporal tracking. KIP has a built-in `Event` type for episodic memory.

**Decision**:
Use KIP's built-in `Event` type for Chronicle events with `event_class: "chronicle:*"` prefix.

**Event Class Values**:
- `chronicle:EntityCreated`
- `chronicle:EntityUpdated`
- `chronicle:RelationshipFormed`
- `chronicle:WorkflowCompleted`
- `chronicle:LibraryForked`

**Rationale**:
1. KIP's `Event` type already has all needed fields (timestamp, participants, summary, context)
2. Leverages `$self`/`$system` memory patterns for consolidation and decay
3. Compatible with KIP MCP server tools
4. No duplicate type definitions

**Consequences**:
- Chronicle events are standard KIP Events
- `event_class` prefix enables filtering: `FILTER(STARTS_WITH(?e.attributes.event_class, "chronicle:"))`
- Gardener can use standard `$system` patterns for event maintenance

---

## DEC-006: Composable Fork Policy

**Status**: ACCEPTED
**Date**: 2026-01-16

**Context**:
Users should be able to fork libraries (Curated → Personal). Need to define what carries over.

**Decision**:
Implement composable fork policy with role-based permissions, defined in library manifest.

**Fork Policy Schema** (in `n-lib.json`):
```json
{
  "fork_policy": {
    "allowed_roles": ["member", "contributor", "steward"],
    "include": {
      "schema": true,
      "seed_data": true,
      "workflows": false,
      "chronicle": false
    },
    "exclude_tags": ["private"],
    "attribution": {
      "source_metadata": true,
      "forked_from_link": true
    }
  }
}
```

**Rationale**:
1. Library owners control what can be forked
2. Role-based permissions respect governance structures
3. Attribution maintains provenance chain
4. Flexible include/exclude enables various fork scenarios

**Consequences**:
- Fork endpoint must validate caller role
- Manifest parser must handle `fork_policy`
- Forked items tagged with `metadata.source: "forked:OriginalLibrary:v1.0"`

---

## DEC-007: Event Storage Strategy

**Status**: ACCEPTED
**Date**: 2026-01-16

**Context**:
High-frequency events (e.g., `EntityViewed`) could overwhelm storage. Need efficient strategy.

**Decision**:
- **Full storage**: Mutation events (create, update, delete, relationship changes)
- **Aggregate only**: View events → `ViewSummary` per entity per week
- **Discard**: Query/search events (ephemeral, logs only)

**Rationale**:
1. Mutation events are critical for graph history and temporal replay
2. View aggregation provides insight without 1000x storage overhead
3. Query events have low value for long-term memory

**Gardener Workflow**: `aggregate-views` runs weekly to compact view data.

**Consequences**:
- Must implement `ViewSummary` concept type
- Gardener needs `aggregate-views` workflow
- Frontend can display view counts from summaries

---

## DEC-008: Gardener Scheduling via Off-Chain Cron

**Status**: ACCEPTED
**Date**: 2026-01-16
**Context**: The Gardener agent needs to run maintenance tasks (prune-orphans, aggregate-views, decay-confidence). Should this run on-chain (canister heartbeat) or off-chain (scheduled worker)?

**Decision**: **Off-chain cron scheduling** via the existing worker infrastructure.

**Options Considered**:
| Option | Pros | Cons |
|:---|:---|:---|
| On-chain heartbeat | Decentralized, no external dependency | Cycles cost, limited execution time, no external API access |
| Off-chain cron | Full compute, external APIs, existing infrastructure | Centralized dependency, requires monitoring |

**Rationale**:
1. Gardener may need to call external APIs (LLM for merge detection)
2. Maintenance tasks can be long-running (beyond canister limits)
3. Worker infrastructure already exists and is battle-tested
4. Can migrate to on-chain later if needed

**Implementation**:
- Add `gardener` role to worker capabilities
- Schedule via cron (e.g., every 6 hours)
- Gardener polls `SleepTask` nodes via KIP

---

## DEC-009: Dual API Surface (Raw KIP + Wrapped)

**Status**: ACCEPTED
**Date**: 2026-01-16
**Context**: Should we expose raw KIP commands, wrapped type-safe APIs, or both?

**Decision**: **Both APIs** with appropriate access controls.

| Layer | Purpose | Access | Security |
|:---|:---|:---|:---|
| **Raw KIP** (`execute_kip`) | AI agents, Gardener, power users | Authenticated + `kip-execute` capability | ACL per Domain |
| **Wrapped APIs** | Frontend, public consumers | Open (read) / Auth (write) | Input validation, bounded |

**Rationale**:
1. Raw KIP enables AI agents to use standard MCP tools
2. Wrapped APIs provide type safety and bounded queries for frontend
3. Defense in depth: frontend never calls raw KIP directly
4. Gradual migration path: start with wrapped, enable raw for trusted agents

**Security Controls for Raw KIP**:
- Caller must have `kip-execute` capability
- Domain ACL enforcement per command
- Rate limiting per caller
- Query complexity limits (depth, result count)
- Audit logging for all mutations

**Future Enhancement (TEE)**:
- **vetKD derived keys**: Workers prove authorization without exposing master credentials
- **Encrypted commands**: Sensitive KIP commands encrypted, decrypted only inside canister
- **Confidential domains**: Domains with `sensitivity: "confidential"` use threshold encryption

**Consequences**:
- Need capability-based access control in backend
- Frontend uses wrapped APIs only
- Workers/agents can use raw KIP with proper auth

---

## Decision Log

| ID | Decision | Status | Date |
|:---|:---|:---|:---|
| DEC-001 | Adopt KIP as standard interface | PROPOSED | 2026-01-16 |
| DEC-002 | Library = Domain + Capsules | PROPOSED | 2026-01-16 |
| DEC-003 | Gardener integrates with $system | PROPOSED | 2026-01-16 |
| DEC-004 | KIP adapter as separate module | PROPOSED | 2026-01-16 |
| DEC-005 | Chronicle events use KIP Event type | ACCEPTED | 2026-01-16 |
| DEC-006 | Composable fork policy | ACCEPTED | 2026-01-16 |
| DEC-007 | Event storage strategy (aggregate views) | ACCEPTED | 2026-01-16 |
| DEC-008 | Gardener off-chain cron scheduling | ACCEPTED | 2026-01-16 |
| DEC-009 | Dual API surface (Raw KIP + Wrapped) | ACCEPTED | 2026-01-16 |

---

## References

- [RESEARCH.md](./RESEARCH.md) - Research analysis
- [PLAN.md](./PLAN.md) - Implementation phases
- [REQUIREMENTS.md](./REQUIREMENTS.md) - Technical requirements
- [FEEDBACK.md](./FEEDBACK.md) - Resolved questions
