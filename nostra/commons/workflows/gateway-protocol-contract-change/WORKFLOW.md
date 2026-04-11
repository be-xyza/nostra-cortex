---
id: gateway-protocol-contract-change
title: Gateway Protocol Contract Change
owner: Systems Steward
updated_at: 2026-04-09
---

# Gateway Protocol Contract Change

## Purpose
Govern endpoint inventory, protocol contract coverage, and parity fixtures when runtime gateway routes or behaviors change.

## Triggers
- Edits to `cortex/apps/cortex-eudaemon/src/gateway/server.rs`
- Changes to gateway parity fixtures or `endpoint_inventory.*`
- Changes to `research/118-cortex-runtime-extraction/GATEWAY_PROTOCOL_CONTRACT_2026-02-16.json`

## Inputs
- The relevant initiative plan and decision record
- Current endpoint inventory and parity fixtures
- Gateway protocol contract JSON

## Lanes
- `inventory-only`: route additions/removals with no schema or behavior shift.
- `contract-change`: request/response, error, or transaction semantics changed.
- `compatibility-risk`: changes that may break fixtures, clients, or replay expectations.

## Analysis Focus
- Inventory parity across `server.rs`, TSV, JSON, fixtures, and exemptions.
- Protocol contract coverage for every `METHOD + PATH` pair.
- Backward-compatibility risk and whether replay or fixture regeneration is required.

## Steps
1. Rebuild the route inventory and compare it to the checked-in parity inventory.
2. Confirm the protocol contract still covers every `METHOD + PATH` pair.
3. Verify fixture count, approved exemptions, and parity-case expectations remain aligned.
4. Record whether the change is additive, schema-shifting, or backward-compatibility-sensitive.

## Outputs
- Updated gateway inventory/contract evidence
- Clear note on compatibility impact and required host follow-up

## Observability
- Track inventory count, fixture count, and exemption count.
- Record whether the change is additive, contract-shifting, or compatibility-sensitive.
- Capture any parity-case churn that signals unstable route ownership.

## Required Checks
- `bash scripts/check_gateway_parity_inventory_sync.sh`
- `bash scripts/check_gateway_protocol_contract_coverage.sh`
