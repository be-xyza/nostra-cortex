# Initiative 132 Batch 0A Contract Hardening Checklist

**Status**: Draft
**Created**: 2026-03-31
**Parent**: [PHASE7_EXECUTION_PLAN.md](/Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/PHASE7_EXECUTION_PLAN.md)

## Goal

Tighten the highest-risk Rust/TypeScript boundary contracts before any deeper Rust parity or extraction work begins.

This batch is intentionally narrow. It is a contract-clarity pass, not a gateway rewrite.

## Scope

### Primary Targets

1. Heap emission payloads
2. Provider runtime envelopes
3. Agent execution lifecycle payloads
4. ACP payloads where ambiguity is currently highest

### Core Principle

Harden the network boundary first:

- explicit discriminators for variant-shaped payloads
- consistent `camelCase` serialization where the active contract expects it
- aligned Rust and TypeScript definitions on the chosen surfaces
- preserve explicitly governed compatibility fields where current consumers still depend on them

## Files To Audit First

### Rust

- [server.rs](/Users/xaoj/ICP/cortex/apps/cortex-eudaemon/src/gateway/server.rs)
- [heap_mapper.rs](/Users/xaoj/ICP/cortex/apps/cortex-eudaemon/src/services/heap_mapper.rs)
- [responses_types.rs](/Users/xaoj/ICP/cortex/apps/cortex-eudaemon/src/services/provider_runtime/responses_types.rs)
- [contracts.rs](/Users/xaoj/ICP/cortex/libraries/cortex-domain/src/agent/contracts.rs)
- [protocol.rs](/Users/xaoj/ICP/cortex/libraries/cortex-runtime/src/policy/protocol.rs)

### TypeScript

- [contracts.ts](/Users/xaoj/ICP/cortex/apps/cortex-web/src/contracts.ts)
- any targeted heap/provider/ACP contract tests under [tests](/Users/xaoj/ICP/cortex/apps/cortex-web/tests)

## Checklist

### Slice 1: Heap Emission Contract

1. Confirm the canonical Rust `EmitHeapBlockRequest` structure and all active TypeScript counterparts match on:
   - required keys
   - optional keys
   - casing
   - nested discriminated payloads
2. Identify any heap payload variants that still deserialize through ambiguous shape matching.
3. Add or tighten tests proving the canonical `camelCase` payload deserializes in Rust and matches the TypeScript interface.

**Expected output**

- one source of truth per field set
- explicit note on any compatibility fields intentionally retained

### Slice 2: Provider Runtime Envelope Contract

1. Audit request/response/event envelopes in `provider_runtime`.
2. Identify where `serde(rename = "type")`, ad hoc `Value`, or permissive flattening is still masking variant drift.
3. Tighten only the externally exposed envelope structure first.
4. Preserve permissive raw payload capture only where required for upstream provider compatibility.

**Expected output**

- clearer typed envelope boundary
- explicit separation between typed contract fields and raw passthrough payload

### Slice 3: Agent Execution Lifecycle Contract

1. Audit the lifecycle payload emitted by the Rust side.
2. Confirm the active schema expected by web/runtime consumers.
3. Verify `camelCase` serialization and any enum/value conventions that affect replay or UI consumption.
4. Flag any fields that should remain optional because they are intentionally phased or derived later.

**Expected output**

- stable lifecycle event payload
- no accidental mismatch between replay/governance expectations and UI contract assumptions

### Slice 4: ACP Ambiguity Audit

1. Identify ACP payloads where ambiguity currently creates the most maintenance risk.
2. Prioritize JSON-RPC request/response/update envelopes over deeper execution internals.
3. Do not attempt to fully redesign ACP in this batch.
4. Produce a clear follow-up list for Batch 2 where broader ACP extraction begins.

**Expected output**

- tightened highest-risk ACP envelopes
- a smaller, better-defined ACP backlog for the extraction batch

## Non-Goals

Do not do any of the following in Batch 0A:

1. split crates
2. rewrite the gateway composition root
3. remove all optional fields categorically
4. introduce a new umbrella core crate
5. redesign provider or ACP architecture beyond boundary hardening

## Verification Commands

### Rust

```bash
cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --test gateway_parity
```

### Web / Contracts

Run the most relevant contract tests for the surfaces touched, especially:

```bash
cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --test gateway_parity
```

and the targeted `cortex-web` contract tests that cover:

- heap API contract
- chat socket / provider event contract
- any ACP- or lifecycle-adjacent consumer contract

### Governance / Portfolio

```bash
bash /Users/xaoj/ICP/scripts/check_nostra_cortex_terminology.sh \
  /Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/README.md \
  /Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/PHASE7_EXECUTION_PLAN.md \
  /Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/BATCH0A_CONTRACT_HARDENING_CHECKLIST.md

python3 /Users/xaoj/ICP/scripts/check_research_portfolio_consistency.py
```

## Exit Criteria

Batch 0A is complete only if:

1. the chosen Rust and TypeScript surfaces are demonstrably aligned
2. variant ambiguity is reduced on the targeted boundaries
3. compatibility fields that remain are documented as intentional
4. no broader architecture rewrite is smuggled into this pass

## Handoff Note

If Batch 0A uncovers structural coupling that cannot be resolved cleanly without moving provider-runtime code, stop after documenting the boundary problem and hand off to Batch 1 rather than forcing extraction inside the contract-hardening pass.
