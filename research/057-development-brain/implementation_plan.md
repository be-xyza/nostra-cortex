# Implementation Plan - Config Service (057)

The goal is to implement a robust `ConfigService` in `nostra-worker` that manages environment-specific configurations (Local, Testnet, Mainnet) and service resolution strategies (e.g., fallbacks for Vector Search).

## Proposed Changes

### [nostra-worker](file:///Users/xaoj/ICP/nostra/worker)

#### [NEW] [src/config_service.rs](file:///Users/xaoj/ICP/nostra/worker/src/config_service.rs)
- Define `ConfigMatrix`, `ServiceConfig`, `ResilienceConfig` structs matching `nostra-config-v1.schema.json`.
- Implement `ConfigService` struct with:
    - `load()`: Reads `nostra_config.json` or falls back to defaults.
    - `get_vector_endpoints()`: Returns primary and fallback URLs.
    - `get_env()`: Returns current environment enum.

#### [MODIFY] [src/main.rs](file:///Users/xaoj/ICP/nostra/worker/src/main.rs)
- Initialize `ConfigService` at startup.
- Log the loaded environment and active services.

## Verification Plan

### Automated Tests
- **Unit Test**: In `config_service.rs`, write a test `test_load_default_config` that verifies the default values when no file is present.
    - Run: `cargo test -p cortex_worker config_service`
- **Integration Test**: Create a temporary `nostra_config.json` with "Testnet" settings, run the worker, and verify via logs or a test function that it picked up the new values.

### Manual Verification
- Run `cargo run` and observe the startup logs indicating "Loaded Config: Local (Default)" or similar.
