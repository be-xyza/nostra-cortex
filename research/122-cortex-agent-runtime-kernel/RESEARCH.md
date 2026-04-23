---
id: '122'
name: cortex-agent-runtime-kernel
title: Cortex Agent Runtime Architecture
type: general
project: nostra
status: active
authors:
- User
tags: []
created: '2026-02-20'
updated: '2026-02-22'
---

# Cortex Agent Runtime Architecture

## 1. Executive Summary

This document defines the architecture and implementation plan for the **Cortex Agent Runtime**, the minimal execution substrate required for AI agents to operate within Nostra Cortex.

The core architectural thesis is that generic LLM agent frameworks (LangChain, AutoGen, CrewAI) are misaligned with Nostra's strict requirements for **deterministic consensus, multi-year durability, and constitutional governance**. Adopting them introduces unacceptable framework drift, maintenance surface area, and non-determinism.

Instead, Nostra will implement a **Minimal Viable Kernel (MVK)** in Rust. This incredibly thin adapter layer uses Temporal workflows for the "agent loop" and standardizes entirely on the OpenAI JSON Tool Schema to convert AI reasoning into highly-typed structural mutations (via the `ActionTarget` primitive).

## 2. Architectural Positioning

*   **Nostra Layer**: Defines platform authority, data constraints, governance space rules, and canonical `GlobalEvent` history.
*   **Cortex Layer**: The execution runtime (Temporal workers) that reliably processes workflows.
*   **Cortex Agent Runtime (`cortex-agents`)**: Lives entirely *inside* the Cortex layer. It is a utility crate running within a Temporal worker. It has no authority to mutate Nostra directly; it only proposes `ActionTargets`.

## 3. Core Design Principles

1.  **Durable by Default**: The "agent loop" is a Temporal workflow. If an agent "sleeps" to wait for user input or block on an asynchronous task, the workflow suspends durationally. There are no ephemeral python sleep loops.
2.  **Authority-Separated**: Agents cannot write to the graph. They parse JSON into an `ActionTarget`, which must be executed by the Host Canister. If the action is protected, it automatically downgrades to a Governance Proposal.
3.  **Determism Wraps Non-Determinism**: LLM network calls (`perceive`, `reason`) are executed strictly as Temporal Activities. The workflow state machine evaluating the result is 100% deterministic.
4.  **A2UI & GlobalEvent Native**: Agents emit `SurfaceUpdate` payloads (for UI rendering) or `GlobalEvent` records (for graph mutations). They do not emit raw conversational markdown.
5.  **Schema Alignment**: Agent Tools use the exact JSON Schema standard expected by OpenAI/Anthropic APIs to prevent ecosystem isolation.

## 4. The Minimal Viable Kernel (MVK) Architecture

We reject the creation of complex Planners, Evaluators, or Memory Adapters in Phase 1. The runtime consists of three primary components.

### 4.1 The Core Agent Trait
The Agent is merely a pure function interface that takes a state history and returns a declarative Plan.

```rust
// cortex-agents/src/kernel.rs
#[async_trait]
pub trait CortexAgent {
    // Takes the workflow's state history, returns a list of tool invocations
    async fn reason(&self, state: &AgentState) -> Result<Plan, AgentError>;
}
```

### 4.2 The Tool Registry
Tools do not execute side-effects directly. They parse the LLM's raw JSON output into the strongly-typed `ActionTarget` struct defined in `shared/specs.md`.

```rust
pub trait CortexTool {
    fn name(&self) -> &'static str;
    fn json_schema(&self) -> serde_json::Value; // Standard OpenAI format

    // Core responsibility: JSON -> Strongly Typed Nostra Mutation Request
    fn parse(&self, input: &serde_json::Value) -> Result<ActionTarget, ToolError>;
}
```

### 4.3 The Authority Guard
The Guard acts as the middleware between the Cortex Worker and the Nostra Host Canister. It is the only component that actually mutates state.

```rust
pub struct AuthorityGuard;
impl AuthorityGuard {
    pub async fn execute_guarded(
        target: ActionTarget,
        actor: Principal
    ) -> Result<ExecutionOutcome, GovernanceError> {
        // 1. Submit ActionTarget to the Host Canister
        // 2. Canister Evaluates Constitutional Constraints
        // 3. Return outcome or error
    }
}
```

### 4.4 The Temporal Workflow (The "Loop")
The agent execution loop runs entirely within Temporal, solving the durability, retry, and crash-recovery problem natively.

```rust
// Inside the Temporal Worker
pub async fn agent_workflow(ctx: WfContext, state: AgentState) -> WfResult<()> {
    loop {
        // NON-DETERMINISTIC: Call the LLM (Wrapped in a Temporal Activity)
        let plan = ctx.activity(ReasoningActivity).input(state.clone()).await?;

        // DETERMINISTIC: Evaluate the plan
        for tool_call in plan.tool_calls {
            let tool = ToolRegistry::get(&tool_call.name)?;

            // Generate the deterministic mutation request
            let action_target = tool.parse(&tool_call.args)?;

            // MUTATION: Execute securely via Authority Guard Activity
            let outcome = ctx.activity(GuardedActionActivity).input(action_target).await?;

            state.push_history(outcome);

            // Emit GlobalEvent so Nostra indexers catch it
            ctx.activity(EmitEventActivity).input(outcome.to_event()).await?;
        }

        if plan.is_complete { break; }
    }
    Ok(())
}
```

### 4.5 The Model Provider Boundary
Inspired by the IronClaw architectural reference, LLM execution is abstracted securely out of the pure domain layer using the `ModelProviderPort` trait in `cortex-domain`. This defines core capabilities (`complete`, `embed`) independently. The concrete execution adapters (like `rig-rs` integrations for OpenAI, Anthropic, etc.) live strictly within `cortex-worker` Temporal Activities. This prevents provider vendor lock-in inside the Nostra execution environments.

## 5. Implementation Roadmap (Phase 1)

1.  **Create `cortex-agents` crate**: Initialize the minimal Rust crate within the Cortex workspace.
2.  **Define `CortexTool` and Tool Registry**: Implement the trait mapping OpenAI JSON schemas to Nostra's `ActionTarget`. Implement 2-3 standard read-only graph tools (e.g., `ReadNode`, `SearchTags`).
3.  **Implement `AuthorityGuard` Activity**: Build the Temporal Activity that submits `ActionTargets` to the Nostra Canister over real ICP RPC.
4.  **Create the Temporal Workflow Shell**: Implement the base `agent_workflow` that runs the Loop.
5.  **Integrate `rig-rs` (LLM Client)**: Inside the `ReasoningActivity`, use a minimal Rust LLM client to execute the actual prompt against GPT-4/Claude.
6.  **Persona Injection**: Implement the logic to query the `AgentPersona` graph node (`017-ai-agent-role-patterns`) at the start of the workflow to build the system prompt dynamically.

## 6. What We Are Explicitly NOT Building

To mitigate the risk of maintenance explosion, `cortex-agents` will **not** include:
*   **Prompt Management/Chaining SDKs**: The prompt is simply constructed from the Graph Persona.
*   **Custom Vector DB Adapters**: Agents must use standard tools (`SearchKnowledgeBase`) to fetch memories. The runtime itself does not magically inject embeddings.
*   **Policy Engines**: The runtime strictly delegates permission evaluations to the Host Canister.

## 7. Next Steps

This document should be merged into the `014-ai-agents-llms-on-icp` research initiative as the architectural truth for the agent execution layer. We will begin implementation of the MVK in the Cortex repository immediately following approval.

---

## 8. Tool Execution Security Model

Derived from: `research/reference/topics/agent-systems/ironclaw` — `src/sandbox/`, `src/safety/`.

### 8.1 Capability Manifest

Every `CortexTool` implementation must declare its required capabilities explicitly. This maps to the `requires.tools` field in the `017-ai-agent-role-patterns` Agent Config Schema:

| Capability | Meaning | Default |
|-----------|---------|---------|
| `http` | Tool may request outbound HTTP calls | `false` — must be explicitly granted |
| `secrets` | Tool needs access to credential material | `false` — injected at `AuthorityGuard` boundary only |
| `tool_invoke` | Tool may invoke other tools transitively | `false` — prevents transitive escalation |

Capabilities are declared at tool registration time and verified by the Tool Registry before `CortexTool.parse()` is called.

### 8.2 Credential Boundary

Credentials are **never** available to `CortexTool.parse()`. They are injected exclusively inside `AuthorityGuard.execute_guarded()` when submitting an `ActionTarget` to the Host Canister. This mirrors the host-boundary injection pattern from ironclaw's WASM sandbox and prevents tools from exfiltrating credentials through their JSON output.

### 8.3 Tool Output Gate

Before any tool call result is injected back into the LLM context window, it should pass through a lightweight wrapper:

1. Tag the result with a clear delimiter: `<tool_result id="…">…</tool_result>`
2. Strip or escape known injection escape sequences from content strings

This is a formatting pass — not a full policy engine. The governance gate (`GateOutcome`) remains the authoritative enforcement point.

### 8.4 Severity Routing Alignment

IronClaw's severity model (`Block` / `Warn` / `Review` / `Sanitize`) maps directly onto the existing `GateOutcome` enum in `cortex-runtime/src/ports.rs`:

| IronClaw Severity | Cortex `GateOutcome` |
|------------------|---------------------|
| Block | `Block` |
| Review | `RequireReview` |
| Simulate | `RequireSimulation` |
| Warn | `Warn` |
| Pass | `Pass` |

No new severity types are needed. The `GovernanceAdapter.evaluate_action_scope()` call on the canister side provides the enforcement.

### 8.5 SSRF Prevention (Future Gate)

Relevant when any tool declares `http` capability. At that point, require an allowlist configuration for permitted HTTP origins — no tool may make unrestricted outbound calls. This is a pre-requisite for `http` capability approval, not a general runtime concern today.

## 9. Implementation Status

### Phase 2 — Production Readiness (Completed 2026-02-22)
- `ModelProviderPort` trait defined in `cortex-agents/src/provider.rs` using the Ironclaw pattern.
- `OpenAiProvider` adapter implemented via `async-openai`.
- Temporal mock facade scaffolded in `cortex-worker/src/temporal.rs`.
- `ArchitectAndEvaluateWorkflow` and `EvaluateSimulationPlanActivity` registered in the worker binary.
- GSMS Translation Engine mapping `ActionTarget` → `SimulationAction` in `cortex-agents/src/translator.rs`.
- A2UI React renderer bootstrapped in `cortex-web/src/components/a2ui/`.

### Phase 3 — Integrating Execution & GSMS Feedback (Completed 2026-02-22)
- `SpaceExecutionHook` domain models mapping `SpaceId` → Agent Initiative in `cortex-domain/src/agent/execution.rs`.
- Gateway API endpoint `/api/kg/spaces/:space_id/agents/initiatives` dispatching workflows.
- `HumanApprovalEvent` / `ApprovalDecision` models in `cortex-domain/src/simulation/feedback.rs`.
- Workflow pauses for human validation via `wait_for_human_approval` in the mock Temporal context.
- `ApprovalControls` and `DiffViewer` widgets added to the A2UI `WidgetRegistry`.
- Desktop `SpaceGraphExplorerView` wired with "Agent Initiative" pipeline action.
