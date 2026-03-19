# 132 — Eudaemon Alpha Initiative

**Status**: Planning
**Created**: 2026-03-07
**Category**: Institutional Intelligence / Agent Architecture

## Summary

Establish the Cortex Stewardship Institute as the first institutional intelligence within the Nostra-Cortex ecosystem. The initiative deploys a ZeroClaw-based research agent (`agent:eudaemon-alpha-01`) on a secured VPS to observe the ContributionGraph, detect patterns, produce insight contributions, and maintain a DPub chronicle of the platform's evolution.

## Objectives

1. **Continuous Improvement** — Analyze architecture, critique designs, propose improvements as graph contributions
2. **Preparation for Cortex-Native Agents** — Research memory architecture, agent governance, and workflow patterns needed for Stage 3 migration
3. **Chronicle** — Maintain a living DPub documenting the ecosystem's evolution from genesis

## Identity Architecture

| Layer | Identity | Type |
|-------|---------|------|
| Nostra (platform) | `institution:cortex-stewardship-institute` | Institution contribution (`Emergent`) |
| Cortex (execution) | `agent:eudaemon-alpha-01` | Agent runtime ID |

## Cross-Initiative Architectural Alignment

Eudaemon Alpha acts as the integration pioneer for ten Nostra-Cortex architectural initiatives:

- **124 (AGUI Heap Mode)**: Workspace is a public Space named `Cortex Stewardship` using the `HeapWorkspaceView`. Eudaemon emits polymorphic blocks (`POST /api/cortex/studio/heap/emit`) and reads context bundles (`POST /blocks/context`).
- **126 (Agent Harness)**: Operates at **Authority L1 (Suggestion-only)**. Emits rigorous `AgentExecutionLifecycle` events to the `GlobalEvent` stream on every cycle.
- **127 (Cortex Repo Ingestion)**: Code-search tool boundaries restrict AST parsing exclusively to sync-fetches inside `cortex-memory-fs/sandboxes/`, blocking raw Nostra repo reads.
- **122 (Agent Runtime Kernel)**: Eudaemon Alpha explicitly serves as the Python VPS prototype simulating the robust Rust "Minimal Viable Kernel" (MVK) defined in 122.
- **121 (Cortex Memory FS)**: Internal episodic mapping, prompting traces, and tool histories are persisted purely to a local, Git-backed filesystem hosted on the VPS, isolating internal reasoning from canonical Nostra graph publications.
- **062 (Model Constitutions)**: System prompt heavily enforces the `AgentDisclosurePattern` to formally separate Nostra logic from base-model safety refusals.
- **047 (Temporal Arch)**: The ZeroClaw runtime must align conceptually with the "Workflow-as-Agent" durable execution loop.
- **080 (DPub Standard)**: The Chronicle is a formally bound `Contribution<DPub>` with continuous living drafts and monthly immutable Editions.
- **113 (CRDT)** & **115 (ViewSpec)**: Agent block emissions must conform to structured CRDT convergence and ViewSpec UI synthesis contracts.
- **130 (Space Governance)**: Eudaemon's Space capability graph must explicitly activate Heap Mode and DPub rendering.

## Evolutionary Lifecycle

| Stage | Description | Runtime |
|-------|------------|---------|
| 1 | External research node | ZeroClaw on VPS (interfacing via Cortex APIs) |
| 2 | Multi-agent research system | Specialized ZeroClaw agents |
| 3 | Native Cortex workers | Rust workers (Temporal pattern) |
| 4 | Institutional intelligence | Fully native |

## Key Decisions

- **Runtime**: ZeroClaw (minimal orchestration) — aligned with Temporal durability constraints
- **Workspace**: 124 Heap Board — NO new primitives needed; fully leverages polymorphic block emission
- **Deployment**: Hostinger VPS (Ubuntu + Docker) — isolation from dev environment pending native migration
- **Submission**: Agent proposes; human steward reviews (via 126 Evaluation Loop gates)
- **Docker Security**: Container NOT `read_only`. Agent has internal autonomy (pip, scratch files). Canonical repos mounted `ro`. Sandbox mounted `rw`.
- **Network**: `internal: true` removed — agent needs outbound HTTPS for Gateway, PyPI, LLM APIs
- **Governance**: Dual-layer — immutable local hard caps (env vars) + graph-native Heap policy blocks for collaborative adjustment
- **Execution Strategy**: Adaptive routing with `volatility_mode` (real-time for volatile codebases, batch for stable)
- **Self-Optimization**: Agent proposes config improvements via `SelfOptimizationProposal` blocks on Heap (L1-gated, never auto-applied)
- **Bootstrap**: On first boot, agent probes APIs and emits `ConfigProposalBlock` to Heap for steward review

## References
- [Original Conversation (PDF)](../reference/inbox/NOSTRA%20-%20Branch%20·%20VPS%20Cortex%20Eudaemon%20Setup.pdf)
- [Eudaemon Charter (from original config)](../reference/inbox/NOSTRA%20-%20Branch%20·%20VPS%20Cortex%20Eudaemon%20Setup.pdf#page=100)
- [Nostra Spec — Institutions](../../nostra/spec.md#L803-L806)
- [AGENTS.md — Nostra-Cortex Separation](../../AGENTS.md#L16-L19)
