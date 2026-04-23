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

---

## DEC-002: Initiative 126 extends (does not replace) Initiative 122
**Date**: 2026-02-24
**Status**: Proposed

**Options Considered**:
1. Parallel architecture independent from 122
2. Extension model that builds governance/replay contracts on top of 122 MVK

**Decision**: Option 2

**Rationale**: Avoids competing agent-runtime architectures and preserves continuity with existing MVK assumptions.

**Implications**: 126 work must explicitly reference 122 boundaries and avoid adding planner/evaluator framework bloat.

---

## DEC-003: V1 cutline is L0-L2 + core payload only
**Date**: 2026-02-24
**Status**: Proposed

**Options Considered**:
1. Implement full L0-L4 and full payload in v1
2. Ship lean v1 (`L0-L2`, core required keys) and defer advanced levels/fields

**Decision**: Option 2

**Rationale**: Reduces delivery risk and prevents overbuilding before governance/operations evidence is established.

**Implications**: `L3-L4` remain contract-defined but blocked/deferred in this initiative scope; extension payload fields are optional.
