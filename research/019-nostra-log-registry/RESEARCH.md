---
id: 019
name: nostra-log-registry
title: 'Research: Nostra System Log Registry'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-30'
---

# Research: Nostra System Log Registry

## Goal
Design a centralized "Log Registry" or "Error Index" for the Nostra ecosystem that is **Standardized**, **User Friendly**, and **AI Friendly**.
This system will allow Agents (Worker), Backend (Canister), and Frontend (Client) to submit logs/errors to a unified immutable record.

## Layered Architecture Role

> [!IMPORTANT]
> **Consolidation Decision**: This initiative owns the **Data Model Layer** of Nostra's unified Observability stack.

| Layer | Owner | Responsibility |
|-------|-------|----------------|
| **Data Model** | **019-Log-Registry** (this) | `LogEntry`, `Span`, `Metric` schemas |
| **Pipeline** | 054-OpenTelemetry | Receiver→Processor→Exporter traits |
| **Consumption** | 033-Cortex-Monitor | Dashboard, Triage, Alerts |

The consolidation rationale came from related observability overlap analysis captured during initiative planning.

---

## Implementation Status

**Status**: ✅ **Implemented** (2026-01-30)

**Location**: [`/nostra/log-registry/`](file:///Users/xaoj/ICP/nostra/log-registry/)

**Key Files**:
- [`types.rs`](file:///Users/xaoj/ICP/nostra/log-registry/src/types.rs) - `LogEvent`, `Span`, `SeverityNumber` with `Storable` trait
- [`pipeline.rs`](file:///Users/xaoj/ICP/nostra/log-registry/src/pipeline.rs) - `Receiver`, `Processor`, `Exporter` traits
- [`exporters/stable_memory.rs`](file:///Users/xaoj/ICP/nostra/log-registry/src/exporters/stable_memory.rs) - Stable storage implementation

**Verification**: `cd nostra && cargo check -p nostra_log_registry` ✅

---

## "Possible and Needed?"
**Yes.**
1.  **AI Analysis**: Agents like the Librarian need access to error history to diagnose issues autonomously ("Self-Healing").
2.  **Decentralized Debugging**: In a decentralized squad (multiple workers), local file logs are insufficient. A shared on-chain or off-chain index is required.
3.  **Audit Trail**: Critical for governance and contribution attribution.

## Key Enhancements (Expanded Scope)

### 1. AI Friendliness (Context is King)
Logs must not just be strings. They must carry structured context.
- **Workflow Tracing**: Every log from a generic agent must carry the `workflow_instance_id` so we can reconstruct the execution path.
- **Entity Correlation**: Errors related to a specific `Entity` must link to its UUID.

### 2. User Friendliness (Flight Recorder)
The UI should feel like a "Black Box" recorder for the ecosystem.
- **Visual Timeline**: See errors spike on a graph.
- **Click-to-Debug**: Clicking a log entry should take you to the related Entity or Workflow.

### 3. Standardization (ICP Alignment)
- Adopt keys compatible with OpenTelemetry where possible (`trace_id`, `span_id`).
- Ensure immutable/tamper-evident logging for Governance actions (as per `032` alignment).

## Proposed Architecture
- **Log Index Module**: A module in the main canister (currently) storing `LogEntry` records.
- **LogEntry Schema**:
  ```motoko
  type LogEntry = {
      id: Text;
      timestamp: Int;
      source: { #agent(Text); #backend; #frontend };
      level: { #info; #warn; #error; #critical };
      message: Text;
      context: ?[(Text, Text)]; // Keys: workflow_id, entity_id, trace_id
  };
  ```

## Integration with Research Initiatives
- **002 (V2 Arch)**: The Log Registry serves as the "Debug Layer" for the modular architecture.
- **013 (Workflow)**: The Log Registry maps 1:1 with Workflow Steps for tracing.
- **017 (Agents)**: Agents consume logs to perform "Self-Repair" tasks.

## Known Issues & Log
This section tracks system-wide issues identified during research/development, acting as a manual appendix to the future digital registry.

### Active Issues
- **[ISSUE-001] Entity Count Fluctuation**
    - **Status**: Investigating
    - **Symptoms**: Entities flip between 0 and N (e.g., 88) on refresh/polling. "Core" libraries appear to toggle indiscriminately.
    - **Suspected Cause**: Frontend Identity persistence failing mid-poll, causing backend `get_enabled_library_ids` to revert to default (empty or core-only) for an anonymous user.
    - **Logged**: 2026-01-18
    - **Action**: Debug logging added to `main.rs`. Need to verify `enabled_ids` stability in console.
