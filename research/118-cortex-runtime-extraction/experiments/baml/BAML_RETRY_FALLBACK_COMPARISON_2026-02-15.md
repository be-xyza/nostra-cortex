# BAML Retry/Fallback vs Cortex Workflow Behavior

Date: 2026-02-15  
Initiative: `research/118-cortex-runtime-extraction`

## Scope
This compares BAML retry/fallback semantics against current Cortex/Nostra workflow execution behavior to identify portable patterns and concrete gaps.

## Evidence Sources
- BAML retry/fallback declarations: `research/reference/topics/agent-systems/baml/engine/baml-runtime/src/cli/initial_project/baml_src/clients.baml`
- BAML strategy runtime: `research/reference/topics/agent-systems/baml/engine/baml-runtime/src/internal/llm_client/strategy/mod.rs`
- BAML fallback strategy: `research/reference/topics/agent-systems/baml/engine/baml-runtime/src/internal/llm_client/strategy/fallback.rs`
- BAML round-robin strategy: `research/reference/topics/agent-systems/baml/engine/baml-runtime/src/internal/llm_client/strategy/roundrobin.rs`
- BAML runtime lookup contracts: `research/reference/topics/agent-systems/baml/engine/baml-runtime/src/runtime_interface.rs`
- Cortex durable trait contract: `nostra/libraries/nostra-workflow-core/src/traits.rs`
- Cortex workflow engine behavior: `nostra/libraries/nostra-workflow-core/src/engine.rs`
- Current desktop workflow executor behavior: `cortex/apps/cortex-desktop/src/services/workflow_executor.rs`
- Existing extraction fallback policy surface: `nostra/extraction/src/lib.rs`

## Side-by-Side Behavior
| Capability | BAML | Cortex/Nostra (current) | Gap |
|---|---|---|---|
| Retry policy declaration | Declarative `retry_policy` blocks (constant + exponential) and per-client policy attachment | `DurableActivity` defines `max_retries()` + `retry_backoff_ms()` defaults | Engine does not currently invoke policy-driven retry loops in `Engine::step()` |
| Fallback provider chain | Declarative `provider fallback` with ordered strategy array | No generalized provider chain in workflow engine; endpoint-level manual retry exists (`/workflows/{id}/retry`) | Missing runtime-level fallback orchestration abstraction |
| Round-robin provider strategy | Declarative `provider round-robin` with runtime index tracking | No equivalent provider scheduler in workflow-core | Missing selection strategy contract for external ops |
| Retry + strategy integration | Strategy providers expose retry policy names via runtime interfaces | Retry and fallback exist in separate subsystems (workflow trait vs extraction policy) | No shared policy model spanning workflow external operations |
| Attempt observability | BAML tracks orchestration scope and node-level execution across providers | Workflow context stores textual history; desktop telemetry includes fallback counters but not per-attempt standardized trace | Missing structured attempt schema for provider execution decisions |

## Key Findings
1. BAML has a cohesive policy model where retry and provider strategy are first-class and composable.
2. Cortex workflow-core exposes retry knobs on traits but does not operationalize them in the default execution engine path.
3. Fallback semantics are present in adjacent systems (`extraction` fallback policy, workflow retry endpoint) but not unified for `AsyncExternalOp`.

## Recommended Experiment Outcome
Adopt the **pattern**, not the DSL:
- Define a runtime-neutral provider execution contract for `AsyncExternalOp`.
- Move retry/fallback policy into an explicit typed config object injected into runtime execution.
- Emit structured attempt events (provider, attempt, outcome, backoff, fallback transition) for deterministic replay and telemetry.

## Immediate Follow-up Hooks
- Prototype code: `research/118-cortex-runtime-extraction/experiments/baml/rust_typed_function_contract_prototype.rs`
- Mapping note: `research/118-cortex-runtime-extraction/experiments/baml/BAML_SIGNATURE_MAPPING_NOTE_2026-02-15.md`
