# Decisions

## 2026-02-23 - Heaper Reverse-Engineering Closeout

1. **Adopt pattern, not runtime**
- We will not adopt Heaper's Go/Node/Caddy runtime as a dependency.
- We will adopt its architectural pattern: collaborative block state + denormalized query projection.

2. **CRDT authority remains Initiative 113**
- Heaper's Yjs behavior informs mapping design only.
- Nostra/Cortex uses Initiative 113 (`ArtifactCrdtMutation`) as canonical CRDT authority.

3. **Heap emission contract standardized**
- `EMIT_HEAP_BLOCK.schema.json` is now the agent payload contract for heap-mode persistence.
- Contract includes semantic relations, files/apps/meta projections, and CRDT mutation projection hints.

4. **Mention policy**
- Mentions are inline content refs first.
- Mentions may be mirrored to graph relations when projection policy requests it.

5. **File key policy**
- Canonical file key policy is `hash:file_size`.
- Import/reconciliation paths must tolerate `hash`-only legacy keys.

## 2026-02-24 - Desktop Source Implementation + Web Parity Rollout Lock

1. **Desktop is authoritative implementation for 124**
- Canonical heap ingest, mapping, projection, query, and action semantics are implemented in `cortex/apps/cortex-desktop`.
- `cortex-domain` remains CRDT primitive authority via Initiative 113.

2. **Canonical heap contract endpoints are active**
- `POST /api/cortex/studio/heap/emit`
- `GET /api/cortex/studio/heap/blocks`
- `POST /api/cortex/studio/heap/blocks/:artifact_id/pin`
- `POST /api/cortex/studio/heap/blocks/:artifact_id/delete`

3. **Web is parity consumer phase**
- `cortex/apps/cortex-web` consumes canonical desktop heap endpoints behind `VITE_HEAP_PARITY_ENABLED`.
- Web parity request-contract tests are included in `tests/heapApiContract.test.ts`.

4. **No Heaper runtime dependency adoption**
- Heaper remains reference architecture only.
- No Go/Node/Caddy/Yjs authority was introduced into Cortex runtime.

## 2026-02-25 - Universal Polymorphic Block Adoption

1. **Broadening Block Payload Constraints**
- Replaced references to "A2UI snapshot persistence" with "Polymorphic payload persistence" across the codebase.
- The CRDT stream and data map universally applies to `a2ui`, `rich_text`, `media`, `structured_data`, and `pointer` primitives.

## 2026-02-25 - Phase A Heap UX Interaction Layer

1. **New Gateway Endpoints**
- `POST /api/cortex/studio/heap/blocks/context` — packages selected blocks into `AgentContextBundle` for Agent Harness consumption.
- `GET /api/cortex/studio/heap/blocks/:artifact_id/export?format=markdown|json` — Markdown export includes YAML frontmatter with tags and mentions as `[[wikilinks]]`. JSON export returns raw polymorphic block payload.
- `GET /api/cortex/studio/heap/blocks/:artifact_id/history` — reads JSONL audit log, filters by artifact_id, returns chronological version timeline.

2. **Block Selection + Multi-Action**
- `HeapBlockWrapper` gains `select_mode`, `is_selected`, `on_select` props with checkbox overlay and primary border feedback.
- `HeapWorkspaceView` manages `HashSet<String>` selection state with floating action bar (Send to Agent / Delete / Clear).

3. **Client-Side Search**
- `⌘K` shortcut toggles search overlay. Client-side filtering across title, tags, mentions, block_type. Live result count. ESC dismisses.

4. **View Modes**
- `HeapViewMode` enum: All (chronological), Unlinked (triage inbox — blocks with no tags/mentions), Pinned.
- Tab-style selector in toolbar with contextual empty states.

5. **Advanced Filter Logic**
- Three-tier tag filtering: OR (additive, green), AND (required, blue), NOT (excluded, red).
- Expandable filter panel with colored operation buttons and removable tag pills.

6. **Keyboard Shortcuts**
- `⌘K` search, `Escape` dismiss/clear, `⌘A` select-all (when in select mode).

7. **Motion Design System Foundation**
- 10 keyframe animations, 6 easing/duration tokens, 10 utility classes, 5 stagger delays in `tailwind_fallback.css`.
- `MotionPolicy` helpers in `theme_policy.rs`. All animations respect `prefers-reduced-motion`.
