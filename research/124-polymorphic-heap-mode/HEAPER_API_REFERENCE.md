# Heaper API Reference (Reverse-Engineered, Validated)

> Source container: `ghcr.io/janlunge/heaper:latest`
> Validation date: 2026-02-23
> Environment: local Docker (`heaper`), Apple Silicon host, linux/amd64 image

This document consolidates runtime evidence from:
- Postgres introspection (`psql` against running container DB)
- `backend-sync` source (`/usr/src/app/backend-sync/dist/server.js`, 2,345 lines)
- Caddy routing config (`/etc/caddy/Caddyfile`, fallback script)
- Frontend bundle route extraction (`/usr/src/app/frontend/assets/*.js`)
- Go backend binary route extraction (`strings /usr/local/bin/backend`)
- Live HTTP probing against `http://localhost:3010`

## 1. Runtime Topology

Heaper runs 4 supervised services in one container:

| Service | Internal Port | Implementation | Role |
|---|---:|---|---|
| Caddy | 80 / 443 | Caddy | Reverse proxy + frontend static hosting |
| Backend API | 3000 | Go binary (`/usr/local/bin/backend`) | Main REST API |
| Backend Sync API | 1233 | Node (`backend-sync`) | Sync/ydoc HTTP endpoints |
| Hocuspocus WS | 1234 | Node (`backend-sync`) | Yjs websocket collaboration |

Host mapping from `docker-compose.yml`:
- `3010 -> 80`
- `1233 -> 1233`
- `1234 -> 1234`

### 1.1 Caddy Route Split

Primary routing behavior:
- `/ws*` -> `localhost:1234`
- `/api/sync/*` -> `localhost:1233`
- `/api/ydoc/*` -> `localhost:1233` (TLS and fallback config)
- `/sync/*` -> `localhost:1233` (TLS and fallback config)
- `/api/*` (other) -> `localhost:3000`
- fallback -> frontend SPA

Implication: backend-sync exposes only a subset of API routes; most `/api/*` routes are served by Go backend.

## 2. Postgres Schema (public: 19 tables)

Observed tables:
- `apps`
- `attributes`
- `auth_requests`
- `block_access`
- `block_relations`
- `block_updates`
- `blocks`
- `coupons`
- `duplicate_files`
- `entitlements`
- `files`
- `licenses`
- `roadmap_interactions`
- `sessions`
- `subscriptions`
- `system`
- `thumbnails`
- `users`
- `workspaces`

### 2.1 Core Tables

#### `blocks`
- Primary key: `id` (`ulid`)
- Content projection columns: `title`, `text`, `title_doc`, `text_doc`
- CRDT base state: `ydocument` (`bytea`)
- Compaction/versioning: `sync_version` (`int`, non-null)
- Ownership/scope: `workspace_id`, `owner_id`
- Lifecycle: `deleted_at`, `permanently_deleted_at`, `cleanup_completed_at`

#### `block_updates`
- Primary key: `id` (`ulid`)
- Foreign-ish pointer: `block_id`
- Incremental CRDT update chunk: `updates` (`bytea`)
- Optional snapshots: `text_doc` (`jsonb`), `ydoc` (`bytea`)

#### `block_relations`
- Primary key: `id` (`ulid`)
- Relation shape: `from_id`, `to_id`, `type`, `meta`
- Scope/audit: `created_by`, `workspace_id`

#### `files`
- Primary key: `id` (`ulid`)
- Block association: `block_id`
- Dedup identity: `hash`
- Metadata: `name`, `mime_type`, `file_size`, `width`, `height`, `duration`, `exif_data`, `is_uploaded`
- Constraint: `UNIQUE (block_id, hash)`

#### `duplicate_files`
- Primary key: `id` (`ulid`)
- Fields: `workspace_id`, `original_block_id`, `duplicate_path`, `hash`, `file_size`, `scan_id`
- Constraint: `UNIQUE (workspace_id, duplicate_path)`

### 2.2 Important Constraint Facts

- `files` dedupe uniqueness is per block: `UNIQUE (block_id, hash)`
- No DB-level uniqueness found for `block_relations (from_id, to_id, type)`
- `duplicate_files` tracks path-based duplicate detection, not only hash global uniqueness

## 3. Yjs / Hocuspocus Model

## 3.1 Document Naming

`backend-sync` accepts multiple naming patterns:
- `{workspaceID}:{blockID}:v{syncVersion}`
- `{workspaceID}:{blockID}`
- `block/{blockID}` (with `workspaceId` query parameter)

## 3.2 Shared Types Observed

Observed in backend-sync/frontend:
- `title_v0` (`Y.XmlFragment`)
- `content_v0` (`Y.XmlFragment`)
- `files_v0` (`Y.Map`)
- `apps_v0` (`Y.Map`)
- `meta_v0` (`Y.Map`)
- `relations_v0` (checked in onStore guard)
- `relations_v1` (`Y.Map`, used for relation hydration/processing)

Note: relation naming is version-fragmented (`relations_v0` and `relations_v1` both appear in runtime code).

## 3.3 Lifecycle (Load/Store/Compaction)

Load path (`onLoadDocument`):
1. Resolve `blockID`/`workspaceID` from doc name.
2. Load `blocks.ydocument` base if present.
3. Apply `block_updates.updates` in chronological order.
4. If no base doc exists, hydrate from Postgres projections (`title`, tags, files).

Store path (`onStoreDocument`):
1. Diff current full doc against persisted state.
2. Persist daily update row in `block_updates` (upsert-by-date behavior).
3. Extract and project:
- `content_v0` -> `blocks.text`, `blocks.text_doc`
- `title_v0` -> `blocks.title`, `blocks.title_doc`
- `apps_v0` -> `apps` table sync
- `files_v0` -> `files`/`thumbnails` sync
- `relations_v1` -> `block_relations` sync logic
- `meta_v0` -> soft/permanent delete timestamps

Compaction:
- Merge base + updates into fresh base
- Write to `blocks.ydocument`
- Increment `blocks.sync_version`
- Delete old `block_updates`
- Explicit endpoint: `POST /api/ydoc/compact`

## 4. Tags, Mentions, Links (Important Semantics)

## 4.1 Tags

Tags are structural relations:
- `block_relations` rows with `type = 'tag'`
- Hydrated into Yjs `relations_v1` map as tag entries

## 4.2 Mentions

Mentions appear as inline ProseMirror nodes:
- Node type: `reference`
- Attrs include `id`, `label`, `type: "mention"`

Important nuance:
- Mentions are inline content primitives in ProseMirror/Yjs,
- but backend-sync can also project mention relations into `block_relations` (`type = 'mention'`) during relation processing.

So the strict statement "mentions are not in block_relations" is not always true in the current backend-sync implementation.

## 4.3 Page Links

Inline ProseMirror node:
- `pagelink` with `blockKey`

## 5. Files and Dedup

- DB dedupe key per block: `(block_id, hash)`
- Yjs key format is inconsistent across code paths:
- add-file path uses `"${hash}:${file_size}"`
- hydration and thumbnail update paths often read/write by `hash` only

This inconsistency matters for deterministic cross-layer projection.

## 6. HTTP Endpoint Inventory

## 6.1 Backend-sync implemented endpoints (source-confirmed)

| Method | Route | Notes |
|---|---|---|
| POST | `/api/ydoc/update` | operations: `add_file`, `add_thumbnails` |
| POST | `/api/ydoc/compact` | compaction trigger |
| GET | `/sync/health` | health |
| POST | `/sync/upload-complete` | mark upload complete in Yjs files map |
| GET | `/api/sync/delta` | changed/deleted block delta feed |
| POST | `/api/sync/push` | push Yjs updates |
| POST | `/api/sync/pull` | pull Yjs state/updates |

## 6.2 Go backend routes relevant to heap/block workflows

Observed from frontend route strings + Go binary strings + probes:
- Auth/session: `/api/auth/register`, `/api/auth/login`, `/api/auth/create_session`, `/api/auth/refresh_token`, `/api/auth/verify_key`, `/api/auth/verify_email`, `/api/auth/verify_session`
- Block CRUD/query: `/api/insert_block`, `/api/update_block`, `/api/delete_block`, `/api/move_block`, `/api/order_blocks`, `/api/get_children`, `/api/query_blocks`, `/api/search_blocks`, `/api/graph_blocks`, `/api/navigation_blocks`, `/api/block_title`, `/api/block_link`, `/api/blocks_for_app`
- Heap: `/api/heap`, `/api/heap/add`, `/api/heap/changed_blocks`, `/api/heap/:id`
- Views/attrs: `/api/update_app_settings`, `/api/update_view`, `/api/update_attrs`, `/api/add_available_column`
- Workspaces/user: `/api/workspaces`, `/api/me`
- Admin: `/api/admin/invite_user`, `/api/admin/pending-users`, `/api/admin/users` (+ more admin routes in binary)
- File/media endpoints: `/api/upload/*`, `/api/file/*`, `/api/download/thumbnail/*`, `/api/status/file/*`

Auth prefix correction:
- API auth endpoints are under `/api/auth/*`.
- `/auth/*` paths in frontend are SPA routes.

## 6.3 Coverage note

- 84 unique `/api|/auth|/sync` paths were extracted from frontend + backend-sync source.
- Go binary strings expose additional parameterized paths not all method-verified.
- For non-heap features (billing/license/admin), treat this as discovery inventory, not full OpenAPI spec.

## 7. Heap-Mode Integration Takeaways for A2UI And Polymorphic Heap

- Use the dual-layer pattern:
- Source-of-truth collaborative state in CRDT artifact layer
- Denormalized query/search projections in relational store

- Preserve semantic split:
- tags = structural graph metadata
- mentions = inline content references, optionally mirrored as structural edges for queryability

- Treat file dedupe as content-addressed (`hash`) with block-scoped uniqueness and explicit projection rules to avoid key drift (`hash` vs `hash:file_size`).

- Keep transport separation explicit:
- websocket collaboration path (Hocuspocus) for real-time edits
- REST sync/compaction endpoints for deterministic background reconciliation.

## 8. Validation Caveats

- Go backend source code was not available in-container; route behavior outside backend-sync is inferred from runtime probing, frontend calls, and binary string extraction.
- Some status codes in probes reflect missing auth/body rather than route absence.
- Relation map versioning (`relations_v0` vs `relations_v1`) appears transitional and should be normalized in Nostra mapping.
