---
id: "100-bible-native-dpub-corpus"
name: "bible-native-dpub-corpus"
title: "Decision Log: Bible as a Native dPub Corpus"
type: "decision"
project: "nostra"
status: draft
authors:
  - "User"
tags:
  - "dpub"
  - "corpus"
  - "legal"
created: "2026-02-05"
updated: "2026-02-05"
---

# Decision Log: Bible as a Native dPub Corpus

Track architectural decisions with rationale for future reference.

---

## DEC-001: Phase-1 KJV source policy (“Latest” + pinned manifest)
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Pick a single named historical edition (e.g., 1769 Oxford) as canonical
2. Use “latest KJV text” from a chosen source and make it reproducible by pinning a SourceManifest at import time

**Decision**: Use the **latest KJV text** available from the chosen Phase-1 source, but require a **BibleSourceManifest** that pins the exact source inputs (hashes + retrieval metadata) so “latest” is reproducible.

**Rationale**: “Latest” is not a stable bibliographic identifier by itself; provenance must make the import deterministic and auditable.

**Implications**:
- Every import run writes a `BibleSourceManifest` artifact and references it from verse/chapter provenance.
- If the upstream source changes, it becomes a new edition_id/translation fork in lineage, not an in-place edit.

---

## DEC-002: Canonical VerseKey standard
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. OSIS IDs (e.g., `Gen.1.1`) as canonical
2. USFM identifiers (e.g., `GEN 1:1`) as canonical
3. Internal IDs only (rejected; portability loss)

**Decision**: Use **OSIS VerseKey** as canonical. Store USFM (and other) aliases as optional fields.

**Rationale**: OSIS maps cleanly to Book/Chapter/Verse and is widely interoperable.

**Implications**: Cross-reference graphs and external interoperability become simpler.

---

## DEC-003: Granularity for dPub V1 bootstrap
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Chapter-as-Artifact in dPub (verses rendered as blocks) — recommended for V1
2. Verse-as-Contribution (31K+ artifacts per translation)

**Decision**: Phase 1 uses **chapter-as-Artifact**; verse-atomic store is Phase 2.

**Rationale**: dPub V1 tooling is chapter-oriented today; verse-atomic introduces scale and resolver/index pressure.

**Implications**: Verse-level forking/lineage is implemented in the Corpus layer first, then surfaced in dPub views.

---

## DEC-004: License labeling policy
**Date**: 2026-02-05
**Status**: 🟡 Proposed

**Options Considered**:
1. Single global label (“Public Domain”)
2. Jurisdiction-aware labels (`public_domain_us`, `crown_copyright_uk`, etc.)

**Decision**: Deferred pending legal steward review; provisional recommendation is jurisdiction-aware representation.

**Rationale**: Scripture texts have complex jurisdictional status; “public domain” is not universally true for KJV.

**Implications**: Export bundles must carry license metadata and potentially enforce region-based warnings in UI.

---

## DEC-005: EPUB alignment strategy (adapter vs canonical)
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Model EPUB internals (OPF/spine/nav/XHTML) as first-class canonical objects
2. Treat EPUB as an adapter/export format derived from dPub Editions (recommended)

**Decision**: EPUB is a **portable view** of an Edition; canonical source remains dPub snapshot (`snapshot.json`) + `NostraBlock`.

**Rationale**: dPub V1 is manifest + edition + snapshot driven; making EPUB canonical would shift the project into “EPUB-native authoring” and complicate integrity/versioning.

**Implications**:
- Export pipeline must be deterministic and auditable (exporter version recorded).
- Import pipeline uses EPUB as an ingestion source only.

---

## DEC-006: Deterministic EPUB constraints
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Use current time/random UUIDs freely in EPUB generation
2. Derive IDs/timestamps from edition inputs and normalize output (recommended)

**Decision**: Normalize output for deterministic builds.

**Rationale**: Portability and verification improve when the same Edition yields the same EPUB bytes (or at least the same file hashes).

**Implications**:
- Stable file ordering, normalized ZIP timestamps, consistent compression.
- Any identifiers (e.g., package UUID) derived from `edition_id`/`dpub_id`/`version`.

---

## DEC-007: Translation and edition identifiers
**Date**: 2026-02-05
**Status**: ✅ Decided

**Decision**:
- `translation_id`: stable semantic identifier (e.g., `kjv`, `web`)
- `edition_id`: derived from `{translation_id}:{source_manifest_id}` (or a stable hash), not from human labels like “latest”

**Rationale**: Enables forks and provenance without ambiguity.

**Implications**: Verse/chapter atoms must carry both `translation_id` and `edition_id` (or a reference to the SourceManifest that implies it).
