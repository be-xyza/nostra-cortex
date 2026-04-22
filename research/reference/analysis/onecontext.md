---
id: onecontext
name: onecontext
title: OneContext
type: reference_analysis
project: nostra
status: draft
portfolio_role: satellite
execution_plane: nostra
authority_mode: recommendation_only
reference_topics: [agent-systems, memory, context]
reference_assets:
  - "research/reference/topics/agent-systems/OneContext"
evidence_strength: strong
handoff_target:
  - "Systems Steward"
authors:
  - "User"
  - "Codex"
tags: [agents, memory, portability]
stewardship:
  layer: "Architectural"
  primary_steward: "Systems Steward"
  domain: "Agents & Execution"
created: "2026-02-20"
updated: "2026-02-20"
---

# OneContext Reference Analysis

## Overview
OneContext is a repository implementing an Agent Self-Managed Context layer, acting primarily as a CLI wrapper for creating and sharing robust context "sessions" between agents.

## Why Intake?
To formalize the initial review of agent memory topologies, specifically to compare the session-based wrapper approach with the Git-centric active reasoning paths outlined in Initiative `121-cortex-memory-fs`.

## Placement
`research/reference/topics/agent-systems/OneContext`

## Intent
Formally examine robust context sessions that flow between systems.

## Initiative Links
- `103-agent-client-protocol-alignment`

## Pattern Extraction
- **Session Portability:** The core concept of OneContext is packaging an interaction session into an exportable chunk that can be rehydrated by another agent or shared over chat interfaces like Slack.
- **Agent-to-Human Handoff:** It structurally focuses on boundaries where a session leaves one actor and enters another.

## Possible Links To Nostra Platform and Cortex Runtime
- Nostra Agent Client Protocol (ACP) handoffs.

## Adoption Decision
**Recommendation:** Reference-only.
- Reject adopting the repository as a dependency.
- Keep the session export pattern in mind for future ACP boundaries, but proceed with the Git-backed Context Memory FS (`121`) as the primary cognitive substrate.

## Known Risks
Overlapping concepts with existing memory layers could cause abstraction conflicts.

## Suggested Next Experiments
None currently.
