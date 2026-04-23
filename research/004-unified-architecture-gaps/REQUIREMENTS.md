---
id: '004'
name: unified-architecture-gaps
title: 'Requirements: Unified Architecture Components'
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Unified Architecture Components

## Component Mapping

| User Request | Missing Component | Proposed Home (Cortex) | Tech Stack |
|--------------|-------------------|---------------|------------|
| "User Management" | **User Registry** | `kg-registry` (expand scope) | Motoko (StableBTreeMap) |
| "Governance Info" | **Governance Host** | New `governance` canister | Rust (IC-GOV framework?) |
| "Cooperative Workflows" | **Workflow Engine** | `cortex-worker` (Execution Layer) | Rust (Durable) |
| "Indexes" | **Discovery Canister** | New `discovery` canister | Motoko / Rust |
| "Routers" | **Event Bus** | `event-hub` canister (PubSub) | Motoko |
| "Agents" | **MCP Server** | `cortex-agents` (Router) | Rust (`ic-rmcp`) |
| "Frontend UIs" | **Unified Component Lib** | Shared Dioxus Crate | Rust (Dioxus) |
