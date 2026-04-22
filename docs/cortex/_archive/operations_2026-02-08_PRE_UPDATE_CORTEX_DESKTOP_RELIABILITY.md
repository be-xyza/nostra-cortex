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
The system pauses the canister and executes a **Native ICP Snapshot**:
```bash
dfx canister snapshot create --network local nostra_backend
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
    dfx canister snapshot load {snapshot_id} --network local nostra_backend
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
| **Logic Layer** | **Local Replica** | Full local subnet simulation (`dfx`). |
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
