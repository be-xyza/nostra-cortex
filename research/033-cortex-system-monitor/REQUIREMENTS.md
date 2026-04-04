---
id: '033'
name: cortex-system-monitor
title: 'Requirements: Cortex System Monitor'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Cortex System Monitor

**Status**: DRAFT  
**Date**: 2026-01-17  
**Initiative ID**: 033  

## 1. Functional Requirements

### 1.1 Data Collection (Backend)
*   **System Metrics**:
    *   **Cycles**: Must report current balance and daily burn rate.
    *   **Memory**: Must report Heap and Stable memory usage.
    *   **Uptime**: Time since last upgrade/deploy.
*   **Operational Metrics**:
    *   **Errors**: Count of `ERROR` and `CRITICAL` logs in the last N hours.
    *   **Workflows**: Number of Active, Queued, and Failed workflow instances.
    *   **Users**: Number of active users (past 24h).
    *   **Frontend Health**: Client-side crash rate (reported via Log Drain).
*   **Triage Items (New)**:
    *   System must generate persistent `TriageItem` records for critical events.
    *   Support lifecycle: `Open` -> `Ack` -> `Resolved`.
*   **Fleet Management (New)**:
    *   **Registry**: Store a list of monitored `CanisterIds`.
    *   **Remote Polling**: Backend must be able to query health/cycles of registered canisters (Worker/Indexers).
    *   **Cycles Actions**: Ability to send cycles to any registered canister.
*   **Economic Integrity (New)**:
    *   **TVL**: Total cycles/tokens locked in active workflows.
    *   **Trust Score**: Aggregated reputation score of active agents/users.
    *   **Anomaly Detection**: Alert on rapid cycle depletion or mass-entity creation (Spam).
*   **API**:
    *   Expose `getSystemStatus() : async SystemStatus` (Public or Restricted to Monitor Role).
    *   Expose `getTriageItems(status) : async [TriageItem]`.
    *   `SystemStatus` Record must include all above metrics.

### 1.2 User Interface (Frontend - Unified App)
*   **Navigation**:
    *   Top Level Tabs: `Dashboard` (Monitor) | `Inbox` (Admin) | `Settings`.
*   **View A: Dashboard (Monitor)**:
    *   **Traffic Lights**: Green/Yellow/Red status.
    *   **Gauges**: Cycles, Memory, API Latency.
    *   **Live Feed**: Scrolling log ticker.
    *   **Fleet View**: List/Grid of all connected canisters (Backend, Worker, etc.) with individual status.
*   **View C: The Cortex Atlas (System Map)**:
    *   **Technology**: D3.js Force-Directed Graph.
    *   **Capabilities**:
        *   **Auto-Layout**: Force simulation that clusters related components.
        *   **Interactive**: Click node to inspect details (e.g., click Canister -> Inspector Panel opens).
        *   **Filtering**: Toggles to show/hide "Libraries", "Agents", or "Workflows".
    *   **Visual Feedback**:
        *   Real-time animations for active calls/signals.
        *   Status colors (Red/Green) overlaid on nodes.
*   **View D: Inbox (Admin)**:
    *   **Layout**: "Linear-style" density: Left sidebar, Middle (List), Right (Context).
    *   **Triage List**: Active `TriageItems` sorted by priority/time.
    *   **Action Deck**: Context-sensitive specific actions (e.g., "Top Up", "Retry", "Approve").
    *   **keyboard Shortcuts**: J/K nav, E to resolve.
*   **Notifications**:
    *   In-app toast notifications for Critical health status.

### 1.3 Off-Chain Integration (Optional Phase)
*   **Prometheus Exporter**:
    *   Expose a text-based HTTP endpoint `/metrics` compatible with Prometheus scraping.

### 1.4 Local Development & Diagnostics (Crucial)
*   **Configuration Validation**:
    *   System must validate critical configuration parameters (e.g., Backend Canister IDs) on startup.
    *   Mismatches between frontend config and actual backend identity must trigger clear, actionable error messages (preventing opaque WASM crashes).
*   **Local Network Visibility**:
    *   **Canister Discovery**: Ability to see all locally running canisters (e.g., via `dfx` state or local registry) within the Monitor UI.
    *   **Connection Status**: Visual indicator of connection health to local canisters.

## 2. Non-Functional Requirements

### 2.1 Performance
*   **Low Overhead**: Monitoring logic must not consume more than 5% of canister instructions.
*   **Latency**: Dashboard load time < 2s.

### 2.2 Security
*   **Access Control**: Only users with `#admin` or `#analyst` role can view sensitive system details (e.g., exact cycle balance, user IPs).
*   **Data Privacy**: Logs must not leak PII unless authorized.

### 2.3 Reliability
*   **Self-Monitoring**: The monitor must be able to report if *it* is failing (e.g., via fail-safe HTTP outcall or dead man's switch).

### 2.4 Modularity (Packaging)
*   **Feature Flags**: All major components (Fleet, Economy, Inbox) must be gated by feature flags.
*   **Graceful Degradation**: If a feature is disabled, the UI must adjust layout without errors (no empty holes).
*   **Tiered API**: API endpoints must validate if the caller's Space/User has the package enabled.

## 3. Constraints
*   **Storage**: On-chain history limited to ~24h to save costs. Long-term history must be offloaded.
*   **IC Environment**: No background cron jobs < 1s. Pulse frequency limited by heartbeat settings.
