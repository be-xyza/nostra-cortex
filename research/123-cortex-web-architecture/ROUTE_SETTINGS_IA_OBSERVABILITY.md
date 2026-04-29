# Route Settings IA Observability

## Scope

This note records the advisory route and settings information architecture contract for Cortex Web. It compares typed routes, seeded navigation entries, utility entries, and A2UI-only candidates without changing route behavior or adding a global settings page.

Authority mode: `recommendation_only`.

## Contract Surfaces

- Fixture: `cortex/apps/cortex-web/src/store/routeIaInventory.fixture.json`
- Type wrapper: `cortex/apps/cortex-web/src/store/routeIaInventory.ts`
- Snapshot id: `system:ux:route-ia-inventory`
- Preview endpoint: `/api/system/ux/route-ia-inventory`
- Validator: `scripts/check_cortex_web_route_ia_inventory.py`

The fixture records route owner, nav source, route class, typed-route status, A2UI fallback allowance, nav visibility, detail tabs, settings-like affordances, operator boundary, readiness, known gaps, and recommended actions.

## Settings Absence Contract

`/settings` remains missing in this stage. Do not build a global settings page yet. Settings-like affordances must declare their current owner route until global settings IA is implemented.

Current settings-like owner routes include:

- `/spaces` and `/spaces/:id`: Space identity, registry readiness, navigation plan, agent policy, and history.
- `/system/providers`: operator-only provider inventory, bindings, discovery, and runtime status.
- `/labs/space-studio`: draft design profile and template import experiments.
- `/heap`: saved views and local layout preferences remain route-owned.

## Current Route Truth

- `/contributions`, `/artifacts`, and `/workflows` are real surfaces but have different host models and readiness states.
- `/system/providers` is visible as an operator utility route and must not be generalized as settings.
- `/discovery` is visible in nav while still under construction.
- `/studio`, `/notifications`, `/metrics`, `/vfs`, `/siq`, `/memory`, `/simulation`, `/institutions`, and `/testing` are seeded A2UI-only candidates without typed route ownership.
- Labs routes remain draft/experiment surfaces.

## Follow-On Validation Targets

- Add route IA smoke coverage for `/spaces`, `/spaces/:id` tabs, `/contributions`, `/artifacts`, `/workflows`, `/system/providers`, and labs routes.
- Add an A2UI-only candidate exemption check so seeded hidden routes do not silently become product navigation.
- Keep `/settings` missing until the settings IA is intentionally designed.
