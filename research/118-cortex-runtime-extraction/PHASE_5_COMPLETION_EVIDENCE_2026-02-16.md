# Initiative 118 - Phase 5 Completion Evidence

Date: 2026-02-16

## Scope Closed

Phase 5 gateway extraction closure criteria are implemented in workspace:

1. Runtime gateway envelope/model foundation completed:
   - Added route-template and path-parameter support in request envelopes.
   - Added route metadata resolution path (`resolve_route`) through `GatewayHostAdapter`.
   - Added deterministic template matcher + precedence controls in `cortex-runtime`.
2. Full API runtime dispatch path enabled in Desktop gateway:
   - Added `/api/*` runtime-dispatch middleware at gateway router boundary.
   - All API requests now pass through `cortex_runtime::gateway::GatewayDispatcher` before host route execution.
   - Legacy host execution path is preserved using loopback dispatch with explicit bypass header (`x-cortex-runtime-legacy-bypass`).
3. Contract-derived route metadata wiring:
   - Desktop runtime host now loads `GATEWAY_PROTOCOL_CONTRACT_2026-02-16.json`.
   - Route matching resolves template + params deterministically for inventory endpoints.
   - Transaction boundary, event-emission defaults, and idempotency semantics are sourced from the contract registry.
4. Local gateway singleton callsites removed from target production flows:
   - `gateway/server.rs` no longer calls `get_gateway()` directly.
   - `resilience_service.rs` no longer calls `get_gateway()` directly.
   - `agent_service.rs` no longer calls `get_gateway()` directly.
   - Runtime-host wrappers mediate queue/export/probe/offline queueing flows.
5. Route worklist artifact generated and partition-locked:
   - `research/118-cortex-runtime-extraction/PHASE_5_ROUTE_WORKLIST_2026-02-16.md`

## New/Updated Validation Tests

1. Runtime matcher tests (`cortex-runtime`):
   - one-param template resolution
   - two-param template resolution
   - static-over-param precedence
   - ambiguous template rejection
2. Desktop runtime-host tests:
   - transaction-boundary count lock (60/7/2/52/2)
   - inventory template resolution without ambiguity

## Validation Commands (Executed)

1. `cargo check --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-runtime`
   - PASS
2. `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-runtime`
   - PASS
3. `cargo check --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-desktop --bin gateway_server`
   - PASS
4. `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-desktop runtime_host::tests`
   - PASS
5. `bash /Users/xaoj/ICP/scripts/check_gateway_parity_inventory_sync.sh`
   - PASS (`inventory=123 fixtures=123 exemptions=0`)
6. `bash /Users/xaoj/ICP/scripts/check_gateway_protocol_contract_coverage.sh`
   - PASS (`inventory=123 contract=123`)
7. `bash /Users/xaoj/ICP/scripts/run_cortex_runtime_freeze_gates.sh`
   - PASS
8. `bash /Users/xaoj/ICP/scripts/check_118_pr_evidence.sh --pr-body-file /Users/xaoj/ICP/tests/fixtures/pr_evidence/valid.md`
   - PASS
9. `bash /Users/xaoj/ICP/tests/scripts/test_check_118_pr_evidence.sh`
   - PASS (negative fixtures fail as expected)

## Governance Notes

1. ADR-017 evidence discipline remains active for merge workflow.
2. Contract/inventory lock invariants remain unchanged (`123/123`, zero exemptions).
3. This artifact records local completion evidence; PR evidence bundle still requires run links + steward records under active governance rules.
