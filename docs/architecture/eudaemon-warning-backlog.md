# `cortex-eudaemon` Warning Backlog

This document records the March 11, 2026 warning inventory used for the handoff stabilization slice.

## Baseline Freeze

Baseline commands:

- `cargo check --manifest-path cortex/apps/cortex-eudaemon/Cargo.toml --all-targets`
- `cargo test --manifest-path cortex/apps/cortex-eudaemon/Cargo.toml system_agents_runs_lists_recent_runs_in_desc_order`
- `cargo test --manifest-path cortex/apps/cortex-eudaemon/Cargo.toml acp_native_entry_reports_unreachable_when_worker_absent`

Pre-cleanup state:

- `cortex-eudaemon` emitted 145 warnings in the default handoff target set.
- Most warnings were dead-code warnings caused by the daemon binary compiling a private copy of the module tree from `src/main.rs`.
- Remaining warning-heavy modules also included dormant service bundles that were still exported into the default build.

## Module Inventory

| Module path | Warning class | Chosen disposition | Reason / retention |
| --- | --- | --- | --- |
| `cortex/apps/cortex-eudaemon/src/main.rs` | duplicate default-build dead code via private `mod gateway; mod services;` | fixed | daemon binary now uses the library crate boundary directly |
| `cortex/apps/cortex-eudaemon/src/services/backend_service.rs` | remove-on-sight dead service module | removed | no active call sites in the canonical runtime path |
| `cortex/apps/cortex-eudaemon/src/services/workflow_service.rs` | duplicate flow/control surface types and dead service module | removed | active flow/catalog logic already lives in `ops_flows.rs` and `gateway/server.rs` |
| `cortex/apps/cortex-eudaemon/src/services/agent_service.rs` | legacy debug/state scaffolding mixed with active search/index code | fixed + scaffold-gated | active indexing/search remains default-build; debug bridge and state-channel code moved behind `service-scaffolds` |
| `cortex/apps/cortex-eudaemon/src/gateway/runtime_host.rs` | legacy offline mutation helpers in canonical runtime host | fixed + scaffold-gated | queue/export/apply/probe remain default-build; debug-only local-gateway mutation helpers are now `service-scaffolds` only |
| `cortex/apps/cortex-eudaemon/src/services/mod.rs` | dormant service modules exported into default build | fixed | removed dead exports and gated dormant scaffolds behind `service-scaffolds` |
| `cortex/apps/cortex-eudaemon/src/services/capabilities_scanner.rs` | dormant scaffold module | scaffold feature | no canonical references from gateway/workbench/runtime startup |
| `cortex/apps/cortex-eudaemon/src/services/console_service.rs` | dormant console bridge | scaffold feature | only used by legacy chat bridge code now gated behind `service-scaffolds` |
| `cortex/apps/cortex-eudaemon/src/services/github_mcp_service.rs` | dormant MCP setup/service scaffold | scaffold feature | not part of canonical runtime/gateway path |
| `cortex/apps/cortex-eudaemon/src/services/lint_service.rs` | dormant service module | scaffold feature | no canonical references in the handoff path |
| `cortex/apps/cortex-eudaemon/src/services/local_connection.rs` | dormant adapter scaffold | scaffold feature | no canonical references in the handoff path |
| `cortex/apps/cortex-eudaemon/src/services/mcp/client.rs` | dormant MCP client abstraction | scaffold feature | MCP stack is retained only as a non-default scaffold |
| `cortex/apps/cortex-eudaemon/src/services/mcp/policy.rs` | dormant MCP policy scaffold | scaffold feature | retained with the gated MCP stack |
| `cortex/apps/cortex-eudaemon/src/services/mcp/toolset_adapter.rs` | dormant MCP toolset scaffold | scaffold feature | retained with the gated MCP stack |
| `cortex/apps/cortex-eudaemon/src/services/mcp/transport.rs` | dormant MCP transport scaffold | scaffold feature | retained with the gated MCP stack |
| `cortex/apps/cortex-eudaemon/src/services/motoko_graph_service.rs` | dormant service module | scaffold feature | no canonical references in the handoff path |
| `cortex/apps/cortex-eudaemon/src/services/cortex_ux.rs` | stale reexports/imports | fixed | unused compatibility imports removed from the active surface |
| `cortex/apps/cortex-eudaemon/src/gateway/server.rs` | stale helper duplicated from active ops flow loader | fixed | removed unused test-only ACP status fetch helper |
| `cortex/apps/cortex-eudaemon/src/services/authz.rs` | unused helper functions | fixed | helper functions removed from default build because they had no live callers |
| `libraries/cortex-domain/src/integrity/integrity_events.rs` | upstream dependency unused import | retained | outside this slice; does not belong to the `cortex-eudaemon` module boundary, but still appears in the handoff cargo check output |

## Post-Cleanup Residual

Post-cleanup `cargo check --manifest-path cortex/apps/cortex-eudaemon/Cargo.toml --all-targets` state:

- `cortex-eudaemon` local warnings: `0`
- external warning still emitted by dependency build: `libraries/cortex-domain/src/integrity/integrity_events.rs`

The strict warning profile budget remains `20` for `cortex_eudaemon` to avoid blocking on upstream workspace warnings outside this slice. Any new `cortex-eudaemon` warning introduced after this slice should be treated as a regression.
