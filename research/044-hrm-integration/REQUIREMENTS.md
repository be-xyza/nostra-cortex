---
id: '044'
name: hrm-integration
title: 'Requirements: HRM Integration'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: HRM Integration

## Functional Requirements
- **Local Execution**: Must run on the user's hardware (Mac M-series) via Sovereign Worker integration.
- **Task Mapping**: Must be able to represent "mundane tasks" (scheduling) as 9x9 Sudoku-like grids.
- **Verification**: Solutions typically produced off-chain must be verifiable on-chain by a deterministic Canister function.
- **Cost Efficiency**: Inference runs off-chain (zero gas), verification runs on-chain (minimal gas).

## Technical Requirements
- **Dependencies**: Python 3.10+ (PyTorch MPS), Rust (Worker), Motoko (Backend Verifier).
- **Interface**:
    - **Worker**: `HrmScheduler` skill (Rust) calling `adapter.py` (Python).
    - **Frontend**: `SchedulerLab.rs` (Dioxus) for visualization.
    - **Backend**: `Verifier.mo` for trustless validation.
