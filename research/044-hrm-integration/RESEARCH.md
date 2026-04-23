---
id: '044'
name: hrm-integration
title: 'Research: HRM (Hierarchical Reasoning Model) Integration'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research: HRM (Hierarchical Reasoning Model) Integration

## 1. Context
The user has identified the [HRM model](https://github.com/sapientinc/HRM) (Hierarchical Reasoning Model) as a potential candidate for enhancing Nostra / Cortex. HRM is a novel recurrent architecture inspired by the human brain's hierarchical processing (slow abstract planning vs. fast detailed computation). A local installation exists at `/Users/xaoj/ICP/HRM`.

## 2. Goals
-   Evaluate HRM's capabilities and suitability for Nostra / Cortex.
-   Determine if HRM can be used for "mundane AI tasks" (e.g., file management, scheduling, simple logic).
-   Identify gaps, challenges, and potential cost savings (e.g., running small specialized models locally instead of large cloud LLMs).
-   Resolve overlaps with existing research (e.g., Workflow Engine, Auto Claude).

## 3. Analysis of HRM
Based on the repository README (v. 2506.21734v3):
-   **Architecture**: Two-module recurrent system (High-level Plan vs. Low-level Compute).
-   **Size**: Extremely efficient (~27M parameters).
-   **Training Efficient**: Achieves high performance with ~1000 samples.
-   **Capabilities**:
    -   **Strengths**: Abstract reasoning (ARC), complex logic puzzles (Sudoku), pathfinding (Mazes).
    -   **Weaknesses (inferred)**: The README focuses on "puzzles" and "reasoning". It does **not** explicitly mention Natural Language Processing (NLP), instruction following for text tasks, or external tool usage. It seems to be a specialized reasoning engine rather than a general-purpose LLM like GPT-4 or Claude 3.5.
    -   **Input/Output**: Likely works on grid-based or tokenized abstractions rather than raw natural language conversation, though this needs verification of the `pretrain.py` and dataset structures.

## 4. Fit for "Mundane AI Tasks"
"Mundane tasks" in the context of Nostra/Cortex usually imply:
-   Parsing user intent from natural language.
-   Executing specific workflows (file operations, API calls).
-   Data transformation.

**Hypothesis**: HRM might not be a drop-in replacement for the "Conversational" or "Instruction Following" layer, but could be a powerful "Reasoning Core" for specific sub-problems, such as:
-   **Workflow Optimization**: Finding the optimal path for a complex workflow (similar to Maze solving).
-   **Scheduling/Logic Constraints**: Solving constraint satisfaction problems (similar to Sudoku).
-   **Pattern Matching**: Identifying abstract patterns in data (ARC).

If we can map "mundane" tasks to these abstract problem spaces, HRM could run locally and cheaply, saving cycles on large LLMs.

## 5. Overlaps & Integration
-   **013-nostra-workflow-engine**: HRM is best positioned as a specialized **Async Worker** (or "Solver Agent"). The Workflow Engine can delegate abstract logic tasks (e.g., "Schedule Optimization") to an HRM worker, which returns the result to be rendered by A2UI. It should **not** replace the main Orchestrator (which requires NLP).
-   **035-auto-claude**: Auto Claude focuses on generalist coding agents. HRM is likely too specialized to participate in the general "Dev Loop" (writing code), but could be used as a sub-routine for specific algorithmic problems if the coding task involves them (e.g., "Optimize this pathfinding algorithm").
-   **043-icp-compute-platform**: HRM is small enough to potentially run on-chain or on edge devices (Nostra Client), aligning with decentralized compute goals.

## 6. Implementation Strategy (Draft)
1.  **Verify Input/Output**: Run `puzzle_visualizer.html` or inspect `dataset/` to see how data is formatted.
2.  **Test "Text" Capability**: Does it handle vocabulary? Or just symbols?
3.  **Prototype**: Create a "Task Scheduler" problem formulated as a Sudoku/Constraint problem and see if HRM solves it.
4.  **Hardware Check**: The codebase currently relies on CUDA/FlashAttention. Running on Mac (MPS) requires porting or finding a CPU-only fallback (which might be slow).


## 6. Feasibility Analysis (2026-01-20)
**Verdict: Feasible for functional use via Sovereign Worker / Hybrid Service.**

### 6.1. Hardware & Environment
-   **Mac Compatibility**: **VERIFIED.**
    -   Successfully patched `models/layers.py` to use `F.scaled_dot_product_attention` (MPS-native) instead of `flash_attn`.
    -   Successfully patched `pretrain.py` and `evaluate.py` to detect `MPS` device dynamically.
    -   Successfully replaced `AdamATan2` with `torch.optim.AdamW` (standard PyTorch).
    -   **Result**: Model initializes and trains on Apple M-series hardware without errors.

### 6.2. Architectural Options Analysis
Based on deep analysis, the "Pure WASM" path is deemed **NON-VIABLE** for the near term.

| Option | Architecture | Status | Pros | Cons |
| :--- | :--- | :--- | :--- | :--- |
| **A** | **Sovereign Worker** (Current) | ⭐⭐⭐⭐⭐ (Best) | Full power, fast iteration, easy debugging. | Trust boundary (off-chain). |
| **B** | **Pure WASM** (ONNX -> wasi-nn) | ⭐⭐ (Avoid) | Ideologically pure. | Fragile, limited ops, no dynamic memory, nightmare to debug. |
| **C** | **External Service** (HTTP Outcalls) | ⭐⭐⭐⭐ (Production) | Scalable, standard. | Latency, cost, trust assumptions (same as worker). |

**Conclusion**: We will proceed with **Option A** (Sovereign Worker) for development and initial deployment.

## 7. Strategic Recommendation
**Principle: Reasoning Off-Chain, Verification On-Chain.**

We should **not** force the HRM model itself onto the ICP chain (WASM). Instead, we should:
1.  **Run HRM Off-Chain**: Use the Sovereign Worker (or later, a cloud service) to perform the heavy "Reasoning" (solving the Sudoku grid).
2.  **Verify On-Chain**: Build a lightweight canister logic that verifies the *solution* (e.g. checks that the Sudoku grid constraints are met).

**Next Step**:
1.  **Trace Reasoning**: Instrument the current Python adapter to capture intermediate steps.
2.  **Verify Determinism**: Ensure the adapter output is deterministic for a given input.
3.  **Build Verifier**: Plan a future Rust/Motoko module that can just *check* the result.

## 8. Agent UI Protocol (2026-01-21)
**Standard**: All Agent-to-User interactions must comply with the **AG-UI Protocol**.
-   **Output**: Agents must NOT render HTML/CSS. They must output AG-UI structured JSON (e.g., `render.form`, `request.confirmation`).
-   **Rendering**: The Nostra Client handles the visual translation using **Shoelace** tokens.
