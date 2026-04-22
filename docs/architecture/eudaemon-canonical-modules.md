# `cortex-eudaemon` Canonical Module Boundary

This document is the authoritative module classification for the handoff stabilization slice.

## Classification Rules

- `canonical_runtime`: default-build runtime logic required for daemon startup or gateway dispatch
- `canonical_ops_surface`: default-build operational surfaces consumed by Workbench or gateway endpoints
- `compatibility_bridge`: still active in default build, but retained as a compatibility seam rather than canonical product authority
- `scaffold_feature`: retained only behind the `service-scaffolds` feature
- `remove_on_sight`: obsolete module that should not remain in the default build

## Canonical Modules

| Module | Class | Notes |
| --- | --- | --- |
| `gateway/server` | `canonical_runtime` | active gateway host and route implementation |
| `gateway/runtime_host` | `canonical_runtime` | active dispatch boundary; local-gateway queue/probe APIs remain default-build |
| `workbench_ux` | `canonical_ops_surface` | registered Workbench route renderer |
| `ops_gates` | `canonical_ops_surface` | gate summary loading and heap drill-in support |
| `ops_agents` | `canonical_ops_surface` | agent run list/detail support |
| `ops_artifacts` | `canonical_ops_surface` | artifact inventory support |
| `ops_flows` | `canonical_ops_surface` | workflow catalog and decision-plane support |
| `cortex_ux` | `canonical_ops_surface` | persisted layout/pattern/capability contract helpers |
| `cortex_ux_store` | `canonical_ops_surface` | layout and contract storage manager |
| `local_gateway` | `compatibility_bridge` | live queue/state backing for local-gateway compatibility endpoints |
| `local_gateway_bridge` | `compatibility_bridge` | adapter seam for local-gateway orchestration |
| `agent_service` | `canonical_ops_surface` | default-build search/index surface; legacy debug bridge code is scaffold-only |
| `workflow_service` | `remove_on_sight` | removed; duplicate legacy service layer |
| `backend_service` | `remove_on_sight` | removed; no canonical references |
| `mcp/client` | `scaffold_feature` | retained only with `service-scaffolds` |
| `mcp/policy` | `scaffold_feature` | retained only with `service-scaffolds` |
| `mcp/toolset_adapter` | `scaffold_feature` | retained only with `service-scaffolds` |
| `mcp/transport` | `scaffold_feature` | retained only with `service-scaffolds` |
| `github_mcp_service` | `scaffold_feature` | dormant MCP bootstrap/setup surface |
| `capabilities_scanner` | `scaffold_feature` | dormant scaffold, not part of canonical handoff path |
| `console_service` | `scaffold_feature` | legacy console bridge retained only with debug scaffolds |
| `lint_service` | `scaffold_feature` | dormant scaffold, not part of canonical handoff path |
| `local_connection` | `scaffold_feature` | dormant scaffold, not part of canonical handoff path |
| `motoko_graph_service` | `scaffold_feature` | dormant scaffold, not part of canonical handoff path |
