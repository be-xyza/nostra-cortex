---
id: "126"
name: "agent-harness-architecture"
title: "Decision Log: Agent Harness Architecture"
type: "decision"
project: "nostra"
status: draft
authors:
  - "User"
tags: ["agent-harness", "governance", "temporal"]
created: "2026-02-24"
updated: "2026-02-24"
---

# Decision Log: Agent Harness Architecture

Track architectural decisions with rationale for future reference.

---

## DEC-001: AgentExecutionRecord uses canonical GlobalEvent envelope
**Date**: 2026-02-24
**Status**: Proposed

**Options Considered**:
1. Custom per-agent event schema outside `GlobalEvent`
2. Canonical `GlobalEvent` envelope + typed payload contract

**Decision**: Option 2

**Rationale**: Preserves replayability, portfolio consistency, and cross-initiative event tooling.

**Implications**: `AgentExecutionRecord` payload schema must version independently while retaining envelope compatibility.
