# Initiative 131 Decisions

## DEC-131-001: Cortex-Owned Tool Loop
**Date**: 2026-03-04
**Status**: Approved

**Decision**: Tool execution (and any MCP integration) remains in `cortex-eudaemon`. The adapter sidecar is treated as a pure model/API compatibility layer.

**Rationale**: Preserves governance parity (authority ladder, approval gates) and prevents tool execution from bypassing Cortex policies.

## DEC-131-002: Responses SSE as the Streaming Contract
**Date**: 2026-03-04
**Status**: Approved

**Decision**: `cortex-eudaemon` consumes `response.*` SSE events from `/v1/responses` and projects progressive `surface_update` A2UI payloads over `/ws`.
