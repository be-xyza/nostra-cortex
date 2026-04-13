# pi-mono Analysis

## Placement
`research/reference/topics/agent-systems/pi-mono`

## Intent
Analyze pi-mono as an AI agent toolkit, to extract patterns for our own agentic CLI, web UI, TUI, and unified LLM API integrations. It provides modular tooling across multiple packages (e.g., `packages/coding-agent`, `packages/ai`, `packages/agent`, `packages/tui`, `packages/web-ui`) that we can study to decouple UI, agent execution, and the LLM API layer.

## Possible Links To Nostra Platform and Cortex Runtime
- **Cortex-eudaemon LLM Abstraction**: The `packages/ai` package offers a unified LLM API that abstracts away provider-specific nuances, serving as a reference for our own model bindings in Rust/Motoko.
- **Cortex Workflows (Agent Core)**: The `packages/agent` layer provides a generic core that could inform how we compose context and tool-calling loops.
- **A2UI Workflows / Artifacts Cockpit**: The `packages/web-ui` and `packages/tui` offer event-driven rendering of agent cognitive loops, parallel to our goals for live A2UI execution monitoring.

## Initiative Links
- None specific at the moment, but strongly relates to ongoing runtime extractions and A2UI synthesis capabilities.

## Pattern Extraction
- **Decoupled Unified LLM API**: Isolates provider schema details (like OpenAI vs Anthropic) into a singular API boundary (`packages/ai`).
- **Separation of Cognitive Loop from UI**: By breaking out the `coding-agent` from the `web-ui` and `tui`, the agent loop acts strictly as a headless engine emitting standard events to be consumed by diverse frontends.
- **Micro-batched vLLM Orchestration**: The `packages/pods` implementation handles specific resource-aware scaling that could serve as a useful comparative architectural reference for local Temporal/Worker VPS nodes.

## Adoption Decision
Retain as reference to preserve optionality for unifying our agent tooling layer. It serves as an architectural blueprint for structuring the "headless agent / event-driven generic UI" paradigm.

## Known Risks
Upstream drift, context mismatch between a generic JS/TS ecosystem vs. an ICP-specific and Temporal-backed runtime, and semantic transplant risk (converting TypeScript loops blindly into Rust/WASM execution paths).

## Suggested Next Experiments
Prototype a unified LLM API boundary for `cortex-eudaemon` inspired by their `packages/ai`, and test if we can adapt their `web-ui` event streams to drive native A2UI cockpit viewspecs for agent feedback loops.

## Experimental Outcomes & Future Migration
- **Decoupled A2UI Terminal POC**: Successfully executed a prototype in `cortex/experiments/a2ui-terminal/`. We mapped synthetic A2UI JSON payloads generated dynamically into headless `pi-tui` terminal blocks. This confirms `cortex-eudaemon` can drive terminal UIs strictly via declarative state.
- **Rust Translation Path**: The TypeScript implementation serves purely as structural validation. The definitive logged path for any forthcoming Cortex CLI is to directly migrate this tested A2UI traversal logic into native Rust, utilizing a framework like `ratatui` to maintain parity with our core backends.
