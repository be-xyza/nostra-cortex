# Initiative 137: Local Model Orchestration

## Goal
Use local Mac models (Ollama, HRM) as sovereign Cortex workers, orchestrated by Eudaemon Alpha (VPS), to reduce latency for mundane tasks and preserve resource sovereignty.

## Architecture
- **Inference**: Local Mac (Ollama/HRM) with Apple Silicon (MPS) acceleration.
- **Orchestration**: Eudaemon Alpha (VPS) routes tasks based on cognitive complexity.
- **Communication**: Phase 6 uses SSH reverse tunnels (`11434:localhost:11434`) to expose local Ollama as a standard provider to the VPS.

## Task Routing Strategy
| Task Type | Target Model | Rationale |
|-----------|--------------|-----------|
| Embeddings | `all-minilm` (local) | Fast, free, low latency (384d). |
| Code Analysis | `llama3.1:8b` (local) | High coding proficiency, private. |
| Cognitive Audit | Cloud (GPT-4o/Claude 3.5) | Maximum reasoning density for governance. |

## Visibility Goals
- Explicit "Sovereign" badges in `ProviderDashboard`.
- Model name visibility in `AgentActivityPanel`.

## Next Steps
1. Establish SSH tunnel connectivity.
2. Update task router in `cortex-eudaemon` to support local-first priority.
3. Validate "wow" UI visibility in `cortex-web`.
