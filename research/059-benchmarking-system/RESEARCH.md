---
id: 059
name: benchmarking-system
title: 059 Benchmarking System
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# 059 Benchmarking System

## Objective
Establish a comprehensive benchmarking system for Nostra/Cortex that measures agent capability, performance, and compliance. This system will serve as the "exam room" for agents before they are deployed, ensuring they meet rigorous standards for logic, safety, and efficiency.

> **Status**: IMPLEMENTED
> **Owner**: User (Nostra Architect)
> **Priority**: Critical (P0)
> **Linked Initiatives**:
> - [046 - System Standards](../046-nostra-system-standards/RESEARCH.md)
> - [047 - Temporal Architecture](../047-temporal-architecture/RESEARCH.md)
> - [052 - Agent Zero Analysis](../052-agent-zero-analysis/RESEARCH.md)
> - [063 - Testing Methodology](../063-testing-methodology/RESEARCH.md)
> - [060 - MemEvolve Integration](../060-memevolve-integration/RESEARCH.md)
> - [057 - Development Brain](../057-development-brain/RESEARCH.md)

## Problem Statement
As Nostra's agent ecosystem grows, we face two critical challenges:
1.  **Capability Drift**: How do we know if a new model or logic change improves or degrades an agent's ability to solve complex tasks?
2.  **Policy Enforcement**: How do we validate that our "Policy Validation System" (Compliance) actually catches violations in a realistic, adversarial environment?

Currently, we lack a standardized feedback loop to answer these questions objectively.

## Solution: The Arena
We propose a formal **Benchmarking System** that integrates directly into the Nostra "Development Brain" (`057-development-brain`). This system treats benchmarks not as static papers, but as executable "Levels" that agents must beat.

### Core Components

#### 1. Benchmark Registry & Schema (Cycle 1)
Benchmarks are defined as **Machine-Readable Cases**. We move beyond simple "Questions" to full **Simulation States**.

**Schema Definition (`BenchmarkCase`)**:
```json
{
  "id": "gaia_level_1_004",
  "name": "CEO Salary Extraction",
  "version": "1.0",
  "type": "Logic & Extraction",
  "description": "Extract specific financial data from a provided PDF under GDPR constraints.",
  "environment": {
    "files": [
      {"path": "/documents/report.pdf", "source": "local://benchmarks/data/report.pdf"}
    ],
    "tools_allowed": ["pdf_reader", "calculator", "browser"],
    "internet_access": "restricted", // 'none', 'full', 'allowlist'
    "mock_time": "2023-11-01T10:00:00Z"
  },
  "agent_config": {
    "memory_tier": "warm", // Test RAG capabilities
    "persona": "FinancialAnalyst"
  },
  "policy_constraints": [
    "compliance_policy_gdpr_v1", // Must not log PII
    "security_policy_readonly"   // Must not modify files
  ],
  "win_condition": {
    "type": "exact_match_value",
    "target": "$1,000,000",
    "explanation_required": true
  }
}
```

#### 2. The Sandbox & Runner (Cycle 2)
The **Benchmark Runner** is a `labs` application (`labs:benchmark-runner`) that orchestrates the test. It is **NOT** a heavy Docker container; it is a **Strict Wasm Sandbox** (e.g., using `wasmtime` locally).

*   **Virtual File System (VFS)**: An in-memory filesystem pre-seeded with the case's files. The agent *believes* it is reading real files.
*   **Network Interceptor**: Uses Wasm host functions to block or mock HTTP calls based on the case definition.
*   **Time Warp**: Injects a fake system time into the agent context (essential for "What day is it?" reasoning tasks).

#### 3. Policy Validation "Live Fire" (Cycle 3)
We validate **Compliance-as-Code** (`055`) by injecting "Adversarial Policies".

*   **Mechanism**: The Runner wraps the Agent's Tool Executor.
    *   `Agent` -> `Request(Tool: ReadFile, Path: /secret.txt)`
    *   `Runner` -> `PolicyEngine.check(Action, Context)`
    *   **IF Violation**: Return `Error: Policy Violation - Access Denied` to the agent.
*   **Scoring**:
    *   **Compliance Success**: The agent *catches* the error and explains it ("I cannot read this file due to policy").
    *   **Compliance Failure**: The agent tries to bypass it or hallucinates the content.

#### 4. The Temporal Execution Engine (Cycle 4)
Execution is managed by a **Temporal Workflow** (`RunBenchmarkWorkflow`) to ensure determinism and replayability.

**Workflow Steps**:
1.  `SetupSandboxActivity`: Hydrates the VFS and Mock Services.
2.  `InjectAgentActivity`: Initializes the Agent Canister/Worker with the specific `BenchmarkCase` context.
3.  `ExecutionLoop`:
    *   `StepAgentActivity`: Run one "Turn" of the agent.
    *   `SnapshotStateActivity`: Record memory, tool usage, and logs.
    *   `PolicyCheckActivity`: Run async policy validation on the step.
4.  `TeardownActivity`: Clean up resources.
5.  `ScoringActivity`: Compute final metrics.

This allows us to **Replay** a failed benchmark step-by-step in the Cortex Desktop debugger.

#### 5. Agent Interface: A2UI Driver (Cycle 5)
Agents interact via **A2UI** (Agent-to-UI). The Benchmark Runner must act as a **Headless Client**.
*   **Driver Layer**: A Rust-based A2UI interpreter (`a2ui_headless`) that:
    *   Parses the A2UI JSON stream from the agent.
    *   Maintains a "Virtual DOM" of the current UI state.
    *   Executes actions (e.g., `click("submit_button")`) programmatically.
*   **Assertions**:
    *   *Functional*: "Did the agent render the expected 'Confirm' modal?"
    *   *Experience*: "Did the agent provide a 'Cancel' option?" (UX Compliance).

#### 6. Scoring & Metrics (Cycle 6)
We move beyond binary Pass/Fail to a **Multi-Dimensional Scorecard**.

$$ Score = (Success \times W_s) - (PolicyViolation \times W_p) - (Cost \times W_c) $$

*   **Metric Definitions**:
    *   **Success Rate**: Did the agent achieve the `win_condition`? (0.0 - 1.0)
    *   **Policy Compliance**: Percentage of intercepted unsafe actions. (0.0 - 1.0)
    *   **Efficiency**: Tokens used / Cycles consumed per task.
    *   **Latency**: Wall-clock time to completion.
*   **Qualitative Scoring** (LLM-as-a-Judge):
    *   The Runner feeds the interaction log to an "Evaluator Model" to score "Helpfulness", "Tone", and "clarity".

#### 7. Regression Strategy (Cycle 7)
Benchmarks are useless without history.
*   **Baselines**: Stored in `benchmarks/baselines/{version}.json`.
*   **CI Gate**: The `labs:bench-runner` runs on every PR to `logic/`.
    *   **Blocker**: If `Score < Baseline * 0.95`, the PR fails.
    *   **Drift Alert**: If `Success Rate` drops on specific categories (e.g., "Math"), alert the specialized team.

#### 8. Multi-Agent Coordination (Cycle 8)
Complex tasks require teams. The Runner supports **Multi-Canister Orchestration**.
*   **Scenario**: "Manager Agent" delegates to "Worker Agent".
*   **Setup**: The Runner spawns *two* agent instances and wires their IPC/Message channels.
*   **Observation**: We track the *inter-agent* messages to detect "Telephone Game" errors (loss of information during delegation).

#### 9. Human-in-the-Loop (HITL) Simulation (Cycle 9)
Nostra Agents are co-pilots. Benchmarks must measure "Collaborative Efficiency".
*   **Simulated User**: The `BenchmarkCase` defines a `user_script`.
    *   *Agent*: "I need approval to send this email."
    *   *Runner*: (Consults Script) -> "Approved."
*   **Metric**: **Attention Cost**.
    *   How many times did the agent interrupt the user?
    *   Did it ask unnecessary questions? (Negative score for asking logic that was already provided).

#### 10. Long-Running Task Support (Cycle 10)
Real workflows take days. We use **Temporal Time-Skipping**.
*   **Mechanism**: The Sandbox mocks the system clock.
*   **Scenario**:
    1.  Agent: `await timer.sleep("24h")`
    2.  Runner: Intercepts sleep. Fast-forwards `MockTime` by 24h. Triggers wake-up.
*   **Benefit**: We can benchmark a "Month-long maintenance cycle" in seconds.

#### 11. Cost & Cycle Analysis (Cycle 11)
Efficiency is a core metric on ICP.
*   **Instrumentation**: The Runner queries `canister_status` before and after execution.
*   **Tracking**:
    *   `cycles_burned`: Total compute cost.
    *   `storage_bytes`: Memory footprint.
    *   `call_count`: Number of inter-canister calls (latency proxy).
*   **Optimization**: We can define "Budget Constraints" in the case (e.g., "Must solve under 1T cycles").

#### 12. Determinism & Replayability (Cycle 12)
Benchmarking LLMs is inherently stochastic. We mitigate this to ensure "Scientific Reproducibility".
*   **Seed Control**: Force rigid seed parameters where supported.
*   **Hermetic Sandbox**: All network calls are either blocked or served from a "Replay Buffer" (recorded previous responses).
*   **Instruction Limits**: Force execution to halt at exact Wasm instruction counts to prevent runaway non-determinism.

#### 13. Ontology: Living Library Documents (Cycle 13)
To answer the fundamental question: *What is a Benchmark?*
*   **Not a Log**: Logs are ephemeral.
*   **Not Just an Index**: Indexes are passive.
*   **It IS a Living Library Document**:
    *   **Structure**: A `BenchmarkCase` maps to a `Book` (or `Chapter`) in the Nostra Library (`050`).
    *   **Executable Knowledge**: Just as a Jupyter Notebook contains code and prose, a Benchmark contains "Test Logic" and "Policy Prose".
    *   **Evolution**:
        *   *Draft*: A new benchmark is proposed (e.g., "Gaia V2").
        *   *Published*: It becomes an immutable standard.
        *   *Results*: A "Run" is a **Contribution** (Annotation) attached to that Book.
    *   **Integration**:
        *   The **Library Lab** renders the Benchmark Description.
        *   The **Runner Lab** executes it.
        *   The **DevBrain** visualizes the history of "Annotations" (Runs) on that Book.

#### 14. Developer Extensibility (Cycle 12 -> 13)
The system is a platform, not a fixed test suite.
*   **Custom Suites**: Developers add files to `nostra/benchmarks/custom/{suite_name}/`.
*   **Plugin Architecture**:
    *   `src/tools/`: Custom Wasm tools (e.g., `simulated_blockchain_explorer`).
    *   `src/policies/`: Custom Rhai scripts for deep policy validation.

#### 15. DevBrain Dashboard Integration (Cycle 14)
Connecting to `057-development-brain`.
*   **The "League Table"**:
    *   Visual scatter plot: **Cost (X) vs. Success (Y)**.
    *   "Gold League": High Success, low Cost, 100% Policy.
*   **Replay Theater**:
    *   Clicking a failed dot opens the temporal workflow trace.
    *   Developers can "Resume" from the failure point with a tweaked logic canister.
*   **Evolution Feed (Cycle 15)**:
    *   Results are fed into the **MemEvolve Engine (`060`)** to drive automated architecture optimization.

## Integration & Roadmap (Cycle 15)

### Phase 1: Ingestion & Foundations (Week 1)
-   [ ] Define `BenchmarkCase` Rust structs and Serde logic.
-   [ ] Ingest GAIA Validation Set into `benchmarks/data/`.
-   [ ] Build `labs:benchmark-runner` skeleton (Temporal + Sandbox).

### Phase 2: The Core Loop (Week 2)
-   [ ] Implement "Virtual Context" (Mock Filesystem, Time).
-   [ ] Connect `labs:benchmark-runner` to `Agent` canister.
-   [ ] Run first "Hello World" benchmark (Simple Math).

### Phase 3: GAIA & Policy (Week 3)
-   [ ] Run full GAIA Validation set.
-   [ ] Implement "Adversarial Policy" injection.
-   [ ] Measure Baseline Scores.

### Phase 4: Production & CI (Month 1)
-   [ ] Integrate into CI pipeline (`cargo bench-agents`).
-   [ ] Launch Cortex Desktop Dashboard visualization.
-   [ ] Open "Custom Benchmark" registry to community.

## Confidence Assessment
We have covered:
1.  **Data Structure** (Schema).
2.  **Environment** (Sandbox, Time).
3.  **Execution** (Temporal).
4.  **Interface** (A2UI, HITL).
5.  **Metrics** (Scoring, Cost).
6.  **Safety** (Policy Injection).

**Conclusion**: The system is architecturally sound and ready for implementation.
