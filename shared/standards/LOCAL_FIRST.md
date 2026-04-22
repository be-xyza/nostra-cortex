# Local-First Capability Standard (Draft)

**Type**: System Standard (Capability)
**Status**: DRAFT
**Scope**: Nostra (Platform) + Cortex (Execution)
**Related**: `shared/standards/TECHNOLOGY_NEUTRALITY.md`, `research/096-offline-sync`, `research/030-artifacts-editor`

---

## 1. Thesis

> **Local-first is a capability, not a substrate.**

Nostra’s **constitutional truth** remains the canonical Graph + Workflow History.
Local-first behavior is implemented via **Adapters** (Cortex + Web clients) that preserve user agency during network loss and safely reconcile with the canonical system.

This standard defines what “local-first” must mean inside Nostra/Cortex, without requiring CRDTs everywhere.

---

## 2. Definitions

### 2.1 Capability Levels

1. **Offline Read**: previously fetched state remains available offline.
2. **Offline Write (Queued)**: user intent is captured locally as queued mutations and replayed later.
3. **Offline Write (Drafted)**: local drafts can diverge and later be merged/forked via workflows.
4. **Real-Time Collaboration**: concurrent multi-user editing with live presence (typically CRDT-backed).

### 2.2 Canonical vs Local

- **Canonical**: the authoritative state in the Graph + Workflow History.
- **Local**: device-scoped state enabling UX continuity (drafts, caches, queued mutations).

Local state is never “truth” by itself; it is proposed intent until it is ratified by canonical workflows.

---

## 3. Required Invariants

### 3.1 Intent Fidelity
Local-first MUST capture the user’s intent in a stable, replayable form:
- Prefer workflow-compatible commands (e.g., KIP or other capability envelopes).
- Never store opaque UI diffs as the only representation of intent.

### 3.2 Idempotency & Replay Safety
Any queued mutation MUST include an idempotency key.
Replaying a queue after reconnect MUST be safe under retries, partial failures, and app restarts.

### 3.3 Causal Ordering (Per Space)
For a given Space, queued mutations MUST preserve the user’s local causal order.
If canonical history advanced while offline, reconciliation must surface one of:
- clean apply
- rejected (precondition failed)
- needs merge (workflow task)

### 3.4 Exportability
Local-first state MUST be exportable for debug + user trust:
- queued mutation log export (JSON)
- local drafts export (structured blocks / markdown)

---

## 4. Conflict Resolution: Preferred Strategy

Default strategy is **Async Drafts (Git-style)**:
- Local edits create draft snapshots.
- Canonical commit occurs via a workflow.
- Divergence triggers a merge/fork workflow with explicit provenance.

CRDT is OPTIONAL and should be used only for high-frequency collaboration surfaces (e.g., the artifacts editor) where the UX demands it.

---

## 5. CRDT Guidance (When It Is Worth It)

Use CRDTs when all are true:
- multi-user concurrent editing is a core workflow (not “nice to have”)
- offline edits must merge automatically with minimal friction
- the data model is document-like (not arbitrary graph mutation)

Recommended scope:
- artifacts editor document state (blocks + inline text), per `research/030-artifacts-editor`.

Non-goals:
- CRDT for the entire Contribution Graph
- CRDT for workflow history

---

## 6. Implementation Notes (Non-Normative)

- “Offline write” is typically implemented by a **LocalGateway** that:
  - persists the queue to IndexedDB (web) or a local store (desktop)
  - hydrates on startup
  - replays with backoff + observability
  - escalates conflicts into workflow tasks (A2UI surfaces)
