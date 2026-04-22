# Reference Analysis: Evaluating AGENTS.md (arXiv-2602.11988v1)

## Intent and Classification
- **Topic**: Agent Systems
- **Classification**: Paper
- **Why here**: Evaluates the effectiveness of `AGENTS.md` files (repository-level context files for coding agents). Findings directly impact Nostra/Cortex's massive `AGENTS.md` contract and agent instruction strategy.
- **Links to Nostra/Cortex**:
  - `AGENTS.md` policy and rule injection
  - Inference cost optimization
  - Agent preflight and guidance constraints

## Key Findings
1. **Performance Impact**: Large, LLM-generated `AGENTS.md` files *reduce* task success rates compared to providing no context (by ~3%), while minimal, developer-written ones improve it marginally (+4%).
2. **Inference Cost**: Utilizing these context files increases inference costs by over 20%.
3. **Behavior**: Agents follow instructions but engage in broader exploration (too much file traversal and testing) when given massive context files, making tasks harder.
4. **Conclusion**: `AGENTS.md` should be kept strictly minimal, outlining only essential tooling or requirements, avoiding "unnecessary requirements" that distract the agent.

## Known Risks
- Current Nostra `AGENTS.md` is ~450 lines long, filled with detailed constitutions, policies, and tech stack details, drastically contradicting the paper's recommendation.
- High risk of agent distraction, context window bloat, and decreased task completion rate.

## Suggested Next Experiments
- Aggressively trim `/Users/xaoj/ICP/AGENTS.md`.
- Move specific constitutions and rules into modular **skills** (e.g., `frontend-design`, `nostra-cortex-dev-core`) that are invoked only when relevant, relying on Antigravity's dynamic skill retrieval instead of a global static injection.

## Metadata
- **Primary Steward**: Research Steward
- **Authority Mode**: recommendation_only
- **Initiative Refs**: None directly, but applies workspace-wide.
