---
id: '044'
name: hrm-integration
title: 'Decisions: HRM Integration'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decisions: HRM Integration

## 001: Initialize Research Initiative 044
- **Date**: 2026-01-20
- **Context**: User requested research on HRM (Hierarchical Reasoning Model).
- **Decision**: Created `044-hrm-integration` to track this work.
- **Status**: Active.

## 002: Verified Mac Compatibility
-   **Date**: 2026-01-20
-   **Context**: Patched code to use MPS and native PyTorch attention.
-   **Decision**: Proceed with HRM as a "Logic Kernel" (Phase 3).
-   **Reasoning**: Execution verified on local hardware.
## 003: Reject Pure WASM (ONNX/wasi-nn)
-   **Date**: 2026-01-20
-   **Context**: Evaluated feasibility of compiling dynamic HRM model to ONNX for canister execution.
-   **Decision**: **REJECT**.
-   **Reasoning**: Technical fragility, lack of support for dynamic control flow, poor debugging experience, and severe performance penalties make this path non-viable.

## 004: Architecture Strategy - Verify On-Chain
-   **Date**: 2026-01-20
-   **Context**: Determining long-term decentralization strategy.
-   **Decision**: Adopt "Reasoning Off-Chain, Verification On-Chain" model.
-   **Reasoning**: It is computationally cheaper and safer to verify a solution on-chain (e.g. check Sudoku constraints) than to generate it. This allows the heavy reasoning engine to remain off-chain (Sovereign Worker) while maintaining trustless verification.
