---
id: 078
name: knowledge-graphs
title: 'Decisions: Knowledge Graphs (021)'
type: general
project: nostra
status: active
authors:
- User
tags: []
created: '2026-02-05'
updated: '2026-03-23'
---

# Decisions: Knowledge Graphs (021)

## DEC-001: Knowledge-System Enrichment Integration
**Date**: 2026-02-05
**Decision**: Integrate knowledge-system enrichment tasks into the current plan.
**Rationale**: Align with GlobalEvent/ResourceRef standards and epistemic integrity goals.
**Status**: APPROVED

## DEC-002: M9 Conditional Accept for `motoko-graph` Staged Migration Contract
**Date**: 2026-02-08
**Decision**: Ratify a **Conditional Accept** posture for `motoko-graph` schema-evolution handling: staged migration (`stageMigrationSnapshot -> upgrade -> applyStagedSnapshot`) is approved for controlled adoption, while direct evolved-module continuity failure remains a known risk.
**Rationale**: M8 demonstrates reproducible restoration after upgrade with explicit migration steps and auditable evidence, but direct continuity remains non-functional without staged restore.
**Status**: APPROVED

## DEC-003: M11 Controlled Pilot Authorization Under Guardrails
**Date**: 2026-02-08
**Decision**: Authorize a controlled pilot posture for `motoko-graph` using the staged migration contract validated in M10. Pilot scope is limited to non-production research paths with explicit rollback triggers and repeated contract checks.
**Rationale**: M10 produced two consecutive clean-lifecycle passes with matching staged-migration invariants, reducing uncertainty enough for a constrained pilot while preserving caution on direct continuity behavior and upstream maintenance risk.
**Status**: APPROVED

## DEC-004: M12 Controlled Pilot Evidence Accepted; Advance to Steward Recommendation Gate
**Date**: 2026-02-08
**Decision**: Accept M12 controlled pilot evidence as complete and valid under guardrails, and advance the initiative to M13 steward recommendation packaging.
**Rationale**: M12 executed required two-pass staged-migration prechecks (both pass with identical invariants) and completed a constrained non-production pilot workload pass with explicit telemetry capture and rollback/escalation documentation.
**Status**: APPROVED

## DEC-005: M13 Steward Recommendation Package Finalized (Watch-First, Conditional M14)
**Date**: 2026-02-08
**Decision**: Finalize steward recommendation package with a watch-first posture: keep `motoko-graph` as a controlled reference candidate, do not promote runtime dependency yet, and allow an optional M14 second pilot only under explicit trigger conditions.
**Rationale**: M10 and M12 provide strong staged-migration and pilot evidence, but unresolved modernization/tooling and upstream-maintenance risks still justify conservative progression under recommendation-only authority.
**Status**: APPROVED

## DEC-006: M13 Closeout Ratified (Watch-First Continuation, Conditional M14 Contract)
**Date**: 2026-02-08
**Decision**: Ratify M13 closeout with reproducibility enrichment and capability variance analysis complete. Maintain `watch-first` posture, keep `authority_mode = recommendation_only`, and keep M14 strictly conditional on explicit trigger criteria.
**Rationale**: Three-run A/B evidence is complete for `motoko-graph` and archive baseline under matched workload envelope, capability/architecture variance is now decision-usable, and residual risks (upstream maintenance, tooling modernization, staged migration procedural dependency) still justify deferred runtime dependency promotion.
**Status**: APPROVED

## DEC-007: M14 Conditional Pilot Executed and Passed (Posture Unchanged)
**Date**: 2026-02-08
**Decision**: Execute conditional M14 second pilot under explicit confidence trigger, with mandatory two-pass staged-migration precheck, and record outcome as `M14_PILOT_PASS`. Maintain `watch-first` posture and keep runtime dependency promotion deferred.
**Rationale**: Trigger condition was explicitly satisfied, prechecks passed with invariant stability, and scoped non-production pilot at increased workload (`180` edges) passed with guardrail compliance. Residual modernization and procedural risks remain open, so decision posture does not change.
**Status**: APPROVED

## DEC-008: Complete M15 Tooling Parity Discovery and Advance to Dual-Path Validation
**Date**: 2026-02-08
**Decision**: Close Phase B (`M15`) with `G1_PARITY_DISCOVERY_COMPLETE`, selecting direct mops replacement as the candidate path for the currently active runtime surface, while keeping fallback wrapper/shim options only for future expanded API restoration.
**Rationale**: Active runtime modules do not currently import `mo:sequence` or `mo:crud`; mops registry lookup shows `sequence`/`crud` unresolved but `base` available; isolated mops probe build passed for the active runtime surface.
**Status**: APPROVED

## DEC-009: Complete M16 Dual-Path Validation with Enrichment; Keep Deferred Promotion Posture
**Date**: 2026-02-08
**Decision**: Close Phase C (`M16`) with `G2_DUAL_PATH_PASS` after dual-path validation (`vessel` baseline vs `mops` candidate) and publish enrichment artifacts (stability, telemetry normalization, risk delta, operability checklist, decision matrix). Keep `watch-first` posture, `authority_mode = recommendation_only`, and defer runtime dependency promotion.
**Rationale**: Both paths passed required build lifecycle, M4 workloads (`120`, `180`), and two consecutive M8 staged migration checks. Candidate path showed minor transient instability (retry usage) but no functional regression or invariant breach.
**Status**: APPROVED

## DEC-010: Publish M17 Steward/Owner Decision Package with Cortex Desktop Capture Guidance
**Date**: 2026-02-08
**Decision**: Publish a formal M17 decision package that operationalizes how to surface M16 evidence (data/workflow/progress) and capture the follow-up steward/owner decision in Cortex Desktop. Keep recommendation posture unchanged (`watch-first`, `recommendation_only`) until explicit steward/owner decision is recorded.
**Rationale**: Technical evidence is complete (`G2_DUAL_PATH_PASS`), and the remaining risk is governance interpretation and controlled decision capture rather than missing runtime data. A deterministic capture workflow reduces ambiguity and prevents partial, unsynchronized decision logging.
**Status**: APPROVED

## DEC-011: Capture M17 Steward/Owner Disposition as Hold Deferred (Watch-First Continues)
**Date**: 2026-02-08
**Decision**: Record final M17 disposition as **Hold Deferred**. Keep `watch-first` posture, keep `authority_mode = recommendation_only`, and do not promote `motoko-graph` as a runtime dependency in this cycle.
**Rationale**: M16 evidence confirms functional no-regression across dual paths, but residual operational and modernization risks remain governance-significant; the current evidence supports continued controlled reference posture rather than dependency promotion.
**Status**: APPROVED

## DEC-012: Apply M19 Decision Event (Hold Deferred)
**Date**: 2026-02-08
**Decision**: Apply captured decision event `kg_decision_20260208_manualseed` with selected option **Hold Deferred**.
**Rationale**: Default hold posture pending explicit steward/owner progression trigger.
**Status**: APPROVED

## DEC-013: Enable M20 Continuous Observability and Decision Analytics
**Date**: 2026-02-08
**Decision**: Approve M20 operational enrichment for `motoko-graph`: add monitoring trend artifacts, gateway trend/run APIs, and Cortex Desktop trend/progress/next-action surfaces while preserving recommendation-only authority and deferred promotion.
**Rationale**: M19 closed a single decision cycle, but recurring operations needed trend-backed visibility and deterministic operator guidance to reduce ambiguity in future steward/owner decisions.
**Status**: APPROVED

## DEC-014: Extend Initiative with Phase D — Schema/Ontology Strategy
**Date**: 2026-03-23
**Decision**: Extend 078-knowledge-graphs with Phase D (M21–M24) covering schema registry consolidation, minimal domain ontology design, knowledge bundle specification, and triple query interface prototyping. Triggered by TrustGraph reference intake (scored 25/35, pattern_value=5).
**Rationale**: TrustGraph analysis surfaced a clear schema vs ontology duality pattern and the Context Core portable knowledge bundle concept. These directly address identified gaps in Nostra's `shared/` commons: scattered JSON schemas, no ontology layer, no knowledge bundle format, and no triple query interface. Phase D scopes this as structured milestones under the existing knowledge graphs initiative rather than a separate initiative.
**Status**: APPROVED

## DEC-015: Keep Phase D Incremental and Nostra-Native
**Date**: 2026-03-23
**Decision**: Deliver Phase D as a staged contract layer over the existing Nostra graph substrate instead of a TrustGraph-style runtime port. Registry consolidation should start as an index over existing schema locations, the ontology layer should remain minimal, and JSON should remain the default interactive protocol while MessagePack is reserved for bulk export/import paths.
**Rationale**: TrustGraph is valuable as a pattern reference, but its Python/Pulsar/Cassandra service mesh is not an ICP-native target. Nostra already has constitutional graph semantics in `shared/specs.md`, so the optimal path is to formalize graph contracts, provenance scopes, and bundle manifests without forcing a platform rewrite.
**Status**: APPROVED
