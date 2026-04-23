# Heaper Methodology -> Polymorphic Heap Mode (Final Decisions)

Date: 2026-02-23
Initiative: `research/124-polymorphic-heap-mode`

This document replaces the prior "gap-only" analysis with validated implementation decisions for Nostra/Cortex heap mode.

## 1. What Was Validated

Validation was performed using:
- Live Postgres introspection from running Heaper container
- Full `backend-sync` runtime source (`dist/server.js`, 2,345 lines)
- Caddy route config and startup scripts
- Frontend bundle route extraction
- Go backend binary route extraction (`strings`)
- Live HTTP probing against container host mapping

Result: enough evidence exists to define a practical polymorphic heap direction for A2UI surfaces without importing Heaper runtime components.

## 2. Final Decisions

### Decision A: Keep Nostra/Cortex architecture, copy pattern only

- Do not adopt Heaper runtime stack (Go backend + Node sync + Caddy) as a dependency.
- Do adopt pattern: collaborative block state + denormalized query projection + semantic relations.

Rationale:
- Aligns with MVK and WASM-first constraints.
- Avoids introducing a second sync authority outside Initiative 113.

### Decision B: Heap block semantics in Nostra

Define heap block as:
- A persisted contribution envelope containing A2UI artifact snapshot.
- Relations split into:
- Tags: structural graph edges (`type = tag`)
- Mentions: inline content refs; may also be mirrored to graph edges for queryability
- Page links: navigational refs

Rationale:
- Matches observed Heaper behavior while preserving Nostra graph governance.

### Decision C: Source of truth vs projection layers

Nostra heap mode must use two layers:
1. CRDT artifact state (source of truth) via Initiative 113 (`ArtifactCrdtMutation`)
2. Query projection layer (search/filter/index columns)

Rationale:
- Heaper uses Yjs + Postgres projection. Equivalent pattern should exist in Nostra with Initiative 113 as the CRDT authority.

### Decision D: Mention handling policy

Mentions are modeled as inline content references, but we will mirror them into graph edges when `projection_hints.mirror_mentions_to_relations = true`.

Rationale:
- Preserves content fidelity and enables graph query surfaces.

### Decision E: File identity policy

Use content-addressed identity (`hash`) with explicit key policy in projection:
- canonical in Nostra contract: `hash:file_size`
- tolerate `hash`-only during import/reconciliation

Rationale:
- Heaper runtime shows mixed keying behavior; explicit policy avoids ambiguity.

## 3. A2UI Polymorphic Heap Contract

`EMIT_HEAP_BLOCK` is the agent-facing payload contract (see `EMIT_HEAP_BLOCK.schema.json`).

Required payload domains:
- Source metadata (`agent_id`, `emitted_at`, request/session)
- Block metadata (`id?`, `type`, `title`, visual metadata)
- A2UI content snapshot (`surface_id`, `tree`, optional `data_model`)
- Relations (`tags`, `mentions`, `page_links`)
- Files/apps/meta projections
- CRDT mutation projection hints (`mutations[]`)

This contract is the bridge between ephemeral A2UI emission and persistent contribution + CRDT state.

## 4. A2UI -> Heap Persistence Flow

1. Agent emits `EMIT_HEAP_BLOCK` payload.
2. Cortex Desktop validates payload schema.
3. Desktop creates/updates contribution envelope in workspace.
4. Desktop maps payload into CRDT mutation stream (`ArtifactCrdtMutation[]`).
5. Gateway applies mutations as canonical artifact state.
6. Projection worker updates query/search graph indexes:
- title/text projections
- relation projections (tag + mention mirror policy)
- file/app projections
7. Heap ViewSpec renders reverse-chronological card/grid view from projection layer.

## 5. Mutation Mapping (Deterministic)

Mapping guidelines:
- `content.a2ui.tree` changes -> `map_set`/`array_insert`/`array_delete`
- rich text edits -> `text_insert`/`text_delete`
- relation additions/removals -> graph edge mutations (+ optional inline mention node edits)
- files/apps/meta updates -> map ops under stable artifact paths

Determinism requirements:
- stable path conventions
- monotonic clocks/version checks
- idempotent replay behavior for background sync

## 6. Heap View Behavior in Cortex

Heap mode UI rules:
- Render each persisted heap block as a card (title, semantic chips, timestamps, actions)
- Support immediate filter facets:
- tags
- mentions
- block type
- recency
- file presence/media type
- Distinguish inline mention badges from structural tag badges in card UI

## 7. Sync and Transport Guidance

- Real-time collaboration: Initiative 113 transport path (Cortex-native)
- Background reconciliation: deterministic pull/push style endpoints in Cortex gateway (pattern borrowed, not endpoint names)
- Compaction: periodic CRDT state compaction to bound replay cost

## 8. What We Explicitly Do Not Copy

- Heaper runtime services (`backend-sync`, Hocuspocus, Go API) as dependencies
- Yjs as a second primary CRDT authority
- Ambiguous relation versioning (`relations_v0` vs `relations_v1`) without governance
- Mixed file-key policy without canonicalization

## 9. Immediate Implementation Steps

1. Adopt `EMIT_HEAP_BLOCK` schema in Cortex Desktop ingest path.
2. Implement schema validation + rejection telemetry.
3. Add deterministic mapper: `EMIT_HEAP_BLOCK` -> `ArtifactCrdtMutation[]`.
4. Add projection worker updates for tags/mentions/files/apps.
5. Implement Heap ViewSpec card renderer + core filters.
6. Add conformance tests:
- payload validation
- deterministic mutation replay
- mention mirror policy behavior
- file key canonicalization

## 10. Acceptance Criteria

Heap mode is "directionally complete" when:
- Agent-emitted A2UI surfaces persist as contributions with CRDT state.
- Tags and mentions are queryable and visually distinct.
- Rehydration from CRDT + projection yields consistent heap cards.
- Background sync and replay are deterministic.
- No external Heaper runtime component is required.

## 11. Implemented Reality Snapshot (2026-02-24)

Desktop source implementation is now active:

1. Canonical emit endpoint implemented:
- `POST /api/cortex/studio/heap/emit`

2. Canonical query and actions implemented:
- `GET /api/cortex/studio/heap/blocks`
- `POST /api/cortex/studio/heap/blocks/:artifact_id/pin`
- `POST /api/cortex/studio/heap/blocks/:artifact_id/delete`

3. Deterministic mapper and canonicalization implemented:
- `cortex/apps/cortex-desktop/src/services/heap_mapper.rs`
- Schema/version validation
- Mention mirror policy
- File key normalization (`hash` -> `hash:file_size`)

4. Desktop heap board now consumes live persisted data:
- `cortex/apps/cortex-desktop/src/components/views/heap_workspace_view.rs`

5. Web parity consumer rollout started (feature flagged):
- `cortex/apps/cortex-web/src/App.tsx`
- `cortex/apps/cortex-web/src/api.ts`
- `cortex/apps/cortex-web/src/contracts.ts`

6. Verification evidence (targeted):
- Desktop heap tests: emit/query/idempotency/mention-mirror/pagination/action semantics
- Web parity tests: request contract checks for query filters, cursor, and action headers

## 2026-02-25 - Adopting the Universal Polymorphic Block

1. **Broadening Block Scope**
- The Heap Block represents a universal atomic unit for the Nostra ecosystem, rather than solely an A2UI artifact.
- `EMIT_HEAP_BLOCK` contract supports `payload_type` consisting of `a2ui`, `rich_text`, `media`, `structured_data`, or `pointer`.
- "A2UI snapshot persistence" methodology is upgraded to "Polymorphic payload persistence".
- DPub and ingestion pipelines are fully aligned to rely on this singular building block methodology.
