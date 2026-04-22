# Rationale: openresponses

## Why Here?
The `openresponses` specification provides a compelling pattern for standardizing the "agentic loop" across different LLM providers. It standardizes how tool calls and streaming responses are structured. In the context of Nostra and Cortex, this represents a potential **Model Abstraction Layer** that could sit between the `cortex-eudaemon` runtime and various external AI providers (e.g., Anthropic, OpenAI, Local models).

## Links to Nostra & Cortex
- **cortex-eudaemon Model Abstraction**: Standardizes the LLM interface, preventing vendor lock-in and simplifying the addition of new models.
- **Agent Workflows**: Provides a structured way to handle tool emitting and execution loops before compiling down to our A2UI frontend representations.

## Known Risks
- The specification is still young and may drift.
- Does not replace our UI layer (A2UI); treating it as a UI protocol would be a mismatch.
- May overlap with our existing MCP (`ic-rmcp/`) tool execution boundaries if not carefully integrated.

## Suggested Next Experiments
- Prototype a `cortex-eudaemon` agent runner that consumes the `openresponses` OpenAPI spec to handle tool-call loop back-and-forth natively, instead of using provider-specific SDKs.

## Stewardship
- **Primary Steward**: Systems Steward
- **Authority Mode**: recommendation_only
- **Escalation Path**: steward_review -> owner_decision
- **Lineage Record**: research/REFERENCE_MIGRATION_LINEAGE.md
- **Initiative Refs**: ["123-cortex-web-architecture"]
