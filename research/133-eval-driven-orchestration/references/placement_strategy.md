# Placement Strategy: Integrating Eval-Driven Orchestration

To answer whether these files require a new initiative and where they should live, we must split the strategy into two horizons: **Tactical (Upgrading our Agent Skills)** and **Strategic (Upgrading the Cortex Engine)**.

The short answer is **No, they do not require a new initiative.** The concepts map perfectly as the concrete implementation details of gaps already identified in existing initiatives.

---

## 1. Tactical Placement (Immediate Iteration)
If the goal is simply to adopt Anthropic's workflow to make us (the Agents) better at writing skills *today*, the files belong directly in our local AI environment.

*   **Location**: the active IDE skill root for `skill-creator` (for example, `$CODEX_HOME/skills/skill-creator/` or `~/.codex/skills/skill-creator/`)
*   **Action**: We copy Anthropic's `eval-viewer` into `assets/`, their python scripts into `scripts/`, and rewrite our local `skill-creator/SKILL.md` to use them.
*   **Initiative Impact**: None. This is just an update to our internal tooling (similar to updating `autoskill` or `writing-skills`).

---

## 2. Strategic Placement (Cortex/Nostra Architecture)
If the goal is to formalize this paradigm so that *Cortex* natively orchestrates eval-driven workflows, we distribute the concepts into our existing "Trinity" architecture.

### A. The Benchmarking & Grader Logic -> Initiative 126 (Agent Harness)
Initiative 126 already defines an `Evaluation Loop Interface` to gate L2 outputs, but it lacks the implementation specifics.
*   **File Location**: `cortex/libraries/cortex-agents/src/harness/evaluation.rs`
*   **Action**: Formalize Anthropic's `timing.json` and metric tracking as fields within the `AgentExecutionRecord`. Implement the "Grader" script logic as the standard L2->L3 Evaluation Gate.

### B. The Eval-Viewer UI -> Initiative 123 (Cortex Web)
Anthropic relies on a standalone Python web server (`generate_review.py`) to show side-by-side prompt variations and collect feedback.
*   **File Location**: `cortex/apps/cortex-web/src/components/EvalViewer/` (and the corresponding `cortex-eudaemon` A2UI route).
*   **Action**: Discard their Python HTML generator. Instead, build a native A2UI projection for Cortex Web. When an agent enters an evaluation state, the Cortex Gateway streams an A2UI payload containing the side-by-side comparison and a structured form that saves directly back to `feedback.json` in the workspace.

### C. Parallel A/B Subagents -> Initiative 013 (Workflow Engine)
Anthropic uses bash parallelism to spawn subagents. We need structured execution.
*   **File Location**: `nostra/backend/workflow_engine/`
*   **Action**: Extend the CNCF Serverless Workflow state definitions to natively support a `ParallelAgentTask`. The engine handles branching the worktrees, dispatching the two strategies, and waiting for the 126 Evaluation Loop to select the winner.

### D. Description Optimization Loop -> Initiative 016 (Skill Sync Service)
Anthropic uses a loop to tune description triggers.
*   **File Location**: A new service or cron-job within Cortex.
*   **Action**: When new schemas, skills, or intents are proposed to the Nostra Knowledge Graph, they must pass this exact Optimization Matrix to ensure they don't corrupt the upstream LLM routing context.

## Recommendation
We do not need `132-eval-driven-development`. We simply need to update the `PLAN.md` files of **126**, **123**, and **013** to incorporate these specific architectural blueprints.

For immediate value, we should execute the **Tactical Placement** first: porting the Python scripts into the active IDE skill root for `skill-creator` so we can use them immediately to build better skills, while queuing the Strategic Placements for the actual Rust/TypeScript implementation phases.
