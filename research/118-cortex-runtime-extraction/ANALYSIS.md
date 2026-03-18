# Cortex Architecture: Final Validated Analysis

> **Author**: Winston (Architect Agent)  
> **Date**: 2026-02-10  
> **Revision**: 4 (Final — all corrections validated)  

---

## 1. Four-Round Summary

| Round | Key Finding |
|---|---|
| **Original Proposal** | Desktop should not be privileged |
| **Round 1** | Extract, don't rewrite; reject greenfield |
| **Round 2** | Declare invariants *before* extraction (Constrained Extraction) |
| **Round 3** | `ic-agent` forbidden, CRDT = runtime, tokio restricted |
| **Round 4** | Four hidden coupling vectors validated and corrected |

---

## 2. Hidden Coupling Vectors — Validation

### Vector A: Time Semantics ✅ VALIDATED — Systemic

`Utc::now()` / `SystemTime::now()` / `Instant::now()` found in **16 runtime-portable services**, **31 call sites** (workspace baseline re-check on 2026-02-16):

| Service | Call Sites | Usage Pattern |
|---|---|---|
| `streaming_transport.rs` | 8 | Timestamps, nonces, staleness checks, backlog IDs |
| `cortex_ux_store.rs` | 5 | Replay timestamps, generation IDs |
| `cortex_ux.rs` | 1 | ISO timestamp helper |
| `acp_event_sink.rs` | 1 | Event timestamps |
| `acp_event_projector.rs` | 1 | Projection timestamps |
| `acp_adapter.rs` | 1 | Nonce generation (nanoseconds) |
| `acp_metrics.rs` | 1 | Metric timestamps |
| `acp_permission_ledger.rs` | 1 | Ledger timestamps |
| `acp_session_store.rs` | 1 | Session timestamps |
| `viewspec.rs` | 2 | Timestamps, event IDs |
| `viewspec_synthesis.rs` | 1 | Set IDs |
| `viewspec_learning.rs` | 2 | Signal timestamps, replay IDs |
| `resilience_service.rs` | 1 | Probe timestamps |
| `agent_service.rs` | 2 | Agent event timestamps |
| `local_gateway.rs` | 2 | Duration tracking |
| `workflow_service.rs` | 1 | Workflow timestamps |

**Assessment**: This is not peripheral. Time is embedded in:
- Event identity (timestamp-based IDs)
- Staleness detection (wall-clock comparison)
- CRDT ordering (lamport + wall-clock)
- Nonce generation (nanosecond precision)

Without `TimeProvider`, the runtime implicitly trusts the host's clock. That means:
- Event ordering becomes host-dependent
- CRDT timestamps become host-dependent  
- Replay determinism is impossible

**Correction accepted.** Add `TimeProvider` trait.

---

### Vector B: Error Types ✅ VALIDATED — Moderate

`std::io::Error` found in 2 runtime-portable services:
- `cortex_ux_store.rs` — file persistence errors propagated
- `file_system_service.rs` — expected (stays in host)

Additional risk: many services use `anyhow::Result` or `String` errors, which could carry OS-specific context in error messages.

**Correction accepted.** Runtime must define its own error enum, not propagate `std::io::Error`.

---

### Vector C: Logging ✅ VALIDATED — 8 Services

`log::` / `tracing::` / `println!` / `eprintln!` found in 8 runtime-portable services:
- `acp_event_sink.rs`, `acp_metrics.rs`, `local_gateway.rs`, `agent_service.rs`, `workflow_service.rs`, `lint_service.rs`, `unified_config.rs`, `local_gateway.rs`

These use env-based filter configuration (`RUST_LOG`, `tracing_subscriber`). If logging config is read from env inside the runtime, you've reintroduced `std::env` through the back door.

**Correction accepted.** Add `LogAdapter` trait. Runtime emits structured log events; host routes them.

---

### Vector D: State Mutation ✅ VALIDATED — Critical

`local_gateway.rs` exposes runtime state through a **global static**:
```rust
static GATEWAY: OnceLock<LocalGateway> = OnceLock::new();
```

With **22+ `.lock().unwrap()` mutex sites** for direct state mutation. Any component in the desktop process — UI, gateway handler, or background task — can reach in and mutate gateway state via the global accessor.

This is the most dangerous coupling vector: if Desktop can mutate runtime state directly after extraction, sovereignty is cosmetic.

**Correction accepted.** Add opaque state boundary: host may not access runtime internals except via defined request/response interfaces.

---

## 3. Cortex Runtime Purity Contract v1.2

```
CORTEX RUNTIME PURITY CONTRACT v1.2

═══ Compilation Constraints ═══

1. MUST compile to wasm32-unknown-unknown with --no-default-features
   → CI: cargo check --target wasm32-unknown-unknown --no-default-features

═══ Forbidden Dependencies (Cargo.toml) ═══

2. MUST NOT depend on:
   - dioxus, dioxus-desktop     (UI framework)
   - portable-pty               (terminal emulation)
   - rfd                        (native file dialogs)
   - home                       (home directory)
   - walkdir                    (filesystem traversal)
   - ic-agent, candid           (ICP-specific)
   - axum, tower, tower-http    (HTTP framework)

═══ Forbidden APIs (source code) ═══

3. MUST NOT use:
   - std::fs, std::env, std::process, std::net
   - tokio::process, tokio::fs, tokio::net
   - std::io::Error in public API surface

═══ Allowed Async Runtime ═══

4. tokio = { default-features = false, features = ["rt", "sync", "time", "macros"] }

═══ Adapter Traits (host provides) ═══

5. StorageAdapter     — all persistence
6. NetworkAdapter     — all network IO (including ICP via cortex-ic-adapter)
7. TimeProvider       — wall-clock, monotonic, and scheduling time
8. LogAdapter         — structured log event emission

═══ Configuration ═══

9. ALL config via RuntimeConfig (host-injected at boot)
   No env var reads inside runtime.

═══ Event Architecture ═══

10. ALL state mutations MUST emit Events via EventBus
    Side-effects go through adapter traits only.

═══ Error Model ═══

11. Runtime errors MUST be:
    - Enum-based (RuntimeError)
    - Substrate-neutral (no OS error variants)
    - Serializable
    - Deterministic

═══ State Sovereignty ═══

12. Runtime state is OPAQUE to host.
    Host may not access or mutate runtime internals.
    All interaction via defined request/response interfaces only.

═══ Testability ═══

13. MUST be testable headless.
    Full test suite without OS, window, or network.
    Mock adapters provided for all traits.
```

---

## 4. Phase 5 Elevated: Gateway Protocol Contract

The critique correctly identifies `server.rs` as more than an HTTP binding. It is currently:
- Request orchestrator
- Lifecycle manager
- Implicit transaction boundary
- Error aggregation surface

**New prerequisite — Phase 4.5**:

Before Phase 5 extraction, produce a **Gateway Protocol Contract** defining:

| Aspect | Definition Required |
|---|---|
| Request schema | Every gateway request type with required/optional fields |
| Response schema | Normalized response envelope |
| Error model | How runtime errors map to gateway error responses |
| Event expectations | Which events each request type emits |
| Transaction boundaries | Which operations are atomic |
| Idempotency | Which operations are safe to retry |

This prevents the 16K-line split from introducing silent behavior changes.

---

## 5. Updated Extraction Sequence

```
Phase 0: Constitutional Declaration
├── Purity Contract v1.2
├── CI firewall script
├── Empty cortex-runtime crate (compiles to wasm32)
├── Empty cortex-ic-adapter crate
└── Trait definitions: StorageAdapter, NetworkAdapter,
    TimeProvider, LogAdapter, EventBus

Phase 1: Event Engine (sovereignty inflection point)
├── acp_event_sink → cortex-runtime/events/
├── acp_event_projector → cortex-runtime/events/
├── Replace Utc::now() with TimeProvider
├── Replace log!/tracing! with LogAdapter
└── Desktop implements all adapter traits

Phase 2: Policy & Collaboration
├── acp_protocol, acp_meta_policy, acp_metrics → runtime/policy/
├── artifact_collab_crdt → runtime/collaboration/
├── acp_permission_ledger, acp_session_store → runtime/policy/
├── Replace std::env with RuntimeConfig
└── Replace std::fs persistence with StorageAdapter

Phase 3: Governance & Workflow
├── governance_client → split: logic (runtime) + ICP (adapter)
├── workflow_engine_client → split: logic (runtime) + ICP (adapter)
├── workflow_service, workflow_executor → runtime/workflow/
└── Define GovernanceAdapter, WorkflowAdapter

Phase 4: Agent & ViewSpec
├── agent_service → split: orchestration (runtime) + spawn (host)
├── viewspec*, viewspec_learning, viewspec_synthesis → runtime/viewspec/
├── cortex_ux compatibility facade (host API) + pure evaluation split to runtime/ux + domain/ux
└── resilience_service → runtime/resilience/

Phase 4.5: Gateway Protocol Contract
├── Document request/response schema
├── Define error normalization model
├── Define event emission expectations
└── Define transaction boundaries

Phase 5: Gateway Protocol Extraction
├── server.rs → protocol logic (runtime) + HTTP binding (host)
├── Eliminate static GATEWAY global
├── Runtime state becomes opaque
└── Desktop = host adapter consuming CortexRuntime trait
```

---

## 6. Validated Trait Surface

```rust
// ═══ Host → Runtime (host provides these) ═══

pub trait StorageAdapter: Send + Sync {
    async fn read(&self, key: &str) -> Result<Option<Vec<u8>>, RuntimeError>;
    async fn write(&self, key: &str, data: &[u8]) -> Result<(), RuntimeError>;
    async fn delete(&self, key: &str) -> Result<(), RuntimeError>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>, RuntimeError>;
}

pub trait NetworkAdapter: Send + Sync {
    async fn request(&self, req: NetworkRequest) -> Result<NetworkResponse, RuntimeError>;
}

pub trait TimeProvider: Send + Sync {
    fn now_utc(&self) -> DateTime<Utc>;
    fn now_monotonic(&self) -> Duration;
    fn sleep(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

pub trait LogAdapter: Send + Sync {
    fn emit(&self, event: RuntimeLogEvent);
}

// ═══ Runtime → Host (runtime exposes these) ═══

pub trait CortexRuntime: Send + Sync {
    async fn handle_request(&self, req: GatewayRequest) -> GatewayResponse;
    fn subscribe_events(&self) -> EventStream;
    async fn start_workflow(&self, spec: WorkflowSpec) -> Result<WorkflowHandle, RuntimeError>;
}
```

---

## 7. Initiative 118 — Final Specification

### `118-cortex-runtime-extraction`

| Field | Value |
|---|---|
| **ID** | 118 |
| **Directory** | `118-cortex-runtime-extraction` |
| **Status** | Draft |
| **Layer** | Architectural |
| **Primary Steward** | Systems Steward |
| **Domain** | Agents & Execution |

### Deliverables

| # | Deliverable | Phase |
|---|---|---|
| 1 | Runtime Purity Contract v1.2 | 0 |
| 2 | CI dependency firewall | 0 |
| 3 | `cortex-runtime` crate | 0–5 |
| 4 | `cortex-ic-adapter` crate | 3 |
| 5 | Adapter traits (Storage, Network, Time, Log) | 0 |
| 6 | Gateway Protocol Contract | 4.5 |
| 7 | Updated Desktop (thin host) | 5 |

### Success Criteria

1. `cortex-runtime` compiles to `wasm32-unknown-unknown --no-default-features`
2. CI purity check passes on every PR
3. Zero `ic-agent`/`candid` in runtime
4. Zero `std::fs`/`std::env`/`std::process`/`std::net` in runtime
5. Zero `Utc::now()` / `SystemTime::now()` in runtime (uses `TimeProvider`)
6. Zero `log!` / `tracing!` in runtime (uses `LogAdapter`)
7. No `std::io::Error` in runtime's public API
8. No global statics (`OnceLock`, `Mutex<...>`) exposing runtime state to host
9. Desktop functions identically after extraction (behavior parity)
10. Full runtime test suite runs headless with mock adapters

---

## 8. The Completed Picture

```
Before Initiative 118:

  ┌──────────────────────────────────────────┐
  │           CORTEX DESKTOP                  │
  │  ┌──────────────────────────────────────┐ │
  │  │  15,976-line server.rs monolith      │ │
  │  │  24 services reading std::env        │ │
  │  │  16 services using std::fs           │ │
  │  │  43+ Utc::now() call sites           │ │
  │  │  ic-agent in 3 services              │ │
  │  │  Global Mutex state                  │ │
  │  └──────────────────────────────────────┘ │
  │  Dioxus UI                                │
  └──────────────────────────────────────────┘

After Initiative 118:

  ┌──────────────────────┐   ┌──────────────────────┐
  │  CORTEX DESKTOP      │   │  FUTURE WEB HOST     │
  │  (thin host adapter) │   │  (peer host adapter)  │
  │  • Dioxus UI(Deprec) │   │  • Browser UI         │
  │  • StorageAdapter    │   │  • StorageAdapter      │
  │  • NetworkAdapter    │   │  • NetworkAdapter      │
  │  • TimeProvider      │   │  • TimeProvider        │
  │  • LogAdapter        │   │  • LogAdapter          │
  └────────┬─────────────┘   └────────┬──────────────┘
           │                          │
           ▼                          ▼
  ┌──────────────────────────────────────────┐
  │           CORTEX RUNTIME                  │
  │  • EventBus (sovereignty center)          │
  │  • Policy/Governance (pure logic)         │
  │  • Workflow (pure logic)                  │
  │  • CRDT Collaboration (pure logic)        │
  │  • Agent Orchestration (pure logic)       │
  │  • ViewSpec Evaluation (pure logic)       │
  │  • Opaque State (no host access)          │
  │                                           │
  │  Compiles to wasm32-unknown-unknown       │
  │  Zero host dependencies                   │
  └──────────────────────────────────────────┘
           │
           ▼
  ┌──────────────────────┐
  │  CORTEX-IC-ADAPTER   │
  │  • ic-agent          │
  │  • candid            │
  │  • ICP canister calls│
  └──────────────────────┘
```

---

> *"Sovereignty is not found in clean code. It is found in declared constraints that clean code must obey."*
