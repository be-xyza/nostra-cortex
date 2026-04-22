# Analysis: mcp-cli

## Placement
- path: `research/reference/repos/mcp-cli`
- kind: `repo`
- topics: `agent-systems`

## Intent
A lightweight, Bun-based CLI for interacting with Model Context Protocol (MCP) servers. It provides an agent-optimized interface with connection pooling (lazy-spawn daemon) to make tool calling fast and token-efficient for LLMs.

## Possible Links To Nostra Platform and Cortex Runtime
Directly relevant to the Cortex MCP adapter. The lazy-spawn connection pooling daemon pattern can be translated to how local Cortex background workers or `cortex-eudaemon` sidecars maintain warm connections to external MCP servers without incurring startup costs on every call. The CLI pattern also fits well as a headless capability for debugging the Cortex agent harness.

## Initiative Links
- `131-openresponses-llm-adapter`: OpenResponses LLM adapter and Agent Harness Activation. (Tool loops and sidecar MCP logic).
- `014-ai-agents-llms-on-icp`: AI Agents and LLMs on ICP.

## Pattern Extraction
- **Lazy-Spawn Connection Pooling**: Starts a background daemon that self-terminates after 60s idle timeout to keep MCP server stdout/stdin pipes warm.
- **Config-Driven Tool Filtering**: Global config allow/deny lists for MCP tools.
- **Token-Efficient Discovery**: Using CLI interfaces as an abstraction layer so LLMs only see full tool schemas on-demand (`mcp-cli info <server> <tool>`) rather than injected upfront.

## Adoption Decision
Adopt the connection pooling pattern (60s idle daemon) as a reference architecture for Cortex's MCP adapter sidecar. Wait on the CLI abstraction layer for LLMs until we measure prompt injection overhead vs direct tool execution.

## Known Risks
Bun/Node.js ecosystem dependency if ported natively; however, the pattern is language-agnostic. The daemon approach could leak open file descriptors if the idle timeout self-termination fails or if heavily concurrent.

## Suggested Next Experiments
- Prototype a Rust-based connection pooler in `cortex-eudaemon` that implements the 60s idle termination pattern for a single local MCP server.
- Measure token usage in Cortex with vs without full MCP schema injection upfront.
