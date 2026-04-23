---
id: '067'
name: unified-protocol
title: 067 - Unified Protocol (Nostra Cortex)
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-03'
---

# 067 - Unified Protocol (Nostra Cortex)

> **Status**: ACTIVE (North Star)
> **Owner**: Nostra Architecture Team
> **Supersedes**: 001, 002, 004, 056
> **Integrates**: 013, 057, 028, 052

## Vision
Nostra is not a monolith. It is a **Unified Protocol** composed of three generic engines (The Trinity) that enable any coordination structure to be defined as data.

## The Trinity

1.  **Data Layer (KG)**: The "Memory" of the system.
    *   *Engine*: `nostra-kg` (Generic Graph + Schema Registry)
    *   *Primitive*: `Entity`
    *   *Origin*: Research 001

2.  **Logic Layer (Flow)**: The "Will" of the system.
    *   *Engine*: `nostra-workflow-engine` (Temporal-style State Machines), canonical implementation at `nostra/backend/workflow_engine/`
    *   *Primitive*: `Process`
    *   *Origin*: Research 013

3.  **Control Layer (Brain)**: The "Interaction" of the system.
    *   *Web*: Nostra Web (User Creation)
    *   *Desktop*: Cortex Desktop (Operator IDE)
    *   *Primitive*: `View` (A2UI)
    *   *Origin*: Research 057 / 028

## Why This Matters
By decoupling these layers, "Nostra" (the application) becomes just a **Configuration Package** (Schemas + Workflows) running on generic infrastructure. This ensures:
*   **Data Integrity**: Enforced by Constitutional Schemas (Data Layer).
*   **Composability**: Workflows can be swapped/forked (Logic Layer).
*   **Modularity**: UIs are generated, not hardcoded (Interface Layer).

## Roadmap
See [PLAN.md](./PLAN.md) for the execution roadmap.
