---
status: in-progress
portfolio_role: anchor
stewardship:
  layer: "Architectural"
  primary_steward: "Systems Steward"
  domain: "Infrastructure"
---
# Plan: Implementing the Nostra Library System Upgrade

> **Status**: IN-PROGRESS
> **Supersedes**: Previous PLAN.md
> **Driven By**: [STRATEGIC_ANALYSIS_REPORT.md](./STRATEGIC_ANALYSIS_REPORT.md)

## 1. Goal Description
To operationalize the "Book" and "Library" as a dynamic, modular knowledge system. This moves us from "Static JSONs" to a **Graph-Backed, VFS-Managed, Agent-Governed** architecture.

Key Enhancements:
*   **Production**: Users can Write Books via the **Artifacts Editor** (VFS).
*   **Consumption**: Users typically Read **Dynamic Canister Data**, not static files.
*   **Maintenance**: **Glass Box Agents** maintain the library, with visible logic traces.
*   **V1 Doctrine**: `CONSOLIDATED_SPEC_V1.md` + `ACCEPTANCE_CHECKLIST_V1.md` are the canonical V1 gates.

## 2. User Review Required
> [!IMPORTANT]
> **VFS Architecture**: This plan assumes we are building the VFS layer (`nostra/backend/vfs`) as a core module.
> **Economic Model**: We are implementing the "Tiered Model" (Private Gas Tanks + Public Subsidies).
> **Sync Policy**: One-way export (local → DPub) until a merge policy is approved by the workflow engine.
> **Snapshot Contract**: Define `SnapshotManifest` + `DocsBundle` schemas before syndication.

## 3. Proposed Changes

### Phase 0: Snapshot Contract (Docs Bundle + Manifest)
**Goal**: A deterministic, portable snapshot for Cortex and agents.
*   **Define** `SnapshotManifest` with commit hash, build time, bundle version, schema versions, file index, and checksums.
*   **Define** `DocsBundle` inclusion/exclusion rules (paths, `_archive` handling, size limits, binary policy).
*   **Resolver**: Rewrite `file:///` references to bundle-relative paths during export.
*   **Publishable Artifact**: Produce `snapshot_manifest.json` + `snapshot.json` + bundle archive for DPub payloads.
*   **Schemas**: `SNAPSHOT_MANIFEST_SCHEMA_V1.json` + `DOCS_BUNDLE_SCHEMA_V1.json` as V1 contract sources.

### Phase 1: The "Write" Cycle (Creation & VFS) ✅
**Goal**: Enable users to creat Artifacts (Books/Docs) in a familiar "File" interface.

#### [DONE] `nostra/backend/modules/vfs.mo`
*   Implement the `VirtualFileSystem` actor.
*   **Capabilities**: `mount(path, canister_id)`, `read(path)`, `write(path)`.
*   **Storage**: Map logical paths to `ContributionId`s or `BlobId`s.

#### [MODIFY] `nostra/frontend/src/labs/artifacts_editor.rs`
*   **New Component**: `ArtifactsEditor`.
*   **Features**:
    *   File Tree Sidebar (browsing VFS).
    *   Markdown Editor (Prosemirror/Monaco).
    *   **NostraBlock** support: Embedding `@[id]` references.

### Phase 2: The "Structure" Cycle (Dynamic Rendering)
**Goal**: Remove static JSONs. Render Library from Graph.

#### [MODIFY] `nostra/frontend/src/labs/library_lab.rs`
*   **Refactor**: Remove `load_books_from_json`.
*   **New Logic**:
    *   Query `VFS` for `/lib/books`.
    *   Resolve `BookManifest` (Ordered Array of Chapters).
    *   Fetch `Chapter` nodes from Graph (`node_provider`).
    *   Render `NostraBlock` stream.

#### [NEW] `nostra/backend/data_layer/schema/manifest.mo`
*   Define the `BookManifest` type (Ordered Sequence).

### Phase 2.25: Lifecycle & Governance (Publishing)
**Goal**: Safe distribution and deterministic references.
*   **Roles**: Define who can publish snapshots and how approvals are recorded.
*   **Pinning**: Provide `latest` plus commit-pinned snapshot refs.
*   **Rollback**: Specify rollback strategy and snapshot retention window.
*   **Conflict Policy**: Read-only until workflow-engine merge rules are approved.

### Phase 2.5: The "Syndication" Cycle (Federation)
**Goal**: DPubs must be discoverable and subscribable without centralized dependencies.
*   **Feed Interface**: Implement `get_feed()` query as canonical JSON feed on DPub Manifest.
*   **RSS/Atom**: Implement stateless `to_xml()` transform as a derived client-side/edge view only.

### Phase 3: The "Automate" Cycle (Agent Governance)
**Goal**: Glass Box Agents & Rights Management.
*   **Rights Logic**: Implement "Smart License" verification in the Editor (warn if mixing CC-BY with Copyright).
*   **Attribution**: Implement cryptographic signing of Chapters.
**Goal**: Glass Box Agents.

#### [NEW] `nostra/frontend/src/cortex/components/inspector_trace.rs`
*   **Component**: `AgentTraceView`.
*   **Logic**:
    *   Fetch `Decision` node from Graph.
    *   Visualize "Chain of Thought".
    *   Link to `CONFIG.md` in VFS.

#### [MODIFY] `nostra/frontend/src/cortex/inspector.rs`
*   Integrate `AgentTraceView` into the Constitutional Audit panel.
*   Add "View Config" button linking to `ArtifactsEditor`.

### Phase 4: The "Resilience" Cycle (Censorship Resistance)
**Goal**: Make DPubs un-killable.
*   **Gateway Interface**: Define `StorageProvider` definition in `067`.
*   **Export Tool**: Build `dpub-to-epub` WASM converter for client-side backup.

## 4. Verification Plan

### Automated Tests
*   `dfx test nostra_backend`: verify VFS mount/read/write logic.
*   `cargo test frontend`: verify `Manifest` parsing.
*   **Snapshot Validation**: verify `SnapshotManifest` schema + checksums.
*   **Reference Resolution**: verify no unresolved `file:///` links in bundle.

### Manual Verification
1.  **Write Test**: Open Artifacts Editor, create `/test/my-book.md`. Save.
2.  **Snapshot Build**: Export a snapshot and confirm manifest fields are populated.
3.  **Pin Test**: Load a commit-pinned snapshot in Cortex Desktop.
4.  **Link Test**: Open a document with a resolved `file:///` reference.

---

## 5. Strategic Alignment: VFS as Glue

To support Cortex Desktop, we explicitly map Nostra Primitives to VFS representations:

| Nostra Primitive | VFS Representation | User Action |
| :--- | :--- | :--- |
| **Space** | Directory / Volume | `mount /space` |
| **Contribution** | File (Typed, Versioned) | `open proposal.md` |
| **Fork** | Branch / Copy with Lineage | `cp -r space/ space-fork/` |
| **Event Log** | Append-only Log File | `tail -f .events` |
| **Governance** | Policy File | `cat .config/gov.yaml` |
| **Provenance** | Extended Attributes (xattrs) | `ls -l` (shows signer) |

This mapping allows "Boring Tools" (File Explorers, Diff Viewers) to interact with high-level Constitutional Primitives.
2.  **Read Test**: Open Library Lab, verify `my-book` appears in "New Arrivals".
3.  **Agent Test**: Trigger "Librarian".
    *   Open Cortex Inspector.
    *   Click "Constitutional Audit".
    *   Verify you see the "Trace" (Thought Process).
    *   Click "View Config" and ensure it opens the Editor.

---

## 5. Constitutional Alignment

| Principle | Constitution | Alignment |
| :--- | :--- | :--- |
| **Everything is a Contribution** | Knowledge Integrity §17 | DPub, Chapters, Manifests are all versioned Contributions. |
| **Forking is Constitutional** | Knowledge Integrity §17 | DPubs can be forked with preserved lineage. |
| **Event Sourcing** | Knowledge Integrity §17 | VFS operations are event-logged; replay supported. |
| **Glass Box Agents** | Agent Charter §8 | Agent decisions surfaced via Inspector Trace. |
| **Capability Containment** | Agent Charter §19 | Agent logic traces visible; no hidden authority. |
| **Censorship Resistance** | Spaces §4 | Federation and export tools preserve user sovereignty. |

### Cross-References

- [Day-0 Primitives](file:///Users/xaoj/ICP/research/034-nostra-labs/NOSTRA_KNOWLEDGE_INTEGRITY_MEMORY_DOCTRINE.md) (§17)
- [Agent Charter](file:///Users/xaoj/ICP/research/034-nostra-labs/NOSTRA_AGENT_BEHAVIOR_AUTHORITY_CHARTER.md) (§19)
- [Spaces Constitution](file:///Users/xaoj/ICP/research/034-nostra-labs/NOSTRA_SPACES_CONSTITUTION.md)

## Alignment Addendum (Constitution + System Standards)

- Labs Constitution: Default to Production patterns unless explicitly labeled as Labs; experiments remain fork-safe and documented.
- Knowledge Integrity & Memory: Preserve lineage, log decisions, and avoid rewriting history; summaries are additive, not replacements.
- Spaces Constitution: All authority and data are space-scoped; cross-space effects are explicit links, not merges.
- Stewardship & Roles: Identify accountable roles per change; unclear authority defaults to recommendation-only.
- Contribution Lifecycle: Renames, merges, archives, and scope changes require explicit rationale and approval.
- Agent Behavior & Authority: Agents operate in observe/recommend/simulate unless execution is explicitly approved.
- Security & Privacy: Least authority, explicit consent, and scoped access; minimize sensitive data exposure.
- Governance & Escalation: Disputes and irreversible actions follow escalation pathways and steward review.
- UI/UX Manifesto: Interfaces must surface provenance, time, and agency; avoid dark patterns.
- Modularity: Strict interfaces, no hardcoded canister IDs, and clean boundary contracts.
- Composability: Actions are workflow-compatible and emit standard events.
- Data Confidence & Integrity: Confidence/reliability metadata is required where applicable.
- Portability: Data must be exportable and WASM-safe; avoid OS-specific dependencies in core logic.
- Durable Execution: State is persisted via stable memory; workflows are replayable.
- Visibility Decoupling: Index/search are async from source of truth.
- Outbox Pattern: External calls are queued with idempotency and retry semantics.
- Verification: Each initiative includes verification steps and records results.
