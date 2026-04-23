# 132 — Eudaemon Alpha Initiative

**Status**: Active
**Created**: 2026-03-07
**Updated**: 2026-03-19
**Category**: Institutional Intelligence / Agent Architecture

## Summary

Initiative 132 establishes Eudaemon Alpha as the first institutional research agent aligned to the current Nostra/Cortex runtime stack. For Phase 6, the canonical deployment target is now:

- **Host**: Hetzner VPS
- **Gateway**: Rust `cortex-gateway` on the same host, bound to `127.0.0.1:3000`
- **Agent loop**: Python Eudaemon Alpha worker in `/Users/xaoj/ICP/eudaemon-alpha/agent`
- **Runtime posture**: Linux `systemd` services, production auth enabled, no Docker assumption

This initiative no longer treats Hostinger or Docker as the active deployment path, and it no longer treats the Rust-native `cortex-eudaemon` runtime as the Phase 6 primary implementation. The Rust-native path remains the migration target for Phase 7+.

The newly reviewed Doubleword batch-strategy transcript is adopted here only as an advisory architecture pattern: Eudaemon should design and synthesize a cognitive audit pipeline, not become the primary batch analyzer itself.

## Objectives

1. **Continuous Improvement**: Analyze architecture, critique designs, and propose governance-safe improvements as graph contributions.
2. **Hetzner Bring-Up**: Make the Python agent plus Rust gateway deployable and repeatable on Hetzner without architecture guesswork.
3. **Migration Readiness**: Preserve parity with the future Rust-native Cortex worker path rather than fork a separate product line.
4. **Chronicle**: Maintain a living DPub documenting ecosystem evolution from genesis.

## Identity Architecture

| Layer | Identity | Type |
|-------|---------|------|
| Nostra (platform) | `institution:cortex-stewardship-institute` | Institution contribution (`Emergent`) |
| Cortex (execution) | `agent:eudaemon-alpha-01` | Agent runtime ID |

`agent:eudaemon-alpha-01` is now a hard deployment contract across:

- Python agent config and env templates
- Gateway auth headers and identity enforcement
- Actor registry state
- Space membership bootstrapping

## Current Runtime Resolution

Before continuing implementation, Initiative 132 is resolved around the following split:

- **Exploratory working material**: Heap blocks and context bundles (`124`)
- **Operational follow-through**: closeout ledgers and workflow checkpoints (`126`, `134`)
- **Executable intent**: workflow artifact pipeline plus compiled action/navigation plans (`130`, `134`)
- **Hosted Phase 6 stack**: Hetzner VPS with loopback-local Rust gateway and Python agent worker
- **Future migration target**: Rust-native Cortex worker parity slice, not Phase 6’s primary runtime

Detailed analysis lives in [WORK_PRIMITIVES_ARCHITECTURE.md](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/WORK_PRIMITIVES_ARCHITECTURE.md).

## Cross-Initiative Alignment

Eudaemon Alpha acts as the integration pioneer for the active Nostra/Cortex stack:

- **124 (AGUI Heap Mode)**: Eudaemon emits heap blocks through `POST /api/cortex/studio/heap/emit`, reads block lists through `GET /api/cortex/studio/heap/blocks`, and packages canonical context through `POST /api/cortex/studio/heap/blocks/context`.
- **126 (Agent Harness)**: Operates at **Authority L1**. Emits `AgentExecutionLifecycle` records each cycle and treats identity enforcement as a production requirement.
- **127 (Cortex Repo Ingestion)**: Code search remains constrained to sandbox roots rather than raw repo mutation.
- **121 (Cortex Memory FS)**: Internal episodic memory remains Git-backed and local to the host.
- **125 (SIQ)**: Deterministic SIQ gates remain authoritative. Any batch-audit layer is evidence enrichment, not release authority.
- **122 (Agent Runtime Kernel)**: The Rust-native runtime remains the long-term target; Phase 6 uses the Python worker as the transitional loop.
- **047 (Temporal Architecture)**: Durable workflow semantics remain the execution model, even while hosted externally on Hetzner.
- **080 (DPub Standard)**: Chronicle drafting stays local in Phase 6; governed promotion remains the next integration step.
- **130 / 133 / 134**: Space capability governance, evaluation, and workflow artifacts remain the controlling portfolio surfaces for any future cognitive audit pipeline.

## Evolutionary Lifecycle

| Stage | Description | Runtime |
|-------|------------|---------|
| 1 | External research node | Python Eudaemon Alpha worker on Hetzner + local Rust gateway |
| 2 | Multi-agent research system | Specialized hosted agents sharing the same governed gateway |
| 3 | Native Cortex workers | Rust workers (Temporal pattern) |
| 4 | Institutional intelligence | Fully native |

## Key Decisions

- **Canonical Phase 6 runtime**: Python Eudaemon Alpha worker plus Rust gateway on the same Hetzner host
- **Canonical gateway URL**: `http://127.0.0.1:3000/api/cortex/studio`
- **Deployment model**: direct Linux services via `systemd`; no Docker requirement for Phase 6
- **Auth posture**: `NOSTRA_AUTHZ_DEV_MODE=0`, `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=0`, `NOSTRA_AGENT_IDENTITY_ENFORCEMENT=1`
- **Submission model**: heap-to-governed-artifact promotion with steward review
- **Operational access**: loopback-local gateway plus SSH access from a local private key
- **Migration posture**: `cortex-eudaemon` is the future parity target, not the current deployment default
- **Cognitive audit posture**: external batch cognition is advisory only; Eudaemon is the architect and synthesizer for audit loops, not the direct high-volume analyzer

## Deployment Surfaces

- Canonical env template: [`.env.hetzner.example`](/Users/xaoj/ICP/eudaemon-alpha/agent/.env.hetzner.example)
- Hetzner runbook: [`eudaemon-alpha-phase6-hetzner.md`](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-hetzner.md)
- Companion repo bootstrap script: [`bootstrap_eudaemon_alpha_hetzner.sh`](/Users/xaoj/ICP/eudaemon-alpha/scripts/bootstrap_eudaemon_alpha_hetzner.sh)
- Production gateway launcher: [`run_cortex_gateway_production.sh`](/Users/xaoj/ICP/scripts/run_cortex_gateway_production.sh)
- Companion repo agent launcher: [`run_eudaemon_alpha_agent.sh`](/Users/xaoj/ICP/eudaemon-alpha/scripts/run_eudaemon_alpha_agent.sh)

The `eudaemon-alpha/` path is a companion implementation repo attached to the root repo as a submodule. Initiative 132 remains authoritative in the root repo.

## References

- [Implementation Plan](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/PLAN.md)
- [Implementation Decisions](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/DECISIONS.md)
- [Work Primitive Architecture](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/WORK_PRIMITIVES_ARCHITECTURE.md)
- [Hetzner Runbook](/Users/xaoj/ICP/docs/cortex/eudaemon-alpha-phase6-hetzner.md)
- [Nostra Spec — Institutions](/Users/xaoj/ICP/nostra/spec.md#L803-L806)
- [AGENTS.md — Nostra/Cortex Separation](/Users/xaoj/ICP/AGENTS.md#L16-L19)
