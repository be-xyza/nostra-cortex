---
id: '006'
name: sentiment-capture-patterns
title: 'Decision Log: Sentiment Capture Patterns'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decision Log: Sentiment Capture Patterns

## DEC-001: Sentiment as Graph Relationships
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. **Property on Entity**: `entity.likes = 5`. Simple, but loses "who" liked it and when. Prone to write contention.
2. **Relationship (Edge)**: `(User)-[:REACTED {type: "🔥"}]->(Entity)`. Tracks provenance, timestamp, flexible.
3. **Separate Canister**: Store likes in a dedicated "Likes Canister". scalable but adds complexity.

**Decision**: **Relationship (Edge)** (Option 2).

**Rationale**:
- **Provenance**: Critical to know *who* reacted for reputation systems and filtering bots.
- **Flexibility**: Can easily add new reaction types without schema migration.
- **Graph-Native**: Aligns with the overall KG architecture.
- **Aggregation**: Can be aggregated asynchronously into a cached property for read performance.

## DEC-002: Hybrid Auto-Detection
**Date**: 2026-01-14
**Status**: ✅ Decided

**Options Considered**:
1. **Strict Manual**: User must click the button.
2. **Regex Implementation**: Scan text for specific unicode characters. Fast, cheap.
3. **AI Extraction**: Send text to LLM to determine sentiment. Accurate but slow/expensive.

**Decision**: **Hybrid Strategy**.
1. **Fast Path**: Regex scan for specific emojis (e.g., ❤️, 👍, 🔥) in user text -> auto-create reaction.
2. **Slow Path**: (Future) AI Agent analyzes text for subtle sentiment and adds "inferred" sentiment edges.

## DEC-003: Asynchronous Aggregation
**Date**: 2026-01-14
**Status**: ✅ Decided

**Rationale**:
- Counting edges on every read is expensive (`O(N)`).
- We will implement a "Materialized View" pattern where a `sentimentScore` property on the Entity is updated periodically or on-write (time-decayed ranking can happen later via indexers).
