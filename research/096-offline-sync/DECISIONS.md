---
id: 096
name: offline-sync
title: 'Decisions Log: Offline Sync & Local-First (096)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-05'
updated: '2026-02-06'
---

# Decisions Log: Offline Sync & Local-First (096)

## 001: Desktop-First Phase 1
**Date**: 2026-02-05
**Status**: ACCEPTED
**Decision**: Phase 1 prioritizes Cortex Desktop for durable LocalGateway persistence and replay.
**Rationale**: Desktop is the operator surface for LocalGateway and fastest path to durable offline writes.

## 002: LocalGateway Queue Persistence Format
**Date**: 2026-02-05
**Status**: ACCEPTED
**Decision**: Persist queued mutations to JSON at `~/.nostra/cortex/local_gateway_queue.json`.
**Rationale**: JSON is transparent, debuggable, and fast to ship for scaffolding.

## 003: Idempotency Key Strategy
**Date**: 2026-02-05
**Status**: ACCEPTED
**Decision**: Use `id` as the primary `idempotency_key`; if missing, derive a new UUID on load.
**Rationale**: Guarantees replay safety without retrofitting legacy queues.

## 004: Conflict Handling UX (Phase 1)
**Date**: 2026-02-05
**Status**: ACCEPTED
**Decision**: Surface conflicts as an A2UI stub task with `retry / fork / discard`. Advanced merge UI is deferred.
**Rationale**: Provides a minimum viable user decision surface while merge policies mature.

## 005: Unified Inbox Information Architecture
**Date**: 2026-02-06
**Status**: ACCEPTED
**Decision**: Use a single inbox with labels/tags instead of a dedicated conflict-only inbox.
**Rationale**: Aligns with A2UI semantics-first routing, reduces UI fragmentation, and keeps all actionable work in one operator stream.

## 006: Inbox Semantics Contract
**Date**: 2026-02-06
**Status**: ACCEPTED
**Decision**: Workflow engine emits `RenderSurface` metadata for inbox routing (`context`, `priority`, `tone`, `density`, `intent`, `theme`, plus provenance fields).
**Rationale**: Metadata-driven rendering avoids per-screen hardcoding and enables deterministic cross-host behavior.

## 007: Action Identity Normalization
**Date**: 2026-02-06
**Status**: ACCEPTED
**Decision**: All mutating inbox actions use `verb:id` identity strings (e.g., `conflict_retry:<mutation_id>`).
**Rationale**: Supports stable parsing, host-specific routing, and safer dedup/archival behavior.

## 008: Theme Defaults for Nostra vs Cortex
**Date**: 2026-02-06
**Status**: ACCEPTED
**Decision**: Host defaults are `nostra` for Nostra surfaces and `cortex` for Cortex surfaces, with optional `meta.theme` override.
**Rationale**: Preserves architectural boundary while keeping one shared policy layer.
