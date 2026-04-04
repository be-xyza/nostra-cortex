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
updated: '2026-04-03'
---

# Decisions: Knowledge Graphs (023)

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

## DEC-016: Treat Existing Phase D Artifacts as Draft Baseline; Use JSON-LD as Interoperability Comparator
**Date**: 2026-04-03  
**Decision**: Recognize the existing Phase D commons artifacts on disk as the active draft baseline for Initiative 078: `shared/standards/knowledge_graphs/schema_registry.toml`, `ontology_manifest.schema.json`, `knowledge_bundle.schema.json`, `triple_query_request.schema.json`, `triple_query_response.schema.json`, and `shared/ontology/core_ontology_v1.json`. Treat the custom ontology manifest as the canonical draft contract for now, while using JSON-LD only as an experimental interoperability projection rather than as an automatic replacement.  
**Rationale**: The initiative plan had drifted behind the actual workspace state and still described the ontology layer as absent. Correcting that drift preserves governance clarity. A constrained JSON-LD projection is still valuable for standards comparison and future interoperability work, but making it canonical now would prematurely collapse an open design question that 078 still needs to validate deliberately.  
**Status**: APPROVED


## DEC-017: Earn Phase D v1 Freeze Through Reference-Aligned Validation, Not Draft Incumbency
**Date**: 2026-04-03  
**Decision**: Treat the current ontology, bundle, and triple-query contracts as **leading v1 candidates**, not automatically frozen contracts. Earn the v1 freeze through a reference-aligned validation lane: ontology sufficiency across research/operations/adversarial Space examples, JSON-LD projection parity, SHACL Core-style constraint coverage, bundle portability and round-trip validation, and a read-only internal triple adapter in `cortex-eudaemon`. Keep semantic authority in Nostra and keep executable translation in Cortex.  
**Rationale**: TrustGraph, JSON-LD 1.1, SHACL, Owlish, Horned OWL, and SPARQL 1.1 all support the architectural direction, but none justify blindly accepting the current draft as the best v1 shape just because it already exists on disk. The right move is to validate the draft against those patterns, freeze only what survives, and explicitly keep the Phase D query facade narrower than a full RDF/SPARQL platform.  
**Status**: APPROVED

## DEC-018: Ratify the Phase D Commons Contracts as the Phase E v1 Baseline
**Date**: 2026-04-03  
**Decision**: Ratify the current ontology, bundle, and triple-query contracts as the active **v1 baseline** for the commons graph layer, based on the freeze-readiness evidence artifact and the now-explicit comparator outcomes. The canonical ontology remains the custom JSON manifest; JSON-LD remains derived-only.  
**Rationale**: Phase E closes the earned-freeze lane with machine-readable evidence rather than draft incumbency. The current four core relations, three provenance scopes, bundle semantics, and read-only triple facade all survived the validation lanes without semantic exceptions.  
**Status**: APPROVED

## DEC-019: Complete M21 Through a Root Standards Registry and Keep Explore on a Derived Read Model
**Date**: 2026-04-03  
**Decision**: Complete M21 by introducing a root `shared/standards/standards_registry.toml`, keeping `shared/standards/knowledge_graphs/schema_registry.toml` as a delegated package registry, and adding `schema_guided_extraction_context.json` for extraction-oriented schema lookup. For downstream consumers, publish an internal Explore topology contract as a **derived read model** rather than a new graph authority surface.  
**Rationale**: The next phase needed a governance-complete discovery index without moving file paths or duplicating semantic authority. A delegated registry design preserves existing package-level knowledge-graph governance while making `shared/standards/` discoverable as one governed surface. The Explore lane needs deterministic graph/topology payloads, but that must stay derived from the canonical triple facade.  
**Status**: APPROVED

## DEC-020: Verify Phase E Graph Contracts Through a Slim Library Harness
**Date**: 2026-04-03  
**Decision**: Verify the Phase E graph contract and retrieval lane through a dedicated `knowledge-graph-tests` Cargo feature in `cortex-eudaemon`, and run the graph-specific integration tests with `--no-default-features --features knowledge-graph-tests`. Keep the full daemon binaries and gateway surfaces bound to `service-scaffolds` rather than forcing the graph harness to compile unrelated execution surfaces.  
**Rationale**: The graph contract lane must stay executable, but the broader `cortex-eudaemon` crate currently carries unrelated gateway and agent-run drift. A slim verification harness preserves truthful Phase E evidence for the read-only graph facade, runtime projector, retrieval pilot, and derived Explore topology model without pretending that the rest of the daemon surface is already green.  
**Status**: APPROVED

## DEC-021: Operationalize the Ratified Graph Baseline as an Internal Daemon Service
**Date**: 2026-04-03  
**Decision**: Promote the ratified graph baseline from test-only helpers into a typed internal `cortex-eudaemon` service layer. The service wraps runtime-backed triple projection, read-only triple querying, bounded graph/vector/hybrid retrieval, and derived Explore topology generation. Keep the entire lane internal-only: no public HTTP graph API, no WebSocket graph surface, and no write semantics in this phase.  
**Rationale**: Phase E proved the contracts; Phase F needs an operational execution path that Cortex code can call without widening the public surface or shifting semantic authority away from Nostra. A typed internal Rust service preserves the boundary: Nostra owns ontology, bundle, and query meaning, while Cortex owns execution, projection, retrieval, and derived read models.  
**Status**: APPROVED

## DEC-022: Add a Separate Phase F Daemon Build Gate Alongside the Graph Contract Lane
**Date**: 2026-04-03  
**Decision**: Keep `knowledge-graph-tests` as the canonical slim verification lane for graph contracts, and add a separate `scripts/check_cortex_eudaemon_default_build.sh` gate for the default-feature daemon build. Treat those as complementary gates: one proves graph semantics and evidence flow, the other proves broader daemon health.  
**Rationale**: The graph lane is now operational and should not be hidden inside unrelated daemon drift, but Phase F also cannot claim operational maturity if the default-feature daemon build has no explicit check. Splitting the gates makes failures legible and avoids forcing graph-contract changes when the issue is really broader daemon integration drift.  
**Status**: APPROVED

## DEC-023: Close Initiative 078 Through Bounded Evaluation, Derived Handoff, and Deferred Graph Promotion
**Date**: 2026-04-03  
**Decision**: Close Initiative `078` with a bounded evaluation-and-handoff package rather than expanding graph semantics further in this phase. The closeout requires: a fixed benchmark suite with graph-native and vector-friendly cases, a case-aligned shared evaluation report against the current `037` knowledge path, deterministic Explore topology handoff validation for both research-space and agent-scope projections, and a written graph-promotion policy that defers new graph-native evidence classes until repeated benchmark and consumer evidence justify promotion.  
**Rationale**: The remaining work in `078` was no longer about proving the commons contracts or the internal runtime lane. The real risk had shifted to overfitting the graph model to a single benchmark gap. Closing the initiative through stronger evaluation, clearer consumer handoff, and explicit promotion criteria preserves the Nostra/Cortex boundary: Nostra keeps semantic authority, Cortex proves execution value, and future graph expansion stays stewarded rather than opportunistic.  
**Status**: APPROVED
