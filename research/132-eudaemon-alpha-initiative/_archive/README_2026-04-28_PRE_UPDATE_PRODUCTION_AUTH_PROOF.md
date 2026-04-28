# 132 — Eudaemon Alpha Initiative

**Status**: Active
**Created**: 2026-03-07
**Updated**: 2026-04-27
**Category**: Institutional Intelligence / Agent Architecture

## Summary

Initiative 132 establishes Eudaemon Alpha as the first institutional research agent aligned to the current Nostra/Cortex runtime stack. For this pass, the root repo is the authoritative planning source, and the currently validated runtime facts are:

- **Host**: Hetzner VPS
- **Gateway**: Rust `cortex-gateway` on the same host, bound to `127.0.0.1:3000`
- **Worker**: Rust `cortex_worker` from `nostra/worker` is the active VPS worker target declared by the current deploy authority; PR #69 restores it as a passive build/preflight binary, while live polling/runtime execution remains gated by production identity and host-mode VPS authority proof
- **Runtime posture**: Linux `systemd` services, production auth enabled, no Docker assumption

Gateway parity passes locally in this checkout. The active VPS contract is `cortex-gateway` plus `cortex_worker`, validated through the runtime authority manifest and operator-local promotion flow. The worker build/preflight blocker is addressed by PR #69, but runtime readiness still requires promotion plus host-mode authority validation. Prompt override remains unverified and should be treated as a future capability rather than a live dependency. Meta-Harness findings are recommendation-only and do not change authority boundaries.

The newly reviewed Doubleword batch-strategy transcript is adopted here only as an advisory architecture pattern: Eudaemon should design and synthesize a cognitive audit pipeline, not become the primary batch analyzer itself. Phase 6 communication and main-cycle analysis stay on the native live cognition lane first.

Hermes is adopted for the next local pass only as a local advisory meta-observer: it may receive Initiative 132, Doubleword, architecture, and provider batch-policy references as read-only source material for architecture observation, contradiction detection, drift detection, bounded audit-unit analysis, and recommendation synthesis, but it must not receive provider credentials, submit batch jobs, poll batch APIs, or perform repository/runtime mutation. Each Hermes pass must remain bounded, deterministic, and auditable, and it must produce only source-linked local findings plus a synthesis artifact.

The Hermes Capability & Discovery Envelope adds a second local planning layer for missing observer lanes and realistic capability expectations. It may classify Hermes-native features, map system cartography and boundary-integrity lanes, translate external-agent patterns into Cortex-native design notes, and draft skill-improvement proposals. It does not enable new tools, activate skills, schedule jobs, run batch infrastructure, or introduce execution adapters.

Hermes runbooks may automate the operator ritual around advisory passes: preflight, one bounded pass, postflight, optional evidence drafting, and manual promotion. They do not grant Hermes unattended agency or mutation authority.

Hermes source packets are now the preferred bounded excerpt/fact layer when a pass does not need broad direct file reading. If Hermes outputs are later surfaced to the user in Cortex Web, the intended route is through existing heap, steward-gate, A2UI approval, and proposal-review primitives rather than a bespoke Hermes control plane.

Hermes is now green for bounded local advisory review and lineage only. This green status does not authorize production identity readiness, production authorization, ICP evidence promotion beyond the recorded evidence note, provider execution, subagents, skill activation, memory authority, unattended execution, or Hermes acting as a workflow authority.

`hermescortexdev` is separate from `hermes132`: it is a local patch-prep developer/operator profile that can produce bounded implementation handoffs from approved local task packets. Its outputs are local operator artifacts until manually promoted into Initiative 132 evidence, heap/proposal surfaces, workflow drafts, or other governed records.

Developer worktree isolation, checkpointing, and immutable evidence promotion are now explicit operator safety controls around Initiative 132. They protect the system-definition layer and steward continuity, but they are not themselves runtime heap, closeout-ledger, or workflow primitives.

Hermes-related local work now has an explicit record-sync gate: if a Hermes profile, runbook, task packet, handoff artifact, stabilization status, or role-boundary correction affects Initiative 132 understanding, it must be promoted into governed records or clearly marked local-only.

## Objectives

1. **Continuous Improvement**: Analyze architecture, critique designs, and propose governance-safe improvements as graph contributions.
2. **Hetzner Bring-Up**: Keep the Rust gateway plus `cortex_worker` VPS contract deployable and repeatable without architecture guesswork.
3. **Migration Readiness**: Preserve parity with the future Rust-native Cortex worker path rather than fork a separate product line.
4. **Chronicle**: Maintain a living DPub documenting ecosystem evolution from genesis.

## Identity Architecture

| Layer | Identity | Type |
|-------|---------|------|
| Nostra (platform) | `institution:cortex-stewardship-institute` | Institution contribution (`Emergent`) |
| Cortex (execution) | `agent:eudaemon-alpha-01` | Agent runtime ID |

`agent:eudaemon-alpha-01` is now a hard deployment contract across:

- Runtime env, gateway auth headers, and worker identity enforcement
- Actor registry state
- Space membership bootstrapping

## Current Runtime Resolution

Before continuing implementation, Initiative 132 is resolved around the following split:

- **Exploratory working material**: Heap blocks and context bundles (`124`)
- **Operational follow-through**: closeout ledgers and workflow checkpoints (`126`, `134`)
- **Executable intent**: workflow artifact pipeline plus compiled action/navigation plans (`130`, `134`)
- **Hosted Phase 6 stack**: Hetzner VPS with loopback-local Rust gateway and `cortex_worker`; historical Python companion references are unvalidated in this checkout
- **Future migration target**: Rust-native Cortex worker parity slice, not Phase 6’s primary runtime

Detailed analysis lives in [WORK_PRIMITIVES_ARCHITECTURE.md](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/WORK_PRIMITIVES_ARCHITECTURE.md).

## Cross-Initiative Alignment

Eudaemon Alpha acts as the integration pioneer for the active Nostra/Cortex stack:

- **124 (Polymorphic Heap Mode)**: Eudaemon emits heap blocks through `POST /api/cortex/studio/heap/emit`, reads block lists through `GET /api/cortex/studio/heap/blocks`, and packages canonical context through `POST /api/cortex/studio/heap/blocks/context`.
- **126 (Agent Harness)**: Operates at **Authority L1**. Emits `AgentExecutionLifecycle` records each cycle and treats identity enforcement as a production requirement.
- **Visibility and approvals**: existing heap solicitations, steward feedback, A2UI approval telemetry, steward-gate validate/apply, viewspec proposal review, workflow-draft proposal review, and agent-contribution approval surfaces are the correct future projection path for Hermes outputs if user-facing visibility is added later.
- **127 (Cortex Repo Ingestion)**: Code search remains constrained to sandbox roots rather than raw repo mutation.
- **121 (Cortex Memory FS)**: Internal episodic memory remains Git-backed and local to the host.
- **125 (SIQ)**: Deterministic SIQ gates remain authoritative. Any batch-audit layer is evidence enrichment, not release authority.
- **122 (Agent Runtime Kernel)**: The Rust-native runtime remains the long-term target; the active VPS worker contract now uses `cortex_worker`, while older Python companion references are historical/unvalidated here.
- **047 (Temporal Architecture)**: Durable workflow semantics remain the execution model, even while hosted externally on Hetzner.
- **080 (DPub Standard)**: Chronicle drafting stays local in Phase 6; governed promotion remains the next integration step.
- **130 / 133 / 134**: Space capability governance, evaluation, and workflow artifacts remain the controlling portfolio surfaces for any future cognitive audit pipeline.
- **ZeroClaw (latest upstream)**: Relevant only as a possible sidecar/auth pattern source for Codex subscription access. It is not the runtime authority, gateway replacement, or workflow substrate.
- **Repo Hygiene Program**: Clean request worktrees, durable checkpoint bundles, and immutable evidence promotion keep system-definition work aligned with Initiative 125 integrity controls without reclassifying Git state as a Cortex runtime primitive.

## Evolutionary Lifecycle

| Stage | Description | Runtime |
|-------|------------|---------|
| 1 | External research node | Hetzner `cortex_worker` + loopback-local Rust gateway; historical Python companion path unvalidated |
| 2 | Multi-agent research system | Specialized hosted agents sharing the same governed gateway |
| 3 | Native Cortex workers | Parity-backed Rust slices (Temporal pattern, stricter typed API boundaries) |
| 4 | Institutional intelligence | Fully native, with OS-level sandboxing required for untrusted execution paths |

## Key Decisions

- **Canonical Phase 6 runtime**: `cortex_worker` plus Rust gateway on the same Hetzner host; older Python companion assets are not current checkout authority
- **Canonical gateway URL**: `http://127.0.0.1:3000/api/cortex/studio`
- **Deployment model**: direct Linux services via `systemd`; no Docker requirement for Phase 6
- **Promotion authority**: operator-local SSH via [`scripts/promote_eudaemon_alpha_vps.sh`](/Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh); GitHub signals promotability only
- **Auth posture**: `NOSTRA_AUTHZ_DEV_MODE=0`, `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`, `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
- **Submission model**: heap-to-governed-artifact promotion with steward review
- **Operational access**: loopback-local gateway plus SSH access from a local private key
- **Migration posture**: `cortex-eudaemon` is the future parity target, but Phase 7 should advance through parity-backed Rust slices and measured extraction from the current `cortex-eudaemon` surface rather than a wholesale replatform
- **Security Posture**: Phase 6 remains a governed Hetzner `systemd` runtime with operator-local SSH promotion; OS-level sandboxing becomes mandatory for the executor slice that runs untrusted code or broader autonomous contribution loops
- **Cognitive audit posture**: external batch cognition is advisory only; Eudaemon is the architect and synthesizer for audit loops, not the direct high-volume analyzer
- **Hermes posture**: Hermes may be used locally as a read-only meta-observer, with one bounded auditable pass, source-linked findings plus synthesis output, and no live batch-provider execution or repository/runtime mutation
- **Hermes developer posture**: `hermescortexdev` may prepare local patch handoffs only; Codex/operator applies implementation separately
- **Hermes record-sync posture**: local Hermes artifacts do not update Initiative 132 by implication; they need explicit evidence/decision/plan promotion or a local-only disposition
- **Hermes capability discovery posture**: Hermes may classify capabilities and propose observer lanes or skill improvements, but the result is local planning material only; feature enablement and execution adapters require later governed decisions
- **Hermes runbook posture**: local runbooks may standardize bounded pass operation, but preflight/postflight, evidence promotion, commits, and pushes remain operator-mediated
- **Provider posture**: low-latency live cognition is the primary Phase 6 path; batch audit stays secondary
- **Subscription posture**: ChatGPT Pro matters only through official Codex subscription access; it is not a generic API-credit source for the worker
- **Developer isolation posture**: request work belongs in clean `.worktrees/`; the shared root worktree is reserved for repo-wide stewardship tasks
- **Evidence posture**: mutable `logs/*` outputs remain local operational artifacts; durable evidence is preserved by promoting immutable copies into governed initiative surfaces

## Validated Phase 7 Sequencing

The validated next-step order for Initiative 132 is:

1. **Contract hardening now**: tighten Rust/TypeScript DTO sync, discriminated payloads, and network-boundary serialization on the currently exposed Gateway and A2UI contracts.
2. **Parity-backed Rust slice next**: port the highest-value Eudaemon loop capabilities into `cortex-eudaemon` without changing the canonical Phase 6 deployment authority model until parity is proven.
3. **Executor isolation before untrusted execution**: apply OS-level sandboxing to the execution slice that runs generated shell/code or broader autonomous contribution actions before those capabilities become real.

The first extraction seams worth pursuing in the current repo are:

- **Provider runtime surface**: provider registry/runtime/client/policy logic
- **ACP / terminal execution surface**: ACP protocol, terminal control, and permission-ledger enforcement
- **Workbench UX / heap projection surface**: heap/workbench/viewspec projection and UX orchestration

Current stage evidence now shows Batch 1 materially advanced on the provider-runtime surface: remote runtime-host discovery moved behind `provider_runtime::discovery`, provider-admin auth-binding helper logic moved behind `gateway::provider_admin`, and the governed parity/operator checks remained green after the extraction. ACP and workbench extraction remain deferred by default rather than being pulled into the same batch.

## Repo Hygiene Alignment

Initiative 132 treats repo hygiene as an operator safety layer with three clear boundaries:

1. Heap blocks remain exploratory runtime material.
2. Closeout ledgers remain runtime follow-through for operational remediation.
3. Workflow checkpoints remain durable execution primitives.

Developer Git worktree isolation does not replace any of those surfaces. It exists to protect the system-definition layer from lost steward updates, dirty-root drift, and ambiguous evidence provenance.

The practical consequences are:

- request work should begin from a clean `.worktrees/` branch
- root-worktree work is reserved for repo-wide stewardship operations
- mutable `logs/*_latest.*` outputs are reproducible local artifacts, not durable Git authority
- evidence that matters should be promoted into governed initiative surfaces and remain source-linked

## Deployment Surfaces

- Hetzner runbook: [`eudaemon-alpha-phase6-hetzner.md`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-hetzner.md)
- Local promotion command: [`scripts/promote_eudaemon_alpha_vps.sh`](/Users/xaoj/ICP/scripts/promote_eudaemon_alpha_vps.sh)
- On-host deploy script: [`ops/hetzner/deploy.sh`](/Users/xaoj/ICP/ops/hetzner/deploy.sh)
- Gateway unit template: [`cortex-gateway.service`](/Users/xaoj/ICP/ops/hetzner/systemd/cortex-gateway.service)
- Worker unit template: [`cortex-worker.service`](/Users/xaoj/ICP/ops/hetzner/systemd/cortex-worker.service)
- Runtime authority check: [`scripts/check_vps_runtime_authority.sh`](/Users/xaoj/ICP/scripts/check_vps_runtime_authority.sh)

Older deployment notes reference an `eudaemon-alpha/` companion implementation repo, but those references are historical only. Initiative 132 remains authoritative in the root repo, and the active VPS deployment contract is the current runbook plus runtime authority manifest.

## Governed Evidence

- [Evidence directory purpose](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/evidence/README.md)
- [Promoted dirty inventory metadata](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/evidence/20260410T072110Z_dirty_inventory.json.meta.json)
- [Batch 1 Decision Gate](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/BATCH1_DECISION_GATE.md)
- [Phase 7 Execution Plan](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/PHASE7_EXECUTION_PLAN.md)
- [Hermes first advisory activation](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/evidence/20260423T083435Z_hermes_first_advisory_activation.md)

## References

- [Implementation Plan](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/PLAN.md)
- [Phase 7 Execution Plan](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/PHASE7_EXECUTION_PLAN.md)
- [Batch 1 Decision Gate](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/BATCH1_DECISION_GATE.md)
- [Implementation Decisions](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/DECISIONS.md)
- [Work Primitive Architecture](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/WORK_PRIMITIVES_ARCHITECTURE.md)
- [Hetzner Runbook](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-hetzner.md)
- [Nostra Spec — Institutions](/Users/xaoj/ICP/nostra/spec.md#L803-L806)
- [AGENTS.md — Nostra/Cortex Separation](/Users/xaoj/ICP/AGENTS.md#L16-L19)
