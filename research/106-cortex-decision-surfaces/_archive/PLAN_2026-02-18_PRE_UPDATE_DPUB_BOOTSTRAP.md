# 106 — Cortex Decision Surfaces

stewardship:
  layer: Systems
  primary_steward: Systems Steward
  domain: Agents & Execution

## Objective
Deliver a single operator-facing decision plane across Cortex Desktop that exposes execution posture, attribution posture, governance scope, replay contract evidence, and release gates for all high-impact mutations.

## Next Phase
- Hardening stream continues in `/Users/xaoj/ICP/research/107-cortex-decision-plane-hardening/PLAN.md` for canonical-source projection, policy durability, mutation-gate synthesis, and drift enforcement.

## Scope
- Additive contract evolution only.
- Surface and enforce deterministic decision actions (`verb:id`).
- Unified inbox path for system and release-gate surfaces.

## Phases
1. Contract Layer
- Extend workflow/governance contracts for execution profile, attribution domains, replay contract, and scope evaluation.
- Regenerate declarations and sync frontend type bindings.

2. Gateway Projection Layer
- Add `/api/system/*` decision projection endpoints and decision action endpoints (`ack`, `escalate`).
- Persist gateway projection/action artifacts with deterministic IDs.

3. Cortex Surface Layer
- Extend A2UI metadata and inbox label routing for Execution/Attribution/Governance/Replay/Release Gate.
- Add operator panes in workflow views with actionable gate controls.

4. Validation Layer
- Compile and test workflow/governance/gateway/frontends.
- Verify deterministic action IDs and strict override payload validation.

## Exit Criteria
- All high-impact decision surfaces include gate and replay metadata.
- No hidden gate path outside inbox/action surfaces.
- Decision actions remain parseable and deterministic.

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
