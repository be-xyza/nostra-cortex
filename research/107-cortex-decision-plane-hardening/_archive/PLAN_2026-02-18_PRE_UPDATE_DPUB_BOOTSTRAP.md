# 107 — Cortex Decision Plane Hardening

stewardship:
  layer: Systems
  primary_steward: Systems Steward
  domain: Agents & Execution

## Objective
Harden Cortex Desktop decision surfaces so canonical canister state is projected first, policy gates are enforced deterministically, and multi-space operator decision paths remain fully legible in a unified inbox.

## Scope
- Additive contract evolution for workflow-engine and governance canisters.
- Canonical-first gateway projections with cache/fallback degradation metadata.
- Deterministic mutation-gate synthesis and lineage references.
- Multi-space decision-plane projections and release-gate evidence surfacing.

## Phases
1. Contracts + declarations sync for replay lineage and gate evaluation.
2. Gateway canister clients + canonical-source projection routing.
3. Unified mutation-gate synthesis (epistemic + governance + replay + test gate).
4. Cortex UI/operator legibility upgrades (source, policy, lineage, active-space routing).
5. Durability and safety hardening for legacy decode and policy persistence.
6. Verification gates and drift checks (`check_did_declaration_sync.sh`).

## Exit Criteria
- All decision surfaces include source-of-truth and degradation metadata.
- Policy block/review semantics are enforced in action endpoints.
- Replay lineage and policy references are surfaced on mutation gate envelopes.
- DID/declaration sync checks pass with zero drift.

## Follow-On
- Phase 8 successor: `/Users/xaoj/ICP/research/108-cortex-decision-plane-security-legibility/PLAN.md`

---

## Cross-Initiative Coordination

> [!WARNING]
> **118 Coordination**: This initiative operates on files targeted for extraction by
> Research 118 (Cortex Runtime Extraction). Do NOT introduce `OnceLock`, `std::fs`,
> `std::env`, `Utc::now()`, or `std::process` in extraction-target files.
> See `research/118-cortex-runtime-extraction/PLAN.md` §File Disposition Matrix.
