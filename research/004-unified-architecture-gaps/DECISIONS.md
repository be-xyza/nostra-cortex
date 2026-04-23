---
id: '004'
name: unified-architecture-gaps
title: 'Decisions: Unified Architecture'
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions: Unified Architecture

## 1. Shared User Index in `kg-registry`
**Context**: We need a single source of truth for user identity across Nostra and KG.
**Decision**: We will expand the scope of the existing `kg-registry` canister in Motoko Maps KG to act as the central User Index.
**Consequences**:
- Nostra v2 must depend on `kg-registry`.
- `kg-registry` becomes a critical shared infrastructure piece.

## 2. Monolith First for Nostra v2
**Context**: Nostra v2 architecture needs to balance modularity with development speed.
**Decision**: Start with a Monolithic architecture for Nostra v2 canisters, BUT implement the `Event Emitter` interface immediately.
**Rationale**: This allows us to move fast now but future-proofs the system for the Global Discovery Index and Governance layers later.

## 3. Generic Governance Host
**Context**: Both projects need governance, but logic shouldn't be duplicated.
**Decision**: Create a generic `governance-host` canister that manages proposals and voting strategies, capable of targeting other canisters.
**Strategy**: Start with "Owner Dictator" strategy (simple passthrough) and upgrade to "Community Vote" later.

## 4. Unified MCP Router
**Context**: AI Agents need to interact with both systems.
**Decision**: Use the `ic-rmcp` based `mcp-server` (from KG) as the single entry point for agents, routing calls to specific project tools.

## 5. Workflow Engine Delegation (2026-01-24)
**Context**: 004 originally designed a basic workflow schema, but requirements grew significantly.
**Decision**: Delegate full Workflow Engine implementation to [013-nostra-workflow-engine](../013-nostra-workflow-engine/PLAN.md).
**Status**: 013 is now ✅ COMPLETED with:
- Serverless Workflow DSL (JSON/YAML)
- Actor-Model execution (State Machine per Instance)
- A2UI integration for human tasks
- Saga compensation pattern
- WorkflowLab in Nostra frontend
**Consequences**:
- 004's `WORKFLOW_DESIGN.md` is deprecated (historical reference only).
- All workflow-related development follows 013's architecture.
