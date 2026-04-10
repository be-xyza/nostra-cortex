# Cortex Git Adapter (Phase 1)

Phase 1 goal: **read-only** GitHub → Nostra ingestion.

## What it does
- Receives GitHub webhooks at `POST /webhooks/github` and verifies `X-Hub-Signature-256`.
- Enforces a repo allowlist from `config/git_adapter_registry.toml`.
- Projects events into Nostra via `executeKip` UPSERT commands.
- Runs a periodic reconciliation loop so state converges if webhooks are missed.
- Exposes operational endpoints: `GET /health`, `GET /ready`, `GET /metrics`.

## Governed predicates
This adapter only emits relationships using predicates in `libraries/nostra-resource-ref` (e.g. `references`, `produces`).
Phase 1 uses `references` only.

## Running (local)
From repo root:
```bash
cargo run -p cortex-git-adapter --manifest-path cortex/Cargo.toml
```

Required env:
```bash
export CORTEX_GIT_ADAPTER_WEBHOOK_SECRET="..."
```

### Nostra sink selection
By default the adapter uses `ic-agent` and requires a backend canister id.
For local development you can also run the optional `icp` sink mode.

`ic-agent` mode (default):
```bash
export NOSTRA_IC_HOST="http://127.0.0.1:4943"
export CANISTER_ID_NOSTRA_KIP="aaaaa-aa" # replace (preferred)
# optional fallback:
export CANISTER_ID_NOSTRA_BACKEND="aaaaa-aa"
```

`icp` mode (opt-in):
```bash
export CORTEX_GIT_ADAPTER_USE_ICP=true
export CORTEX_GIT_ADAPTER_ICP_PROJECT_ROOT="/path/to/icp-project" # must contain an icp.yaml with a canister that exposes `executeKip`
export CORTEX_GIT_ADAPTER_ICP_CANISTER="nostra_backend" # optional override
```
Note: this repo currently does not ship an `icp.yaml` that defines a KIP/`executeKip` canister, so `ic-agent` mode is the expected default.

Optional env:
```bash
export CORTEX_GIT_ADAPTER_PORT=8787
export CORTEX_GIT_ADAPTER_BIND=127.0.0.1
export CORTEX_GIT_ADAPTER_REGISTRY_PATH="cortex/apps/cortex-git-adapter/config/git_adapter_registry.toml"
export NOSTRA_WORKSPACE_ROOT="$PWD"

# Hardening:
export CORTEX_GIT_ADAPTER_MAX_REQUEST_BODY_BYTES=1048576
export CORTEX_GIT_ADAPTER_REQUEST_TIMEOUT_SECS=10
export CORTEX_GIT_ADAPTER_DELIVERY_RETENTION_DAYS=30
export CORTEX_GIT_ADAPTER_RECONCILE_PER_PAGE=50
export CORTEX_GIT_ADAPTER_RECONCILE_MAX_PAGES=10

# Phase 2 (requires KIP canister upgrade):
export CORTEX_GIT_ADAPTER_KIP_EMIT_ATTRIBUTES=true
export CORTEX_GIT_ADAPTER_STORE_AUTHOR_EMAIL=false
export CORTEX_GIT_ADAPTER_KIP_METHOD="execute_kip_mutation" # or "executeKip"

# GitHub API for reconciliation:
export GITHUB_TOKEN="ghp_..."
# or GitHub App:
export GITHUB_APP_ID="123"
export GITHUB_APP_INSTALLATION_ID="456"
export GITHUB_APP_PRIVATE_KEY_PATH="/path/to/private-key.pem"
```

## Registry
See `config/git_adapter_registry.toml`.
