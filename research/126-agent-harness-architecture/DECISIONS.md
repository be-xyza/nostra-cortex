---
id: "126"
name: "agent-harness-architecture"
title: "Decision Log: Agent Harness Architecture"
type: "decision"
project: "nostra"
status: complete
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
**Status**: Accepted

**Options Considered**:
1. Custom per-agent event schema outside `GlobalEvent`
2. Canonical `GlobalEvent` envelope + typed payload contract

**Decision**: Option 2

**Rationale**: Preserves replayability, portfolio consistency, and cross-initiative event tooling.

**Implications**: `AgentExecutionRecord` payload schema must version independently while retaining envelope compatibility.

---

## DEC-002: Initiative 126 extends (does not replace) Initiative 122
**Date**: 2026-02-24
**Status**: Accepted

**Options Considered**:
1. Parallel architecture independent from 122
2. Extension model that builds governance/replay contracts on top of 122 MVK

**Decision**: Option 2

**Rationale**: Avoids competing agent-runtime architectures and preserves continuity with existing MVK assumptions.

**Implications**: 126 work must explicitly reference 122 boundaries and avoid adding planner/evaluator framework bloat.

---

## DEC-003: V1 cutline is L0-L2 + core payload only
**Date**: 2026-02-24
**Status**: Accepted

**Options Considered**:
1. Implement full L0-L4 and full payload in v1
2. Ship lean v1 (`L0-L2`, core required keys) and defer advanced levels/fields

**Decision**: Option 2

**Rationale**: Reduces delivery risk and prevents overbuilding before governance/operations evidence is established.

**Implications**: `L3-L4` remain contract-defined but blocked/deferred in this initiative scope; extension payload fields are optional.

---

## DEC-004: L1 materializes local proposal artifacts in v1
**Date**: 2026-02-24
**Status**: Accepted

**Options Considered**:
1. Defer L1 until explicit governance submit-proposal canister method exists
2. Materialize proposal bridge artifacts locally with governance-visible lineage

**Decision**: Option 2

**Rationale**: Current governance candid does not expose a generic proposal submit endpoint and v1 still needs enforceable L1 behavior.

**Implications**: Proposal bridge artifacts are the canonical v1 L1 output and remain migration candidates when native canister proposal endpoints are introduced.

---

## DEC-005: Event sink is local JSONL first, remote optional
**Date**: 2026-02-24
**Status**: Accepted

**Options Considered**:
1. Decision-surface JSON only
2. CloudEvent-compatible lifecycle emission with local durability and optional remote sink

**Decision**: Option 2

**Rationale**: Preserves deterministic local evidence and enables progressive rollout of remote event transport without blocking execution path.

**Implications**: Lifecycle event emission can run best-effort by default and fail-closed only when explicitly configured.

---

## DEC-006: Runtime `agent_id` is resolved from request/header with env fallback
**Date**: 2026-02-24
**Status**: Accepted

**Options Considered**:
1. Fixed runtime identity constant for all lifecycle records
2. Resolve identity from runtime context with safe fallback

**Decision**: Option 2

**Rationale**: Preserves lineage clarity per run and reduces ambiguity in lifecycle records without breaking existing clients.

**Implications**: Resolution order is `x-cortex-agent-id` header -> request payload `agentId` -> `NOSTRA_AGENT_ID`/`NOSTRA_DEFAULT_AGENT_ID` -> default constant.

---

## DEC-007: Fail-closed sink mode requires explicit non-empty sink URL
**Date**: 2026-02-24
**Status**: Accepted

**Options Considered**:
1. Allow fail-closed mode without validating sink configuration
2. Reject execution lifecycle emission when fail-closed is enabled and sink URL is missing/blank

**Decision**: Option 2

**Rationale**: Prevents silent misconfiguration where operators think strict remote durability is active but no sink is configured.

**Implications**: `NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED=true` now enforces presence of `NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL`.
