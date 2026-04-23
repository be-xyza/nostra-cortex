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
