# Eudaemon Alpha - Workspace Resolution

Date: 2026-03-12

## Resolved Answer

Eudaemon should use the existing Cortex heap workspace, not an artifact-only workaround and not a newly invented primitive.

## What Changed

The previous version of this document assumed there was no canonical heap board yet and therefore recommended modeling agent scratch work as artifact-like contributions. That is no longer the correct stage architecture.

Initiative 124 now provides the working surface:
- `POST /api/cortex/studio/heap/emit`
- `GET /api/cortex/studio/heap/blocks`
- `POST /api/cortex/studio/heap/blocks/context`

## Current Workspace Model

1. Space-level governance still matters:
   - the agent belongs to a governed space
   - capability overlays and DPub activation remain space-scoped

2. Working material belongs on the heap:
   - notes
   - charts
   - scratch explorations
   - pointers
   - A2UI surfaces

3. Durable outputs are promoted out of the heap:
   - proposal
   - DPub chapter update
   - workflow draft
   - other governed contribution

## Why This Is Better

1. It uses the runtime primitives Cortex already implements.
2. It keeps exploratory work in the execution layer, where mutability and iteration belong.
3. It preserves Nostra authority for promoted contributions and initiative lineage.
4. It avoids inventing a second agent-only workspace model before Initiative 132 can even bind to the current stack.

## Canonical Direction

For Initiative 132, the heap is the working board.
The research initiative remains the governing plan.
Promoted outputs become governed contributions.

See `WORK_PRIMITIVES_ARCHITECTURE.md` and `PLAN.md` for the full architecture.
