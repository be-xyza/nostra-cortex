---
id: "097-nostra-cortex-alignment-decisions"
name: "nostra-cortex-alignment-decisions"
title: "Decision Log: Nostra/Cortex Alignment Remediation"
type: "decision"
project: "nostra"
status: active
authors:
  - "User"
  - "Codex"
tags: [alignment, remediation]
created: "2026-02-03"
updated: "2026-02-08"
---

# Decision Log: Nostra/Cortex Alignment Remediation

Track architectural and governance decisions made during alignment work.

---

## DEC-001: Create a New Alignment Initiative
**Date**: 2026-02-03
**Status**: ✅ Decided

**Options Considered**:
1. Extend `research/046-nostra-system-standards/`
2. Create a new initiative for alignment remediation

**Decision**: Create `research/097-nostra-cortex-alignment/`.

**Rationale**: Keeps remediation work explicit, auditable, and separable from standards evolution.

**Implications**: All remediation tracking, compliance artifacts, and decisions live in 097.

---

## DEC-002: Canonical Workflow Engine Path
**Date**: 2026-02-03
**Status**: ✅ Decided

**Options Considered**:
1. `nostra/backend/workflow_engine/`
2. `src/nostra_workflow_engine/`
3. Maintain both as co-equal

**Decision**: Canonical path is `nostra/backend/workflow_engine/`.

**Rationale**: Aligns with canister packaging and `nostra/dfx.json` production configuration.

**Implications**: Prototype paths must be marked as non-canonical or deprecated in docs.

---

## DEC-003: Status Source of Truth
**Date**: 2026-02-03
**Status**: ✅ Decided

**Options Considered**:
1. `PLAN.md` overrides status index
2. `RESEARCH_INITIATIVES_STATUS.md` overrides plans

**Decision**: `PLAN.md` is the source of truth.

**Rationale**: Plans encode initiative intent and current state, while the index is derivative.

**Implications**: Status index must be synchronized to plan headers/notes.

---

## DEC-004: Alignment Addendum Format
**Date**: 2026-02-03
**Status**: ✅ Decided

**Options Considered**:
1. Minimal checkboxes
2. Short narrative per principle

**Decision**: Add a short narrative per principle.

**Rationale**: Narrative preserves context and avoids shallow compliance signaling.

**Implications**: Each `PLAN.md` receives an appended alignment addendum.

---

## DEC-005: Type Contract Source of Truth
**Date**: 2026-02-03
**Status**: ✅ Decided

**Options Considered**:
1. Keep legacy reference to `backend/types.mo`
2. Treat Motoko `types.mo` as canonical for all interfaces
3. Treat Candid `.did` as the canonical public contract

**Decision**: Candid `.did` files are the source of truth for public interfaces. Motoko domain types and Rust bindings must align to those contracts.

**Rationale**: Interfaces span Motoko and Rust; Candid is the shared, language-neutral contract and prevents drift between canisters and frontend bindings.

**Implications**: Update `AGENTS.md` guidance; refresh `nostra/src/declarations/**` and `nostra/frontend/src/types.rs` when `.did` definitions change.

---

## DEC-006: Security Audit Findings (RSA Marvin)
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Block builds until `rsa` has a fixed release
2. Accept risk with documentation and monitoring
3. Replace RSA usage immediately across affected crates

**Decision**: Accept risk with documentation and monitoring; plan a targeted migration off RSA where feasible.

**Rationale**: No upstream fix is available; blocking builds would halt delivery. The affected scope is limited to `frontend` and `cortex_worker`, and we can plan a migration path without halting progress.

**Implications**: Track in alignment research; schedule evaluation of RSA replacement or alternative crypto primitives in subsequent initiatives.

---

## DEC-006A: Snapshot Manifest + Docs Bundle Contract
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Ad hoc bundles per tool
2. Standardized `SnapshotManifest` + `DocsBundle` contract

**Decision**: Standardize a `SnapshotManifest` and `DocsBundle` contract for all published knowledge snapshots.

**Rationale**: Cortex and agents need deterministic, verifiable snapshots that can be pinned and reloaded.

**Implications**: `080-dpub-standard`, `057-development-brain`, and `036-project-guide-integration` must reference the shared contract.

---

## DEC-007: One-Way Sync Policy Until Merge Rules
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Bidirectional edits immediately
2. One-way export (local → DPub) until merge policy is approved

**Decision**: Enforce one-way export until workflow-engine merge rules are formalized.

**Rationale**: Prevents schema drift and reconciliation errors while the canonical workflow engine is still in Phase 1.

**Implications**: Native authoring features remain gated; read-only consumption is allowed.

---

## DEC-008: Minimal Workflow Subset for Flow Graph MVP
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Support the full workflow DSL from day one
2. Define a minimal subset for MVP and expand later

**Decision**: Define a minimal subset for MVP: `api`, `event`, `cron`, `noop`, `agent` (if available), `worker` (if available).

**Rationale**: Enables deterministic graph derivation and UI delivery without blocking on the full DSL.

**Implications**: Non-core step types are omitted from MVP visualization until their contracts stabilize.

---

## DEC-009: Lineage Edges Included in Flow Graph
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Emit/subscribe edges only
2. Emit/subscribe edges + Nostra contribution lineage edges

**Decision**: Include lineage edges in the derived flow graph as a distinct `lineage` variant.

**Rationale**: Aligns execution with knowledge history and makes “execution creates knowledge” visible.

**Implications**: Graph derivation must join workflow steps with Nostra contribution lineage data.

---

## DEC-010: Layout Storage in Nostra with Cortex Cache
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Store layout only in Cortex
2. Store layout only in Nostra
3. Store layout in Nostra and cache in Cortex

**Decision**: Store layout as a Nostra `FlowLayout` contribution and cache in Cortex for performance.

**Rationale**: Preserves history and governance while keeping UI responsive.

**Implications**: Layout changes are auditable and versioned; Cortex must implement cache invalidation on layout updates.

---

## DEC-011: Role Semantics Doctrine + Deprecate “Mayor”
**Date**: 2026-02-06
**Status**: ✅ Decided

**Options Considered**:
1. Keep “Mayor” as the canonical coordinator title
2. Treat “Mayor” as a purely cosmetic persona and document disclaimers
3. Deprecate civic/sovereign metaphors and formalize a role semantics doctrine (layer-true titles)

**Decision**: Deprecate “Mayor” in canonical Nostra/Cortex docs and protocols; adopt a Role Semantics Doctrine and prefer `Steward`/`Maintainer` (Nostra legitimacy) and `Operator`/`Worker` (Cortex execution).

**Rationale**: “Mayor” implies political agency and discretionary authority, which becomes future-hostile as agents increasingly submit proposals, execute workflows, and enforce rules under procedural, auditable, forkable governance.

**Implications**:
- Canonical plans/specs should avoid introducing new `mayor`-named roles/persona IDs.
- Legacy references may remain only for historical mapping (e.g., Gastown terminology), with explicit deprecation notes.
- UI surfaces should use layer-true names (e.g., `Steward Console`, `Operator Console`) and reserve `Orchestrator`/`Intention Compiler` for internal module naming.

## DEC-011A: Cross-Initiative Resolution Matrix + Governance Gates
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Resolve alignment ad hoc per initiative
2. Maintain a single resolution matrix with explicit gates

**Decision**: Maintain `RESOLUTION_MATRIX.md` for all initiatives and treat cross-initiative changes as gated actions with explicit decisions.

**Rationale**: Alignment is only durable if every initiative is tracked against the same standards and any exceptions are visible and accountable.

**Implications**: Alignment completion requires a resolved matrix, logged decisions, and recommendation-only handling when authority is unclear.

## DEC-011B: Append-Only Layout History with Retention Cap
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Store only the latest layout per workflow + graph version
2. Keep full history indefinitely
3. Keep append-only history with a fixed retention cap

**Decision**: Persist append-only layout history per workflow + graph version with a fixed retention cap (25 entries).

**Rationale**: Maintains change traceability while limiting storage growth in the backend canister.

**Implications**: Workbench can show recent layout edits and export delta reports without unbounded storage.

---

## DEC-012: Pin dfx to 0.29.2 to Avoid Color Panic
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Stay on `dfx` 0.30.x and accept build panics
2. Pin to the latest working 0.29.x release
3. Build from source or patch `dfx` locally

**Decision**: Pin `dfx` to 0.29.2 (via `dfxvm`) to restore `dfx build`.

**Rationale**: 0.30.x panics with `ColorOutOfRange` when setting stderr color in this environment. 0.29.2 builds successfully without the panic.

**Implications**: Use `dfxvm default 0.29.2` for local builds until 0.30.x fixes the issue.

---

## DEC-013: Inline Layout Preview Stats in Workbench History
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Show preview stats only in the header summary
2. Show preview stats inline per history item

**Decision**: Show preview stats inline for the selected history entry (while keeping the header summary).

**Rationale**: Inline context reduces cognitive load when scanning multiple layout revisions.

**Implications**: Workbench stores a lightweight preview key (timestamp) alongside the preview summary string.

---

## DEC-014: Placeholder Plans for Missing PLAN.md Initiatives
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Leave initiatives without `PLAN.md` and track as unresolved
2. Create minimal placeholder plans with alignment addenda

**Decision**: Create minimal placeholder `PLAN.md` files for initiatives missing plans, labeled as draft and research-only pending scope confirmation.

**Rationale**: Ensures constitutional alignment coverage without assuming implementation scope, while preserving a clear path to formal planning once ownership and scope are defined.

**Implications**: Resolution matrix marks these initiatives as `plan-stub` with scope pending steward confirmation; follow-up required to confirm authority and update statuses.

---

## DEC-015: Default Steward Assignment for Scoping Initiatives
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Defer steward assignment until each initiative is reviewed
2. Assign a default steward to unblock scoping alignment

**Decision**: Assign **Nostra Team** as the default steward for all scoping initiatives until explicit owners are confirmed.

**Rationale**: Enables consistent governance while keeping scope draft and recommendation-only.

**Implications**: Status labels updated to “Draft (Scoping; Steward Confirmed)” and plans note steward confirmation pending owner approval.

---

## DEC-016: Tri-Axial Stewardship Metadata in PLAN Frontmatter
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Document stewardship only in plan body text
2. Store stewardship metadata in `PLAN.md` frontmatter (machine-readable)
3. Create a separate registry outside the plans

**Decision**: Require a `stewardship` block in `PLAN.md` frontmatter, using tri-axial taxonomy (Layer, Steward Role, Domain).

**Rationale**: Makes ownership legible to humans and agents, supports automation, and preserves constitutional boundaries without changing plan intent.

**Implications**: Standards updated to include enums; validation and stewardship matrix scripts added.

---

## DEC-017: Heuristic Stewardship Backfill for Legacy Plans
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Leave legacy plans without stewardship until manual classification
2. Apply a heuristic, keyword-based backfill to unblock validation and then review

**Decision**: Apply a deterministic, keyword-based backfill for stewardship metadata across legacy plans, with follow-up review as needed.

**Rationale**: Enables immediate conformance to stewardship metadata requirements while preserving a review path for nuanced classification.

**Implications**: Stewardship assignments for non-scoping plans should be reviewed and adjusted by stewards as initiatives progress.

---

## DEC-018: Stewardship Review Checklist
**Date**: 2026-02-06
**Status**: ✅ Decided

**Options Considered**:
1. Rely on informal stewardship judgment
2. Add a lightweight checklist to standards

**Decision**: Add a lightweight stewardship review checklist to `docs/architecture/standards.md`.

**Rationale**: Ensures consistency and truthfulness in stewardship metadata without adding process overhead.

**Implications**: Future stewardship edits should reference the checklist and archive before update.

---

## DEC-019: Resolution Matrix Generated from Repo Truth
**Date**: 2026-02-06
**Status**: ✅ Decided

**Options Considered**:
1. Maintain `RESOLUTION_MATRIX.md` by manual edits
2. Generate `RESOLUTION_MATRIX.md` from initiative directories, plan/spec presence, and status index

**Decision**: Add `scripts/generate_resolution_matrix.py` and regenerate `RESOLUTION_MATRIX.md` from repository state.

**Rationale**: Manual updates drifted (`101` listed while `102` initiative directories existed). Generation makes matrix totals auditable and repeatable.

**Implications**:
- Resolution summary now reflects 102 initiatives and detects duplicate ID prefixes (`021` appears twice).
- Duplicate numeric prefix is treated as a governance/data-quality issue; recommendation-only action is to renumber one initiative prefix under explicit steward approval.

**Follow-up**: Resolved by DEC-021 (initiative renumbered to `078-knowledge-graphs`).

---

## DEC-020: Status Index Reconciled to PLAN Truth
**Date**: 2026-02-06
**Status**: ✅ Decided

**Options Considered**:
1. Keep status index at 101 and accept missing row drift
2. Reconcile index/table to include all initiatives and current `PLAN.md` statuses

**Decision**: Update `research/RESEARCH_INITIATIVES_STATUS.md` to 102 items and add the missing `100-bible-native-dpub-corpus` row (`draft`), then regenerate downstream matrices.

**Rationale**: Alignment policy sets `PLAN.md` as source of truth; index and matrix outputs must match discovered initiative inventory.

**Implications**: Status/reporting artifacts now align on total counts and avoid silent omissions.

---

## DEC-021: Renumber Duplicate Knowledge Graphs Initiative to 078
**Date**: 2026-02-06
**Status**: ✅ Decided

**Options Considered**:
1. Keep both initiatives on `021` and carry duplicate-prefix exceptions indefinitely
2. Renumber the duplicate Knowledge Graphs initiative to the next available slot and update all references

**Decision**: Renumber the duplicate Knowledge Graphs initiative to `078-knowledge-graphs` and update links, status rows, and generated matrices.

**Rationale**: Removes identifier ambiguity and keeps initiative indexing machine-addressable without scope or content changes.

**Implications**:
- Duplicate-prefix condition is cleared in `RESOLUTION_MATRIX.md`.
- Cross-initiative links now target `research/078-knowledge-graphs/`.

---

## DEC-022: dPub Snapshot Contract Closeout (Dual-file + JSON-canonical feed)
**Date**: 2026-02-06
**Status**: ✅ Decided

**Options Considered**:
1. Keep SnapshotManifest mapped to `snapshot.json`
2. Split payload and contract into separate files
3. Make RSS/Atom the canonical feed output

**Decision**: Adopt dual-file snapshot semantics (`snapshot.json` payload + `snapshot_manifest.json` contract) and keep feed source-of-truth as canonical JSON (`feed.json`/JSON query output) with RSS/Atom as derived views only.

**Rationale**: Eliminates naming drift between plans/specs/runtime, preserves backward compatibility for existing readers, and provides deterministic contract verification for Cortex/Desktop workflows.

**Implications**:
- `publish_dpub_edition_v2` is added without removing v1.
- v1 writes `snapshot_manifest.json` with fallback `commit_hash = \"unknown-local\"`.
- Alignment scripts enforce no active plan/spec maps SnapshotManifest to `snapshot.json` or treats XML as feed source-of-truth.

---

## DEC-023: Reject Program-Level Merge for Now; Keep Tracking Active
**Date**: 2026-02-06
**Status**: ✅ Decided

**Options Considered**:
1. Execute program-level merge recommendations immediately
2. Reject program-level merges for now while retaining portfolio tracking data

**Decision**: Reject program-level merges at this time. Keep all program bundles and steward queue signals as tracking-only.

**Rationale**: Preserves visibility and prioritization data without forcing structural consolidation before steward-ready timing.

**Implications**:
- `research/STEWARD_REVIEW_QUEUE.md` program rows are tracking-only (merge rejected).
- `research/programs/*.md` remain active as portfolio tracking indices.
- Merge policy can be revisited later via explicit steward approval.

---

## DEC-024: Enforce Portfolio Consistency via Automated Check + CI
**Date**: 2026-02-06
**Status**: ✅ Decided

**Options Considered**:
1. Keep portfolio consistency as manual stewardship practice only
2. Add automated consistency checks and CI gate for research portfolio artifacts

**Decision**: Add automated portfolio consistency validation (`scripts/check_research_portfolio_consistency.py`) and enforce it in CI (`.github/workflows/research-portfolio.yml`).

**Rationale**: Prevents drift across `PLAN.md` frontmatter, status index, program tracking docs, and steward queue policy.

**Implications**:
- Invalid/missing `status` or `portfolio_role` is now a detectable regression.
- `RESEARCH_INITIATIVES_STATUS.md` and `current_folders.txt` must stay synchronized with actual initiative directories.
- Queue policy now hard-fails if `merge-review` appears while merge-rejected policy is active.

---

## DEC-025: Closeout Verification Requirement for Reference Initiatives
**Date**: 2026-02-06
**Status**: ✅ Decided

**Options Considered**:
1. Keep reference initiatives without explicit verification artifacts
2. Require `VERIFY.md` for reference initiatives (completed/superseded lineage)

**Decision**: Require `VERIFY.md` on reference initiatives; backfilled closeout verification docs for current reference set.

**Rationale**: Gives a concrete closure trail for human and agent navigation and reduces ambiguous "complete but unverified" states.

**Implications**:
- Steward queue no longer raises high-priority closeout actions for reference initiatives with `VERIFY.md`.
- Consistency checker enforces `VERIFY.md` presence for `portfolio_role: reference`.

---

## DEC-026: Reference Workspace Is a Research Extension (Constitution-Gated)
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Keep `reference/` as a separate top-level workspace indefinitely
2. Treat reference as a canonical extension under `research/` with staged migration

**Decision**: Canonicalize the reference workspace under `research/reference` and treat it as part of the research system, with one-cycle compatibility support for `/reference`.

**Rationale**:
- Aligns evidence repositories with the research pipeline they inform.
- Reduces semantic drift between initiative decisions and supporting evidence.
- Keeps constitutional controls explicit during a sensitive scope-change action.

**Implications**:
- Canonical root becomes `/Users/xaoj/ICP/research/reference`.
- Legacy path `/Users/xaoj/ICP/reference` remains temporary compatibility only.
- Policy/docs must use recommendation-only defaults and steward escalation for structural actions.

---

## DEC-027: Sensitive Action Classification and Transition Authority Mode
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Execute relocation as ordinary maintenance
2. Classify relocation as `rename/scope-change` with constitutional escalation controls

**Decision**: Classify the relocation as a sensitive structural action (`rename/scope-change`) and enforce `recommendation_only` transition mode until steward closeout.

**Rationale**:
- Root relocation changes operational boundaries and lineage interpretation.
- Constitutional doctrine requires explicit escalation and audibility for irreversible or high-impact structural changes.

**Implications**:
- Every migration stage must be logged in lineage artifacts.
- Compatibility closure requires steward-reviewed decision logging.

---

## DEC-028: Reference Contract Normalization and Advisory Enforcement
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Keep informal reference intake metadata and manual review
2. Normalize metadata contract and add advisory checker before hard CI gating

**Decision**: Normalize reference intake contract and add advisory enforcement for constitutional metadata fields before any hard CI fail policy.

**Rationale**:
- Improves authority clarity (`primary_steward`, `authority_mode`, `escalation_path`) and lineage integrity (`lineage_record`, `initiative_refs`).
- Supports phased rollout without blocking active work.

**Implications**:
- `docs/reference/README.md` defines the canonical contract and key names.
- New advisory checker flags missing stewardship/escalation/linkage fields.
- Hard CI enforcement is deferred until a future steward-approved cycle.

---

## DEC-029: Reference Compatibility Closure (Legacy Alias Retired)
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Keep `/Users/xaoj/ICP/reference` compatibility alias beyond the initial transition cycle
2. Retire compatibility alias after dependency elimination and preserve lineage in governance records

**Decision**: Retire `/Users/xaoj/ICP/reference` compatibility alias and keep `/Users/xaoj/ICP/research/reference` as the sole canonical root.

**Rationale**:
- First-party governance and research docs were migrated to canonical paths.
- Continued dual-path support increases drift risk and weakens semantic authority boundaries.
- Closure criteria in `research/REFERENCE_MIGRATION_LINEAGE.md` were satisfied.

**Implications**:
- Canonical references must use `research/reference/...`.
- Historical mapping and rationale remain preserved in lineage and decision logs.
- Advisory checker remains active for analysis metadata backlog, but path authority is now single-root.

---

## DEC-030: Approve ACP Intake Promotion to `agent-systems` and Link to Initiative 103
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Keep ACP in `research/reference/inbox` as pending intake
2. Promote ACP to `research/reference/topics/agent-systems` and update catalog/analysis links

**Decision**: Promote ACP to `research/reference/topics/agent-systems/agent-client-protocol` and treat intake as resolved.

**Rationale**:
- User explicitly approved the sensitive structural move.
- ACP scored high on ecosystem fit and adapter value for Cortex interoperability.
- Initiative `103-agent-client-protocol-alignment` now contains a concrete ACP-to-Nostra event mapping artifact for controlled pilot gating.

**Implications**:
- `research/reference/index.toml`, `research/reference/index.md`, and `research/reference/analysis/agent-client-protocol.md` must reference the topic path.
- Future ACP work should proceed under initiative 103 with recommendation-only execution until implementation approval is granted.

---

## DEC-031: Approve `motoko-graph` Promotion to `data-knowledge` and M2 Hardening
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Keep `motoko-graph` in `research/reference/inbox` pending more validation
2. Promote to `research/reference/topics/data-knowledge`, complete M2 hardening, and retain watch status

**Decision**: Promote `motoko-graph` to `research/reference/topics/data-knowledge/motoko-graph`, apply M2 compatibility hardening, and keep adoption status as watch (not a runtime dependency yet).

**Rationale**:
- User explicitly approved proceeding with all next steps, including sensitive structural promotion.
- M1 baseline validation passed after environment and tooling remediation.
- M2 hardening removed known compiler and script fragility without expanding public graph APIs.

**Implications**:
- `research/reference/index.toml`, `research/reference/index.md`, and `research/reference/analysis/motoko-graph.md` must reference the promoted topic path.
- `research/reference/topics/data-knowledge/motoko-graph/test.sh` now enforces reproducible environment constraints (`TERM=xterm-256color`, vessel `0.4.x`).
- Next adoption gate is M3 behavior/scale hardening evidence; authority mode remains `recommendation_only`.

---

## DEC-032: Close M3 Baseline for `motoko-graph` and Keep Watch Status
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Treat M3 baseline pass as sufficient for runtime dependency adoption
2. Mark M3 complete but keep watch status until graph-operation validation (M4)

**Decision**: Mark M3 as passed and keep `motoko-graph` in watch status; defer dependency adoption until M4 graph-operation and scale evidence is complete.

**Rationale**:
- M3 confirms reproducible runtime harness behavior and captures memory/cycle telemetry.
- Current canister behavior still validates loop/scaffold behavior, not full graph operation semantics (`createNode`, `createEdge`, `walk`).
- Adoption without operation-level evidence creates avoidable integration risk.

**Implications**:
- `research/reference/analysis/motoko-graph.md` records M3 telemetry and `M3_PASS`.
- `research/reference/index.toml` next-experiment note now points to M4 operation-level validation.
- Authority mode remains `recommendation_only`; no runtime dependency promotion yet.

---

## DEC-033: Close M4 Graph-Operation Validation and Keep Watch Status Pending M5
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Promote `motoko-graph` immediately to dependency-candidate after M4 pass
2. Record M4 pass, keep watch status, and gate adoption on upgrade/migration evidence (M5)

**Decision**: Record `M4_PASS` and keep `motoko-graph` in watch status. Defer dependency adoption until M5 validates stable-memory upgrade behavior and workflow modernization feasibility.

**Rationale**:
- M4 now demonstrates reproducible operation-level behavior (`createNode`, `createEdge`, `readNode`, `readEdge`, `walk`) with telemetry under workload.
- Remaining integration risk is lifecycle-oriented (upgrade safety and long-term tooling sustainability), not basic operation correctness.
- Recommendation-only authority mode requires explicit evidence and staged gates before runtime dependency promotion.

**Implications**:
- `research/reference/analysis/motoko-graph.md` records M4 command outcomes, metrics, and `M4_PASS`.
- `research/reference/index.toml` next-experiment guidance advances to M5 (`stable-memory`/migration gate).
- Authority mode remains `recommendation_only`; runtime adoption decision is deferred.

---

## DEC-034: Close M5 Upgrade Validation; Defer Adoption Pending M6 Scale Envelope
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Promote `motoko-graph` to dependency-candidate immediately after M5 upgrade continuity pass
2. Record M5 pass, keep watch status, and require M6 scale-envelope + migration strategy closure before adoption

**Decision**: Record `M5_PASS`, keep `motoko-graph` in watch status, and defer adoption until M6 defines scale thresholds and migration strategy decisions.

**Rationale**:
- M5 upgrade-path checks passed: seeded graph state persisted through `upgrade`, and reset behavior under `reinstall` was confirmed.
- Migration feasibility probing showed direct `mops` substitution is not immediate (`sequence` and `crud` not found in registry lookup), so workflow modernization needs explicit design.
- Recommendation-only authority mode favors staged evidence closure before runtime dependency promotion.

**Implications**:
- `research/reference/analysis/motoko-graph.md` records M5 metrics, continuity results, and migration-feasibility findings.
- `research/reference/index.toml` now advances next-experiment guidance to M6 (scale envelope + migration strategy).
- Authority mode remains `recommendation_only`; runtime adoption decision remains deferred.

---

## DEC-035: Close M6 Scale Envelope; Defer Adoption Pending M7 Policy Closeout
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Promote `motoko-graph` to dependency-candidate after M6 linear scale results
2. Record M6 pass, keep watch status, and require explicit M7 dependency policy + schema-evolution evidence

**Decision**: Record `M6_PASS`, keep watch status, and defer adoption until M7 closes policy thresholds and schema-evolution upgrade evidence.

**Rationale**:
- M6 profiling across 120/180/240 edges showed stable near-linear behavior with consistent per-edge costs.
- Upgrade continuity is validated (M5), but schema-evolution migration safety is not yet demonstrated.
- Tooling modernization remains unresolved; direct `mops` substitution is still blocked by missing legacy dependencies in registry search.

**Implications**:
- `research/reference/analysis/motoko-graph.md` now includes M6 tier metrics and slope summaries.
- `research/reference/index.toml` advances suggested next experiments to M7 policy closeout.
- Authority mode remains `recommendation_only`; runtime adoption decision remains deferred.

---

## DEC-036: Mark M7 Schema-Evolution Gate Blocked; Require M8 Migration Remediation
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Accept M7 as passed based on policy threshold availability and partial upgrade checks
2. Mark M7 blocked because evolved-module upgrade continuity failed, and require migration remediation before adoption

**Decision**: Mark M7 as `BLOCKED`. Keep `motoko-graph` in watch status and require M8 migration-path remediation followed by M7 revalidation.

**Rationale**:
- Evolved-module upgrade evidence showed continuity failure (`edgeCount` reset from seeded value to `0` after upgrade).
- Policy thresholds from M6 are available, but they are insufficient without upgrade continuity under schema/interface evolution.
- Local runtime instability during repeated scripted runs is a secondary environment issue; the primary blocker is continuity failure under successful upgrade.

**Implications**:
- `research/reference/analysis/motoko-graph.md` records `M7_BLOCKED` with exact reproduction evidence.
- `research/reference/index.toml` advances next-step guidance to M8 remediation and M7 re-run.
- Authority mode remains `recommendation_only`; runtime dependency adoption remains blocked.

---

## DEC-037: Accept M8 Migration Remediation Evidence; Keep Adoption Deferred Pending M9 Policy
**Date**: 2026-02-07
**Status**: ✅ Decided

**Options Considered**:
1. Keep M7 blocked without remediation path and halt progress
2. Implement and accept explicit staged migration remediation evidence (M8), then require policy ratification before adoption

**Decision**: Accept `M8_PASS` as valid remediation evidence for schema-evolution migration path, while keeping runtime dependency adoption deferred until M9 policy ratification and M7 revalidation criteria are finalized.

**Rationale**:
- M8 demonstrates reproducible recovery after evolved-module upgrade by persisting staged snapshot metadata and restoring graph state post-upgrade.
- Direct continuity remains broken (`M8_DIRECT_POST_UPGRADE_EDGE_COUNT=0`), so remediation is procedural rather than implicit.
- Dependency adoption requires explicit policy agreement on whether this staged migration contract is acceptable for initiative `078`.

**Implications**:
- `research/reference/analysis/motoko-graph.md` records M8 metrics and remediation behavior.
- `research/reference/index.toml` advances next-step guidance to M9 policy ratification and M7 criteria revalidation.
- Authority mode remains `recommendation_only`; no runtime dependency promotion yet.

---

## DEC-038: Record M9 Conditional Accept Ratification and Defer Adoption Pending Revalidation Evidence
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Keep `motoko-graph` blocked until direct continuity works without staged restore
2. Accept staged migration as a controlled schema-evolution contract and defer adoption until contract-conformant revalidation evidence is stable

**Decision**: Adopt option 2. M9 is ratified as **Conditional Accept** (see `DEC-002` in `/Users/xaoj/ICP/research/078-knowledge-graphs/DECISIONS.md`), while runtime adoption remains deferred pending contract-conformant M7 revalidation evidence.

**Rationale**:
- M8 demonstrates reproducible upgrade recovery using staged snapshot persistence + restore.
- Direct post-upgrade continuity remains non-functional without staged restore.
- Recommendation-only authority requires explicit stability evidence before dependency promotion.

**Implications**:
- `research/reference/analysis/motoko-graph.md` must record `M9_CONDITIONAL_ACCEPT` and explicit M7 contract gates.
- `research/reference/index.toml` must point next steps to M10 (contract-conformant M7 revalidation with policy checklist evidence).
- No dependency promotion occurs in this stage.

---

## DEC-039: Approve Test Catalog Initiative Placement and Filesystem-Canonical v1
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Keep test evidence fragmented across local tool outputs
2. Introduce canister-canonical test storage immediately
3. Create dedicated initiative with filesystem-canonical contract and Cortex projection

**Decision**: Create `research/105-cortex-test-catalog` and adopt filesystem-canonical test artifacts for v1 (`logs/testing/*`) with read-only Cortex Desktop projection.

**Rationale**:
- Fastest path to unify local IDE agent execution evidence, CI checks, and Cortex visibility.
- Aligns with existing machine-readable closeout artifact patterns used in knowledge-engine hardening.
- Reduces migration risk by deferring canister-canonical persistence to a later phase.

**Implications**:
- `shared/standards/testing/*` becomes the contract baseline for local IDE agents.
- Gateway/API additions in Cortex Desktop must expose catalog, runs, gates, and health as read-only views.
- CI policy moves advisory-first, then blocking after the stabilization window.

---

## DEC-040: Record M11 Controlled Pilot Authorization for `motoko-graph` (Adoption Still Deferred)
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Keep watch-only status after M10 and defer any pilot authorization
2. Authorize controlled pilot posture under strict guardrails while keeping dependency promotion deferred

**Decision**: Adopt option 2. M11 controlled pilot posture is authorized (see `DEC-003` in `/Users/xaoj/ICP/research/078-knowledge-graphs/DECISIONS.md`) with recommendation-only authority and explicit rollback criteria.

**Rationale**:
- M10 provided two consecutive clean-lifecycle passes with identical staged migration invariants.
- Residual risks remain (procedural dependency, upstream maintenance stagnation), so broad adoption is still not approved.
- Controlled pilot keeps progress moving while preserving governance guardrails.

**Implications**:
- `research/reference/analysis/motoko-graph.md` advances to `M11_CONTROLLED_PILOT_APPROVED` and points next gate to M12 pilot execution evidence.
- `research/reference/index.toml` suggested next experiments advance from M11 decision to M12 pilot execution under staged migration regression checks.
- No runtime dependency promotion is approved in this stage.

---

## DEC-041: Record M12 Pilot Evidence Pass; Advance to M13 Steward Recommendation
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Keep M12 open and request more pilot evidence before closing
2. Accept current M12 evidence and move to steward recommendation packaging

**Decision**: Adopt option 2. M12 evidence is accepted as complete for this stage (see `DEC-004` in `/Users/xaoj/ICP/research/078-knowledge-graphs/DECISIONS.md`), and the next gate is M13 steward recommendation.

**Rationale**:
- Required pre-run two-pass staged-migration checks were executed and passed with identical invariants.
- Controlled non-production pilot workload completed with PASS and captured telemetry.
- Guardrail posture remains intact: recommendation-only authority and no dependency promotion.

**Implications**:
- `research/reference/analysis/motoko-graph.md` advances to `M12_PILOT_PASS` with `M13_STEWARD_RECOMMENDATION_REQUIRED`.
- `research/reference/index.toml` advances suggested next experiments to M13 recommendation packaging.
- Runtime dependency adoption remains deferred.

---

## DEC-042: Record M13 Steward Recommendation Package (Watch-First, Conditional M14)
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Recommend immediate dependency progression after M12 pilot success
2. Recommend watch-first posture with optional M14 second pilot under explicit triggers

**Decision**: Adopt option 2. M13 recommendation package is finalized (see `DEC-005` in `/Users/xaoj/ICP/research/078-knowledge-graphs/DECISIONS.md`) with watch-first posture and conditional M14 path.

**Rationale**:
- M10 and M12 evidence establish strong controlled-adoption viability.
- Residual modernization and upstream-maintenance risks are still open.
- Recommendation-only authority and constitutional guardrails favor conservative progression.

**Implications**:
- `research/reference/analysis/motoko-graph.md` advances to `M13_RECOMMENDATION_READY`.
- `research/reference/index.toml` advances suggested next experiments to conditional M14 or continued watch-only monitoring.
- No runtime dependency promotion is authorized in this stage.

---

## DEC-043: Record M13 Closeout Completion and Watch-First Continuation Contract
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Trigger M14 immediately after M13 package completion
2. Complete closeout enrichment (A/B reproducibility + capability variance + modernization plan), keep watch-first default, and trigger M14 only if explicit criteria are met

**Decision**: Adopt option 2. M13 closeout is complete and synchronized with initiative `078` (`DEC-006`), while runtime dependency adoption remains deferred under recommendation-only authority.

**Rationale**:
- Reproducibility evidence now includes three-run A/B matrix with aggregate statistics under a matched workload envelope.
- Capability differences between `motoko-graph` and original implementation are explicitly documented with adoption implications.
- Residual risks remain open (tooling modernization and procedural migration dependency), so automatic M14 execution is not justified.

**Implications**:
- `research/reference/analysis/motoko-graph.md` advances to closeout-complete watch-first state with conditional M14 triggers.
- `research/reference/index.toml` suggested next experiments move to post-closeout monitoring + trigger-based M14.
- No runtime dependency promotion is approved in this stage.

---

## DEC-044: Record M14 Conditional Pilot Pass and Preserve Watch-First Posture
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Keep M14 unexecuted after M13 closeout
2. Execute M14 under explicit confidence trigger with precheck contract and scoped pilot

**Decision**: Adopt option 2. M14 was executed under the conditional contract and passed (`M14_PILOT_PASS`), while overall posture remains watch-first and recommendation-only (mirrors `DEC-007` in `/Users/xaoj/ICP/research/078-knowledge-graphs/DECISIONS.md`).

**Rationale**:
- Explicit trigger condition for additional confidence evidence was present.
- Two-pass staged-migration precheck passed before pilot execution.
- Scoped non-production pilot passed at higher workload without guardrail violation.

**Implications**:
- `research/reference/analysis/motoko-graph.md` advances with M14 evidence and unchanged deferred-promotion posture.
- `research/reference/index.toml` advances next-step language to post-M14 monitoring + modernization milestone reassessment.
- No runtime dependency promotion is authorized in this stage.

---

## DEC-045: Record M15 Tooling Parity Discovery Completion and Phase C Entry
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Keep vessel-only path and defer parity discovery work
2. Complete parity discovery now and define a constrained Phase C dual-path validation contract

**Decision**: Adopt option 2. M15 parity discovery is complete (mirrors `DEC-008` in `/Users/xaoj/ICP/research/078-knowledge-graphs/DECISIONS.md`) and Phase C dual-path validation is now the next recommendation-only step.

**Rationale**:
- Active runtime modules currently depend only on local + `mo:base/*` imports.
- `mops` registry availability is sufficient for active `base` path, while legacy `sequence`/`crud` remain unresolved in current registry lookup.
- Isolated mops probe build passed for active runtime surface, enabling evidence-driven dual-path comparison in next phase.

**Implications**:
- `research/reference/analysis/motoko-graph.md` records `M15_PARITY_DISCOVERY_PASS`.
- `research/reference/index.toml` suggested next experiments advance to Phase C (`G2_DUAL_PATH_PASS`) no-regression validation.
- Runtime dependency promotion remains deferred under `recommendation_only`.

---

## DEC-046: Record M16 Dual-Path Validation Pass and Enrichment Completion (Posture Unchanged)
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Hold progression pending additional retries or wider workload envelope before closing Phase C
2. Close Phase C now when dual-path pass criteria and enrichment package are complete, while preserving deferred-promotion posture

**Decision**: Adopt option 2. M16 dual-path validation is complete (`G2_DUAL_PATH_PASS`, mirrors `DEC-009` in `/Users/xaoj/ICP/research/078-knowledge-graphs/DECISIONS.md`) with enrichment artifacts published and recommendation-only guardrails unchanged.

**Rationale**:
- Both paths (`vessel`, `mops`) passed required build lifecycle checks, M4 workloads (`120`, `180`), and two-pass M8 staged migration checks.
- Candidate path required retries on one workload step but converged within policy budget, with no invariant regression.
- Risk and operability deltas are now explicit and decision-usable for steward/owner follow-up.

**Implications**:
- `research/reference/analysis/motoko-graph.md` records `G2_DUAL_PATH_PASS` and M16 enrichment outcomes.
- `research/reference/index.toml` advances next-step guidance to post-M16 monitoring and modernization milestone reassessment under recommendation-only authority.
- Runtime dependency promotion remains deferred.

---

## DEC-047: Record M17 Decision-Capture Guidance for Cortex Desktop (Governance-Oriented Next Step)
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Keep post-M16 state as documentation-only and defer decision UX/process definition
2. Publish explicit Cortex Desktop guidance for evidence surfacing and steward/owner decision capture contract

**Decision**: Adopt option 2. Publish M17 guidance artifact (mirrors `DEC-010` in `/Users/xaoj/ICP/research/078-knowledge-graphs/DECISIONS.md`) to standardize data/workflow/progress surfacing and follow-up decision capture semantics.

**Rationale**:
- Remaining uncertainty is governance disposition, not technical validation completeness.
- Explicit capture contract prevents drift between Desktop-facing state and canonical decision/reference logs.
- Recommendation-only authority is preserved while enabling deterministic steward/owner action.

**Implications**:
- `research/078-knowledge-graphs/M17_CORTEX_DESKTOP_DECISION_CAPTURE_GUIDE.md` becomes the operational reference for decision-session execution.
- `research/reference/analysis/motoko-graph.md` advances to `M17_DECISION_PACKAGE_READY`.
- `research/reference/index.toml` next-step language shifts from validation execution to steward/owner decision capture workflow.

---

## DEC-048: Record Final M17 Disposition as Hold Deferred and Freeze Promotion
**Date**: 2026-02-08
**Status**: ✅ Decided

**Options Considered**:
1. Advance toward dependency progression after M16 pass
2. Hold deferred, preserve watch-first posture, and continue recommendation-only governance
3. Open an additional evidence cycle immediately before disposition

**Decision**: Adopt option 2. Final M17 disposition is captured as **Hold Deferred** (mirrors `DEC-011` in `/Users/xaoj/ICP/research/078-knowledge-graphs/DECISIONS.md`).

**Rationale**:
- Dual-path validation and enrichment are complete and useful, but do not eliminate modernization and operational governance risks.
- Recommendation-only authority favors conservative continuation absent explicit owner-level risk acceptance for promotion.
- A clear captured disposition removes ambiguity and closes the current decision cycle.

**Implications**:
- `research/reference/analysis/motoko-graph.md` advances to `M17_DECISION_CAPTURED_HOLD_DEFERRED`.
- `research/reference/index.toml` next-step guidance moves to watch-first monitoring with trigger-based re-open conditions.
- Runtime dependency promotion remains deferred.

---

## DEC-049: Mirror M19 Decision Event Apply (Hold Deferred)
**Date**: 2026-02-08
**Status**: ✅ Decided

**Decision**: Mirror application of decision event `kg_decision_20260208_manualseed` from initiative `078` with selected option **Hold Deferred**.

**Rationale**:
- Governance decision was captured through controlled event workflow.
- Cross-log synchronization is required for consistency.

**Implications**:
- Reference analysis/index are synchronized to the applied disposition.
- Authority mode remains `recommendation_only`.

---

## DEC-050: Mirror M20 Continuous Observability Enablement
**Date**: 2026-02-08
**Status**: ✅ Decided

**Decision**: Mirror `078/DEC-013` by enabling M20 continuous observability for `motoko-graph`, including trend artifacts, gateway trend/run endpoints, and Desktop operational analytics panels.

**Rationale**:
- M19 proved single-event governance execution; M20 adds repeatable operational telemetry and explicit next-action guidance.
- Trend-backed visibility improves steward/owner decision quality without changing authority boundaries.

**Implications**:
- `research/reference/analysis/motoko-graph.md` advances to `M20_CONTINUOUS_OBSERVABILITY_READY`.
- `research/reference/index.toml` next-step language shifts to trend-backed weekly monitoring and trigger-based reopen criteria.
- `authority_mode` remains `recommendation_only`; dependency promotion remains deferred.

---

## DEC-051: Canonical Nostra/Cortex Filesystem Boundary and Workspace Cutover
**Date**: 2026-02-17
**Status**: ✅ Decided

**Options Considered**:
1. Keep Cortex execution crates under `nostra/` and rely on naming-only boundaries
2. Separate into immediate multi-repo split
3. Keep monorepo and establish canonical split workspaces with phased path cutover

**Decision**: Adopt option 3. The repository remains monorepo, with canonical platform root `nostra/` and canonical execution root `cortex/`. Cortex desktop and `cortex-*` libraries move to `cortex/*` with active CI/script path contract updates.

**Rationale**:
- Preserves governance and coordination advantages of a single repository.
- Eliminates layer ambiguity that previously mixed execution runtime under platform-root paths.
- Reduces operational drift by centralizing path resolution through shared variables/contracts.

**Implications**:
- Canonical execution workspace is now `cortex/Cargo.toml`; platform workspace remains `nostra/Cargo.toml`.
- Active automation and verification lanes consume canonical Cortex path variables in CI/scripts.
- Root workspace remains compatibility-only during transition; no endpoint or public schema semantics change.
