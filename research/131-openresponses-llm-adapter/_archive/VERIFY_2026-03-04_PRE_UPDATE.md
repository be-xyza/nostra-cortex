# Initiative 131 Verification Log

## Preflight
- `bash scripts/check_agent_preflight_contract.sh`
- `bash scripts/check_dynamic_config_contract.sh`
- `bash scripts/check_antigravity_rule_policy.sh`

## Manual Smoke Test (Local)
1. Start adapter sidecar: `./run_open_responses_server`
2. Verify adapter health: `curl http://127.0.0.1:<port>/v1/models`
3. Start daemon: `CORTEX_AGENT_RUNTIME_MODE=gateway_primary ./scripts/run_cortex_daemon.sh`
4. Start web: `./scripts/run_cortex_web.sh dev`
5. Trigger agent contribution and confirm streaming planner transcript appears.
