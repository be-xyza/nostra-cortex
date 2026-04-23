---
id: '021'
name: kip-integration
title: 'Research: Capability-First Architecture (formerly KIP Integration)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-01'
updated: '2026-02-01'
---

# Research: Capability-First Architecture (formerly KIP Integration)

**Status**: PIVOTED
**Date**: 2026-02-01

## Context
Originally, Initiative 021 aimed to make KIP (Knowledge Interaction Protocol) the *foundational layer* of Nostra.
However, deep analysis revealed a conflict between KIP's language-first approach and Nostra's constitutional invariants (Versioning, Scope Preservation, Provenance).

## Decision: The "Adapter" Pattern
We have decided to downgrade KIP to an **Adapter** and elevate **Capability Interfaces** to the foundational Layer 0.

### The New Stack
1.  **Layer 0: Core Primitives (MCI/QCI)**
    - Strictly typed Rust/Motoko interfaces.
    - Enforce invariants *before* any language parsing.
    - Defined in [CAPABILITY_INTERFACES.md](./CAPABILITY_INTERFACES.md).

2.  **Layer 1: Adapters**
    - **KIP Adapter**: Parses KQL/KML -> MCI/QCI.
    - **GraphQL Adapter**: (Future) Parses GraphQL -> MCI/QCI.
    - **Candid Adapter**: Direct canister calls -> MCI/QCI.

3.  **Layer 2: Clients**
    - Cortex UI, Agents, CLI.

## Value
This ensures that "History is Sacred" and "Space Sovereignty" are enforced by the *system*, not by a specific query language parser.

## References
- [CAPABILITY_INTERFACES.md](./CAPABILITY_INTERFACES.md)
- [DECISIONS.md](./DECISIONS.md) (_Updated_)
