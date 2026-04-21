# Placement

- Artifact ID: `paper-2026-balakrishnan-logact`
- Type: `paper`
- Topic: `agent-systems`
- Placement path: `research/reference/knowledge/agent-systems/2026_balakrishnan_logact/`
- Status: `reviewed`

## Scorecard

| Field | Score | Rationale |
|---|---:|---|
| ecosystem_fit | 5 | Directly addresses reliable agent execution, approvals, audit, and recovery in shared environments. |
| adapter_value | 5 | Strong source for a typed execution-log adapter and pre-execution voter/decider surfaces in Cortex. |
| component_value | 4 | Maps well onto runtime services, workflow replay, and agent supervision, but not every Nostra surface needs a shared log. |
| pattern_value | 5 | The paper contributes multiple reusable patterns: append-before-act logging, typed mailboxes, pluggable voters, quorum decisions, semantic recovery, and supervisor introspection. |
| ux_value | 3 | Mostly backend/runtime oriented, though it implies better operator-facing visibility and intervention tooling. |
| future_optionality | 5 | Opens paths for multi-agent coordination, operator replay, semantic health checks, and steered approvals. |
| topic_fit | 5 | Best fit is existing `agent-systems`; it is not a new topic. |

## Intent

Assess whether LogAct offers architecture or governance patterns that Nostra/Cortex should adopt for agent reliability. The goal is not to copy the paper wholesale, but to identify which abstractions can sharpen current Cortex execution traces, approval gates, and recovery surfaces.

The paper's central claim is that agents should log intended actions before execution, then route those intentions through pluggable voters and a decider before an executor mutates the environment. This changes the log from passive observability into an active control plane.

## Possible Links To Nostra Platform and Cortex Runtime

1. Nostra already treats durable execution and history as first-class concerns. `nostra/spec.md` explicitly frames durable workflows, auditability, and long-lived execution as core platform properties.
2. Cortex already has deterministic event projection primitives. `nostra/libraries/cortex-domain/src/events.rs` defines stable event IDs and projection kinds, which is adjacent to LogAct's typed AgentBus entries.
3. Cortex web/runtime already exposes approval-gated agent contribution paths. `cortex/apps/cortex-web/src/api.ts` includes `/api/kg/spaces/:space_id/agents/contributions/:run_id/approval`, and parity fixtures in `cortex/apps/cortex-eudaemon/tests/fixtures/gateway_baseline/parity_cases/post_api_kg_spaces_param_space_id_agents_contributions_param_run_id_approval.json` show approval as a governed runtime contract.
4. Initiative 123 already requires steward-gated mutation parity across runtime boundaries, which is philosophically aligned with LogAct's voter and decider separation.
5. Initiative 118 already pushes Cortex toward explicit adapters for storage, network, time, logging, and event buses. LogAct suggests one concrete next shape for the event/log side of that architecture: typed intent-first runtime logs instead of post-hoc traces alone.

## Initiative Links

- `118` Cortex Runtime Extraction: relevant for event bus, log adapter, deterministic replay, and runtime purity boundaries.
- `123` Cortex Web Architecture: relevant for approval envelopes, runtime parity, and operator-facing governance controls.
- `125` System Integrity + Quality: relevant for evidence, reproducible runtime artifacts, and gate visibility.

## Pattern Extraction

### Patterns We Already Align With

1. Durable execution as a core runtime concern. Nostra already positions workflows as long-running and restartable rather than ephemeral.
2. Auditability and lineage. Nostra treats history as sacred, and Cortex already emits deterministic event IDs for projected runtime updates.
3. Approval as a first-class mutation control. Existing agent contribution approval endpoints show the repo already recognizes that agent outputs sometimes require explicit acceptance.
4. Boundary-aware architecture. Initiative 118 already prefers explicit adapters and host-neutral boundaries that could support a shared execution-log substrate.

### Patterns We Partially Align With

1. Event logging versus intent logging. Current surfaces show event and replay thinking, but not yet a canonical typed log where intended actions become durable before execution.
2. Human approval versus programmable voter quorum. The repo has steward-gated mutation patterns, but not yet an extensible pre-execution voter layer that mixes deterministic and model-based checks.
3. Replay and recovery versus semantic recovery. Existing replay contracts and runtime sync surfaces are promising, but recovery still appears more operational than introspection-driven.
4. Multi-agent coordination. Mailbox-style coordination appears conceptually compatible with A2UI and contribution-run surfaces, but there is no obvious shared-bus abstraction for cross-agent gossip or supervision.

### Patterns We Do Not Yet Align With

1. A first-class AgentBus or typed shared log dedicated to each logical agent.
2. Isolated Driver, Voter, Decider, and Executor roles with independent permissions.
3. Policy changes recorded as typed runtime log entries rather than external configuration only.
4. Supervisor introspection over multiple agents' logs to cut duplicate work and share discovered fixes.

### Recommended Adoption Shape

1. Do not attempt a platform-wide rewrite around LogAct.
2. Adopt a narrower "intent log" pattern inside Cortex runtime for one or two high-risk execution surfaces.
3. Keep Nostra as authority over policy and approval semantics, while Cortex implements execution-time voter/decider mechanics.
4. Treat LLM-based voters as supporting checks only. Pair them with deterministic rule-based gates and clear steward overrides.

## Related Findings

1. `AgentDojo` shows why this matters: realistic tool-using agents remain vulnerable and difficult to evaluate under prompt injection, even without complex production side effects. That makes pre-execution controls more valuable than post-hoc logs alone.
2. `Defeating Prompt Injections by Design` (CaMeL) is relevant because it secures the control/data-flow boundary with explicit capabilities. For Nostra/Cortex, this suggests pairing LogAct-style intent logs with capability-scoped tool permissions so approvals are not just semantic but authority-aware.
3. `LlamaFirewall` is relevant because it treats guardrails as pluggable, real-time monitors. Its scanners and online static analysis complement LogAct's voter model well: use deterministic scanners and policy engines as first-pass voters, then reserve LLM-based review for ambiguous cases.
4. `CodeAct` is relevant as a caution. It expands agent power by using executable code as the action space, which makes LogAct-like pre-execution visibility even more important if Cortex agents gain broader code execution powers.
5. `Reflexion` is relevant but weaker as a reliability substrate. Episodic reflective memory can improve performance, but it does not provide durable intent logging, quorum checks, or recovery guarantees. It is better viewed as a payload that could run on top of a LogAct-like bus.
6. Classical shared-log and durable-workflow systems remain useful reference points. LogAct's main contribution is not logging itself, but moving the shared log into the agent control loop before execution and enriching it with typed safety and policy semantics.

## Adoption Decision

Adopt selectively as a reference architecture and prototype target.

The strongest near-term fit is a Cortex runtime experiment that adds typed pre-execution log entries for agent intentions, plus a rule-based approval voter for dangerous mutations. The next-best fit is operator tooling: log-backed supervision and replay views that make pending actions, approvals, and recovery decisions visible in one place.

The weakest fit is trying to make every workflow or Nostra contribution run through a heavy shared-log protocol. Nostra should continue to define authority and governance semantics; Cortex should selectively add LogAct-style execution controls where open-ended agent behavior creates real reliability risk.

## Known Risks

1. The paper's strongest recovery claims still depend on LLM introspection, so "recoverable" does not equal formally safe.
2. A richer runtime log can become a second authority system if it is allowed to redefine policy without clear steward ownership.
3. Shared-log adoption can add latency and complexity. The paper reports low overhead overall, but its own safety setup still adds noticeable latency when voters are enabled.
4. Cross-agent locking and concurrency discipline remain underspecified in the paper and would need a stronger Nostra authority model before production use.

## Suggested Next Experiments

1. Build a minimal `CortexIntentBus` prototype for one agent-run path with typed entries: `intent`, `vote`, `decision`, `result`, `mail`, and `policy`.
2. Add a deterministic risk classifier for file deletion, destructive workflow calls, or governance-affecting mutations and require approval before execution.
3. Surface pending intents and approval decisions in operator UI so stewards can inspect what is about to happen, not only what already happened.
4. Run a recovery experiment on a long-running workflow step using existing replay data plus a semantic diagnostic pass, then compare operator effort and correctness.
5. Test a supervisor agent that reads multiple run logs and emits suggestions or deduplication hints, but keep it advisory-only until reliability is proven.
