# Gate Summary Heap Blocks (SIQ + Testing)

## Purpose
Provide a **heap-block-compatible**, rolling “latest” representation of operational gate results so operators can capture the current SIQ/Testing status in the Heap without bespoke frontend widgets.

This is an enrichment layer: Workbench remains the primary drill-in surface; Heap blocks are compact summaries + pointers.

## Schema
- Structured payload schema id: `nostra.heap.block.gate_summary.v1`
- Canonical schema file: `shared/standards/heap/gate_summary_block.schema.json`

Required fields (normalized to snake_case in `structured_data`):
- `schema_id`, `kind`, `generated_at`, `latest_run_id`
- `overall_verdict`, `required_gates_pass`
- `counts` (object), `failures` (array)

Optional:
- `render_hints.primary_route` (Workbench drill-in)
- `raw` (included only when size-clamped)

## Emission Endpoint (Gateway)
`POST /api/system/gates/emit-heap-block` (operator role required)

Request (camelCase):
```jsonc
{
  "schemaVersion": "1.0.0",
  "workspaceId": "nostra-governance-v0", // hint (space/workspace)
  "kind": "siq",
  "artifactId": "gate_summary_siq_latest"
}
```

Defaults:
- `artifactId` omitted → `gate_summary_siq_latest` or `gate_summary_testing_latest` depending on `kind`

Response:
```jsonc
{
  "schemaVersion": "1.0.0",
  "accepted": true,
  "kind": "siq",
  "workspaceId": "nostra-governance-v0",
  "heapWorkspaceId": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "artifactId": "gate_summary_siq_latest",
  "blockId": "01ARZ3NDEKTSV4RRFFQ69G5FAW",
  "emittedAt": "2026-03-04T19:12:34Z"
}
```

Notes:
- `workspaceId` is accepted as either a Heap workspace ULID or a non-ULID “space/workspace hint”.
- When `workspaceId` is non-ULID, the gateway resolves a deterministic `heapWorkspaceId`:
  - If env override exists: `CORTEX_HEAP_WORKSPACE_ID_FOR_SPACE_<SPACE_ID_UPPER_SANITIZED>` → use that ULID.
  - Else derive a deterministic ULID from seed: `heap_workspace:<workspaceId_hint>` (empty hint → `heap_workspace:default`).
- `artifactId` is the rolling “latest” identifier (stable).
- `blockId` is a **deterministic ULID** derived from the emit request id (stable for identical inputs) and satisfies Heap ULID validation.

Error codes:
- `INVALID_GATE_KIND` (400)
- `GATE_ARTIFACT_MISSING` (503)
- `GATE_ARTIFACT_INVALID` (422)
- `GATE_SUMMARY_HEAP_EMIT_FORBIDDEN` (403)

## Audit Trail
Each successful emit appends a structured JSONL audit event:
- Path: `${NOSTRA_DECISION_SURFACE_LOG_DIR}/actions/gate_emit_audit.jsonl` (falls back under `logs/system/decision_surfaces/actions/`)
- Logs surface: exposed via the curated logs stream id `gate_emit_audit`

## Artifact Sources (Allowlisted)
Emission reads rolling “latest” artifacts only (no filesystem browsing):
- SIQ: `NOSTRA_SIQ_LOG_DIR/siq_gate_summary_latest.json`
- Testing: `NOSTRA_TESTING_LOG_DIR/test_gate_summary_latest.json`

## Workbench UX Hook
Workbench surfaces can project a primitives-only button that triggers the frontend action handler:
- Action descriptor: `emitGateSummaryToHeap?kind=<siq|testing>&workspaceId=<space_id>`

## Heap Focus (Optional UX)
The Heap UI supports selecting a block via query param:
- `/heap?focus=gate_summary_siq_latest`
