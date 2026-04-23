---
id: '013'
name: nostra-workflow-engine
title: Feedback & Open Questions
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Feedback & Open Questions

## Open Questions

### Q1: Workflow Upgradability
- **Question**: What happens to active instances when a `WorkflowDefinition` is updated?
- **Hypothesis**: Active instances continue on the *old* definition (Versioned WFDs).
- **Status**: Needs Decision.

### Q2: Agent Identity
- **Question**: How do we prevent "rogue agents" from spamming the task queue?
- **Proposal**: Agents must stake tokens or have a Reputation Badge to claim tasks.

### Q3: Visualization Tech
- **Question**: D3.js vs ReactFlow vs Sigma.js for the Canvas?
- **Context**: Need "Force Directed" (D3/Sigma) for flexible graphs, but "Orthogonal Routing" (ReactFlow) for neat diagrams.
- **Status**: Evaluating.
