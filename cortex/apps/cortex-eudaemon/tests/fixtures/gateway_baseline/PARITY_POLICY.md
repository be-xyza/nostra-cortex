# Gateway Parity Policy

The gateway parity suite is a hard progression gate for runtime extraction work.

- No extraction phase advances unless parity suite passes.
- Parity fixtures are golden; any intentional contract drift must update fixtures and include rationale in Research 118 decisions.
- Representative endpoint classes must remain covered: system, cortex, acp, workflow, metrics, testing, kg, and websocket.
- Inventory lock is mandatory: `inventory_count == fixture_count + approved_exemptions_count`.
- `approved_exemptions_count` defaults to `0`. Any non-zero exemption requires explicit Research 118 rationale.
- Nondeterministic endpoints must declare normalization rules (`mode`, `ignored_fields`, `required_keys`) in each fixture.
- Endpoint change protocol: any endpoint add/remove/rename requires synchronized updates in the same PR for:
  - `endpoint_inventory.tsv`
  - `endpoint_inventory.json`
  - `parity_cases/*.json`

## Command Contract

- Non-mutating inventory/fixture sync check:
  - `bash scripts/check_gateway_parity_inventory_sync.sh`
- Manual inventory + parity fixture refresh (local maintenance only):
  - `python3 scripts/refresh_gateway_parity_fixtures.py`
- Full pre-Phase-2 freeze gate run:
  - `bash scripts/run_cortex_runtime_freeze_gates.sh`

> CI must run checks only (no auto-write of fixtures). Fixture regeneration is a deliberate local maintenance action.
