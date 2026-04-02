# Initiative 132 Phase 7 Execution Plan

**Status**: Draft
**Created**: 2026-03-31
**Scope**: Phase 7 Rust parity, contract hardening, and execution-isolation sequencing

## Purpose

This document converts the validated Phase 7 posture for Initiative 132 into an actionable implementation sequence.

It assumes the following constraints are already fixed:

1. Phase 6 remains the canonical deployment model:
   - Python Eudaemon Alpha worker
   - Rust gateway
   - Hetzner VPS
   - operator-local SSH promotion
   - `systemd` services
2. Phase 7 is a parity-backed Rust hardening program, not a wholesale runtime replacement.
3. OS-level sandboxing is introduced for executor slices that run untrusted code or autonomous contribution paths, not as an immediate whole-stack replacement for the current runtime contract.

## Success Criteria

Phase 7 should be considered on track only if each batch produces one of two outcomes:

1. A validated reduction in boundary drift, runtime ambiguity, or app-surface bloat.
2. A justified no-go result showing the seam is not ready and should remain in the current app crate.

The goal is not extraction for its own sake. The goal is to reduce coupling without destabilizing the current authority model.

## Batch Order

1. Batch 0: Contract Hardening
2. Batch 1: Provider Runtime Extraction
3. Batch 2: ACP / Terminal Execution-Control Extraction
4. Batch 3: Workbench UX / Heap Projection Extraction
5. Batch 4: Executor Isolation Hardening

## Batch 0: Contract Hardening

### Goal

Reduce network-boundary drift before moving more execution logic into Rust.

### Primary Scope

- Gateway request/response DTOs
- A2UI payload DTOs
- TypeScript contract sync on the highest-risk surfaces
- Tagged variant payloads where the domain is explicitly variant-shaped

### In Scope

- `EmitHeapBlockRequest` and related heap emission contracts
- agent execution lifecycle payloads
- provider-runtime request/response envelopes
- ACP JSON-RPC payloads where contract ambiguity is currently high

### Out of Scope

- blanket removal of every optional field
- compatibility-breaking rewrites of legacy payloads without an explicit migration plan
- general crate extraction

### Candidate Files

- [server.rs](/Users/xaoj/ICP/cortex/apps/cortex-eudaemon/src/gateway/server.rs)
- [contracts.rs](/Users/xaoj/ICP/cortex/libraries/cortex-domain/src/agent/contracts.rs)
- [responses_types.rs](/Users/xaoj/ICP/cortex/apps/cortex-eudaemon/src/services/provider_runtime/responses_types.rs)
- [contracts.ts](/Users/xaoj/ICP/cortex/apps/cortex-web/src/contracts.ts)

### Exit Criteria

1. Rust and TypeScript boundary payloads match on the chosen surfaces.
2. Variant payloads use explicit discriminators where ambiguity currently causes drift risk.
3. Existing compatibility fields are preserved only where a named contract still requires them.

### Verification

- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --test gateway_parity`
- targeted contract tests in `cortex-web`
- terminology and portfolio checks

## Batch 1: Provider Runtime Extraction

### Goal

Isolate provider runtime logic so live-provider and batch-audit adapters can evolve without further inflating the main gateway surface.

### Owned Surface

- `provider_runtime/config.rs`
- `provider_runtime/client.rs`
- `provider_runtime/policy.rs`
- `provider_runtime/responses_types.rs`
- `provider_runtime/sse.rs`
- `provider_runtime/tool_loop.rs`
- `provider_probe.rs`

### Keep In App Layer

- route wiring in the gateway
- operator auth and role gating
- startup env resolution that remains tightly bound to `GatewayState`

### Recommended Form

Start as a tighter module/package boundary inside the existing workspace. Promote to its own crate only if reuse or dependency pressure becomes concrete.

### Risks

- extracting config too early can duplicate startup/env logic
- over-generalizing this surface can turn it into a generic LLM framework rather than a Cortex execution adapter

### Exit Criteria

1. Provider runtime logic has a narrower import surface into the gateway.
2. Operator-only execution-infrastructure visibility remains intact.
3. The extracted surface still enforces server-side execution eligibility and does not make discovered inventory executable implicitly.

### Verification

- gateway parity checks
- provider policy tests
- operator-role access tests

## Batch 2: ACP / Terminal Execution-Control Extraction

### Goal

Separate protocol, session, ledger, and execution-policy responsibilities from the broader app surface while preserving the current execution authority model.

### Owned Surface

- `acp_protocol.rs`
- `acp_adapter.rs`
- `acp_event_sink.rs`
- `acp_event_projector.rs`
- `acp_session_store.rs`
- `acp_permission_ledger.rs`
- `acp_metrics.rs`
- `acp_meta_policy.rs`

### Keep In App Layer

- concrete terminal spawning
- filesystem service integration
- HTTP route wiring
- startup/pilot toggles

### Recommended Form

Extract adapter ports and protocol logic together. Avoid creating a thin “types-only” module that leaves all behavioral coupling in place.

### Risks

- ACP is still coupled to terminal and filesystem services
- a protocol-only extraction may add indirection without reducing maintenance load

### Exit Criteria

1. ACP protocol handling can be reasoned about without reading the full app gateway.
2. Permission ledger and session semantics remain fail-closed.
3. Terminal execution remains governed by explicit policy checks.

### Verification

- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --test acp_gateway_integration`
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --test acp_staging_operationalization`

## Batch 3: Workbench UX / Heap Projection Extraction

### Goal

Reduce the largest projection/presentation bloat pool without destabilizing gateway composition.

### Owned Surface

- `workbench_ux.rs`
- `viewspec.rs`
- `viewspec_learning.rs`
- `viewspec_synthesis.rs`
- `cortex_ux.rs`
- `cortex_ux_store.rs`
- optionally `heap_mapper.rs`
- optionally `heap_nesting.rs`

### Keep In App Layer

- route handlers
- runtime state wiring
- React/web rendering specifics

### Recommended Form

Start as a module-boundary cleanup first. Only promote to a crate after the projection and UX contracts are more stable.

### Risks

- this seam is large enough to invite premature abstraction
- mixing projection policy with product-specific UX could recreate the same bloat in a new location

### Exit Criteria

1. Projection/model logic is separated from route orchestration.
2. Heap/workbench/viewspec transforms are testable without the full gateway stack.
3. The boundary is useful to both headless runtime and web-facing consumers.

### Verification

- targeted unit tests for projection/model transforms
- relevant `cortex-web` contract tests
- gateway parity checks for affected endpoints

## Batch 4: Executor Isolation Hardening

### Goal

Introduce OS-level sandboxing only when the runtime actually executes untrusted code, generated shell/code, or broader autonomous contribution flows.

### Trigger Conditions

Any one of the following should move this batch from deferred to active:

1. Eudaemon executes generated shell commands beyond tightly governed internal tooling.
2. Eudaemon executes arbitrary code evaluation loops.
3. Eudaemon gains broader autonomous contribution behavior that can mutate repo or host state beyond current L1/L2 expectations.

### Required Controls

- executor-specific filesystem restrictions
- executor-specific network restrictions
- explicit capability restrictions
- bounded resource controls
- preserved operator-only execution-infrastructure surfaces

### Preferred First Step

Harden existing `systemd` service posture first, then add OS-level sandboxing to the narrower executor slice rather than the whole gateway/runtime stack.

### Exit Criteria

1. Untrusted execution paths are isolated from host secrets and unrelated filesystem roots.
2. Sandboxed execution cannot implicitly escalate through discovered provider/runtime inventory.
3. Local debugging and operator authority flows remain legible.

### Verification

- governed smoke tests for allowed paths and denied paths
- explicit network-denial checks
- runtime-authority contract verification

## No-Go Rules

Do not proceed with a batch if:

1. it requires replacing the canonical Phase 6 deployment model before parity is shown
2. it introduces a new umbrella core crate without a measured dependency problem
3. it breaks operator-only execution-infrastructure visibility
4. it weakens the current authority model to make extraction easier

## Suggested Ownership Model

1. **Batch 0**: contract hardening first, with parity and web-contract checks as gates
2. **Batch 1**: provider runtime surface
3. **Batch 2**: ACP / execution-control surface
4. **Batch 3**: workbench UX / heap projection surface
5. **Batch 4**: executor-specific sandboxing only when trigger conditions are met

## Recommended First Implementation Ticket

The best first ticket is:

`Batch 0A — Gateway / TypeScript contract hardening for heap emission, provider runtime envelopes, and lifecycle payloads`

This gives the highest leverage with the lowest architectural risk, and it improves every later extraction batch.
