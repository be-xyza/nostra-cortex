---
id: "103-agent-client-protocol-alignment-research"
name: "agent-client-protocol-alignment-research"
title: "Research: Agent Client Protocol Against Nostra and Cortex"
type: "research"
project: "nostra"
status: in_review
authors:
  - "User"
  - "Codex"
tags: [protocol, agents, interoperability, acp]
created: "2026-02-07"
updated: "2026-02-07"
---

# Research: Agent Client Protocol Against Nostra and Cortex

**Date**: 2026-02-07
**Status**: DRAFT
**Context**: ACP is now cataloged at `research/reference/topics/agent-systems/agent-client-protocol` and is being evaluated as a possible interoperability protocol for Cortex agent clients and Nostra-aligned execution controls.

## 1. Executive Summary
ACP is a strong candidate as an external interaction protocol for Cortex because it already standardizes session lifecycle, tool-call reporting, permission gating, filesystem/terminal delegation, and incremental updates over JSON-RPC. The main risk is that ACP assumes a trusted local editor-client boundary, while Nostra requires stronger authority scoping, provenance, and durability constraints.

## 2. Core Questions
1. Can ACP session semantics map cleanly to Nostra workflow and event standards?
2. Can ACP permission and tool-call models enforce Nostra constitutional authority boundaries?
3. What adapter hardening is required for filesystem, terminal, and extension surfaces?
4. Should ACP be treated as a production integration target, pilot target, or watch-only reference?

## 3. Findings and Analysis

### 3.1 Protocol Fit Signals
- ACP uses JSON-RPC 2.0 with clear method/notification boundaries, which aligns well with adapter-based integration.
- Session primitives (`session/new`, `session/prompt`, `session/update`, `session/cancel`) match Cortex conversational execution loops.
- Permission request flow (`session/request_permission`) supports user consent checkpoints compatible with Nostra authority doctrine.
- `session/update` carries plan entries, tool lifecycle state, and message chunks that can be projected into Nostra event timelines.

### 3.2 Primary Gaps Against Nostra and Cortex Standards
- Trusted-client assumptions around `fs/*` and `terminal/*` need explicit policy wrappers for least privilege and path/command boundaries.
- ACP session replay (`session/load`) is optional and does not itself guarantee durable history semantics; Nostra requires durable execution lineage.
- `_meta` and extension methods are flexible but require schema governance to avoid incompatible custom payload drift.

### 3.3 Initial Mapping Matrix
| ACP Surface | Nostra/Cortex Target | Preliminary Fit | Notes |
|---|---|---|---|
| `initialize` + capabilities | Cortex adapter handshake | High | Direct map to capability negotiation and feature flags |
| `session/prompt` + `session/update` | Workflow execution + event stream | Medium-High | Requires standardized event projection |
| `session/request_permission` | Stewardship/authority checkpoints | Medium-High | Needs explicit policy mapping per mode |
| `fs/read_text_file`, `fs/write_text_file` | Workspace adapter boundary | Medium | Must enforce space-scoped path policy |
| `terminal/*` | Controlled execution broker | Medium | Needs command allow/deny and audit trail |
| `_meta`, custom `_*` methods | Extensibility/observability | Medium | Requires namespacing and validation constraints |

## 4. Recommendations
- Keep this initiative in `draft` and `recommendation_only` until policy wrappers are defined.
- Treat ACP as a pilot candidate for Cortex client interoperability, not as a direct trust boundary.
- Gate any pilot behind three controls:
  1. Policy-enforced filesystem and terminal adapters.
  2. Event projection from ACP updates into Nostra-standard event envelopes.
  3. Trace/provenance requirements for `_meta` propagation and extension methods.

## 5. Spike Implementation Snapshot (2026-02-07)
- Added Rust ACP adapter spike at `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_adapter.rs`.
- Exposed minimal ACP gateway routes backed by the adapter in `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/gateway/server.rs`:
  - `POST /api/acp/fs/read_text_file`
  - `POST /api/acp/fs/write_text_file`
  - `POST /api/acp/terminal/create` (policy validation)
- Implemented policy-enforced wrappers for:
  - `fs/read_text_file` (absolute path + root boundary + line/limit constraints)
  - `fs/write_text_file` (absolute path + root boundary checks)
  - `terminal/create` validation (allowlisted commands, cwd boundary, env allowlist, output limit cap)
- Added session-update to Nostra projection mapping helper for stable event translation.
- Verification run: `cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml acp_adapter` passed (`7` tests).

## 6. Pilot Hardening Snapshot (2026-02-07)
- Added ACP JSON-RPC runtime endpoint support at `POST /api/acp/rpc`.
- Added lifecycle handlers:
  - `initialize`
  - `session/new`
  - `session/load`
  - `session/prompt`
  - `session/cancel`
  - `session/set_config_option`
  - `session/request_permission`
  - `terminal/create`
  - `terminal/output`
  - `terminal/wait_for_exit`
  - `terminal/kill`
  - `terminal/release`
- Added `_meta` validation profile enforcement with reserved trace keys and `nostra.*` namespace requirements.
- Added hybrid durability path:
  - local JSONL event persistence
  - CloudEvent emit path toward log-registry boundary
  - local gateway outbox fallback on emit failure
- Added permission ledger with pilot-scoped policy behavior for `allow_always` and `reject_always`.
- Added session replay durability and persisted turn/update sequencing.

## 7. Recommendation Update
Recommendation advances from “watch” to **pilot** pending steward review, with explicit residual risks captured in:
- `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_GATE_REPORT.md`
- `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/ADOPTION_RECOMMENDATION.md`
