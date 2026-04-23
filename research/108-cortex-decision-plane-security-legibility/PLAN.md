---
status: active
portfolio_role: anchor
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Agents & Execution"
---

# 108 — Cortex Decision Plane Security + Canonical Legibility

stewardship:
  layer: Systems
  primary_steward: Systems Steward
  domain: Agents & Execution

## Objective
Harden Cortex Desktop decision operations so actor identity, policy gating, and replay lineage are canonical, deterministic, and space-scoped while preserving additive compatibility.

## Scope
- Governance actor-role binding APIs and policy snapshot retrieval.
- Workflow replay lineage APIs and deterministic decision digest surfacing.
- Gateway signed-intent staged enforcement (`off|warn|required_p0_p1|required_all`).
- Space-scoped operator surfaces, telemetry, and structured risky-action quality forms.
- CI checks for DID/declaration drift and no-panic decode guardrails.

## Phases
1. Contract additions + declaration mirrors (governance/workflow engine).
2. Stable persistence for governance actor-role bindings.
3. Gateway gate orchestration v2 with canonical actor-policy evaluation.
4. Cortex decision-surface UX enforcement and multi-space visibility.
5. Verification gate expansion: determinism, auth staging, degraded transparency.

## Exit Criteria
- High-impact actions are policy-evaluated with actor principal context.
- Decision envelopes include `source_of_truth`, `policy_ref`, `policy_version`, `lineage_id`, and auth status metadata.
- Replay lineage queries return deterministic digest + evidence for mutation reconstruction.
- Signed-mode staging behavior is test-covered and deterministic.

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
