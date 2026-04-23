---
id: "131"
name: "openresponses-llm-adapter"
title: "OpenResponses LLM Adapter + Agent Harness Activation"
type: "plan"
project: "cortex"
status: active
portfolio_role: supporting
authority_mode: recommendation_only
execution_plane: "cortex"
authors:
  - "Codex"
tags:
  - "cortex"
  - "agents"
  - "llm"
  - "open-responses"
  - "responses-api"
stewardship:
  layer: "Execution"
  primary_steward: "Systems Steward"
  domain: "Agent Harness"
created: "2026-03-04"
updated: "2026-03-04"
---

# Initiative 131: OpenResponses LLM Adapter + Agent Harness Activation

## Objective
Activate Cortex agent runs by replacing the deterministic planner stub in `cortex-eudaemon` with an LLM-driven planner using a local `open-responses-server` adapter sidecar (Responses API + SSE), while keeping tool execution and governance inside Cortex.

## Scope (v1)
1. Agent runs only (`/api/spaces/:space_id/agent/contribution` path): planner uses adapter.
2. Preserve A2UI-over-WebSocket contract to `cortex-web` (repeat `surface_update` events).
3. Tool loop remains Cortex-owned; sidecar MCP hosting is disabled.

## Non-Goals (v1)
1. Migrating other LLM callers (e.g., `cortex-worker`) to the adapter.
2. Enabling sidecar-hosted tools (MCP/web search/file search) that bypass Cortex governance.

## Acceptance Criteria
1. With adapter running, agent contribution runs show a streaming planner transcript (Markdown) before GSMS evaluation.
2. If adapter is unavailable and fail mode is `fallback`, the run completes using deterministic fallback with a surfaced note.
3. Governance ladder (`L0-L2`) and replay/lifecycle artifacts remain intact (Initiative 126).
