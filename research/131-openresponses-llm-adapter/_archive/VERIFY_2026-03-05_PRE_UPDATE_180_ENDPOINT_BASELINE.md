# Initiative 131 Verification Log

## Preflight
- `bash scripts/check_agent_preflight_contract.sh`
- `bash scripts/check_dynamic_config_contract.sh`
- `bash scripts/check_antigravity_rule_policy.sh`

## Manual Smoke Test (Local)
Date: 2026-03-04

### Sidecar
1. Start adapter sidecar: `./run_open_responses_server`
2. Verify adapter health: `curl http://127.0.0.1:8080/health`
   - observed: `{"status":"ok","adapter":"running"}`
3. Verify adapter routes: `curl http://127.0.0.1:8080/openapi.json | jq '.paths | keys'`
   - observed: `["/","/health","/responses","/v1/chat/completions","/{path_name}"]`

### Daemon + Adapter Status
1. Start daemon (example): `CORTEX_GATEWAY_PORT=3555 CORTEX_AGENT_RUNTIME_MODE=gateway_primary CORTEX_LLM_ADAPTER_URL=http://127.0.0.1:8080 ./scripts/run_cortex_daemon.sh`
2. Verify daemon sees adapter: `curl http://127.0.0.1:3555/api/system/llm-adapter/status`
   - observed: `adapterHealth.status=ok`, `openapiPaths` includes `/responses`
   - note: `upstreamModelsError` expected for Ollama (`/v1/models` returns 404)

### Web
1. Start web: `./scripts/run_cortex_web.sh dev`
2. Trigger agent contribution and confirm streaming planner transcript appears.

## Automated Checks (Repo)
Date: 2026-03-04

- Preflight + config governance:
  - `bash scripts/check_agent_preflight_contract.sh` (PASS)
  - `bash scripts/check_dynamic_config_contract.sh` (PASS)
  - `bash scripts/check_antigravity_rule_policy.sh` (WARN: existing advisory finding)
- Gateway inventory + protocol contract alignment:
  - `python3 scripts/refresh_gateway_parity_fixtures.py` (refreshed inventory=175)
  - `bash scripts/check_gateway_parity_inventory_sync.sh` (PASS)
  - `bash scripts/check_gateway_protocol_contract_coverage.sh` (PASS)
  - `bash scripts/check_gateway_contract_descriptors_strict.sh` (PASS)
- Rust tests:
  - `cd cortex && CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/cortex-eudaemon-target cargo test -p cortex-eudaemon --tests` (PASS)
