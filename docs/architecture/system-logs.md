# System Logs (Curated Streams + Cursor Tail)

## Purpose
Provide a **principle-aligned** operational logs surface for Cortex Workbench:

- **Curated allowlist** of structured system streams (no filesystem browsing).
- **Deterministic inventory** ordering.
- **Role-gated** access per stream.
- **Cursor-based tail** over HTTP (poll + byte-offset cursor).

This supports production handoff by making `/logs` useful without violating governed discovery or dynamic source constraints.

## APIs

### Inventory
`GET /api/system/logs/streams`

Returns stable-sorted stream descriptors:

- `streamId`, `label`, `format` (`json`, `jsonl`, `text`)
- `requiredRole`
- `source` (the governing `NOSTRA_*_LOG_DIR` root)
- `description`

### Tail
`GET /api/system/logs/streams/:stream_id/tail?cursor=<u64>&limit=<u32>`

Cursor semantics:

- For `jsonl`/`text`, `cursor` is a **byte offset** into an append-only file.
- If the file shrinks or `cursor` exceeds file length, the server resets to `0` and returns `reset=true`.
- `nextCursor` returns the next byte offset to request.

Events are projected as:

- `ts`, `level`, `subsystem`, `message`
- `raw` (parsed JSON for `jsonl`/`json`)
- `rawTextLine` (only when a line cannot be parsed as JSON or for `text`)

## Stream Policy
- Stream IDs map to **fixed paths** resolved from existing `NOSTRA_*_LOG_DIR` configuration.
- Unknown `streamId` returns `404`.
- Streams with elevated requirements (e.g. steward audit) return `403` to lower roles.

## Workbench Projection
`/logs` (Workbench A2UI) consumes the inventory + tail endpoints and selects a stream via:

- `node_id=log_stream:<stream_id>:cursor:<u64>`

The Workbench projection is read-only; structural promotion and governance actions remain steward-gated.
