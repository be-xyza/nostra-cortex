---
status: draft
stewardship:
  layer: Systems
  primary_steward: Systems Steward
  domain: Agents & Execution
---

# Initiative 118 — Execution Plan

## Cortex Runtime Purity Contract v1.3

```
CORTEX RUNTIME PURITY CONTRACT v1.3

═══ Compilation ═══

1. MUST compile to wasm32-unknown-unknown with --no-default-features

═══ Forbidden Crates (Cargo.toml) ═══

2. MUST NOT depend on:
   dioxus, dioxus-desktop, portable-pty, rfd, home, walkdir,
   ic-agent, candid, axum, tower, tower-http

═══ Forbidden APIs (source) ═══

3. MUST NOT use:
   std::fs, std::env, std::process, std::net,
   tokio::process, tokio::fs, tokio::net,
   std::io::Error in public API,
   log!, tracing!, println!, eprintln!,
   Utc::now(), SystemTime::now(), Instant::now()

   > [!NOTE]
   > **StorageAdapter v1** is intentionally minimal (`read`, `write`, `delete`, `list`).
   > Research 085 (File Infrastructure) defines a richer surface including tiered storage,
   > versioning, deduplication, and economics. StorageAdapter will evolve when 085 reaches
   > implementation. Do not over-design the v1 surface.

═══ Allowed Async ═══

4. tokio = { default-features = false, features = ["rt", "sync", "time", "macros"] }

═══ Adapter Traits (host provides) ═══

5.  StorageAdapter  — all persistence
6.  NetworkAdapter  — all network IO
7.  TimeProvider    — wall-clock, monotonic, scheduling
8.  LogAdapter      — structured log emission

═══ Configuration ═══

9.  ALL config via RuntimeConfig (injected at boot)

═══ Events ═══

10. ALL state mutations emit Events via EventBus

═══ Error Model ═══

11. RuntimeError enum — substrate-neutral, serializable, no OS variants

═══ State Sovereignty ═══

12. Runtime state OPAQUE to host — request/response interface only

═══ Adapter Constraint ═══

13. Adapters are side-effect providers only — never mutation authorities

═══ Determinism ═══

14. BTreeMap where iteration order affects output
15. Document f32/f64 rounding behavior in scoring

═══ Testability ═══

16. Full test suite headless with mock adapters
```

---

## Extraction Phases

### Phase 0: Constitutional Declaration

**Prerequisites** (must complete before crate creation):
- Audit `nostra-workflow-core/alignment.rs` to ensure it remains substrate-neutral (no `candid` imports). This unblocks clean extraction of workflow types into `cortex-domain`.
- Document OnceLock elimination plan for 9 extraction-target files (see Appendix B)

**Phase 0 closure rule (locked)**:
- Phase 0 is complete only when every `METHOD + PATH` pair in `endpoint_inventory.tsv`
  has exactly one parity fixture in `parity_cases/`, or is listed in
  `approved_exemptions.json` with explicit rationale.
- Default rule: `approved_exemptions_count == 0`.

**Deliverables**:
- [x] Record gateway behavior baseline
  - Capture request/response fixtures for all gateway endpoints (inventory-locked, one fixture per endpoint)
  - Store fixtures in `cortex/apps/cortex-desktop/tests/fixtures/gateway_baseline/`
  - Maintain both inventory formats:
    - `endpoint_inventory.tsv` (canonical source for endpoint count)
    - `endpoint_inventory.json` (machine-readable mirror)
  - Enforce count lock in tests:
    `inventory_count == fixture_count + approved_exemptions_count`
  - Default policy keeps `approved_exemptions_count == 0`
- Runtime Purity Contract v1.3 committed
- CI firewall scripts (domain purity + runtime purity)
- `cortex/libraries/cortex-domain/` crate — pure, sync, `#![forbid(unsafe_code)]`, compiles to wasm32
- `cortex/libraries/cortex-runtime/` crate — async orchestration, restricted tokio, compiles to wasm32
- `cortex/libraries/cortex-ic-adapter/` crate — ICP adapter
- Port trait definitions in `cortex-runtime`: `StorageAdapter`, `NetworkAdapter`, `TimeProvider`, `LogAdapter`, `EventBus`
- `DomainError` enum in `cortex-domain`
- `RuntimeError` enum in `cortex-runtime`
- `RuntimeConfig` type in `cortex-runtime`
- `EdgeKind` enum in `cortex-domain` — typed relationship taxonomy (see ADR-014)
- Graph traversal utilities in `cortex-domain` — cycle detection, topological sort, dependency walk
- `StructuralDiff` type in `cortex-domain` — pure graph diff (nodes/edges added/removed/changed)
- Workspace `Cargo.toml` updated with new members

**Verification**: 
- `cargo check -p cortex-domain --target wasm32-unknown-unknown` passes
- `cargo check -p cortex-runtime --target wasm32-unknown-unknown --no-default-features` passes
- Domain CI purity check passes (no async, no unsafe, no forbidden deps)
- Runtime CI purity check passes (no candid/ic-agent, restricted tokio)
- Graph traversal unit tests pass (`cargo test -p cortex-domain -- graph`)
- `nostra-workflow-core` compiles without `candid` dependency
- `gateway_parity` test passes with full inventory lock and zero approved exemptions

---

### Phase 1: Event Engine (Sovereignty Inflection Point)

**Extract**:
- `acp_event_sink.rs` → `cortex-runtime/src/event_bus.rs`
- `acp_event_projector.rs` → `cortex-runtime/src/event_bus.rs`
- Event schema types → `cortex-domain/src/events.rs`

**Modifications**:
- Replace `Utc::now()` with `TimeProvider` (runtime) or time parameter (domain)
- Replace `log!`/`tracing!` with `LogAdapter`
- Replace `std::fs` persistence with `StorageAdapter`

**Desktop changes**: Implement `StorageAdapter` (filesystem-backed), `TimeProvider` (chrono-backed), `LogAdapter` (tracing-backed)

**Verification**:
- CI purity checks pass for both crates
- `cortex_runtime_v0` remains default-off behind feature and runtime toggle
- ACP event-path parity matrix passes across all `AcpSessionUpdateKind` variants
- Shadow mismatch tests fail on non-allowed drift and allow timestamp-only drift
- Cloud event parity tests pass with timestamp normalization at second precision
- Desktop behavior unchanged under legacy path

---

### Stop Gate Before Phase 2

Phase 2 and higher are blocked until all conditions below are green in a single run:
- Phase 0 verifications
- Phase 1 verifications
- No parity inventory gaps
- No unresolved shadow mismatches
- Closure evidence recorded in `PHASE_0_1_CLOSURE_EVIDENCE_2026-02-15.md`
- Entry packet prepared and complete:
  `PHASE_2_ENTRY_PACKET_2026-02-15.md`

### Pre-Phase-2 Operational Checklist

The CI job `cortex-runtime-freeze-gates` in `.github/workflows/test-suite.yml` is the
authoritative freeze gate before any Phase 2 kickoff branch is approved.

- `cortex-runtime-freeze-gates` is green on latest `main`
- `cortex-runtime-freeze-gates` is green on latest PR candidate
- `initiative-118-evidence-gate` is green on latest PR candidate
- Enforced merge path:
  - Preferred: branch protection marks `cortex-runtime-freeze-gates` as required before merge
  - Interim (no ruleset/required-check support): merges are steward-only and require PR evidence bundle (freeze-gate run URL + attached `logs/testing/freeze_gates/*` + inventory lock counts)
- PR evidence format is mandatory for 118-scope changes:
  - `118_SCOPE_APPLIES=yes`
  - `Freeze gate run URL: https://github.com/<owner>/<repo>/actions/runs/<id>`
  - `Inventory counts: inventory=<n> fixtures=<n> exemptions=<n>`
  - `Evidence files attached: yes`
- Out-of-scope override is allowed only when CI path matcher confirms non-118 scope:
  - `118_SCOPE_APPLIES=no`
- `check_gateway_parity_inventory_sync.sh` reports synchronized inventory
- `approved_exemptions_count == 0` in `approved_exemptions.json`
- ACP shadow tests show no non-allowed mismatch regressions

Operational command contract:
- Non-mutating sync check: `bash scripts/check_gateway_parity_inventory_sync.sh`
- Full freeze gate run: `bash scripts/run_cortex_runtime_freeze_gates.sh`
- PR evidence validation: `bash scripts/check_118_pr_evidence.sh --pr-body-file <path>`
- Manual local refresh only: `python3 scripts/refresh_gateway_parity_fixtures.py`

### Phase 2 Entry Unblock Status (2026-02-15)

- `cortex/Cargo.toml` and `cortex/apps/cortex-desktop/Cargo.toml` restored and tracked.
- Freeze-gate runtime assets restored:
  - `cortex-domain`, `cortex-runtime`, `cortex-ic-adapter`
  - gateway parity baseline fixtures + inventory lock
  - freeze/evidence scripts and CI workflow guards
- Local gate contract rerun and green:
  - `bash scripts/check_gateway_parity_inventory_sync.sh`
  - `bash scripts/run_cortex_runtime_freeze_gates.sh`
  - `bash scripts/check_118_pr_evidence.sh --pr-body-file tests/fixtures/pr_evidence/valid.md`
  - `bash tests/scripts/test_check_118_pr_evidence.sh`
- Remaining unfreeze prerequisite:
  - record green CI runs for `cortex-runtime-freeze-gates` on latest `main` and PR candidate, then steward records formal unfreeze for Phase 2 PR-1 only.

---

### Phase 2: Policy & Collaboration

**Extract**:
- `acp_protocol.rs` (1,015 lines) → **split**: protocol types to `cortex-domain/policy/`, stateful logic to `cortex-runtime/policy/protocol.rs` (4 OnceLock, 6 std::env — all must be eliminated)
- `acp_adapter.rs` (527 lines) → **split**: `OperationAdapter` trait to `cortex-runtime/policy/adapter.rs`, evidence implementation stays in host
- `acp_meta_policy.rs` (96 lines) → `cortex-domain/policy/meta.rs` (zero violations — pure domain)
- `acp_metrics.rs` (331 lines) → `cortex-runtime/policy/metrics.rs` (2 OnceLock, 3 std::env)
- `acp_permission_ledger.rs` (167 lines) → `cortex-runtime/policy/permissions.rs`
- `acp_session_store.rs` (288 lines) → `cortex-runtime/policy/sessions.rs`
- `artifact_collab_crdt.rs` (466 lines) → `cortex-domain/collaboration/crdt.rs` (3 Utc::now — time injection required)

**Modifications**:
- Replace `std::env` with `RuntimeConfig`
- Replace `std::fs` with `StorageAdapter`
- Eliminate 6 OnceLock instances across `acp_protocol.rs` and `acp_metrics.rs`
- Audit `HashMap` usage in policy paths

**Verification**: CI purity check passes. CRDT convergence tests pass in domain crate. Zero OnceLock in extracted code.

---

### Phase 3: Governance, Workflow & Streaming

**Extract (with split)**:
- `governance_client.rs` (148 lines) → logic to `cortex-runtime/governance/`, ICP calls to `cortex-ic-adapter/`
- `workflow_engine_client.rs` (371 lines) → logic to `cortex-runtime/workflow/`, ICP calls to `cortex-ic-adapter/`
- `workflow_service.rs` (515 lines) → `cortex-runtime/workflow/service.rs` (5 std::env → RuntimeConfig)
- `workflow_executor.rs` (127 lines) → `cortex-runtime/workflow/executor.rs` (zero violations)
- `streaming_transport.rs` (1,178 lines) → **4-way split** (highest-complexity extraction target):
  - Streaming protocol types → `cortex-domain/streaming/types.rs`
  - Streaming orchestration → `cortex-runtime/streaming/transport.rs`
  - ICP transport (ic_agent, candid) → `cortex-ic-adapter/streaming.rs`
  - Process management (Command) → host
- `dfx_client.rs` (344 lines) → `cortex-ic-adapter/dfx.rs` (ICP CLI wrapper)

**New traits**: `GovernanceAdapter`, `WorkflowAdapter`, `StreamingTransportAdapter`

**Verification**: CI purity check passes. Zero `ic-agent` in runtime. Zero `std::process::Command` in runtime. Streaming protocol tests pass with mock transport.

---

### Phase 4: Agent, ViewSpec & UX

**Extract (with split)**:
- `agent_service.rs` (486 lines) → orchestration to `cortex-runtime/agents/`, process spawn to Desktop host (2 OnceLock to eliminate)
- `viewspec.rs` (848 lines) → `cortex-runtime/viewspec/`
- `viewspec_learning.rs` (594 lines) → `cortex-runtime/viewspec/learning.rs`
- `viewspec_synthesis.rs` (343 lines) → `cortex-runtime/viewspec/synthesis.rs`
- `cortex_ux` compatibility facade (new in `cortex-desktop/src/services/cortex_ux.rs`) → stable host API while splitting pure scoring to `cortex-domain/ux/`, orchestration to `cortex-runtime/ux/`, persistence to host
- `cortex_ux_store.rs` (511 lines) → **split**: types to `cortex-domain/ux/types.rs`, persistence to host (2 OnceLock, 5 Utc::now)
- `theme_policy.rs` (184 lines) → **split**: policy rules to `cortex-domain/theme/`, file loading to host (2 OnceLock)
- `resilience_service.rs` (163 lines) → `cortex-runtime/resilience/`

**Determinism audit**: Document `f32` scoring behavior in `cortex_ux` and `viewspec_learning`.

**Workspace re-baseline (2026-02-16)**:
- Prior references to legacy `services/cortex_ux.rs` line counts were stale for this workspace state.
- Phase 4 executes against current tracked files in `cortex/apps/cortex-desktop/src/services/` and keeps a compatibility facade for route/component/gateway contracts during extraction.

**Verification**: CI purity check passes. ViewSpec evaluation tests pass headless. Zero OnceLock in extracted code.

---

### Phase 4.5: Gateway Protocol Contract

**Deliverable**: Document defining:
- Request/response schema for every gateway endpoint
- Error normalization model
- Event emission expectations per request type
- Transaction boundaries
- Idempotency semantics

**No code changes in this phase.**

---

### Phase 5: Gateway Protocol Extraction

**Extract (with split)**:
- `gateway/server.rs` (15,975 lines, **126 purity violations**) → protocol logic to `cortex-runtime/gateway/`, HTTP binding stays in Desktop
- `local_gateway.rs` (1,119 lines) → part of gateway protocol extraction (2 OnceLock)
- `gateway_config.rs` (79 lines) → config types feed `RuntimeConfig` (2 OnceLock, 4 std::env)
- Eliminate `static GATEWAY: OnceLock<LocalGateway>` global and all gateway OnceLock instances
- Runtime state becomes opaque via `CortexRuntime` trait

**Desktop becomes**: thin HTTP adapter → `CortexRuntime` trait consumer

**Verification**: CI purity check passes. All gateway endpoints functional. Runtime test suite fully headless. Desktop behavior parity confirmed.

---

## Success Criteria

1. `cortex-runtime` compiles to `wasm32-unknown-unknown --no-default-features`
2. CI purity check passes on every PR
3. Zero `ic-agent`/`candid` in runtime
4. Zero `std::fs`/`std::env`/`std::process`/`std::net` in runtime
5. Zero `Utc::now()`/`SystemTime::now()` in runtime (uses `TimeProvider`)
6. Zero `log!`/`tracing!` in runtime (uses `LogAdapter`)
7. No `std::io::Error` in runtime public API
8. No global statics exposing runtime state to host
9. Desktop functions identically after extraction
10. Full runtime test suite runs headless with mock adapters

## Behavior Parity Verification Plan

Success Criteria #9 requires "Desktop functions identically after extraction." This requires:

1. **Pre-extraction baseline**: Before Phase 1 begins, record request/response pairs for all gateway endpoints
2. **Regression test suite**: Automated tests that replay recorded requests against post-extraction Desktop and compare responses
3. **Per-phase gate**: Each phase must pass the regression suite before the next phase begins
4. **Rollback path**: Each phase must be independently revertible via feature flags or conditional compilation. Desktop must remain buildable from `main` at every phase boundary.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| Phase 5 gateway split introduces behavior changes | High | Gateway Protocol Contract (Phase 4.5) |
| `TimeProvider` migration across 43+ sites | Medium | Incremental per-phase replacement |
| `streaming_transport.rs` 4-way split complexity | High | Phase 3 — extract protocol types first, transport last |
| `OnceLock` in 9 extraction-target files (not just server.rs) | Medium | Eliminate per-phase with OnceLock plan (Appendix B) |
| `nostra-workflow-core` may regress to substrate-specific deps | Medium | Phase 0 prerequisite — enforce no-`candid` guardrail |
| No behavior parity tests exist | High | Pre-extraction baseline recording (see above) |
| Float determinism in scoring | Low | Document, don't fix unless divergence proven |
| HashMap iteration order | Low | Audit and replace with BTreeMap where needed |
| Active initiatives disrupted (106–116) | Medium | Extract incrementally, never break Desktop build |

---

## Appendix A: File Disposition Matrix

> [!NOTE]
> **Matrix freshness (2026-02-16)**: This matrix is a historical classification snapshot.
> Phase 4 is re-baselined to current tracked files and the `cortex_ux` compatibility facade.
> Current line counts for frequently-referenced files: `cortex_ux.rs` (568),
> `streaming_transport.rs` (1,178), `acp_protocol.rs` (637). Re-validate line-level split
> decisions before starting each phase.

All 84 Cortex Desktop source files classified. **D** = Domain, **R** = Runtime, **A** = Adapter, **H** = Host-Only, **S** = Split.

### Services (41 files, 13,855 lines)

| File | Lines | Phase | Dest | Key Violations |
|---|---|---|---|---|
| `streaming_transport.rs` | 1,178 | 3 | **S** | 7 utc, 2 ic, 2 once, 1 cmd |
| `local_gateway.rs` | 1,119 | 5 | **S** | 2 once, 2 fs |
| `cortex_ux.rs` | 568 | 4 | **S** | 3 env, 1 utc, 3 fs |
| `acp_protocol.rs` | 637 | 2 | **S** | 6 env, 4 once |
| `viewspec.rs` | 848 | 4 | R | 2 utc |
| `viewspec_learning.rs` | 594 | 4 | R | 2 utc |
| `terminal_service.rs` | 536 | — | H | 5 once, 1 cmd |
| `acp_adapter.rs` | 527 | 2 | **S** | 1 env, 1 fs, 1 utc |
| `workflow_service.rs` | 515 | 3 | R | 5 env, 1 utc |
| `cortex_ux_store.rs` | 511 | 4 | **S** | 5 utc, 2 once, 2 env |
| `github_mcp_service.rs` | 510 | — | H | 1 env, 1 fs, 1 cmd |
| `agent_service.rs` | 486 | 4 | **S** | 2 utc, 2 once, 1 cmd |
| `artifact_collab_crdt.rs` | 466 | 2 | D | 3 utc |
| `workflow_engine_client.rs` | 371 | 3 | **S** | 2 ic, 1 cmd, 1 env |
| `motoko_graph_service.rs` | 364 | — | H | clean |
| `dfx_client.rs` | 344 | 3 | A | 1 env, 1 fs, 1 cmd |
| `viewspec_synthesis.rs` | 343 | 4 | R | 1 utc |
| `acp_metrics.rs` | 331 | 2 | R | 2 once, 3 env, 3 fs |
| `acp_session_store.rs` | 288 | 2 | R | 1 utc, 2 fs, 1 env |
| `testing_service.rs` | 240 | — | H | clean |
| `theme_policy.rs` | 184 | 4 | **S** | 2 once, 2 env, 1 fs |
| `acp_event_sink.rs` | 176 | 1 | R | 2 env, 1 fs, 1 utc |
| `acp_permission_ledger.rs` | 167 | 2 | R | 1 env, 1 fs, 1 utc |
| `theme.rs` | 163 | — | H | 1 env |
| `resilience_service.rs` | 163 | 4 | R | 1 utc |
| `governance_client.rs` | 148 | 3 | **S** | 2 ic, 1 cmd, 1 env |
| `supervisor.rs` | 145 | — | H | 1 cmd |
| `snapshot_service.rs` | 136 | — | H | 1 cmd, 1 env, 1 utc |
| `workflow_executor.rs` | 127 | 3 | R | clean |
| `acp_event_projector.rs` | 121 | 1 | R | 2 utc |
| `backend_service.rs` | 99 | — | H | clean |
| `acp_meta_policy.rs` | 96 | 2 | D | clean |
| `unified_config.rs` | 86 | 0 | H | clean |
| `gateway_config.rs` | 79 | 5 | **S** | 4 env, 2 once |
| `file_system_service.rs` | 79 | 1 | H | 1 env, 1 fs |
| `lint_service.rs` | 71 | — | H | 1 env, 1 fs |
| `local_connection.rs` | 61 | — | H | 1 cmd |
| `version_manager.rs` | 49 | — | H | clean |
| `console_service.rs` | 47 | — | H | 2 once |
| `auth_service.rs` | 12 | — | H | clean |

### Gateway (3 files, 15,997 lines)

| File | Lines | Phase | Dest |
|---|---|---|---|
| `server.rs` | 15,975 | 5 | **S** (126 violations) |
| `state.rs` | 20 | 5 | H |

### Components (33 files, 8,590 lines) — All Host-Only

### Root (4 files, 359 lines) — All Host-Only

---

## Appendix B: OnceLock Elimination Matrix

**Inventory valid as of 2026-02-14 codebase scan.**

| File | Static Name | Extraction-Relevant | Phase |
|---|---|---|---|
| `acp_protocol.rs` | `ACP_RUNTIME` | ✅ | 2 |
| `acp_protocol.rs` | `ENV_LOCK` | ✅ | 2 |
| `acp_metrics.rs` | `ACP_METRICS` | ✅ | 2 |
| `cortex_ux_store.rs` | `STORE` | ✅ | 4 |
| `streaming_transport.rs` | `MANAGER` | ✅ | 3 |
| `theme_policy.rs` | `CACHE` | ✅ | 4 |
| `console_service.rs` | `CONSOLE_CHANNEL` | ❌ Host | — |
| `server.rs` | `NONCES`, `LOCK` ×2 | ✅ | 5 |
| `terminal_service.rs` | `OUTPUT_CHANNEL`, `INPUT_CHANNEL`, `ACP_TERMINALS` | ❌ Host | — |
| `agent_service.rs` | `STATE_CHANNEL` | ✅ | 4 |
| `local_gateway.rs` | `GATEWAY` | ✅ | 5 |
| `gateway_config.rs` | `LOCK` | ✅ | 5 |
| `log_view.rs` | `LOG_CHANNEL` | ❌ Host | — |

9 files with OnceLock that **must be eliminated** during extraction:

| File | Count | Phase | Action |
|---|---|---|---|
| `acp_protocol.rs` | 4 | 2 | Replace with `RuntimeConfig` injection |
| `acp_metrics.rs` | 2 | 2 | Replace with adapter-injected state |
| `streaming_transport.rs` | 2 | 3 | Replace with injected transport handle |
| `agent_service.rs` | 2 | 4 | Replace with runtime-managed state |
| `cortex_ux_store.rs` | 2 | 4 | Replace with `StorageAdapter` |
| `theme_policy.rs` | 2 | 4 | Replace with config injection |
| `server.rs` | ~20+ | 5 | Primary Phase 5 target — `CortexRuntime` trait |
| `local_gateway.rs` | 2 | 5 | Part of gateway extraction |
| `gateway_config.rs` | 2 | 5 | Config → `RuntimeConfig` |
3 files with OnceLock that are **host-only** (acceptable):
- `terminal_service.rs` (5), `console_service.rs` (2), `log_view.rs` (1)
