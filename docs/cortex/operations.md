# Cortex Operations Manual: Safe Upgrades & Configuration

> **Version**: 1.0.0
> **Status**: APPROVED
> **Target Audience**: Node Operators, Power Users

## 1. The Safe Upgrade Lifecycle

Cortex enforces a strict "No Data Loss" policy during upgrades. The system performs automatic verifiable snapshots before applying any changes.

### Step 1: Pre-Flight Check
Before initiating an upgrade, the **Local Environment Orchestrator** checks:
1.  **Disk Space**: Ensure enough space for a full snapshot (2x current Stable Memory).
2.  **Port Availability**: Ensure local ports (8000, 4943) are free.
3.  **Network Health**: Verify connection to the replica (Local or Mainnet).

### Step 2: Atomic Snapshot
The system pauses the canister and executes a **Native ICP Snapshot** using the current `icp-cli` workflow:
```bash
icp canister snapshot create nostra_backend
```
*   **Verification**: The snapshot hash is computed and stored in `snapshot.json`.
*   **Failure Mode**: If snapshot fails, the upgrade is **ABORTED** immediately.

### Step 3: Wasm Install
The new `wasm` binary is installed via `upgrade` mode (preserving stable memory).
*   **Argument Migration**: If the init arguments have changed, the `VersionManager` handles the transformation.

### Step 4: Health Probe
Post-upgrade, the system runs a "Smoke Test":
1.  **Canister Status**: Must report `Running`.
2.  **Health Endpoint**: `GET /api/v1/health` must return `200 OK`.
3.  **Data Integrity**: A query is run to verify a known entity (e.g. the Root Space) exists.

### Step 5: Rollback (If Needed)
If the Health Probe fails:
1.  **Alert**: User is notified of failure.
2.  **Restore**: The system automatically calls:
    ```bash
    icp canister snapshot load {snapshot_id}
    ```
3.  **Report**: A failure report is generated for analysis.

---

## 2. Resource Configuration: Hybrid vs Sovereign

To mitigate local system bloat, Cortex supports two operating modes.

### Mode A: Hybrid (Default)
Optimized for low-resource devices (MacBook Air, Entry-Level). Data ownership is retained, but heavy compute is federated.

| Component | Location | Responsibility |
| :--- | :--- | :--- |
| **Control Plane** | **Local** | UI, Keys, Configuration, Governance. |
| **Logic Layer** | **ICP Mainnet** | Smart Contracts, Data Storage (Canisters). |
| **Vector Index** | **Remote** | Nostra Cloud / Community Node (e.g., ELNA). |
| **LLM Inference** | **Remote** | OpenAI / Anthropic / Groq (API-based). |
| **Resource Usage** | **< 500MB RAM** | Lightweight Electron/Tauri wrapper. |

### Mode B: Sovereign (Power User)
Optimized for privacy and independence. Requires substantial hardware (M-Series Pro/Max).

| Component | Location | Responsibility |
| :--- | :--- | :--- |
| **Control Plane** | **Local** | UI, Keys, Configuration, Governance. |
| **Logic Layer** | **Local Replica** | Full local subnet simulation (`icp-cli`). |
| **Vector Index** | **Local** | Local Qdrant / LanceDB instance. |
| **LLM Inference** | **Local** | Ollama / Llama.cpp (7B+ models). |
| **Resource Usage** | **8GB - 32GB RAM** | Heavy compute load. |

### Configuration Toggle
Users can switch modes in `Settings > Infrastructure`.
*   **Switching from Hybrid -> Sovereign**: Triggers a "Full Sync" (downloading Canister State and Vector Indices). This is a heavy background operation.
*   **Switching from Sovereign -> Hybrid**: Archives local state and switches pointers to remote APIs.

---

## 3. Knowledge Engine Operations

For local-first knowledge engine orchestration (ingest, hybrid retrieval, grounded ask, shadow parity, benchmark gates), use:

- `docs/cortex/knowledge-engine-local-runbook.md`
- `scripts/knowledge-closeout-benchmark.sh`
- `docs/cortex/knowledge-engine-closeout-report.md`

---

## 4. Cortex Desktop Reliability Operations

Use these controls for local reliability hardening of `cortex-desktop`.

### Launcher single-instance lock
- Launcher script: `cortex/apps/cortex-desktop/run_cortex.command`
- Lock file: `~/.cortex-desktop/cortex-desktop.pid`
- Behavior:
1. If lock PID is alive, launcher prints `already running` and exits `0`.
2. If lock PID is stale, launcher removes stale lock and proceeds.
3. Lock cleanup is registered via shell trap on normal/error exit.

### Gateway readiness probe
- Readiness endpoint: `GET /api/system/ready`
- Response shape:
```json
{
  "ready": true,
  "gateway_port": 3000,
  "icp_network_healthy": true,
  "dfx_port_healthy": true,
  "notes": []
}
```
- `icp_network_healthy` is the canonical readiness field.
- `dfx_port_healthy` remains as a legacy compatibility alias during migration.
- Launcher performs a post-launch readiness probe loop (up to 15s) against:
  - `http://127.0.0.1:<gateway_port>/api/system/ready`
- Timeout behavior: warning only; launcher does not force-kill the app.

### Gateway port source
- Default gateway base URL: `http://127.0.0.1:3000`
- Optional override: `CORTEX_GATEWAY_PORT=<numeric-port>`
- Invalid override falls back to `3000`.

### Deterministic closeout command
Run:
```bash
bash /Users/xaoj/ICP/scripts/cortex-desktop-closeout-check.sh
```

Command outputs:
- Run artifact: `logs/testing/runs/<run_id>.json`
- Updated compatibility payload in:
  - `logs/testing/test_gate_summary_latest.json` under `.compatibility.cortex_desktop_closeout`

---

## 5. Authz Rollout Operations (Nostra↔Cortex Program Scope)

Use these controls for hybrid authz rollout across Group A/B/C program endpoints.

### Engine mode controls
- Global default engine:
  - `NOSTRA_AUTHZ_ENGINE_MODE=legacy|shadow|enforce` (default `legacy`)
- Per-group override:
  - `NOSTRA_AUTHZ_GROUP_A_MODE=inherit|legacy|shadow|enforce`
  - `NOSTRA_AUTHZ_GROUP_B_MODE=inherit|legacy|shadow|enforce`
  - `NOSTRA_AUTHZ_GROUP_C_MODE=inherit|legacy|shadow|enforce`
- Notes:
1. `inherit` uses global mode.
2. Promotion is expected to happen group-by-group (A -> B -> C).

### Identity trust controls
- Dev-only header fallback:
  - `NOSTRA_AUTHZ_DEV_MODE=true`
  - `NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER=true`
- Program claims activation:
  - `NOSTRA_AUTHZ_REQUIRE_PROGRAM_CLAIMS=true`
  - Enables additive claim checks for high-risk program resources (heap mutate, artifact publish, capability graph structural updates, DPub mutating run/export).
- In non-dev mode, mutating endpoints require verified principal-bound identity and return `403 ACTOR_IDENTITY_UNVERIFIED` when unresolved.

### Metrics and mismatch telemetry
- Metrics endpoint: `GET /api/metrics/authz`
- Counter families:
  - `authz_decision_total`
  - `authz_shadow_mismatch_total`
  - `authz_identity_unverified_total`
  - `authz_policy_compile_fail_total`
- Shadow artifacts:
  - Latest summary: `logs/alignment/authz_shadow_mismatch_latest.json`
  - Append-only events: `logs/alignment/authz_shadow_mismatch_events.jsonl`

### Staging rollout profile (minimal)
Use this as a compact starting point for staged promotion while keeping global behavior stable:

```bash
export NOSTRA_AUTHZ_ENGINE_MODE=legacy
export NOSTRA_AUTHZ_GROUP_A_MODE=shadow
export NOSTRA_AUTHZ_GROUP_B_MODE=legacy
export NOSTRA_AUTHZ_GROUP_C_MODE=legacy
export NOSTRA_AUTHZ_REQUIRE_PROGRAM_CLAIMS=false
```

### Quick validation sequence (no extra tooling)
1. Verify authz metrics surface:
```bash
curl -sS http://127.0.0.1:3000/api/metrics/authz | jq .
```
2. Verify Group A/B/C mode labels in decision counters:
  - check `authz_decision_total[].mode`
3. Verify shadow mismatch stream is active when a group is in `shadow`:
```bash
tail -n 20 /Users/xaoj/ICP/logs/alignment/authz_shadow_mismatch_events.jsonl
```
4. Verify claim gate activation when needed:
```bash
export NOSTRA_AUTHZ_REQUIRE_PROGRAM_CLAIMS=true
```
Then call a high-risk mutator with role-only identity and confirm deny with `requiredClaims` present in authz details.

### Compact metrics payload example
```json
{
  "schemaVersion": "1.0.0",
  "generatedAt": "2026-03-04T12:00:00Z",
  "authz_decision_total": [],
  "authz_shadow_mismatch_total": [],
  "authz_identity_unverified_total": [],
  "authz_policy_compile_fail_total": 0
}
```
