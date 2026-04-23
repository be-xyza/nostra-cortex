---
id: '046'
name: nostra-system-standards
title: 'Decisions: Nostra System Standards (046)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-04'
---

# Decisions: Nostra System Standards (046)

## DEC-001: Separation of Concerns
- **Decision**: Adopt the "Three Pillars" architecture (Modularity, Composability, Portability).
- **Rationale**: Prevents technical debt as the ecosystem scales by enforcing strict horizontal boundaries.
- **Status**: Ratified.

## DEC-002: UI Architecture
- **Decision**: Mandate A2UI for agent-driven interfaces and Native Dioxus/React renderers for native/web shell clients.
- **Rationale**: decouple's core logic from presentation; allows for cross-platform rendering (Web, Native, CLI).
- **Status**: Implemented in Nostra Core.

## DEC-003: Durable Execution
- **Decision**: All side-effects must be durable (Temporal pattern).
- **Rationale**: Ensures reliability on the Internet Computer's asynchronous actor model.
- **Status**: Ratified (Integration with 047).

## DEC-004: Semantic Primitives
- **Decision**: Standardize on the "Core 5" (Entity, Link, Agent, Event, Locus).
- **Rationale**: Unified data structures for the Knowledge Graph, execution history, and contextual sovereignty.
- **Status**: Ratified.

## DEC-005: Canonical Interface Contracts
- **Decision**: Treat Candid `.did` files as the canonical public interface contract across Nostra canisters.
- **Rationale**: Candid is language-neutral and prevents drift between Motoko domain types and Rust/JS bindings.
- **Status**: Ratified.

## DEC-006: Stream vs Feed Terminology
- **Decision**: Use **stream** for append-only, time-ordered sequences (events, timelines). Use **feed** for materialized/published views (e.g., dPub `feed.json`, RSS/Atom-like outputs).
- **Rationale**: Prevents semantic drift and avoids conflating realtime transport/streams with published feeds; keeps dPub terminology aligned with syndication conventions without forcing breaking API renames.
- **Status**: Implemented in Nostra docs (2026-02-04).
