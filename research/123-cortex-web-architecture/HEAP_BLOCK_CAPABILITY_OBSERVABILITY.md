# Cortex Web Heap / Block Capability Observability

## Scope

This note defines the second observability slice for Cortex Web frontend hardening. It builds on the shell surface inventory and focuses only on Heap and block capabilities:

- Heap view modes
- Heap action zones
- block card actions
- block detail tabs
- create panel modes
- upload and extraction status surfaces
- steward gate visibility
- relation editor visibility
- chat overlay visibility
- comment sidebar visibility

It does not classify shell routes, settings information architecture, or route visibility. Those are covered by `SHELL_SURFACE_OBSERVABILITY.md`.

## Contract Fixture

The seed fixture is:

- `cortex/apps/cortex-web/src/store/heapBlockCapabilityInventory.fixture.json`

The local validator is:

- `python3 scripts/check_cortex_web_heap_block_capability_inventory.py`

The fixture is exposed in preview mode at:

- `/api/system/ux/heap-block-capability-inventory`

## Current Capability Map

The current Heap surface exposes these action zones:

- `heap_page_bar`
- `heap_selection_bar`
- `heap_card_menu`
- `heap_detail_header`
- `heap_detail_footer`

The fallback action plan exposes these actions:

- `create`
- `regenerate`
- `refine_selection`
- `export`
- `history`
- `publish`
- `synthesize`
- `pin`
- `delete`
- `discussion`
- `edit`

Create panel modes are:

- `create`
- `generate`
- `upload`
- `chat`
- `plan`

Detail tabs are:

- `preview`
- `relations`
- `attributes`
- `code`

## Current Gaps

High priority:

- Destructive delete is exposed in fallback action paths without a fixture-level confirmation invariant.

Medium priority:

- General edit and relation editing are semantically conflated.
- Comment sidebar state is local UI state, not governed Heap persistence.
- Chat overlay can intercept other controls.
- Regenerate is exposed in action plans but does not yet have a clear runtime command contract in the inventory.

## Follow-On Validation Targets

Next validation should add Playwright and/or contract tests for:

- action menu availability by selected count
- create/upload/chat overlay collisions
- destructive action confirmation metadata
- relation editor authority split from generic edit
- comment persistence authority
