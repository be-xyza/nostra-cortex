---
id: "103-agent-client-protocol-alignment"
name: "agent-client-protocol-alignment"
title: "Agent Client Protocol Alignment Study Plan"
type: "plan"
project: "nostra"
status: draft
portfolio_role: satellite
execution_plane: "cortex"
authority_mode: "recommendation_only"
reference_topics: [agent-systems, editor-platforms]
reference_assets:
  - "research/reference/topics/agent-systems/agent-client-protocol"
  - "research/reference/analysis/agent-client-protocol.md"
evidence_strength: "hypothesis"
handoff_target:
  - "Research Steward"
  - "Systems Steward"
authors:
  - "User"
  - "Codex"
tags: [protocol, agents, interoperability, acp]
stewardship:
  layer: "Architectural"
  primary_steward: "Research Steward"
  domain: "Interop"
created: "2026-02-07"
updated: "2026-02-07"
---

# Agent Client Protocol Alignment Study Plan

## Overview
Evaluate Agent Client Protocol (ACP) as an interoperable boundary between Cortex agent execution and editor-class clients, and determine how ACP can map to Nostra governance, workflow, and evidence standards without weakening authority or durability guarantees.

---

## User Review Required

> [!IMPORTANT]
> Confirm the default direction after this study:
> 1. Recommendation-only with no implementation.
> 2. Build a minimal ACP adapter spike in Cortex.
> 3. Promote ACP integration to an active implementation initiative.

---

## Proposed Architecture

### Component 1: Protocol Surface Inventory
- Baseline ACP methods/notifications: `initialize`, `session/new`, `session/prompt`, `session/update`, `session/request_permission`, `session/cancel`.
- Optional but high-value surfaces: `session/load`, `session/set_config_option`, `fs/*`, `terminal/*`, extension methods (`_*`) and `_meta` propagation.

### Component 2: Nostra-Cortex Mapping Matrix
- Map ACP `session/update` events to Nostra `Event` and workflow timeline artifacts.
- Map ACP tool call lifecycle to Cortex worker execution states and audit logging.
- Map ACP permission options to Nostra authority mode boundaries (`recommendation_only` vs approved execution).

### Component 3: Security and Constitutional Gate
- Validate ACP assumptions around trusted local file/terminal access against Nostra least-authority constraints.
- Define adapter guardrails for path scoping, command execution policy, and provenance metadata.

### Component 4: Adoption Decision Package
- Produce a recommendation with scored fit, transfer risks, and experiment gates.
- Record go/no-go criteria for a future ACP adapter in Cortex.

---

## Implementation Phases

### Phase 1: Protocol Baseline and Evidence Capture
- [x] Inventory ACP protocol primitives from local reference repo docs/schema.
- [x] Capture normative constraints into a comparison worksheet.
- [x] Lock initial scorecard and evidence confidence.

### Phase 2: Semantic Mapping Against Nostra and Cortex
- [x] Map ACP session lifecycle to workflow engine and contribution timelines.
- [x] Map ACP permissions to stewardship and escalation paths.
- [x] Map ACP extensibility (`_meta`, `_*`) to Nostra observability and schema standards.

### Phase 3: Risk, Gaps, and Mitigation Design
- [x] Identify security gaps (filesystem, terminal, permission bypass, metadata trust).
- [x] Identify durability gaps (session replay and history guarantees).
- [x] Propose adapter-level mitigations and policy checks.

### Phase 4: Recommendation and Handoff
- [x] Produce adoption recommendation (`reject`, `watch`, `pilot`, `adopt`).
- [x] Define implementation entry criteria and verification gates.
- [x] Hand off decision package to steward review.

#### Phase 4 Closure Evidence
- Pilot hardening implementation: `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_protocol.rs`
- Event sink and fallback queue: `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/acp_event_sink.rs`
- Terminal lifecycle completion: `/Users/xaoj/ICP/cortex/apps/cortex-desktop/src/services/terminal_service.rs`
- Gate report: `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_GATE_REPORT.md`
- Adoption package: `/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/ADOPTION_RECOMMENDATION.md`

---

## Verification Plan

### Automated Checks
```bash
# Ensure initiative artifacts exist and are linked
ls -la /Users/xaoj/ICP/research/103-agent-client-protocol-alignment

# Ensure ACP intake analysis exists
ls -la /Users/xaoj/ICP/research/reference/analysis/agent-client-protocol.md

# Ensure reference catalog includes ACP entry
grep -n "agent-client-protocol" /Users/xaoj/ICP/research/reference/index.toml

# Run ACP adapter spike tests
cargo test --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml acp_adapter

# Run ACP pilot hardening test set
RUSTFLAGS='-Awarnings' cargo test -q --manifest-path /Users/xaoj/ICP/cortex/apps/cortex-desktop/Cargo.toml acp_
```

### Manual Verification
1. Confirm ACP method and lifecycle mapping references local source docs.
2. Confirm constitutional constraints are preserved as recommendation-only until approval.
3. Confirm initiative linkages exist in reference analysis and index metadata.

---

## File Structure (Proposed)
```
research/103-agent-client-protocol-alignment/
|-- PLAN.md
|-- RESEARCH.md
|-- REQUIREMENTS.md
|-- ACP_NOSTRA_EVENT_MAPPING_MATRIX.md
|-- DECISIONS.md
|-- FEEDBACK.md
`-- _archive/
```

## Alignment Addendum (Constitution + System Standards)

- Labs Constitution: Default to Production patterns unless explicitly labeled as Labs; experiments remain fork-safe and documented.
- Knowledge Integrity & Memory: Preserve lineage, log decisions, and avoid rewriting history; summaries are additive, not replacements.
- Spaces Constitution: All authority and data are space-scoped; cross-space effects are explicit links, not merges.
- Stewardship & Roles: Identify accountable roles per change; unclear authority defaults to recommendation-only.
- Contribution Lifecycle: Renames, merges, archives, and scope changes require explicit rationale and approval.
- Agent Behavior & Authority: Agents operate in observe/recommend/simulate unless execution is explicitly approved.
- Security & Privacy: Least authority, explicit consent, and scoped access; minimize sensitive data exposure.
- Governance & Escalation: Disputes and irreversible actions follow escalation pathways and steward review.
- UI/UX Manifesto: Interfaces must surface provenance, time, and agency; avoid dark patterns.
- Modularity: Strict interfaces, no hardcoded canister IDs, and clean boundary contracts.
- Composability: Actions are workflow-compatible and emit standard events.
- Data Confidence & Integrity: Confidence/reliability metadata is required where applicable.
- Portability: Data must be exportable and WASM-safe; avoid OS-specific dependencies in core logic.
- Durable Execution: State is persisted via stable memory; workflows are replayable.
- Visibility Decoupling: Index/search are async from source of truth.
- Outbox Pattern: External calls are queued with idempotency and retry semantics.
- Verification: Each initiative includes verification steps and records results.
