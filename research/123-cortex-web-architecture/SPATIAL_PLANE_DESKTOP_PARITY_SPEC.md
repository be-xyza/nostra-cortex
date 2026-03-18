# SpatialPlane Desktop Parity Spec (Phase 5)

## Purpose
Define the renderer-agnostic contract required for desktop host parity with the web `SpatialPlane` implementation delivered in Phase 5.

## Scope
- In scope: command semantics, event semantics, replay invariants, and host compliance requirements.
- Out of scope: desktop renderer implementation details, collaboration sync, and governance authority changes.

## Contract Baseline
- `SpatialPlane` payload requires:
  - `plane_id`
  - `surface_class`
  - `commands[]`
- Supported ops:
  - `create_shape`
  - `update_shape`
  - `delete_shape`
  - `focus_bounds`
- Supported shape kinds:
  - `note`
  - `arrow`
- Execution-only mutating behavior:
  - `surface_class` must be `execution` for mutating replay to execute.

## Command Semantics
1. `create_shape`:
   - Creates or replaces shape state by `shape.id`.
   - Duplicate create for same `id` in a stream resolves to latest command in sequence.
2. `update_shape`:
   - Applies partial updates on existing shape state by `shape.id`.
   - Update on unknown shape is ignored (no implicit creation).
3. `delete_shape`:
   - Deletes by `shape_id`.
   - Delete on unknown shape is a no-op.
4. `focus_bounds`:
   - Updates viewport target only.
   - Must not mutate shape state.

## Event Semantics (`cortex:a2ui:event`)
- Event types are locked to:
  - `button_click`
  - `approval`
  - `spatial_shape_click`
  - `spatial_adapter_loaded`
  - `spatial_adapter_fallback`
  - `spatial_adapter_replay`
  - `spatial_adapter_replay_failed`
- Required envelope fields:
  - `eventType`
  - `timestamp` (recorded at ingestion layer)
  - host/run metadata in gateway payload (`run_id`, `space_id`, `mode`, `surface_variant`, `host`)

## Renderer-Agnostic Behavior
1. Host must support deterministic command replay independent of renderer library.
2. Host must provide fallback rendering path when adapter is unavailable.
3. Host must classify adapter failures as:
   - `adapter_unavailable`
   - `adapter_replay_failed`
   - `contract_invalid`
4. Feature flags remain default-off in production paths.

## Replay Invariants
1. Deterministic:
   - Same command stream produces the same final shape state.
2. Idempotent:
   - Reapplying identical stream must not duplicate shape state.
3. Reconcile-first:
   - Existing canvas state is cleared/reconciled before applying replay snapshot.
4. Bounded validation:
   - Missing IDs, invalid dimensions, and out-of-bounds coordinates are rejected as contract errors.

## Desktop Host Compliance Checklist
1. Accept the same `SpatialPlane` payload as web.
2. Preserve command ordering semantics exactly.
3. Emit only locked `cortex:a2ui:event` event types.
4. Support evidence emission to:
   - `POST /api/cortex/viewspecs/experiments/spatial/events`
   - `GET /api/cortex/viewspecs/experiments/spatial/runs/:run_id`
5. Pass replay determinism and idempotence fixtures before implementation promotion.

## Cross-Link
- Governing initiative plan: `research/115-cortex-viewspec-governed-ui-synthesis/PLAN.md`
