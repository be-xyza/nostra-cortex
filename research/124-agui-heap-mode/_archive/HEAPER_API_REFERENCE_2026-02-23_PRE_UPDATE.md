# Heaper API Reference (Reverse-Engineered)

> **Source:** Extracted from the `ghcr.io/janlunge/heaper:latest` Docker container on 2026-02-23.
> The container frontend JS failed to boot (FTS5 SQLite WASM bug on Apple Silicon), so all schemas were extracted via direct Postgres introspection and reading the backend-sync `server.js` source.

---

## 1. Architecture Overview

Heaper runs three internal services inside a single Docker container:

| Service | Port | Technology | Purpose |
|---------|------|------------|---------|
| **Caddy** (reverse proxy) | 80 (→ mapped to host 3010) | Go | Routes frontend, API, and WS traffic |
| **Go API Server** | 1233 | Go + Express (Node) | REST API for auth, block CRUD, workspace management |
| **Hocuspocus Sync** | 1234 | Node.js (TypeScript) | WebSocket server for Yjs CRDT document sync |

The frontend is a Vue 3 + Vite SPA served by Caddy, using **sql.js (SQLite WASM)** as a client-side cache and **Yjs** for real-time collaboration.

---

## 2. Postgres Schema (19 Tables)

### 2.1 `blocks` — The Universal Unit

| Column | Type | Notes |
|--------|------|-------|
| `id` | `ulid` PK | Auto-generated ULID |
| `title` | `text` | Plain-text title extracted from `title_v0` Yjs fragment |
| `color` | `text` | Block accent color |
| `icon` / `icon_type` | `text` | Block icon metadata |
| `main_tag` | `ulid` FK | Primary tag (Block ID of the tag block) |
| `value_type` | `text` | Attribute value type for typed blocks |
| `value_options` | `jsonb` | Options for value type (e.g., select options) |
| `text` | `text` | Plain-text content extracted from `content_v0` |
| `title_doc` / `text_doc` | `jsonb` | ProseMirror JSON of title/content |
| `ydocument` | `bytea` | **Compacted Yjs document state** |
| `type` | `varchar(20)` | Block type enum |
| `default_app` | `ulid` FK | Default app/view for this block |
| `workspace_id` | `ulid` FK | Owning workspace |
| `owner_id` | `ulid` FK | Owning user |
| `encrypted_blob` | `bytea` | E2E encrypted payload |
| `sync_version` | `int` | Monotonically increasing compaction version |
| Timestamps | `timestamp` | `created_at`, `updated_at`, `deleted_at`, `permanently_deleted_at` |

### 2.2 `block_relations` — The Graph Edges

| Column | Type | Notes |
|--------|------|-------|
| `id` | `ulid` PK | |
| `from_id` | `ulid` FK | Source block |
| `to_id` | `ulid` FK | Target block |
| `type` | `text` | **Relation type** (e.g., `'tag'`) |
| `meta` | `jsonb` | Arbitrary metadata |
| `created_by` | `ulid` FK | User who created the relation |
| `workspace_id` | `ulid` FK | |

**Key insight:** Tags are stored as `block_relations` where `type = 'tag'` and `from_id` is the tag block, `to_id` is the tagged block. Mentions are inline ProseMirror `reference` nodes within the Yjs document—they are NOT separate `block_relations` rows.

### 2.3 `block_updates` — CRDT Incremental State

| Column | Type | Notes |
|--------|------|-------|
| `id` | `ulid` PK | |
| `block_id` | `ulid` FK | |
| `updates` | `bytea` | **Yjs incremental update binary** |
| `text_doc` | `jsonb` | Snapshot of ProseMirror JSON at update time |
| `ydoc` | `bytea` | Full Yjs doc snapshot (for daily compaction) |

### 2.4 Other Tables

| Table | Purpose |
|-------|---------|
| `workspaces` | Workspace metadata (with `heap_id`, `sidebar_order` jsonb, `encrypted` bool) |
| `files` | File metadata with SHA256 `hash` for deduplication, `block_id` FK |
| `apps` | View configurations (with `app_type`, `filter` jsonb, `sort` jsonb, `mapping` jsonb) |
| `attributes` | Typed attribute values on relations (string/num/date/json/block ref) |
| `users` | User accounts with EdDSA `public_key` for auth |
| `block_access` | ACL entries (`block_id`, `user_id`, `type` = owner/editor/viewer/none) |
| `thumbnails` | Generated image thumbnails |
| `duplicate_files` | SHA256 deduplication tracking |

---

## 3. Yjs Document Structure

Each block's Yjs document contains these shared types:

| Shared Type | Yjs Type | Purpose |
|-------------|----------|---------|
| `title_v0` | `Y.XmlFragment` | Block title as ProseMirror XML |
| `content_v0` | `Y.XmlFragment` | Block body content as ProseMirror XML |
| `relations_v1` | `Y.Map` | Tag relations (`key = from_id`, value = `Y.Map { type: "tag" }`) |
| `files_v0` | `Y.Map` | File attachments (`key = hash:fileSize`, value = file metadata Y.Map) |
| `apps_v0` | `Y.Map` | App/view configurations |

**Document naming:** `{workspaceID}:{blockID}:v{syncVersion}` (e.g., `01J8ABC:01J8XYZ:v3`)

### CRDT Lifecycle

1. **onLoadDocument:** Load `blocks.ydocument` base + apply all `block_updates` in order
2. **onChange:** (empty — debounced to onStoreDocument)
3. **onStoreDocument:** Extract `title_v0` → plain text → update `blocks.title`; Extract `content_v0` → plain text → update `blocks.text`; Save incremental Yjs update to `block_updates` (one per day, appended)
4. **Compaction:** Periodically merge all `block_updates` into `blocks.ydocument`, increment `sync_version`, delete old updates

---

## 4. ProseMirror Schema (Content Nodes)

### Block Nodes

| Node | Attrs | Notes |
|------|-------|-------|
| `doc` | — | Root: `title block*` |
| `title` | — | Custom node for block titles |
| `paragraph` | — | Standard paragraph |
| `heading` | `level: 1-6` | H1-H6 |
| `codeBlock` | `language` | Fenced code |
| `blockquote` | — | |
| `bulletList` / `orderedList` | `tight`, `start`, `type` | |
| `listItem` / `todo_item` | `checked` (todo) | |
| `todo_list` | — | |
| `horizontalRule` | — | |
| `file` | `files: []` | File attachment embed |

### Inline Nodes

| Node | Attrs | Notes |
|------|-------|-------|
| `pagelink` | `blockKey` | **Block-to-block link** (navigational) |
| `reference` | `id`, `label`, `type: "mention"` | **Mention node** — inline reference to another block |
| `referenceSearch` | `type: "mention"` | Active mention search input |
| `hardBreak` | — | |

### Marks

`link` (href, target), `textStyle` (color), `bold`, `code`, `italic`, `strike`

---

## 5. REST API Endpoints (Port 1233)

### Auth
| Method | Route | Notes |
|--------|-------|-------|
| POST | `/auth/register` | (frontend route) |
| POST | `/auth/login` | (frontend route) |
| POST | `/auth/create_session` | |
| POST | `/auth/verify_key` | EdDSA public key verification |
| POST | `/auth/refresh_token` | |
| POST | `/auth/verify_email` | |

### Block CRUD
| Method | Route | Notes |
|--------|-------|-------|
| POST | `/api/insert_block` | Create a new block |
| POST | `/api/update_block` | Update block metadata |
| POST | `/api/delete_block` | Soft delete |
| POST | `/api/move_block` | Move between workspaces |
| POST | `/api/order_blocks` | Reorder blocks |
| GET | `/api/get_children` | Get child blocks |
| GET | `/api/query_blocks` | Query/filter blocks |
| GET | `/api/search_blocks` | Full-text search |
| GET | `/api/graph_blocks` | Get blocks for graph view |
| GET | `/api/navigation_blocks` | Sidebar navigation |
| GET | `/api/block_title` | Get block title only |
| GET | `/api/block_link` | Get block link metadata |
| GET | `/api/blocks_for_app` | Get blocks filtered by app |

### Heap
| Method | Route | Notes |
|--------|-------|-------|
| GET | `/api/heap` | Get the heap (reverse-chronological block feed) |
| POST | `/api/heap/add` | Add block to heap |
| GET | `/api/heap/changed_blocks` | Get recently changed blocks |

### Views & Attributes
| Method | Route | Notes |
|--------|-------|-------|
| POST | `/api/update_app_settings` | Update view/app config |
| POST | `/api/update_view` | Update view |
| POST | `/api/update_attrs` | Update block attributes |
| POST | `/api/add_available_column` | Add column to table view |

### Sync (Hocuspocus internal)
| Method | Route | Notes |
|--------|-------|-------|
| POST | `/api/ydoc/update` | Add file or thumbnails to YDoc (`{ operation: "add_file" \| "add_thumbnails" }`) |
| POST | `/api/ydoc/compact` | Trigger YDoc compaction |
| POST | `/sync/upload-complete` | Mark file upload as complete |
| GET | `/sync/health` | Health check |

### Workspaces & Admin
| Method | Route | Notes |
|--------|-------|-------|
| GET | `/api/workspaces` | List workspaces |
| GET | `/api/me` | Current user profile |
| POST | `/api/admin/invite_user` | Admin invite |
| GET | `/api/admin/pending-users` | |
| GET | `/api/admin/users` | |

---

## 6. Key Architectural Insights for A2UI Integration

### Tags vs. Mentions — The Critical Distinction

| Concept | Heaper Implementation | Nostra Mapping |
|---------|----------------------|----------------|
| **Tag** | `block_relations` row with `type = 'tag'`, synced to `relations_v1` Y.Map | `SpaceGraph Edge` (structural) |
| **Mention** | Inline ProseMirror `reference` node with `type: "mention"`, `id`, `label` | Inline `ContributionRef` |
| **Page Link** | Inline ProseMirror `pagelink` node with `blockKey` | `SpaceGraph Edge` (navigational) |

**Key takeaway:** Tags are **structural metadata** (stored in both Postgres AND the Yjs doc). Mentions are **content-level references** (stored only inside the ProseMirror document within Yjs).

### File Deduplication

Files use SHA256 `hash` as the deduplication key. The `files` table has a unique constraint on `(block_id, hash)`. In the Yjs document, files are stored in `files_v0` keyed by `hash:fileSize`.

### CRDT Storage Strategy

Heaper does NOT make the entire block a single CRDT. Instead it uses a **dual-layer approach**:
1. **Yjs layer** (`ydocument` bytea + incremental `block_updates`): Handles real-time collaborative editing of title, content, relations, and file metadata
2. **Postgres layer** (denormalized `title`, `text`, `text_doc` columns): Extracted plain-text and ProseMirror JSON snapshots for full-text search, API queries, and non-collaborative reads

This means the Yjs document is the **source of truth** for content, while Postgres acts as a **search and query cache**.
