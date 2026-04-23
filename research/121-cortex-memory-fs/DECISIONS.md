---
id: '121'
name: cortex-memory-fs
title: 'Decisions for Initiative 121: Cortex Memory FS'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-20'
updated: '2026-02-20'
---

# Decisions for Initiative 121: Cortex Memory FS

## ADR-121-001: Separation of Episodic Memory (FS) and Semantic Memory (DPub)

**Date**: 2026-02-20
**Status**: Accepted

**Context**:
We need a robust mechanism to manage the memory of LLM-based agents executing long-horizon tasks. Initially, there were assumptions that we might expand the DPub schema or use dedicated databases to handle all agent context. However, unstructured DB approaches hinder agent branching, debugging, and cross-session portability. Moreover, expanding DPub to log ephemeral reasoning conflates "Working Memory" with published, canonical "Semantic Knowledge".

**Decision**:
1. **Episodic/Working Memory** will be implemented as a Git-backed local filesystem (Cortex Memory FS). Agents will use explicit semantic commands (`branch`, `commit`, `merge`) to manage their trajectory and isolate experimental thinking.
2. **Semantic/Publication Memory** will remain governed by Nostra DPubs. DPubs will serve as the curated, confidence-weighted "textbooks" that agents ingest for pristine context, and publish to only upon successfully synthesizing their working branches.

**Consequences**:
- Agents gain zero-cost branching for sandboxed experimentation (e.g., Godot simulations, Labs).
- DPub remains a clean, high-confidence artifact layer, preventing the "Monster Node" anti-pattern and aligning perfectly with ICP's deterministic time constraints (DTS).
- Our architecture fundamentally aligns with SOTA agent workflows, dramatically improving our ability to orchestrate parallel memory swarms.
