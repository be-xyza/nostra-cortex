# Rationale: open-responses-server

## Why Here?
`open-responses-server` provides a plug-and-play middleware that maps various AI backends (Ollama, vLLM, Groq) to the OpenAI Responses API. In the context of Nostra and Cortex, this acts as a concrete implementation of an LLM orchestration and abstraction layer. It natively supports MCPs (Model Context Protocol), which aligns perfectly with our existing `ic-rmcp/` architecture.

## Links to Nostra & Cortex
- **cortex-eudaemon Model Abstraction**: Acts as a drop-in layer to multiplex LLM backends while maintaining a consistent API contract for the execution shell.
- **MCP Integration**: The server explicitly supports MCPs around Chat Completions and Responses APIs, matching our tool-calling and governance model.
- **Local Dev Ecosystem**: Allows Cortex to easily swap between local models (Ollama) and cloud models (Anthropic/OpenAI) seamlessly during workflow execution.

## Known Risks
- It is a Python-based server; Cortex and Nostra are heavily Rust/WASM-based. Running it as a sidecar introduces another moving part to the deployment footprint.
- The project is early-stage, which implies upstream drift and potential stability issues for production workloads.

## Suggested Next Experiments
- Spin up `open-responses-server` locally via Docker alongside `cortex-eudaemon`.
- Point a Cortex agent workflow at this server, backed by a local Ollama model, and verify that MCP tool calls (via `ic-rmcp/`) execute correctly using the Responses API format.

## Stewardship
- **Primary Steward**: Systems Steward
- **Authority Mode**: recommendation_only
- **Escalation Path**: steward_review -> owner_decision
- **Lineage Record**: research/REFERENCE_MIGRATION_LINEAGE.md
- **Initiative Refs**: ["123-cortex-web-architecture"]
