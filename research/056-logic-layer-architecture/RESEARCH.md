---
id: '056'
name: logic-layer-architecture
title: Logic Layer Architecture Research
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Logic Layer Architecture Research

## Objective
To investigate, validate, and design a "logic layer" for Nostra/Cortex that provides flexible, relational data modeling, formulas, workflows, and rules. This layer must fit Nostra's "OS-Native" and "Canister-First" architecture, moving beyond traditional SaaS limitations.

## Core Findings
- **Baserow** is the correct **Strategic Core**. Its "Schema as Data" philosophy aligns with Nostra's need for an evolving Knowledge Graph.
- **NocoDB** provides excellent **Tactical Reference** for "UI as a Viewport," separating the grid interface from the data source.
- **Rules Engines** (like `json-rules-engine`) follow standard patterns that can be adapted to Rust/Wasm.

> - [060 - MemEvolve Integration](../060-memevolve-integration/RESEARCH.md)

## Architectural Analysis
### 1. Schema & Data Layer (The "Baserow" Influence)
Baserow uses a robust Django model system to define fields. For Nostra/ICP, we must adapt this to **Rust Structures** and **Stable Memory**.
- **Pattern**: `FieldType` definitions separate storage (`models.py`) from logic (`formula`).
- **Nostra Adaptation**:
    - **Schema**: Store user-defined schemas as serialized data in Stable Memory (using `ic-stable-structures` or `Graphiti`).
    - **Fields**: Implement a standard `Field` trait in Rust that handles serialization, validation (`prepare_value_for_db`), and formula type conversion.

### 2. Logic & Formula Engine
Baserow uses **ANTLR4** to parse formulas into an AST, then compiles them to Python expressions or SQL.
- **Nostra Adaptation**:
    - **Parser**: Use a Rust-based parser (e.g., `pest` or `nom`, or port the ANTLR grammar) to generate an AST.
    - **Execution**: Compile the AST to **WASM** or interpret it safely within the Canister. This ensures deterministic execution, crucial for ICP.
    - **Constraint**: Must handle "Gas" (Cycles) limits for complex formulas (recursion depth limits).

### 3. Workflow & Event Bus
Nostra already uses **Temporal** (Initiative 047) for durable execution. The missing piece is the **Trigger**.
- **Pattern**: Event Sourcing.
- **Nostra Adaptation**:
    - **Event Bus**: A lightweight internal bus (or `ic-event-hub`) that allows "Rules" to subscribe to "Data Changes".
    - **Bridge**: When a Rule triggers, it spawns a **Temporal Workflow**.

## Proposed Architecture: The "Logic Substrate"
### Layer 1: The Graph (Storage)
- **Technology**: **NostraGraph** (Custom Rust Canister using `ic-stable-structures`).
- **Role**: Store Entities, Links, and Properties properties as nodes/edges.
- **Reference**: Aligns with `040-nostra-schema-standards` (SpgType) and `053` (Elastic Agent).
- **Note**: "Graphiti" is used only as a reference for the *Search Algorithm* (Layer 2), not the Persistence Layer.

### Layer 2: The Engine (Compute)
- **Technology**: Rust Canister (Logic Service).
- **Role**:
    - Validates writes against the user-defined Schema.
    - Evaluates Formulas (Computed Fields).
    - Checks "Trigger Conditions" (e.g., `Field > X`).

### Layer 3: The Viewport (UI)
- **Technology**: **Lit + Lion + Shoelace** (Web) / **Dioxus** (Native).
- **Role**: Renders the "Table View" or "Kanban View" efficiently using virtualization (like NocoDB/Grid.js), but purely as a projection of Layer 1.
- **Reference**: See `055-data-grid-analysis` for detailed rendering, pipelining, and virtualization strategies.

## Deep UX/UI & Architectural Patterns (Cycle 1-15 Synthesis)

### 1. Rendering Strategy (NocoDB Pattern)
Instead of using heavy 3rd party libraries, we should adopt NocoDB's **Manual Virtualization** (`InfiniteTable.vue`):
- **Chunking**: Fetch data in chunks of 50/100 (`fetchChunk`).
- **Caching**: Store retrieved rows in a client-side `Map<rowIndex, Row>` (`cachedRows`).
- **Placeholder Math**: Manually calculate `placeholderStartRows` and `placeholderEndRows` based on `scrollTop` and fixed `rowHeight`.
- **Why**: This gives us pixel-perfect control over the "Skeleton" state which is critical for the "Premium" feel (Rule #4 of Web Dev).

### 2. Logic Engine (Baserow Pattern)
For Formulas and dynamic logic, we must use an **AST Interpreter** (`baserow/contrib/database/formula/ast`):
- **Structure**: `BaserowExpression` (Base) -> `BaserowFunctionCall` (Node) -> `BaserowFieldReference` (Leaf).
- **Typing**: Strict typing (`BaserowFormulaValidType`) to ensure formulas like `concat(number, text)` fail gracefully or auto-cast.
- **Portability**: This AST structure is easily portable to Rust (Layer 2) for high-performance execution.

### 3. History & Audit (Baserow Pattern)
- **Pattern**: `RowHistoryHandler` with a `Provider` registry.
- **Implementation**: Every Action (Create/Update/Delete) emits a signal. The Handler catches it, serializes the *Diff*, and stores it in an append-only log.
- **Nostra Adaptation**: This perfectly matches the **Event Log** in the `NostraGraph` canister.

### 4. Integration & Webhooks (NocoDB Pattern)
- **UI**: A single "Webhook Drawer" that switches schema based on `Type` (Slack/Discord/URL).
- **Validation**: Client-side validation of payloads (detecting cyclic calls to self).
- **Testing**: Built-in "Test Webhook" button using sample data (`testWebhook`).

### 5. Interaction State (Optimistic UI)
Baserow uses a "hybrid" approach:
- **Immediate**: Update the local Vuex store (UI updates instantly).
- **Async**: Send request to backend.
- **Rollback**: If backend fails, revert the Vuex store.
- **Recommendation**: This is mandatory for a "smooth" feel on ICP (where calls take ~2s). We cannot wait for consensus for every keystroke.

## Extended Architecture Analysis (Cycles 16-30 Synthesis)

### 6. View Implementations (NocoDB & Baserow)
- **Kanban (NocoDB)**: Uses a `useKanbanViewStore` composable that maintains a `Map<stackId, Row[]>`. Drag-and-drop triggers local state updates (Optimistic UI) followed by an API call via a Command Pattern for Undo/Redo.
    - *Insight*: State must be grouped by the "Stacking Field" locally to avoid constant re-fetches.
- **Calendar (NocoDB)**: Uses a "Range-based" fetch strategy (`from_date`, `to_date`). The store manages `activeCalendarView` (Month/Week/Day) and generates SQL filters dynamically.
- **Gallery (Baserow)**: Implemented as a standard Paginated View but with a specific `GalleryLimitOffsetPagination` and `field_options` (cover image, visible fields). The UI is just a rendering variation of the Grid.
- **Map (NocoDB)**: Uses a dedicated `MapType` store. It requires a `GeoData` column. Updates are broadcast via `updateMapMeta`.
- **ERD (NocoDB)**: A read-only visualization that fetches *all* Table Metas and Relations at once. It does not load row data, only schema.

### 7. Signals & Real-time (Baserow)
- **Pattern**: Django Signals + Celery + WebSockets.
- **Flow**:
    1.  Model Change (e.g., `row.save()`).
    2.  Django Signal (`@receiver`) triggers `transaction.on_commit`.
    3.  On Commit: Queue a Celery Task (`broadcast_to_group`).
    4.  Worker: Pushes message to Redis/Combustion channel.
    5.  WebSocket Server: Broadcasts to connected clients.
- **Nostra Adaptation**: We can simulate this using `ic-event-hub` or pure standard PubSub within the canister, emitting "Update Events" that the Frontend polls or subscribes to.

### 8. Batch Operations (Baserow)
- **Pattern**: `ActionType` registry.
- **Implementation**: `BatchRowsView` accepts a list of items. It delegates to `CreateRowsActionType.do()`.
- **Optimization**: It uses `bulk_create` (SQL) for performance but wraps it in a loop if Signals are required for each row (trade-off between speed and reactivity).
- **UX**: The API supports `client_session_id` and `client_undo_redo_action_group_id` to group batch operations into a single Undo step.

### 9. Sync & Connectors (NocoDB)
- **Pattern**: `SyncSource` model + `SyncService`.
- **Architecture**: A `SyncSource` configures the connection (MySQL/PG). The Service runs a "Sync Job" (ETL) that reads the external source and upserts into the internal NocoDB tables (`nc_...`).
- **Insight**: They do not query external sources live for Views (usually). They "Cache" the data into NocoDB tables to enable their high-performance features (Formula, Virtual Columns). "External View" is a separate concept.

### 10. Snapshot & Trash (Baserow)
- **Snapshots**: Full JSON serialization of the Application tree stored in S3/MinIO. It is a "Cold Backup".
- **Trash**: Soft Delete with a cascading `TrashEntry`. Restoring a parent (table) restores children (fields/rows) if they were deleted *with* the parent.


## Ultra-Deep Architecture Analysis (Cycles 31-45 Synthesis)

### 11. Formula Engine Strategy (Baserow vs. NocoDB)
- **Baserow (Recommended)**: Uses an **AST-based** approach (`function_defs.py`).
    - *Mechanism*: Parses formula string -> AST -> Recursive `to_django_expression()` calls.
    - *Functions*: Implemented as Python classes (`BaserowAdd`, `BaserowConcat`) translating directly to Database functions (e.g., `Coalesce`, `Cast`).
    - *Why Nostra?*: An AST is language-agnostic. We can parse the formula in Rust and compile it to WASM or optimized logic for the Canister.
- **NocoDB**: Uses a **Query Building** approach (`formulaQueryBuilderv2.ts`).
    - *Mechanism*: Recursively builds a KNEX query string.
    - *Risk*: Tightly coupled to SQL dialects. Harder to port to a generic "Logic Canister".

### 12. State Management (Baserow Vuex Pattern)
- **Structure**: Modules for `view`, `field`, `table`.
- **Optimization**:
    - `populateView(view)`: Decorates the API response with UI-only properties (`_` prefix like `_.loading`, `_.hover`). This separates "Server Data" from "UI State" cleanly.
    - **Optimistic Updates**: `createFilter` immediately adds to the store (`commit('ADD_FILTER')`), then calls API. On failure, it rolls back (`commit('DELETE_FILTER')`).
- **Lesson**: Nostra's frontend must assume success for speed (ICP latency) and rollback on consensus failure.

### 13. File Storage & Deduplication (Baserow Pattern)
- **Logic**: `UserFileHandler` (Cycle 44).
- **Deduplication**: Files are named by their **SHA256 Hash**.
    - If User A uploads `image.png` and User B uploads the same image, they point to the exact same file on S3/MinIO.
    - *Benefit*: Massive storage savings for duplicated assets.
- **Security**: NocoDB's `AuditsService` (Cycle 43) implements "Signed URLs" for attachments in audit logs to prevent permanent public access.

### 14. Audit & Activity Logs
- **Baserow**: `Action` based (Undo/Redo is the Audit Log).
- **NocoDB**: Explicit `Audit` model.
- **Architecture**:
    - **Signed URLs**: Attachments in logs must be re-signed on access (Cycle 43). Static links in logs are a security risk.

### 15. Plugin Architecture (Baserow)
- **Registry Pattern**: `registries.py` defines abstract base classes (`Plugin`, `ApplicationType`).
- **Discovery**: Plugins register themselves at startup.
- **Nostra Adaptation**: A "Registry Canister" that allows developers to deploy "Logic Modules" (WASM) that the main Graph Canister can call.


## Omega Synthesis (Cycles 46-60)

### 16. Application Builder vs. Table View
- **Baserow**: Distinct `DataSource` model connects "Pages" to "Services" (Tables).
    - *Insight*: Do not mix "UI Building" with "data modeling". The "Builder" should be a separate mode that *consumes* the Graph, rather than being part of the Grid View.

### 17. System Reliability & Realtime
- **WebSockets**: Baserow uses `realTimeHandler.js` with **Exponential Backoff** (0s, 5s...) for reconnection.
    - *Adaptation*: Nostra needs a similar "Canister Watchdog" in the frontend to handle IC replica upgrades/downtime gracefully.
- **Trash**: Baserow uses **Periodic Tasks** (`Celery`) to clean old trash (30 days).
    - *Adaptation*: Use `IC Timers` (Heartbeat) to prune "Soft Deleted" graph nodes automatically.

### 18. User Experience Patterns
- **Onboarding**: Baserow uses a `Step Registry` pattern. State is simple (`completed_onboarding` boolean on User Profile).
- **Search**: NocoDB explicitly defines `isSearchableColumn`. avoiding "Search All" performance traps.
- **Color**: Baserow pre-defines 14 pastel palettes in `colors.js`.
    - *Recommendation*: Copy this palette. It allows algorithmic assignment of "Distinct but Pleasant" colors to users/tags without user input.

### 19. Developer Experience (DX)
- **OpenAPI**: NocoDB generates Swagger specs *dynamically* from the user-defined schema.
    - *Strategic Value*: Nostra should generate `candid` and `OpenAPI` specs for every user-created "Space", turning it into a programmable API instantly.

### 20. Configuration & Licensing
- **Config**: NocoDB's `NcConfig` class unifies Env vars and SQLite meta-data.
- **Licensing**: Baserow calculates "Seat Usage" dynamically based on Roles.
    - *Nostra*: In a DAO context, "Seats" are likely "Staked Neurons" or "Gov tokens", but the *calculation logic* remains similar.

## Final Architectural Verdict (Cycles 1-60)

The **Baserow Backend (Django/Python)** provides the strongest reference for **Schema/Logic/Plugins** (The "Brain").
The **NocoDB Frontend (Vue/Nuxt)** provides the strongest reference for **Visualizations/Grid/SmartSheet** (The "Face").

**Nostra's Path:**
1.  **Storage**: `ic-stable-structures` (Rust) mimicking Baserow's `Field` pattern.
2.  **Logic**: WASM-based "Formula Engine" (AST-based like Baserow).
3.  **UI**: Virtualized Grid (like NocoDB) but backed by a Baserow-style `Vuex` store for optimistic updates.
4.  **Real-time**: `ic-event-hub` broadcasting "Row Signals" (Baserow pattern).
