# Crate Separation Strategy — Validation

> **Date**: 2026-02-11  
> **Context**: Round 6 of Initiative 118 architectural analysis  
> **Question**: Should `cortex-runtime` use internal modules or separate crates per onion layer?

---

## 1. Verdict

**The argument for separate crates is correct. But the proposal needs three corrections.**

| Claim | Validated? | Evidence |
|---|---|---|
| Separate crates enforce boundaries harder than modules | ✅ Correct | Compiler prevents inward imports — no CI grep needed |
| Greenfield = cheapest time to decide | ✅ Correct | No migration, no API breakage, no contributor confusion |
| Single-crate boundaries erode over time | ✅ Correct — and already proven in this codebase | See §2 |
| Proposed 5-crate structure is optimal | ⚠️ Partially — see §4 |
| Workspace overhead is minimal | ✅ Correct — workspace already has 15 members |

**Previous recommendation (ADR-012: modules-only) is superseded.** The greenfield argument and existing workspace precedent justify the pivot.

---

## 2. The Smoking Gun: Existing Library Purity Violations

The strongest evidence for separate crates comes from **this project's own libraries**:

| Crate | Intended Role | Purity Violation |
|---|---|---|
| `nostra-core` | Core domain types | Depends on `candid` (ICP-specific) + `log` (env-coupled) |
| `nostra-workflow-core` | Workflow domain | Depends on `log` (env-coupled); historical `candid` coupling was removed from alignment types |
| `nostra-ui-core` | UI types | ✅ Clean — only `serde`, `serde_json` |
| `nostra-cloudevents` | Event types | ✅ Clean — only `serde`, `chrono`, `thiserror`, `url`, `uuid` |

`nostra-core` and `nostra-workflow-core` demonstrate exactly the drift the proposal warns about. `candid` leaked into core library boundaries historically, and `log` is still present in workflow core. Cultural boundaries were insufficient. These were well-intentioned libraries that drifted because there was no structural enforcement.

`nostra-ui-core` and `nostra-cloudevents` stayed clean because they were small and narrowly scoped. That's not a guarantee — it's luck.

**This is empirical proof that module-level purity erodes in this codebase.**

---

## 3. Workspace Precedent

The Nostra workspace already has **15 members**:

```
nostra/
├── apps/cortex-desktop/
├── backend/governance/
├── backend/workflow_engine/
├── extraction/
├── frontend/
├── labs/memory_evolution/
├── libraries/nostra-cloudevents/
├── libraries/nostra-core/
├── libraries/nostra-test-kit/
├── libraries/nostra-ui-core/
├── libraries/nostra-workflow-core/
├── log-registry/
├── shared/
├── streaming/
├── worker/
└── scripts/ingest_gaia/
```

Adding 3–5 more crates to `libraries/` is not a structural departure. It's consistent with existing practice. The workspace overhead argument against separate crates is invalid — the project already pays that cost.

---

## 4. Three Corrections to the Proposal

### Correction 1: `cortex-ports` Should Not Be a Separate Crate

The proposal suggests:
```
cortex-domain/
cortex-application/
cortex-ports/         ← trait definitions
cortex-runtime/       ← glue
```

But `cortex-ports` would contain only trait definitions. Both `cortex-domain` and `cortex-application` need these traits. If ports is separate, you get a diamond dependency:

```
cortex-domain    → cortex-ports
cortex-application → cortex-ports + cortex-domain
```

This works, but adds a crate for ~200 lines of trait signatures. More importantly, port traits are **part of the application layer's interface specification**. They belong at the layer that orchestrates — the application layer.

**Recommendation**: Merge ports into `cortex-application`. Domain stays pure with zero dependencies on port traits. Application defines ports and orchestrates domain.

### Correction 2: `cortex-runtime` as Glue Is Unnecessary

The proposal has a separate `cortex-runtime` as "thin integration layer" between domain and application. But application already depends on domain. What does glue add?

If `cortex-runtime` is just `pub use cortex_domain; pub use cortex_application;`, it's ceremony.

**Recommendation**: Keep `cortex-runtime` as the **application-layer crate name**. Don't have both `cortex-application` and `cortex-runtime`.

### Correction 3: Naming Should Follow Existing Convention

Existing crates use `nostra-*` prefix. New crates should use `cortex-*` prefix (since Cortex is the product). This is already implied but worth being explicit:

```
cortex-domain     (not nostra-cortex-domain)
cortex-runtime    (not nostra-cortex-runtime)
cortex-ic-adapter (already planned)
```

---

## 5. Optimal Crate Structure (Corrected)

```
nostra/libraries/
    cortex-domain/          ← Pure, sync, deterministic
        Cargo.toml              deps: serde, chrono, thiserror, uuid
        src/
            lib.rs
            crdt.rs
            policy.rs
            scoring.rs
            events.rs
            errors.rs
            workflow.rs

    cortex-runtime/         ← Async orchestration + port traits
        Cargo.toml              deps: cortex-domain, tokio (restricted), serde
        src/
            lib.rs              ← CortexRuntime trait + builder
            ports/
                storage.rs
                network.rs
                time.rs
                log.rs
                governance.rs
            event_bus.rs
            governance.rs
            workflow.rs
            viewspec.rs
            ux.rs
            resilience.rs

    cortex-ic-adapter/      ← ICP adapter (already planned)
        Cargo.toml              deps: cortex-runtime (for port traits), ic-agent, candid
        src/
            lib.rs
```

Host crates:
```
nostra/apps/
    cortex-desktop/         ← Existing app, becomes thin host
        Cargo.toml              deps: cortex-runtime, cortex-ic-adapter, dioxus, axum
```

### Dependency Graph

```
cortex-desktop ──→ cortex-runtime ──→ cortex-domain
       │                  │
       ├──→ cortex-ic-adapter ──→ cortex-runtime
       │
       ├──→ dioxus, axum (host-only)
```

All arrows point inward. Domain depends on nothing project-specific. Runtime depends only on domain. Adapters depend on runtime (for port traits). Host depends on everything.

---

## 6. What `cortex-domain` Cargo.toml Looks Like

```toml
[package]
name = "cortex-domain"
version = "0.1.0"
edition = "2021"
description = "Pure domain logic for the Cortex sovereign runtime"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", default-features = false, features = ["serde"] }
thiserror = "2.0"
uuid = { version = "1", features = ["v4", "serde"] }

# NO tokio
# NO async-trait
# NO log/tracing
# NO candid
# NO ic-agent
# NO std::fs, std::env, std::process, std::net
```

### Purity Check for Domain

```bash
#!/bin/bash
DOMAIN_TOML="cortex/libraries/cortex-domain/Cargo.toml"
DOMAIN_SRC="cortex/libraries/cortex-domain/src"

# Check forbidden crate dependencies
FORBIDDEN_CRATES="tokio|async-trait|log |tracing|candid|ic-agent|axum|dioxus|portable-pty|rfd|home|walkdir"
if grep -E "$FORBIDDEN_CRATES" "$DOMAIN_TOML"; then
    echo "FAIL: cortex-domain has forbidden dependencies"
    exit 1
fi

# Check forbidden API usage
FORBIDDEN_APIS="std::fs|std::env|std::process|std::net|tokio::|async fn|\.await"
if grep -rE "$FORBIDDEN_APIS" "$DOMAIN_SRC/"; then
    echo "FAIL: cortex-domain uses forbidden APIs"
    exit 1
fi

# WASM compilation check
cargo check -p cortex-domain --target wasm32-unknown-unknown
```

Note the addition: `async fn` and `.await` are forbidden in domain. This is enforceable because domain is sync-only. The compiler enforces it via the absence of tokio in dependencies.

---

## 7. What `cortex-runtime` Cargo.toml Looks Like

```toml
[package]
name = "cortex-runtime"
version = "0.1.0"
edition = "2021"
description = "Sovereign execution runtime for Cortex"

[dependencies]
cortex-domain = { path = "../cortex-domain" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", default-features = false, features = ["serde"] }
thiserror = "2.0"
uuid = { version = "1", features = ["v4", "serde"] }
tokio = { version = "1", default-features = false, features = ["rt", "sync", "time", "macros"] }
async-trait = "0.1"

# NO candid, ic-agent, axum, dioxus, etc.
```

---

## 8. Impact on ADR-012

**ADR-012 must be superseded.** The decision changes from "internal module structure" to "separate crates per onion layer."

New ADR-012 replaces the module-only approach with the 3-crate structure (`cortex-domain`, `cortex-runtime`, `cortex-ic-adapter`).

---

## 9. Risk Analysis

| Risk | Severity | Mitigation |
|---|---|---|
| Workspace grows larger | Low | Already 15 members — 3 more is marginal |
| Cross-crate refactoring friction | Low | Types shared via `cortex-domain` |
| Version coordination | Low | Path dependencies in workspace |
| Compilation time increase | Low | Incremental compilation; domain is small |
| Over-fragmentation pressure | Medium | Hold at 3 crates. Resist per-feature crates. |

---

## 10. The Deciding Evidence

Three pieces of evidence made this decision:

1. **`nostra-core` already leaked `candid`**. Cultural discipline failed. Compiler enforcement would have prevented it.

2. **Workspace already has 15 members**. The overhead argument is moot — the cost is already paid.

3. **Greenfield advantage is time-limited**. After Phase 1 extraction, domain code exists. Splitting it into a separate crate later means migration. Doing it now means clean creation.

The argument is not theoretical. The evidence is in the existing `Cargo.toml` files.
