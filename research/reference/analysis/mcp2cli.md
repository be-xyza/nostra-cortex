# Analysis: mcp2cli

## Placement
- path: `research/reference/repos/mcp2cli`
- kind: `repo`
- topics: `agent-systems`

## Intent
Turns any MCP server or OpenAPI spec into a CLI at runtime with zero codegen. Optimizes LLM context windows by allowing the agent to discover tools via CLI arguments (`--list`, `--help`) instead of injecting full JSON schemas into the system prompt, saving 96-99% of tokens per turn.

## Possible Links To Nostra Platform and Cortex Runtime
Highly relevant to Cortex's agent execution layer. The dynamic OpenAPI/MCP to CLI adapter concept validates that token-efficient tool discovery is necessary for scalable agents. This could influence the A2UI protocol or the `CortexRuntime` by demonstrating an on-demand schema fetching mechanism.

## Initiative Links
- `131-openresponses-llm-adapter`: Agent harness tool loops.
- `014-ai-agents-llms-on-icp`: LLMs and external workers on ICP.

## Pattern Extraction
- **Dynamic CLI Generation**: Reads an OpenAPI spec or queries MCP definitions at runtime to build `argparse` structures without any recompilation.
- **Token-Efficient Encoding (TOON)**: Dedicated output mode that compresses JSON responses into an LLM-friendly compact text format.
- **Unified Adapter Model**: Exposes both HTTP REST (OpenAPI) and MCP (stdio/SSE) behind a unified execution interface.
- **Automatic OAuth / Auth Management**: Built-in state for tokens.

## Adoption Decision
Adopt the token-efficient LLM rendering patterns (TOON) and the on-demand schema discovery as core constraints for the Cortex agent harness. Implement unified adapters for OpenAPI+MCP inside the Cortex runtime following `mcp2cli`'s abstractions.

## Known Risks
Dynamic runtime adaptation can be brittle if upstream specs change mid-conversation. The token savings approach forces the LLM to expend inference steps exploring the CLI rather than executing the task immediately. Python-bound logic.

## Suggested Next Experiments
- Compare token costs and speed between A) injecting 10 MCP tools upfront vs B) letting a Cortex-runner agent discover them using an `mcp2cli`-like mechanism.
- Evaluate compressing A2UI state updates using the TOON encoding pattern.
