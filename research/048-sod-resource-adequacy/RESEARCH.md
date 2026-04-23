---
id: 048
name: sod-resource-adequacy
title: 'Research: Slice-of-Day (SOD) Resource Adequacy Fixes'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: Slice-of-Day (SOD) Resource Adequacy Fixes

**Status**: DRAFT
**Date**: 2026-01-21
**Driven By**: ELNA Vector DB Capacity Failure (042 Benchmark)
**Related**: `042-vector-embedding-strategy`, `047-temporal-architecture`

---

## 1. The Diagnosis
**Why Legacy Models (ELNA) Fail at Scale:**

1.  **Capacity Bottlenecks**:
    *   Legacy systems assume aggregation (monthly/seasonal) or statistical smoothing (ELCC).
    *   SOD requires *hour-by-hour* compliance across the full year.
    *   ELNA's monolithic state updates choke on the combinatorial explosion (10K+ loads × 8760 hours).
    *   *Symptom*: `Instruction Limit Exceeded` at >800 vectors.

2.  **Data Resolution Failure**:
    *   Static/monolithic models recompute from scratch.
    *   Failing to cache partial results leads to inevitable timeout on ICP.

3.  **Sensitivity to Consumption Shapes**:
    *   SOD is shape-driven (peak-coincident), not just energy-driven.
    *   Systems must be natively time-aware to capture these dynamics.

---

## 2. The Fix: Hybrid Time-Slicing Architecture

We must treat **Time as a First-Class Primitive**.

### A. Conceptual Architecture

| Layer | Component | Responsibility |
|:---|:---|:---|
| **Truth Engine** | **ICP / NOSTRA** | Deterministic orchestration, versioning, audit, state authority. |
| **Compute Engine** | **Worker Nodes** | Heavy SOD computation, GPU time-slicing, scenario generation. |
| **Storage** | **Modal-Aware DB** | Time-sliced state, intermediate results (Slice-level caching). |
| **Verification** | **Agents** | Scenario validation, compliance flags. |

### B. Technical Implementation

1.  **Time-Slicing (Native SOD)**:
    *   Move away from aggregation.
    *   Break computation into "Micro-Slices" (e.g., 24-hour chunks or specific load groups).
    *   **Off-Chain Fix**: `VectorService` implements micro-batching (insert 50, yield, insert 50).
    *   **On-Chain Fix**: `TimeSlicedIndex` canister design (sharded by time/slice).

2.  **Resource Virtualization (MicroSlicing)**:
    *   Priority-aware load partitioning.
    *   **Implementation**: Separate canisters or workers for high-risk slices.

3.  **Incremental Recomputation**:
    *   Only recompute affected slices.
    *   **Required**: Slice-level caching of intermediate results.

### C. Missing Pieces to Build (Phase 2)
1.  **Incremental Recompute Logic**: Dependency graph for slices.
2.  **Slice-Level Caching**: Store `compliance_flag` per slice.
3.  **Deterministic Replay**: Version-linked results for audit.

---

## 3. Deployment Strategy (Hybrid)

*   **ICP**: Acts as the Orchestrator and Verifier. It holds the "Gold Copy" of the results.
*   **Rust Workers**: Perform the heavy lifting (embedding generation, large-scale indexing).
*   **GPU**: Off-chain only (via Workers), efficiently utilized via time-slicing.

---

## 4. Next Steps
1.  **Protocol Update**: Define `TimeSlicedIndex` schema in `040`.
2.  **Worker Update**: Implement micro-batching in `VectorService`.
3.  **Architecture**: Formalize the "Truth Engine vs Compute Engine" split in `047`.
