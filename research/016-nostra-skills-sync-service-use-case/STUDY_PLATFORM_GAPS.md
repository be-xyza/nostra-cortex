---
id: "016-study-platform-gaps"
name: "study-platform-gaps"
title: "Study 16.1: Closing Platform Gaps"
type: "study"
project: "nostra"
status: draft
authors:
  - "Antigravity"
tags: ["workflow", "agents", "standards", "architecture"]
created: "2026-01-16"
updated: "2026-01-16"
---

# Study 16.1: Closing Platform Gaps

**Context**: The "Skills Sync Service" (016) has stress-tested the Nostra architecture and revealed 4 critical gaps in the current research for Workflows (013), Contributions (008), and Agents (014). This study analyzes these gaps and proposes optimal paths forward.

## 1. Gap Analysis: Synchronous Workflows (013)

### The Problem
The current Workflow Engine (013) is designed around an **Event-Driven State Machine** (Async).
*   **Scenario**: User clicks "Sync" -> Step 1 (Trigger) -> Step 2 (Merge) -> Step 3 (Response).
*   **Current Reality**: Canisters process asynchronously. A "Sync" request might queue a task, requiring the client to poll for the result.
*   **Friction**: For an API-like service (Latency < 2s), this "Polling" architecture adds unacceptable overhead and complexity for simple CLI tools.

### Optimal Path Forward: "Fast-Path" Execution
We recommend introducing a **Synchronous Execution Mode** for specific workflow types.

*   **Definition**: If a Workflow consists *only* of `SystemOp` and `AgentTask` steps (no `UserTask` or `LongWait`), it can be marked as `mode: sync`.
*   **Implication**: The engine executes all steps in a single Canister Message (or atomic set) and returns the loop output immediately.
*   **Constraint**: Total execution cycles must stay within the single-message limit.

**Action**: Update `013/RESEARCH.md` to define `ExecutionMode: Async | Sync` property.

## 2. Gap Analysis: Agent Verification and Semantic Merging

### The Problem
The "Semantic Merge" relies on an AI Agent. AI is non-deterministic.
*   **Scenario**: Agent is asked to "Merge A and B" via a Nostra Workflow step.
*   **Current Reality**: Forcing Nostra workflows (`013`) to handle non-deterministic LLM execution bloats the platform definition and creates extreme Sybil vulnerabilities.
*   **Risk**: Corrupted skills are merged, or agents game the bounty system.

### Optimal Path Forward: Cortex Agent Runtime (122) & Agent Harness (126)
We recommend explicitly decoupling the execution of AI tasks from Nostra's state machine.

*   **Mechanism**:
    *   **Nostra (Authority)**: Defines the workflow and the final Authority Guard proposal requirements.
    *   **Cortex Runtime (Execution - `122`)**: The actual semantic merge runs inside the Cortex Agent Runtime Kernel safely wrapped in Temporal.
    *   **Agent Harness (`126`)**: To prevent Sybil attacks, agents operate at L1/L2 authority and submit a deterministic `AgentExecutionRecord` (Replay Artifact) which is reviewed by human or highly-trusted governed loops before merging.
*   **Benefit**: Nostra remains a lean, deterministic state machine, while Cortex handles the complex and messy reality of LLM sandboxing and verification.

**Action**: Delegate non-deterministic execution concerns entirely to `122-cortex-agent-runtime-kernel` and `126-agent-harness-architecture`.

## 3. Gap Analysis: Bounty Prioritization (008)

### The Problem
Contribution Types (008) lists `Bounty` as a Phase 3 (Future) feature.
*   **Scenario**: We need agents to report bugs ("Reflections") to improve the system.
*   **Current Reality**: Without an economic incentive (`Bounty`), optimizing "Reflections" relies on altruism, which does not scale for autonomous agents (who pay computation costs).
*   **Impact**: The "Feedback Loop" (Section 8 of 016) is broken.

### Optimal Path Forward: "Micro-Bounties" as Phase 1
We recommend prioritizing a simplified `Bounty` type immediately.

*   **MVP Scope**: A `Bounty` is just a "Promise of Payment" attached to a `WorkflowTrigger`.
*   **Flow**: `on: Reflection.Created` -> `if: Verified` -> `action: Ledger.Transfer`.
*   **Why**: This closes the economic loop for 016.

**Action**: Update `008/PLAN.md` to move Bounty to Phase 1.

## 4. Gap Analysis: Client Agent Protocol (014)

### The Problem
AI Research (014) focuses on Server-Side agents. The Skills Service relies on **Client-Side Agents** (The CLI).
*   **Scenario**: User runs `nostra-agent sync`.
*   **Current Reality**: There is no standard documented for how a CLI tool:
    1.  Authenticates (SIWE/Key).
    2.  Discovers the Space Workflow.
    3.  Formats the `Reflection` artifact.

### Optimal Path Forward: The "Nostra Agent Interface" (NAI)
We recommend defining a lightweight protocol for external clients in 014.

*   **Specs**:
    *   **Auth**: `Bearer <SessionToken>` generated via `dfx identity` or `SIWE`.
    *   **Discovery**: Standard `.well-known/nostra.json` in the Space Canister.
    *   **Artifacts**: JSON-Schema for standard types (`Reflection`, `Issue`).

**Action**: Add "Client Protocol" section to `014/RESEARCH.md`.

## 5. Summary of Recommendations for 016

To execute `016-nostra-skills-sync-service-use-case`, we rely on:
1.  **Sync Execution** in 013 (Critical for Latency).
2.  **Verifier Step** in 013 (Critical for Quality).
3.  **Active Bounties** in 008 (Critical for Telemetry).
4.  **CLI Auth Standard** in 014 (Critical for Security).

This study confirms that **016 cannot be fully realized without these platform updates**.
