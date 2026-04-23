---
id: 018
name: nostra-library-registry
title: 'Decision Log: Living Libraries'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decision Log: Living Libraries

Track architectural decisions for the Living Library concept.

---

## DEC-001: Retain "Library" Terminology

**Date**: 2026-01-16
**Status**: ✅ Decided

**Options Considered**:
1. Rename to "Repository" (GitHub-style)
2. Rename to "Knowledge Pack" or "Domain Pack"
3. Rename to "Module" or "Package"
4. Retain "Library" with explicit redefinition

**Decision**: Option 4 - Retain "Library" with Living System semantics

**Rationale**:
- "Repository" implies version control and code management (PRs, Issues) which contradicts our "no code management" design
- "Pack" is unfamiliar and too generic
- "Module" overlaps with Motoko module system
- "Library" aligns with physical library metaphor: curated knowledge collections, organized by domain, browsed and discovered

**Implications**:
- Documentation must clearly distinguish from "programming library"
- UI should emphasize the "living/growing" aspect rather than "static package"

---

## DEC-002: Three Library Archetypes

**Date**: 2026-01-16
**Status**: ✅ Decided

**Options Considered**:
1. Single library type with permissions
2. Two types: System vs. User
3. Three archetypes: Curated, Personal, Shared

**Decision**: Option 3 - Three distinct archetypes

**Rationale**:
- **Curated**: Publisher-maintained reference knowledge (like textbooks)
- **Personal**: User-owned growing knowledge (like a personal journal/garden)
- **Shared**: Community-stewarded commons (like a wiki)
- Each has fundamentally different governance and growth patterns

**Implications**:
- `LibraryArchetype` variant type added to schema
- Different access control rules per archetype
- UI must distinguish visual presentation by archetype

---

## DEC-003: Chronicle as Append-Only Event Log

**Date**: 2026-01-16
**Status**: ✅ Decided

**Options Considered**:
1. Store full graph snapshots periodically
2. Store event log (Chronicle) and compute snapshots on demand
3. Hybrid: Event log + periodic materialized snapshots

**Decision**: Option 3 - Hybrid approach

**Rationale**:
- Pure snapshots: Storage-heavy, loses granular history
- Pure event log: Replay is expensive for large histories
- Hybrid: Event log provides full history; periodic snapshots optimize temporal queries

**Implications**:
- Chronicle is append-only (immutable history)
- Snapshot service runs periodically (e.g., daily/weekly)
- `getGraphAtTime()` uses nearest snapshot + event replay

---

## DEC-004: Personal Library = One Per Principal

**Date**: 2026-01-16
**Status**: ✅ Decided

**Options Considered**:
1. One Personal Library per user (single garden)
2. Multiple Personal Libraries per user (many gardens)

**Decision**: Option 1 - One Personal Library per principal (for MVP)

**Rationale**:
- Simplifies mental model: "My Library" is unambiguous
- Reduces fragmentation of personal knowledge
- Multiple can be added later as a feature upgrade

**Implications**:
- `createPersonalLibrary` fails if user already has one
- `getMyLibrary` always returns the single library
- Future: May add "Sub-libraries" or "Collections" within Personal Library

---

## DEC-005: Fork Creates Independent Chronicle

**Date**: 2026-01-16
**Status**: ✅ Decided

**Options Considered**:
1. Fork shares chronicle with parent library
2. Fork starts fresh chronicle, records only fork event
3. Fork copies chronicle history and continues independently

**Decision**: Option 2 - Fresh chronicle with fork event

**Rationale**:
- Shared chronicle would leak activity between independent libraries
- Copying history is expensive and confusing (whose events are whose?)
- Fresh start with "LibraryForked" event clearly marks lineage

**Implications**:
- Fork event includes `parentLibraryId` in metadata
- Forked library's Chronicle starts at timestamp of fork
- Historical analysis can reconstruct lineage via parent references

---

## DEC-006: Pattern Detection Deferred to Phase 3

**Date**: 2026-01-16
**Status**: ✅ Decided

**Decision**: Pattern detection (clusters, hubs, orphans) is Phase 3 scope

**Rationale**:
- Requires significant graph algorithm implementation
- Depends on stable Chronicle and temporal query infrastructure
- High value but not blocking core Living Library functionality

**Implications**:
- Phase 1 & 2 focus on Chronicle, growth tracking, and Timeline Scrubber
- Pattern detection integrates with AI agent framework (017)
