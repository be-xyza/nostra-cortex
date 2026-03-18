# Initiative 118 — Gateway Protocol Contract (Phase 4.5)

Date: 2026-02-16

## Scope

- Defines the gateway request/response contract for all inventory-locked endpoints before Phase 5 extraction.
- Endpoint inventory source: `/Users/xaoj/ICP/cortex/apps/cortex-desktop/tests/fixtures/gateway_baseline/endpoint_inventory.tsv`
- Contract JSON artifact: `/Users/xaoj/ICP/research/118-cortex-runtime-extraction/GATEWAY_PROTOCOL_CONTRACT_2026-02-16.json`

## Contract Summary

- Total endpoints: **123**
- `GET` endpoints: 62
- `POST` endpoints: 60
- `PUT` endpoints: 1

### Endpoint Class Coverage

- `acp`: 8
- `cortex`: 73
- `kg`: 6
- `metrics`: 2
- `other`: 4
- `system`: 18
- `testing`: 5
- `workflow`: 5
- `ws`: 2

## Required Per-Endpoint Fields

- `method`
- `path_template`
- `request_schema`
- `response_schema`
- `error_normalization`
- `event_emissions`
- `transaction_boundary`
- `idempotency_semantics`

## Contract Rules

- `method + path_template` is the unique endpoint contract key.
- Every inventory endpoint must have exactly one contract entry.
- Request and response schemas are normalized envelopes for extraction safety.
- Error normalization defines stable machine-readable error semantics before runtime split.
- Event emission and transaction-boundary semantics are explicit per endpoint to prevent silent drift.
- Idempotency semantics are declared per endpoint and govern replay behavior for mutating calls.

## Notes

- This Phase 4.5 artifact is a strict prerequisite for Phase 5 gateway extraction (ADR-010).
- Endpoint behavior remains parity-locked by the gateway baseline fixture suite.
