# Initiative 118 — Architectural Decisions

## ADR-001: Constrained Extraction over Emergent Core

**Decision**: Declare runtime purity invariants before beginning extraction.
Do not let the runtime boundary emerge organically from Desktop refactoring.

**Rationale**: Dependency audit shows host-specific APIs penetrate 58% of services
(`std::env` in 24/41). Without declared constraints, extraction transfers
Desktop assumptions into the runtime permanently.

**Status**: Accepted

---

## ADR-002: ic-agent Forbidden in Runtime

**Decision**: `ic-agent` and `candid` must not appear in `cortex-runtime` or
`cortex-domain`. ICP interaction moves to a separate `cortex-ic-adapter` crate.

**Rationale**: Only 3 files use `ic-agent` (`governance_client`, `streaming_transport`,
`workflow_engine_client`). Clean extraction boundary. Runtime must be substrate-neutral
per §1.1 (Hexagonal Architecture).

**Status**: Accepted

---

## ADR-003: CRDT Logic Belongs in Domain

**Decision**: `artifact_collab_crdt.rs` moves to `cortex-domain/crdt.rs`.

**Rationale**: File is already pure — 467 lines, zero `std::fs`/`std::env`/`std::process`,
zero `unsafe`, zero `async fn`/`.await`, only `chrono`, `serde`, `std::collections`.
Contains deterministic merge algebra, convergence tests, and idempotent operation handling.

**Status**: Accepted

---

## ADR-004: Tokio Restricted Feature Set

**Decision**: Allow `tokio` in `cortex-runtime` with `default-features = false` and
only `rt`, `sync`, `time`, `macros` features. Forbid in `cortex-domain`.

**Rationale**: Forbidden tokio APIs used only in host-specific files (terminal,
supervisor, gateway binding). No runtime-portable service uses them. Feature
restriction maintains WASM compatibility. Domain stays sync-only.

**Status**: Accepted

---

## ADR-005: Injectable Time via TimeProvider

**Decision**: All wall-clock and monotonic time access in `cortex-runtime` must go
through a `TimeProvider` trait. `cortex-domain` must accept time as a parameter,
never calling `Utc::now()` or `SystemTime::now()` directly.

**Rationale**: 43+ time call sites across 15 runtime-portable services. Time is
embedded in event identity, CRDT ordering, staleness detection, and nonce
generation. Without injection, replay determinism is impossible and host clock
becomes implicit authority.

**Status**: Accepted

---

## ADR-006: Structured Logging via LogAdapter

**Decision**: Neither `cortex-domain` nor `cortex-runtime` may use `log!`,
`tracing!`, `println!`, or `eprintln!`. `cortex-runtime` uses `LogAdapter` trait.
`cortex-domain` has no logging at all.

**Rationale**: 8 runtime-portable services use log/tracing with env-based config.
This creates a back door for `std::env` dependency. Structured log events ensure
host controls routing without runtime coupling.

**Status**: Accepted

---

## ADR-007: Opaque Runtime State

**Decision**: Host may not access or mutate runtime internal state. All interaction
via defined request/response interfaces only.

**Rationale**: `local_gateway.rs` uses `static GATEWAY: OnceLock<LocalGateway>` with
22+ `.lock().unwrap()` mutex sites. Any desktop component can reach in and mutate
state. Without opaque boundary, sovereignty is cosmetic.

**Status**: Accepted

---

## ADR-008: Substrate-Neutral Error Types

**Decision**: Runtime and domain public APIs must not expose `std::io::Error` or
OS-specific error variants. Domain defines its own `DomainError` enum. Runtime
defines `RuntimeError`.

**Rationale**: OS-flavored error types leak host assumptions through type system.
Runtime errors must be serializable and deterministic.

**Status**: Accepted

---

## ADR-009: Adapters as Side-Effect Providers Only

**Decision**: Adapters provide data and perform side-effects on request. They may
not mutate runtime state. Runtime decides all state transitions.

**Rationale**: Without this constraint, adapters become covert control planes.
Sovereign mutation authority must reside in runtime logic.

**Status**: Accepted

---

## ADR-010: Gateway Protocol Contract Before Phase 5

**Decision**: Before extracting the 16K-line `server.rs`, produce a Gateway Protocol
Contract defining request/response schema, error normalization, event emission
expectations, transaction boundaries, and idempotency semantics.

**Rationale**: `server.rs` is not just an HTTP binding — it acts as orchestrator,
lifecycle manager, and implicit transaction boundary. Splitting without defined
semantics risks silent behavior changes.

**Status**: Accepted

---

## ADR-011: Determinism Audit

**Decision**: During extraction, audit for:
- `HashMap` iteration order in deterministic output paths (use `BTreeMap` where needed)
- `f32`/`f64` cross-platform rounding in policy scoring and confidence models

**Rationale**: `cortex_ux.rs` uses `f32` for 10+ scoring fields with weighted
calculations. `acp_metrics.rs` uses `f64` for success rates. `viewspec_learning.rs`
uses `f32` for confidence weights. WASM vs native float behavior can differ.
Not a blocker, but must be documented.

**Status**: Accepted

---

## ADR-012: Crate Separation Strategy for Cortex Runtime

**Status**: Accepted  
**Date**: 2026-02-12  
**Supersedes**: ADR-012 Rev A (Module-Only Onion Structure)

### Context

Cortex is undergoing architectural realignment to remove Desktop privilege,
enforce host neutrality, establish deterministic event-driven sovereignty,
and enable Web and Server hosts as equal peers.

Prior proposals recommended enforcing Onion Architecture using internal
modules inside a single `cortex-runtime` crate. Empirical audit revealed
this approach is insufficient:

| Existing Crate | Intended Role | Purity Violation |
|---|---|---|
| `nostra-core` | Core domain types | Depends on `candid` (ICP-specific) + `log` |
| `nostra-workflow-core` | Workflow domain | Depends on `log` (env-coupled); historically had `candid` leakage in alignment path |
| `nostra-ui-core` | UI types | ✅ Clean |
| `nostra-cloudevents` | Event types | ✅ Clean |

Cultural boundaries were insufficient to prevent drift. Structural enforcement
is required.

### Decision

Cortex Runtime SHALL use separate crates per onion layer:

```
nostra/libraries/
    cortex-domain/
    cortex-runtime/
    cortex-ic-adapter/
```

#### `cortex-domain` (Layer 1 — Domain Core)

Pure, deterministic, synchronous business logic.

**Contains**: CRDT algebra, policy validation, scoring formulas, workflow state
machines, event schema definitions, `DomainError` enum, contribution lifecycle rules.

**Constraints**:
- `#![forbid(unsafe_code)]`
- No `async fn`, no `.await`
- No `tokio`
- No `log` / `tracing`
- No `candid` / `ic-agent`
- No `std::fs` / `std::env` / `std::process` / `std::net`
- Must compile to `wasm32-unknown-unknown`

**Dependencies (allowed)**: `serde`, `serde_json`, `chrono` (no-default-features),
`thiserror`, `uuid`.

**Validated**: `artifact_collab_crdt.rs` (467 lines), `acp_meta_policy.rs` (97 lines),
`viewspec_synthesis.rs` (344 lines) all have zero `unsafe`, zero `async`, zero `.await`.

#### `cortex-runtime` (Layer 2 — Orchestration)

Async orchestration layer with port trait definitions.

**Contains**: EventBus, governance orchestration, workflow scheduler, request
routing, agent orchestration, port traits (`StorageAdapter`, `NetworkAdapter`,
`TimeProvider`, `LogAdapter`, `GovernanceAdapter`).

**Dependencies**: `cortex-domain`, `tokio` (restricted), `async-trait`, `serde`.  
**Forbidden**: `candid`, `ic-agent`, UI frameworks, HTTP frameworks.

#### `cortex-ic-adapter` (Layer 3 — Infrastructure)

ICP-specific adapter implementation.

**Dependencies**: `cortex-runtime`, `ic-agent`, `candid`.  
**Must NOT be depended upon by**: `cortex-domain`, `cortex-runtime`.

#### Host Crates (Layer 4)

`cortex-desktop` becomes a thin host: UI rendering, HTTP binding, adapter
implementations. Depends on `cortex-runtime` + adapter crates.

### Dependency Graph

```
cortex-desktop ──→ cortex-runtime ──→ cortex-domain
       │
       └──→ cortex-ic-adapter ──→ cortex-runtime
```

All arrows point inward. `cortex-domain` depends on nothing project-specific.

### Rationale

1. **Structural over cultural enforcement**: Compiler prevents inward import
   violations. No CI grep can match this guarantee.
2. **Greenfield opportunity**: No migration burden, no API breakage, no contributor
   confusion. Pivoting later is exponentially more expensive.
3. **Sovereignty alignment**: Smaller auditable domain surface, deterministic
   replay validation, safer governance mutation surface, host plurality.
4. **Workspace precedent**: Already 14+ members. Adding 3 is consistent, not novel.

### Constraints

1. **No additional layer crates** without a new ADR. The structure is intentionally
   limited to `cortex-domain`, `cortex-runtime`, `cortex-ic-adapter`.
2. **Domain CI purity check** must verify: no forbidden deps, no async, no
   unsafe, no OS APIs, successful wasm32 compilation.
3. **Runtime CI purity check** must verify: no candid/ic-agent, restricted tokio,
   wasm32 compilation with `--no-default-features`.

### Migration

Initiative 118 phases remain unchanged. Extraction mapping:
- Pure logic → `cortex-domain`
- Orchestration → `cortex-runtime`
- ICP calls → `cortex-ic-adapter`
- Host bindings remain in `cortex-desktop`

Phase 0 includes workspace crate creation before extraction begins.

---

## ADR-013: Adapter Strategy — Static Trait Dispatch with Registry-Ready Identity

**Status**: Accepted  
**Date**: 2026-02-12

### Context

Three proto-adapter systems already exist in the codebase:
- `OperationRegistry` (312 lines) — `register_adapter`/`execute` with `Arc<dyn OperationAdapter>`
- `EmbeddingProvider` (137 lines) — `async trait` with `model_id()`, `Arc<dyn>` dispatch
- `WorkflowStore` (109 lines) — persistence port with `save`/`load`/`list`/`delete`

All three use compile-time trait dispatch. None require dynamic loading or
a formal registry. This proves static trait adapters are sufficient at current scale.

### Decision

Phase I uses **static trait adapters, host-provided implementations**.

All port traits in `cortex-runtime/ports/` SHALL implement a minimal
`AdapterIdentity` trait to future-proof registry integration:

```rust
trait AdapterIdentity {
    fn adapter_id(&self) -> &str;      // e.g. "nostra/ic-storage-v1"
    fn adapter_version(&self) -> &str;
    fn capabilities(&self) -> &[&str]; // e.g. ["storage:read", "storage:write"]
}
```

**No dynamic loading**. No WASM adapter packaging. No registry infrastructure.

A future governance-controlled adapter registry (Phase III) requires a separate
initiative when:
- `cortex-runtime` is stable (Phases 0-2 complete)
- Governance mutation model is formalized
- Multiple hosts are live
- Third-party adapter demand exists

### Rationale

1. Three existing proto-adapter systems prove static dispatch works at current scale.
2. `OperationRegistry` already has register/lookup/execute — a full registry is not
   needed until governance controls adapter lifecycle.
3. `AcpPublishEvidenceAdapter` uses `std::process::Command` — proof that some adapters
   can never be WASM-packaged. This limits registry scope to network/storage/compute.
4. Overbuilding extension infrastructure before core stabilizes creates fragility.

### Constraints

- `AdapterIdentity` is mandatory for all port trait implementations
- No `#[wasm_bindgen]` or dynamic loading until Phase III initiative
- Adapter fragmentation beyond the 5 defined ports requires ADR approval

---

## ADR-014: Constitutional Maturity Ladder — Graph Core and Structural Integrity

**Status**: Accepted  
**Date**: 2026-02-12

### Context

Research 118 extracts the Cortex runtime from Desktop-specific assumptions into
a substrate-neutral, deterministic architecture. This extraction creates a
unique opportunity: as `cortex-domain` matures, it can acquire the ability to
reason about Nostra's own architecture — validating coherence, detecting drift,
and eventually simulating governance outcomes.

This capability must emerge in **strict sequence** aligned with Research 118
phases. Building it prematurely would violate ADR-001 (Constrained Extraction
over Emergent Core) by allowing the reasoning layer to emerge organically
before its substrate is clean.

### Decision

Cortex SHALL acquire constitutional reasoning capabilities via a 4-layer
maturity ladder, each layer gated on the completion of specific Research 118
phases:

| Layer | Capability | Gate | Scope |
|---|---|---|---|
| **0** | Deterministic graph core | Phase 0 | `EdgeKind` enum, graph traversal, cycle detection, topological sort, structural diff |
| **1** | Structural Integrity Query Schema (SIQS) | Phase 1-2 complete | `IntegrityRule`, predicate evaluation, `cargo test integrity_engine` |
| **2** | Governance-wired integrity enforcement | Phase 3-4 complete | Pre-vote integrity checks, post-execution validation, risk scoring |
| **3** | Governance Simulation Mode (GSMS) | Post-118 (separate initiative) | Ephemeral forks, mutation replay, structural diff, impact analysis |

#### Layer 0 — Graph Core (Phase 0)

`cortex-domain` SHALL include:

- `EdgeKind` enum — typed relationship taxonomy replacing untyped `Text` edges:
  `depends_on`, `contradicts`, `supersedes`, `implements`, `invalidates`,
  `requires`, `assumes`, `constitutional_basis`
- Graph traversal utilities — cycle detection, topological sort, dependency walk
- `StructuralDiff` type — pure diff of graph states (nodes/edges added/removed)

All graph utilities must:
- Be pure, sync, deterministic
- Compile to `wasm32-unknown-unknown`
- Have zero forbidden dependencies per Purity Contract v1.3
- Be testable headless via `cargo test -p cortex-domain -- graph`

#### Layer 1 — SIQS (After Phase 1-2)

`cortex-domain` SHALL include structural integrity evaluation:

- `IntegrityRule` struct — declarative graph validation rules
- `IntegrityPredicate` — node/edge selection + constraint
- `evaluate_rule(rule, graph) → Vec<IntegrityViolation>`
- No canister integration, no governance wiring, no UI

SIQS rules are **not** contributions at this layer. They are domain-level
validation functions. Promotion to first-class contributions requires a
separate ADR once the contribution lifecycle model is mature.

#### Layer 2 — Governance Integration (After Phase 3-4)

Requires composable proposal mutations. Not specified here — separate ADR
when Phase 3-4 delivers composable governance execution.

#### Layer 3 — GSMS (Post-118)

Requires copy-on-write or overlay graph semantics for ephemeral forks.
Not specified here — separate initiative proposal when Layer 2 is stable.

### Rationale

1. **Sequencing over ambition**: Each layer enables the next. No layer requires
   infrastructure that doesn't exist at its scheduled time.
2. **Graph core is not optional**: Typed edges and traversal utilities are
   foundational infrastructure that `cortex-domain` needs regardless of SIQS.
   Graph type safety prevents semantic drift in the relationship model.
3. **SIQS is domain logic**: Structural integrity validation is deterministic
   graph evaluation — exactly the kind of pure function `cortex-domain` is
   designed for. It is not governance logic; it is graph algebra.
4. **No standalone tool**: Constitutional reasoning lives inside Nostra, not in
   an external application. Building it externally would create dual truth
   sources, schema drift, and signal that the platform cannot reason about
   itself.

### Constraints

1. No layer may be built before its gate (Research 118 phase) is complete
2. SIQS rules must not depend on async, time, or IO — pure graph evaluation only
3. `EdgeKind` enum changes require ADR approval (same governance as adapter ports)
4. GSMS requires a separate initiative proposal — it is not in scope for 118

---

## ADR-015: File Disposition Audit — Complete Extraction Surface

**Decision**: Every Cortex Desktop source file (84 total) has been classified into
one of five disposition categories: Domain, Runtime, Adapter, Split, or Host-Only.
This disposition matrix is the authoritative guide for which files are extracted
in which phase.

**Rationale**: PLAN.md originally named ~18 of 41 service files, leaving 23 without
phase assignment. Empirical audit revealed:
- `streaming_transport.rs` (1,178 lines) requires a **4-way split** — the most
  complex extraction target, more than `governance_client.rs` or `workflow_engine_client.rs`
- OnceLock appears in 17 files (9 extraction-target files requiring elimination, not just `server.rs`)
- `nostra-workflow-core/alignment.rs` now uses substrate-neutral `ActorId`; Phase 0 keeps a regression guard to prevent reintroduction of `candid`
- 14 service files are host-only and should never be extraction targets

**Evidence**:
- 28 files / ~12,160 lines require extraction
- 56 files / ~26,000 lines stay host-only
- `server.rs` alone: 15,975 lines, 126 violations

**Status**: Accepted

**Artifacts**: See PLAN.md Appendix A (Disposition Matrix), Appendix B (OnceLock Matrix)

---

## ADR-016: GSMS Overlay Strategy — Diff-Based Replay

**Status**: Accepted
**Date**: 2026-02-14
**Resolves**: ADR-014 Layer 3 gate ("⚠️ Depends on CoW semantics")

### Context

ADR-014 defined Layer 3 (GSMS — Governance Simulation Mode) as requiring
"Copy-on-write or overlay graph (ephemeral forks without stable memory cost)."
This was deferred as "Post-118, separate initiative" pending resolution of the
overlay strategy.

A 4-round architectural validation determined:
1. GSMS is the natural culmination of 118 Layer 0-3, not a separate initiative
2. Research 119 (Nostra Commons) and 091 (Nostra Bench) converge into the
   same activation point
3. No new primitives are required — only wiring of existing designs

Three overlay strategies were evaluated:

| Strategy | Memory | Complexity | Alignment |
|---|---|---|---|
| Clone-on-fork | ❌ High (full graph copy) | Low | Violates "ephemeral without stable memory cost" |
| Structural sharing | ✅ Low (persistent data structure) | High (deep graph rework) | Over-engineered for current maturity |
| **Diff-based replay** | ✅ Low (heap-only, bounded) | Medium (mutation replay engine) | **Aligns with event-sourced architecture** |

### Decision

GSMS SHALL use **diff-based replay** for simulation sessions.

#### Workflow

```
1. Load graph (read-only reference)           → `before: &Graph`
2. Record base hash                           → `graph_root_hash`
3. Replay scenario mutations into heap copy   → `after: Graph` (ephemeral)
4. Evaluate SIQS on virtual state             → `evaluate_all(rules, &after)`
5. Compute structural diff                    → `diff(&before, &after)`
6. Emit SimulationReport
7. Drop `after`                               → No stable memory write
```

#### Constraints

- `after` is bounded by `max_mutations_per_session` (prevents memory exhaustion)
- `after` is **never** written to stable memory
- Simulation events are **never** logged to Chronicle (hard boundary)
- All replay steps are pure functions — no async, no IO, no wall-clock

#### SimulationSession Metadata

Every session must record:

```rust
pub struct SimulationSession {
    pub session_id: String,
    pub scenario_id: String,
    pub seed: u64,
    pub graph_root_hash: String,
    pub commons_version: String,
    pub siqs_version: String,
    pub max_mutations: usize,
    pub mutation_count: usize,
    pub structural_diff: StructuralDiff,
    pub violations: Vec<IntegrityViolation>,
    pub aborted: bool,           // true if caps exceeded
    pub abort_reason: Option<String>,
}
```

### Rationale

1. **Event-sourced alignment**: Chronicle events are already mutation records.
   Replaying a mutation sequence against a virtual graph is the same paradigm.
2. **`StructuralDiff` already exists**: Designed in 118 Layer 0 as
   `diff(before: &Graph, after: &Graph) -> StructuralDiff`.
3. **No infrastructure debt**: Clone-on-fork requires stable memory management.
   Structural sharing requires persistent data structures. Diff-based replay
   requires only what Layer 0 already provides.
4. **Mutation caps are natural**: Capping replay length directly bounds memory.
   No separate memory management required.

### Impact on ADR-014

This decision resolves ADR-014 §Layer 3:
- "Post-118 (separate initiative)" → "118 Layer 3 (activated by ADR-016)"
- "⚠️ Depends on CoW semantics" → "✅ Diff-based replay (no CoW needed)"
- "GSMS requires a separate initiative proposal" → "GSMS activation spec:
  `GSMS_ACTIVATION.md`"

---

## ADR-017: Phase 0/1 Closure Gate Before Phase 2+

**Status**: Accepted  
**Date**: 2026-02-15

### Decision

Initiative 118 must not advance to Phase 2+ until a single gate run confirms:
- Phase 0 baseline and inventory lock are complete
- Phase 1 event-engine slice parity is complete under feature flag
- Purity, terminology, wasm, and parity tests are all green

### Mandatory gate conditions

1. Gateway parity inventory lock is enforced with:
   `inventory_count == fixture_count + approved_exemptions_count`
2. Default exemptions policy remains:
   `approved_exemptions_count == 0`
3. ACP shadow tests fail on non-allowed drift and allow timestamp-only drift.

### Evidence

Closure evidence is recorded in:
`research/118-cortex-runtime-extraction/PHASE_0_1_CLOSURE_EVIDENCE_2026-02-15.md`

Gate run outcome on 2026-02-15: all required checks passed.

### Enforcement

Phase 2 work (`acp_protocol`/policy layer extraction) is frozen until ADR-017
gate conditions are met and documented.

### Unfreeze authority and conditions

Unfreeze authority: Initiative steward(s) for 118 and platform architecture steward.
Unfreeze requires all of the following:

1. `cortex-runtime-freeze-gates` CI job is green on latest `main`.
2. `cortex-runtime-freeze-gates` CI job is green on latest PR candidate.
3. `approved_exemptions_count == 0` unless a new ADR explicitly authorizes exceptions.
4. No unresolved ACP shadow mismatch regressions in current freeze-gate logs.
5. `PHASE_2_ENTRY_PACKET_2026-02-15.md` checklist is complete and attached to kickoff PR.

### ADR-017 Execution Note (2026-02-15, Phase 2 entry unblock branch)

Implemented controls and evidence updates:

1. Restored tracked runtime/gate assets required by freeze policy.
2. Re-enabled CI jobs in `.github/workflows/test-suite.yml` with preflight guards.
3. Re-ran local freeze/evidence command contract with green results:
   - `bash scripts/check_gateway_parity_inventory_sync.sh`
   - `bash scripts/run_cortex_runtime_freeze_gates.sh`
   - `bash scripts/check_118_pr_evidence.sh --pr-body-file tests/fixtures/pr_evidence/valid.md`
   - `bash tests/scripts/test_check_118_pr_evidence.sh`
4. Completed Phase 2 PR-1 extraction slice for `acp_meta_policy` into
   `cortex-domain::policy::meta` and wired desktop policy usage through the domain API.

Unfreeze authority remains steward-controlled. Final unfreeze grant still requires CI run URLs showing `cortex-runtime-freeze-gates` green on latest `main` and PR candidate.

### ADR-017 Unfreeze Grant Record (2026-02-16)

Unfreeze prerequisites are satisfied and recorded:

1. PR candidate run green (`cortex-runtime-freeze-gates`):
   https://github.com/be-xyza/cortex-dev/actions/runs/22048766714
2. Latest `main` run green (`cortex-runtime-freeze-gates`):
   https://github.com/be-xyza/cortex-dev/actions/runs/22048828212
3. Steward-authorized merge for Phase 2 PR-1:
   https://github.com/be-xyza/cortex-dev/pull/2

Scope of grant: **Phase 2 PR-1 only** (`acp_meta_policy` extraction).  
Later Phase 2 slices remain independently gated.

---

## ADR-018: Feature-Gated AsyncExternalOp Policy Integration

**Status**: Accepted  
**Date**: 2026-02-15

### Context

BAML intake experiments (reference `research/reference/topics/agent-systems/baml`) validated three portable runtime patterns:
1. Explicit retry policy as typed configuration
2. Strategy-driven provider selection (`single`, `fallback`, `round_robin`)
3. Structured attempt traces suitable for deterministic replay and policy audit

Before this ADR:
- `nostra-workflow-core` exposed async external steps but had no bounded retry strategy semantics
- repeated `tick()` while paused could redispatch indefinitely
- runtime policy experiment types existed only in `cortex-runtime` research surface

### Decision

Adopt BAML-derived patterns in two bounded layers:

1. Add feature-gated policy experiment module to `cortex-runtime`:
   - feature: `baml-policy-experiments`
   - module: `cortex-runtime/src/policy_experiments.rs`

2. Wire policy-aware dispatch semantics into `nostra-workflow-core` `AsyncExternalOp`:
   - `AsyncRetryPolicy` + `AsyncRetryStrategy`
   - `AsyncProviderStrategy` (`single`, `fallback`, `round_robin`)
   - bounded attempt budget with deterministic provider selection
   - explicit failure when configured attempt budget is exhausted

### Rationale

1. Keeps adoption constitutional and low-risk:
   - patterns are integrated without introducing external DSL/runtime dependency.
2. Converts implicit behavior into deterministic policy:
   - retry/fallback selection is now serializable and testable.
3. Preserves forward compatibility with Initiative 118 extraction phases:
   - feature-gated runtime module remains optional while policy semantics mature.

### Scope and Constraints

- This ADR does **not** adopt BAML syntax or codegen runtime.
- Public canister interfaces remain sourced from Candid contracts.
- Policy semantics are runtime-side execution behavior, not platform schema authority.

### Evidence

- Runtime feature module:
  - `nostra/libraries/cortex-runtime/src/policy_experiments.rs`
- Workflow-core policy wiring:
  - `nostra/libraries/nostra-workflow-core/src/primitives.rs`
  - `nostra/libraries/nostra-workflow-core/src/engine.rs`
  - `nostra/libraries/nostra-workflow-core/src/builder.rs`
  - `nostra/libraries/nostra-workflow-core/src/debates.rs`
- Phase task mapping:
  - `research/118-cortex-runtime-extraction/experiments/baml/BAML_PHASE_TASKLIST_2026-02-15.md`

---

## ADR-019: Phase 2 Policy and Collaboration Extraction Closure

**Status**: Accepted  
**Date**: 2026-02-16

### Decision

Close Initiative 118 Phase 2 as implementation-complete in workspace, with freeze/evidence gates green and endpoint parity preserved.

### Closed Phase 2 slices

1. CRDT extraction into `cortex-domain::collaboration::crdt`
2. ACP operation adapter contract ownership in `cortex-runtime::policy::adapter`
3. Session store extraction into `cortex-runtime::policy::sessions`
4. Permission ledger extraction into `cortex-runtime::policy::permissions`
5. Metrics extraction into `cortex-runtime::policy::metrics`
6. Protocol split:
   - domain-facing JSON-RPC types in `cortex-domain::policy::types`
   - stateful runtime dispatch/orchestration in `cortex-runtime::policy::protocol`
   - thin desktop host shim retained in `cortex-desktop`

### Determinism and purity outcome

- Extracted runtime/domain policy paths are free of extracted `OnceLock`, `std::env`, `std::fs`, and direct wall-clock APIs.
- Session/permission/metrics runtime modules remain deterministic for externally visible ordering and replay surfaces.
- Gateway parity inventory lock remains intact: `inventory=123 fixtures=123 exemptions=0`.

### Validation evidence

Local gate contract re-run on 2026-02-16:

1. `bash scripts/check_gateway_parity_inventory_sync.sh` (PASS)
2. `bash scripts/run_cortex_runtime_freeze_gates.sh` (PASS)
3. `bash scripts/check_118_pr_evidence.sh --pr-body-file tests/fixtures/pr_evidence/valid.md` (PASS)
4. `bash scripts/check_118_pr_evidence.sh --pr-body-file tests/fixtures/pr_evidence/phase2_slice_template.md` (PASS)
5. `bash tests/scripts/test_check_118_pr_evidence.sh` (PASS)

Primary evidence artifact:
`research/118-cortex-runtime-extraction/PHASE_2_COMPLETION_EVIDENCE_2026-02-16.md`

### Governance constraint retained

Remote merge governance still requires steward authorization records per Phase 2 slice PR and attached freeze/evidence proof under ADR-017 controls.

---

## ADR-020: Phase 3 Governance/Workflow/Streaming Extraction Implementation

**Status**: Accepted  
**Date**: 2026-02-16

### Decision

Implement Initiative 118 Phase 3 extraction slices in workspace with runtime/domain/adapter boundaries enforced and freeze/evidence gates green.

### Implemented Phase 3 slices

1. Runtime port expansion in `cortex-runtime::ports`:
   - `GovernanceAdapter`
   - `WorkflowAdapter`
   - `StreamingTransportAdapter`
   - normalized governance/workflow contract types
2. Domain streaming protocol type surface added:
   - `cortex-domain::streaming::types`
3. Runtime modules added:
   - `cortex-runtime::governance`
   - `cortex-runtime::workflow::{service,executor}`
   - `cortex-runtime::streaming::transport`
4. ICP adapter implementations added:
   - `cortex-ic-adapter::governance`
   - `cortex-ic-adapter::workflow`
   - `cortex-ic-adapter::streaming`
   - `cortex-ic-adapter::dfx`
5. Desktop shims updated to consume extracted surfaces:
   - governance/workflow clients delegate to `cortex-ic-adapter`
   - workflow service signature/config logic delegates to `cortex-runtime::workflow::service`
   - workflow executor planning delegates to `cortex-runtime::workflow::executor`
   - dfx helper delegation through `cortex-ic-adapter::dfx`

### Purity and boundary outcome

1. `cortex-runtime` remains free of `ic-agent` and `candid`.
2. `cortex-runtime` remains free of forbidden wall-clock/logging APIs checked by freeze gates.
3. Streaming protocol contract types are substrate-neutral in domain crate.
4. ICP-specific transport and canister calls are isolated in adapter crate.

### Validation evidence

Local gate contract re-run on 2026-02-16:

1. `bash scripts/check_gateway_parity_inventory_sync.sh` (PASS)
2. `bash scripts/run_cortex_runtime_freeze_gates.sh` (PASS)
3. `bash scripts/check_118_pr_evidence.sh --pr-body-file tests/fixtures/pr_evidence/valid.md` (PASS)
4. `bash tests/scripts/test_check_118_pr_evidence.sh` (PASS)

Primary evidence artifact:
`research/118-cortex-runtime-extraction/PHASE_3_COMPLETION_EVIDENCE_2026-02-16.md`

### Governance constraint retained

ADR-017 controls remain active for remote merge workflow:
- per-slice PR evidence bundle
- freeze/evidence run links
- steward authorization record per merge slice

---

## ADR-021: Phase 4 Agent/ViewSpec/UX Runtime-Domain Extraction Implementation

**Status**: Accepted  
**Date**: 2026-02-16

### Decision

Implement Initiative 118 Phase 4 extraction surfaces in workspace by introducing:
1. New `cortex-domain` namespaces for `ux`, `theme`, `viewspec`.
2. New `cortex-runtime` namespaces for `viewspec`, `ux`, `agents`, `resilience`.
3. New runtime adapter ports for UX contract storage, theme policy, and agent process boundaries.
4. Desktop-side compatibility delegation that preserves existing gateway/router/component contracts.

### Implemented surfaces

1. `cortex-domain`:
   - `src/ux/{types,scoring}.rs`
   - `src/theme/policy.rs`
   - `src/viewspec/{types,validation,learning,synthesis}.rs`
2. `cortex-runtime`:
   - `src/viewspec/{store_keys,digest,service}.rs`
   - `src/ux/{service,feedback}.rs`
   - `src/resilience/service.rs`
   - `src/agents/{types,service}.rs`
3. `cortex-runtime::ports` additions:
   - `UxContractStoreAdapter`
   - `ThemePolicyAdapter`
   - `AgentProcessAdapter`
4. Desktop compatibility/cutover boundaries:
   - `services/cortex_ux.rs` now delegates core pure logic to `cortex-domain::ux`.
   - `services/theme_policy.rs` normalization/style flows delegate to `cortex-domain::theme::policy`.
   - `services/cortex_ux_store.rs` implements `UxContractStoreAdapter`.
   - `services/resilience_service.rs` score computation delegates to `cortex-runtime::resilience::calculate`.
   - `services/agent_service.rs` delegates modality/idempotency shaping helpers to `cortex-runtime::agents::service`.
   - `gateway/server.rs` ViewSpec digest helper delegates to `cortex-runtime::viewspec::viewspec_digest_hex`.

### Contract constraints retained

1. Existing `/api/cortex/viewspecs*`, `/api/cortex/layout*`, `/api/cortex/feedback*`, `/api/cortex/preferences/theme-policy`, `/api/cortex/runtime/*`, `/api/metrics/resilience`, and `/api/search` route surface remains unchanged.
2. `services::cortex_ux` API remains stable for router/components/gateway call sites.
3. Runtime modules avoid direct wall-clock calls and other forbidden purity APIs.

### Validation evidence

Local contract executed on 2026-02-16 after implementation:
1. `cargo check --manifest-path nostra/Cargo.toml -p cortex-domain` (PASS)
2. `cargo check --manifest-path nostra/Cargo.toml -p cortex-runtime` (PASS)
3. `cargo check --manifest-path nostra/Cargo.toml -p cortex-desktop --bin gateway_server` (PASS)
4. `cargo check --manifest-path nostra/Cargo.toml -p cortex-desktop --bin cortex_desktop` (PASS)

Full freeze/evidence/parity suite remains required before merge under ADR-017 controls.

---

## ADR-022: Phase 4.5 Gateway Protocol Contract Closure and Phase 5 Unblock Gate

**Status**: Accepted  
**Date**: 2026-02-16

### Decision

Close Phase 4.5 by establishing a machine-validated gateway protocol contract that is inventory-locked to the parity endpoint baseline, and make this contract a mandatory freeze-gate check before Phase 5 extraction work.

### Implemented controls

1. Contract artifacts added:
   - `research/118-cortex-runtime-extraction/GATEWAY_PROTOCOL_CONTRACT_2026-02-16.md`
   - `research/118-cortex-runtime-extraction/GATEWAY_PROTOCOL_CONTRACT_2026-02-16.json`
2. Contract coverage gate added:
   - `scripts/check_gateway_protocol_contract_coverage.sh`
3. Freeze gate chain updated to enforce contract coverage in-line:
   - `scripts/run_cortex_runtime_freeze_gates.sh`
4. Phase 4.5 evidence recorded:
   - `research/118-cortex-runtime-extraction/PHASE_4_5_COMPLETION_EVIDENCE_2026-02-16.md`

### Contract invariants

1. `inventory_count == contract_entries_count`
2. Every inventory `METHOD + PATH` appears exactly once in contract entries.
3. Contract entries must include:
   - `method`
   - `path_template`
   - `request_schema`
   - `response_schema`
   - `error_normalization`
   - `event_emissions`
   - `transaction_boundary`
   - `idempotency_semantics`

### Phase 5 unfreeze condition

Phase 5 extraction is unblocked only when all are true in one local/CI run:

1. `bash scripts/check_gateway_parity_inventory_sync.sh` passes.
2. `bash scripts/check_gateway_protocol_contract_coverage.sh` passes.
3. `bash scripts/run_cortex_runtime_freeze_gates.sh` passes with contract coverage gate enabled.
4. No parity inventory exemptions are introduced (`approved_exemptions_count == 0`).

### Rationale

Phase 5 targets the 15K-line gateway monolith and carries highest drift risk. Contracting endpoint semantics before extraction prevents silent request/response, error, event, transaction, and idempotency regressions during runtime-host split.

---

## ADR-023: Phase 5 Incremental Gateway Runtime Scaffolding and Controlled Cutover Start

**Status**: Accepted  
**Date**: 2026-02-16

### Decision

Start Phase 5 with runtime gateway contract scaffolding and a controlled initial cutover (`/api/health`) while preserving existing route declarations and parity behavior for the full endpoint inventory.

### Implemented scope

1. Added `cortex-runtime::gateway` module surface:
   - envelope types
   - runtime state cache for idempotency replay
   - dispatcher with normalized 404 handling and deterministic replay behavior
2. Extended `CortexRuntime` API and runtime config for gateway extraction.
3. Added desktop runtime host composition layer:
   - `gateway/runtime_host.rs`
4. Started runtime dispatch integration in gateway server on health endpoint.
5. Removed production `OnceLock` usage from targeted Phase 5 files.

### Rationale

1. Establishes compile-time and test-verified runtime gateway primitives before mass endpoint migration.
2. Preserves inventory and parity gates while reducing extraction risk.
3. Keeps route inventory tooling stable by retaining declaration locality in `gateway/server.rs`.

### Validation evidence

1. `research/118-cortex-runtime-extraction/PHASE_5_EXECUTION_EVIDENCE_2026-02-16.md`
2. `bash scripts/run_cortex_runtime_freeze_gates.sh` (PASS with contract coverage gate enabled)
3. `cargo test --manifest-path nostra/Cargo.toml -p cortex-runtime` (PASS, gateway dispatcher tests included)
