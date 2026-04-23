---
id: '004'
name: unified-architecture-gaps
title: 'Unified Architecture Gap Analysis: Nostra v2 & Motoko Maps KG'
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Unified Architecture Gap Analysis: Nostra v2 & Motoko Maps KG

**Date**: 2026-01-24
**Status**: ACTIVE
**Context**: Coordinating architecture between [Nostra v2](../002-nostra-v2-architecture/PLAN.md) and [Motoko Maps KG](../001-multi-project-architecture/PLAN.md).
**Naming Standard**: Aligned with [Nostra Cortex](../README.md) (Nostra = Platform, Cortex = Execution).

> [!NOTE]
> **Workflow Engine (Section 1.3)** is now fully implemented in [013-nostra-workflow-engine](../013-nostra-workflow-engine/PLAN.md) as part of the **Cortex Execution Layer**. This initiative focuses on the remaining gaps: Governance, Discovery, and Agent infrastructure.


---


## Overview

This research analyzes the gaps between the two major projects and proposes a unified architecture.
Detailed breakdowns are available in:
- **[PLAN.md](PLAN.md)**: Roadmap and execution steps.
- **[DECISIONS.md](DECISIONS.md)**: Key architectural decisions.
- **[REQUIREMENTS.md](REQUIREMENTS.md)**: Component mapping and requirements.
- **[FEEDBACK.md](FEEDBACK.md)**: Questions and feedback.

---

## 1. Missing Systems Map

The complete operational picture requires mapping these missing components that sit *between* or *above* the two defined projects.

### 1.1 User Management (Unified Profile)
**Current State**:
- **Nostra**: Mentions "Profile" module (preferences, history).
- **KG**: `Registry` canister tracks `owner` and `collaborators` (Principal-based).
- **Gap**: No shared identity profile. If I am "Alice" in Nostra, I should be "Alice" in KG.

> **Solution**: See [PLAN.md](PLAN.md#1-unified-user-identity) and [DECISIONS.md](DECISIONS.md#1-shared-user-index-in-kg-registry).

### 1.2 Governance System (Future Home)
**Current State**:
- **Nostra**: Deferred to v3.
- **KG**: "ICP Canon" updates via DAO.
- **Gap**: Where does the logic live?

> **Solution**: See [PLAN.md](PLAN.md#2-governance-system) and [DECISIONS.md](DECISIONS.md#3-generic-governance-host).

### 1.3 Workflow Engine (Process Design)
**Current State**:
- **Nostra**: Simple permissions/roles.
- **Gap**: No way to define *processes* (e.g., "Elect a Steward", "Request Funding", "Task Assignment Protocol").

> **Solution**: See [PLAN.md](PLAN.md#3-cooperative-workflow-engine).

### 1.4 Indexing & Router (Discovery Layer)
**Current State**:
- **Nostra**: "Activity" streams (temporal).
- **KG**: Project-isolated queries.
- **Gap**: "Show me everything about 'DeFi' across all my Nostra spaces and KG projects."

> **Solution**: See [PLAN.md](PLAN.md#4-global-discovery-index).

### 1.5 Agents & Automation (Cortex Layer)
**Current State**:
- **KG**: `ic-rmcp` based `mcp-server` canister.
- **Nostra**: "Controlled writes" / "Draft mode".
- **Gap**: How does a **Cortex Agent** *act* on Nostra?


> **Solution**: See [PLAN.md](PLAN.md#5-agents--automation-mcp) and [DECISIONS.md](DECISIONS.md#4-unified-mcp-router).
