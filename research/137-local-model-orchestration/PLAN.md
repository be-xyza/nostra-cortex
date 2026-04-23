---
id: '137'
name: local-model-orchestration
title: 'Initiative 137: Local Model Orchestration'
type: general
project: nostra
status: active
authors:
- User
tags: []
created: '2026-03-23'
updated: '2026-03-23'
---

# Initiative 137: Local Model Orchestration

## Goal
Use local Mac models (Ollama, HRM) as local Cortex workers, orchestrated by Eudaemon Alpha (VPS), to reduce latency for mundane tasks and preserve resource locality.

## Architecture
- **Inference**: Local Mac (Ollama/HRM) with Apple Silicon (MPS) acceleration.
- **Orchestration**: Eudaemon Alpha (VPS) routes tasks based on cognitive complexity.
- **Communication**: Phase 6 uses SSH reverse tunnels (`11434:localhost:11434`) to expose local Ollama as a standard provider to the VPS.
- **Provider model**: Keep the provider registry additive and backward-compatible, with provider family, model profile, runtime instance, device identity, environment identity, and locality metadata surfaced as a structured topology block.
- **Locality labels**: Prefer `Local`, `Tunneled`, and `Cloud` in the UI; keep `Sovereign` internal only if needed for execution policy.

## Task Routing Strategy
| Task Type | Target Model | Rationale |
|-----------|--------------|-----------|
| Embeddings | `all-minilm` (local) | Fast, free, low latency (384d). |
| Code Analysis | `llama3.1:8b` (local) | High coding proficiency, private. |
| Cognitive Audit | Cloud (GPT-4o/Claude 3.5) | Maximum reasoning density for governance. |

## Visibility Goals
- Explicit `Local` / `Tunneled` / `Cloud` badges in `ProviderDashboard`.
- Provider family and model visibility in `ProviderDashboard` and `AgentActivityPanel`.
- Keep device and environment IDs behind hover or drill-down detail.

## Next Steps
1. Establish SSH tunnel connectivity.
2. Update task router in `cortex-eudaemon` to support local-first priority.
3. Extend the provider registry and frontend to show the additive topology block and refresh/discover local providers.
4. Validate the simplified provider UI in `cortex-web`.
