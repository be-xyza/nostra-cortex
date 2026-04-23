---
id: 080
name: dpub-standard
title: 'Decisions: dPub V1 (080)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-04'
updated: '2026-02-10'
---

# Decisions: dPub V1 (080)

**Date**: 2026-02-04
**Status**: Active (Labs)

## D-001 — Publish order follows Manifest
**Decision**: Edition hash computation and snapshot ordering follow `DPubManifest.chapters` order (with fallback to content order for missing entries).
**Rationale**: Keeps edition integrity aligned with the curated narrative order in the dPub manifest.
**Implication**: Reordering the manifest changes the edition Merkle root deterministically.

## D-002 — “Arranged” license requires explicit override token
**Decision**: Publishing is blocked when license contains `Arranged` unless an override token is supplied.
**Rationale**: Enforces minimum rights compatibility without a full treaty system in V1.
**Implication**: Publisher must provide a token in UI/canister API to proceed.

## D-003 — Feeds are native JSON + derived RSS/Atom
**Decision**: Feed source of truth is `feed.json`; RSS/Atom are derived client-side.
**Rationale**: Single source of truth reduces divergence and simplifies V1 syndication.
**Implication**: Any new feed format must be derived from the same JSON source.

## D-004 — Local-first portability via ZIP export
**Decision**: Provide a client-side ZIP export of `/lib/dpubs/<slug>` as V1 portability.
**Rationale**: Ensures data exportability without external pinning dependencies.
**Implication**: Export uses browser Blob download; storage remains VFS-first.

## D-005 — VFS dPub paths are canonical
**Decision**: Canonical dPub storage paths are `/lib/dpubs/<slug>/dpub.json` + editions under `/editions/<ver>/`.
**Rationale**: Stable pathing enables deterministic sync and tooling across Labs and canisters.
**Implication**: All reads/writes must normalize to these paths.

## D-006 — Cross-space treaty gate in Labs UI
**Decision**: Labs UI blocks cross-space reading/export unless a treaty token is provided and the viewer space is set.
**Rationale**: Prevents accidental re-export of private space content before full treaty enforcement is implemented.
**Implication**: Readers see a restricted placeholder; RSS/Atom/ZIP/publish are blocked without a token.

## D-007 — Canister feed access enforces treaty when viewer space provided
**Decision**: `get_dpub_feed` enforces treaty when a viewer space is provided and differs from the dPub space, unless a token is supplied.
**Rationale**: Prevents cross-space feed leakage at the service boundary while keeping backward compatibility.
**Implication**: Clients must pass `viewer_space_did` + `treaty_token` to access cross-space feeds.

## D-008 — Safe default requires current space set in Labs
**Decision**: When treaty enforcement is enabled, Labs UI treats missing/empty `current_space_did` as restricted.
**Rationale**: Authority is unclear without a current space; safe default is recommendation-only.
**Implication**: Users must set a current space before cross-space reads or syncing dPub data.

## D-009 — Guarded dPub VFS access requires viewer space
**Decision**: Add guarded `read_dpub_file_guarded` / `list_dpub_files_guarded` APIs that require a viewer space and enforce treaty at the service boundary.
**Rationale**: Ensures cross-space access control is enforced at the canister layer, not just in UI.
**Implication**: dPub sync operations should use guarded endpoints with `viewer_space_did` + optional treaty token.

## D-010 — Treaty-enforced sync uses guarded dPub endpoints
**Decision**: When treaty enforcement is enabled, Labs sync uses guarded dPub sync (dpubs-only) and requires a current space.
**Rationale**: Prevents cross-space leakage via unguarded VFS listing and reads.
**Implication**: Full `/lib` sync requires disabling treaty enforcement, or separate guarded APIs for other directories.

## D-011 — Guarded full VFS sync requires treaty token
**Decision**: Add guarded `read_vfs_guarded` / `list_vfs_guarded` APIs that require both a viewer space and treaty token for non‑dPub paths.
**Rationale**: Enables full `/lib` sync under treaty enforcement without silently relaxing access controls.
**Implication**: Full sync is allowed only when a treaty token is supplied; otherwise dpubs‑only sync is used.

## D-012 — Dual-file snapshot contract naming
**Decision**: Keep `snapshot.json` as immutable edition payload and use `snapshot_manifest.json` as the SnapshotManifest contract artifact.
**Rationale**: Runtime and reader implementations already load `snapshot.json` as content payload; separating contract metadata avoids semantic drift and preserves deterministic validation.
**Implication**: Publish flows must write `edition_manifest.json`, `snapshot.json`, and `snapshot_manifest.json`; schema/docs/checks must treat JSON feed as canonical and RSS/Atom as derived views only.

## D-013 — BookManifest renamed to DPubManifest
**Decision**: Rename `BookManifest` to `DPubManifest` in `manifest.mo`; rename `title` field to `titleCache` and add `VersionedRef` support.
**Rationale**: Per Consolidated Spec §2.2, "Book" is a UI label, not the canonical type identity. `DPub` is the system concept.
**Implication**: `book.mo` and `book_engine.mo` remain as legacy references but must not be extended. All new Manifest usage should reference `DPubManifest`.

## D-014 — SchemaStatus lifecycle formalized
**Decision**: Add `SchemaStatus` (draft/proposed/approved/deprecated/archived) to `SchemaDefinition` in `types.mo` with lifecycle transition enforcement in the Schema Registry actor.
**Rationale**: 026-nostra-schema-manager requires governance hooks; without status tracking, schema changes bypass review entirely.
**Implication**: All system schemas bootstrap as `#approved`. User-created schemas default to `#draft`. Status transitions enforced: draft→proposed→approved→deprecated→archived. Breaking change to `SchemaDefinition` type.

## D-015 — Ingest pipeline is workflow-governed
**Decision**: The knowledge ingest pipeline is implemented as a `WorkflowDefinition` (`knowledge-ingest-pipeline-v1`) with 6 quality gates, not as a monolithic function.
**Rationale**: Workflow governance ensures human review at extraction and integration steps. System ops validate source legitimacy, schema coherence, and constitutional constraints automatically.
**Implication**: Ingestion requires a running workflow instance. Direct `ingest()` calls bypass governance gates — callers should prefer starting the workflow via `startWorkflowV2`.

## D-016 — Publish edition requires human approval gate
**Decision**: The publish edition workflow (`publish-edition-v1`) includes a mandatory `user_task` review step before creating an immutable edition snapshot.
**Rationale**: Editions are citation targets — once published, the Merkle root is immutable. Human review prevents premature publication of incomplete or incorrect content.
**Implication**: Automated publishing (e.g., CI/CD) must still signal the approval step, even if the approval is pro-forma.

## D-017 — Snapshot contracts enforce bundle integrity
**Decision**: SnapshotManifest records per-file hashes and a Merkle content root. `validateBundle` checks all hashes at read time, rejecting any bundle with mismatched or missing files.
**Rationale**: Editions are immutable citation targets. Without hash verification, content could drift after publication, breaking citations and trust.
**Implication**: All exported dPub bundles must include a `snapshot_manifest.json` alongside content. Reference resolution must run before manifest finalization to ensure `unresolvedCount == 0`.

## D-018 — Cross-space access is treaty-governed
**Decision**: Federated data access between spaces requires an active `Treaty` with explicit `AccessScope` (entity types, hop depth, operations).
**Rationale**: Uncontrolled cross-space traversal risks data leakage. Treaties provide auditable, revocable access grants with lifecycle state enforcement (proposed→active→suspended→revoked).
**Implication**: All cross-space graph queries must check treaty status and scope before returning results.

## D-019 — VFS writes pass through pre-write hooks
**Decision**: The VFS constructor accepts an optional `PreWriteHook` callback invoked before any `writeFile` operation. If the hook returns `#err`, the write is rejected.
**Rationale**: Enables schema validation, constitutional constraint checks, and agent config validation at the storage layer without coupling VFS to specific validators.
**Implication**: Breaking change to `VirtualFileSystem` constructor (now takes 2 args). All call sites updated to pass `null` for backward compatibility.

## D-020 — Treaty store is canister-hosted with stable array persistence
**Decision**: Treaties are stored as stable `[(Text, Treaty)]` in the backend canister with 7 CRUD endpoints (create, activate, suspend, revoke, getTreaty, getTreaties, checkTreatyAccess). All mutations are admin-gated and audit-logged.
**Rationale**: Treaties govern cross-space access and must survive upgrades. Stable array follows the same pattern as `govProposals` for consistency.
**Implication**: Treaty operations are synchronous within the canister. Cross-canister treaty verification will require inter-canister calls in future phases.

## D-021 — Edition artifacts carry computed confidence surfaces
**Decision**: `EditionArtifact` includes a `ConfidenceSurface` computed from chapter-level confidence scores during `assembleEdition`.
**Rationale**: Confidence is a first-class quality signal per the Data Confidence system standard. Embedding it at publish time ensures every citation target carries trust metadata.
**Implication**: `assembleEdition` now requires `chapterConfidences` parameter; callers must supply per-chapter scores.

## D-022 — Agent configs and traces are VFS artifacts (E6)
**Decision**: Agents store configuration and decision traces as JSON files in the VFS under `/agents/<agentId>/`.
**Rationale**: Treating agent state as file artifacts inherits existing VFS access controls, versioning (future), and tooling (Inspector), enabling "Glass Box" observability.
**Implication**: `main.mo` endpoints `registerAgent` and `logAgentTrace` must write to VFS; PreWriteHook must validate these paths.

## D-023 — Graph relationships are extracted from content references (E7)
**Decision**: `@[Entity]` and `#[Tag]` patterns in `NostraBlock` content are parsed and materialized as graph edges (`#references`, `#tagged`) upon request.
**Rationale**: Explicit linking during content creation is high-friction. Automatic extraction from natural syntax encourages interconnectivity without UI overhead.
**Implication**: `linkReferences` endpoint added to allow retroactive linking of existing content.

## D-024 — Schema hygiene is automated via purity analysis (E8)
**Decision**: A system gardener module analyzes schema usage (orphan types) and semantic distance (Levenshtein) to generate governance tasks.
**Rationale**: Knowledge graphs tend toward entropy. Automated hygiene ensures the ontology remains clean and usable without manual auditing.
**Implication**: `runningSchemaHygiene` is a heavy operation; results are audit-logged and surfaced as potential sleep tasks.

## D-025 — Schema exploration is a backend-rendered A2UI surface (E5)
**Decision**: The schema explorer UI is generated entirely by the backend as an A2UI JSON surface, identifying entity types and counts from the live graph.
**Rationale**: Eliminates the need for custom frontend code updates to browse the ontology. The backend is the source of truth and can best visualize its own schema.
**Implication**: `getSchemaExplorerSurface` query endpoint added; no frontend deployment required to ship E5.
