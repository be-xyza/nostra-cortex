# pi-mono Analysis

## Placement
`research/reference/topics/agent-systems/pi-mono`

## Intent
Analyze `pi-mono` as a reference repository for modular agent tooling. The validated scope is a shared TypeScript monorepo containing a provider-agnostic LLM layer (`packages/ai`), a headless agent runtime (`packages/agent`), a coding-agent CLI (`packages/coding-agent`), reusable browser UI (`packages/web-ui`), terminal rendering primitives (`packages/tui`), and optional GPU pod tooling (`packages/pods`).

## Possible Links To Nostra Platform and Cortex Runtime
- **Cortex provider/model adapters**: `packages/ai` is a useful comparison point for provider discovery, tool-calling-first model selection, token/cost accounting, and OpenAI-compatible transport seams.
- **Cortex runtime agent orchestration**: `packages/agent` exposes an explicit event lifecycle for assistant streaming and tool execution, which is directly relevant to `cortex-runtime` agent loop contracts.
- **Cortex host-side workbench surfaces**: `packages/web-ui` shows how a browser chat/artifact surface can sit on top of a shared agent core without becoming the authority layer.
- **Operator-installed capability bundles**: `packages/coding-agent` adds extensibility through skills, prompts, themes, extensions, and installable packages, which is relevant structurally if treated as a host capability and not a default runtime primitive.

## Initiative Links
- `118-cortex-runtime-extraction`: relevant for event buses, agent/tool lifecycle boundaries, and provider abstraction.
- `124-agui-heap-mode`: relevant for host-side projection and workbench/UI consumption patterns.
- `132-eudaemon-alpha-initiative`: relevant for live-provider boundaries, agent loop evolution, and future A2UI feedback surfaces.

## Pattern Extraction
- **Unified LLM layer with tool-calling scope**: `packages/ai` centers on tool-capable models, supports multiple providers and OpenAI-compatible APIs, and exposes streaming events plus cross-provider handoff patterns.
- **Headless agent core with explicit event sequencing**: `packages/agent` documents `agent_start`, `message_update`, `tool_execution_start`, `tool_execution_end`, and configurable parallel versus sequential tool execution with preflight/post hooks.
- **Multiple frontends over one agent/runtime substrate**: the CLI, browser UI, and terminal UI are separate packages over shared lower layers instead of a single inseparable application.
- **Extensibility as a first-class host concern**: the coding agent emphasizes installable skills, prompt templates, extensions, themes, and packages, which is informative for operator-side capability composition.
- **Remote model hosting as a separate operational surface**: `packages/pods` isolates GPU pod and vLLM management from the agent core, which is a useful boundary example even though it is not a direct fit for Nostra authority surfaces.

## Adoption Decision
Retain as an `agent-systems` reference. The repository is a strong pattern source for Cortex-side agent/runtime ergonomics and host UI layering, but it should inform boundary design rather than be transplanted directly into Nostra platform authority or Rust/WASM execution surfaces.

## Known Risks
- Upstream churn is high: the repository shows active internal refactoring and frequent releases, so assumptions can stale quickly.
- The trust model is looser than ours: `pi` packages and extensions run with full system access, which requires explicit operator gating if any analogous capability model is ever adopted in Cortex hosts.
- Storage assumptions differ: the browser UI uses IndexedDB-backed local storage and the CLI persists local sessions, which does not map directly to governed Nostra authority surfaces.
- Language/runtime mismatch remains substantial: the repo is primarily TypeScript/browser oriented, so extraction into Rust/WASM/ICP should happen at the contract level, not by direct code migration.

## Suggested Next Experiments
- Compare `pi-agent-core` event sequencing and tool lifecycle with current `cortex-runtime` agent contracts to identify the smallest parity slice worth porting.
- Prototype a thin Cortex provider adapter surface inspired by `pi-ai`, limited to provider/model selection, token-cost accounting, and OpenAI-compatible transport seams.
- Evaluate whether a host-only capability package model could inform Cortex operator-installed extensions without inheriting `pi`'s default full-access trust posture.
