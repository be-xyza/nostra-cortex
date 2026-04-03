---
id: '033'
name: cortex-system-monitor
title: 'Research: Cortex System Monitor'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-29'
---

# Research: Cortex System Monitor

> **Layer**: Cortex (Execution) — Observability & Runtime Services

**Status**: DRAFT  
**Date**: 2026-01-17  
**Initiative ID**: 033  

## Layered Architecture Role

> [!IMPORTANT]
> **Consolidation Decision**: This initiative owns the **Consumption Layer** of Nostra's unified Observability stack.

| Layer | Owner | Responsibility |
|-------|-------|----------------|
| **Data Model** | 019-Log-Registry | `LogEntry`, `Span`, `Metric` schemas |
| **Pipeline** | 054-OpenTelemetry | Receiver→Processor→Exporter traits |
| **Consumption** | **033-Cortex-Monitor** (this) | Dashboard, Triage, Alerts |

**This initiative is the ONLY consumer of observability data.** It should NOT define its own log schemas—use 019's schemas. It should NOT implement pipelines—use 054's traits.

---

## Strategic Timing Recommendation
*User Question: Implement now or after Workflow Builder?*

**Direct Answer**: Split the implementation.
1.  **Phase 2 (Monitor/Dashboard)**: Implement **NOW**. It is a "Production Foundation" requirement. You cannot safely deploy the Workflow Builder or run Agents without ability to see Cycles/Memory/Error rates.
2.  **Phase 3 (Admin/Inbox)**: Implement **AFTER** Workflow Builder. The "Inbox" manages the *output* of workflows (signals, failures, approvals). Building the inbox before the engine that fills it is premature optimization.

## 1. Overview
The "Cortex System Monitor" is proposed as an observation apparatus to manage the Nostra/Cortex ecosystem at scale. As the system grows with more agents, workflows, and knowledge entities, a centralized dashboard is needed to ensure system health, performance, and stability.

**Crucially, it also serves as a diagnostic tool for local development**, validating configurations (e.g., Canister ID mismatches) and visualizing local network topology to prevent opaque failures.

## 2. Core Questions

### How would this work?
The System Monitor would function as a reliable "Watchdog" layer sitting above the functional components (Knowledge Graph, Workflow Engine, Agents).

*   **Data Collection**: 
    *   **Pull**: FE/External services poll canisters for status (Heartbeat, Memory, Cycles).
    *   **Push**: Canisters emit structured logs (via `019-log-registry`) for critical events.
*   **Aggregation**: A dedicated `Monitor` module or canister aggregates these signals into "Health Scores".
*   **Visualization**: A "Mission Control" dashboard (extending the "System" tab) displaying real-time gauges, traffic lights (Green/Yellow/Red), and active alerts.

### What would it cover?
1.  **Infrastructure Health**:
    *   Cycles balance & burn rate (prediction of depletion).
    *   Heap memory usage / Stable memory usage.
    *   Canister status (Running/Stopping/Stopped).
2.  **Application Logic Health**:
    *   Log Registry Error Rate (spikes in ERROR logs).
    *   Workflow Engine Status (stalled workflows, queue depth).
    *   Agent Activity (busy/idle states).
3.  **Security Monitoring**:
    *   Unauthorized access attempts (from 031 RBAC logs).
    *   Anomalous mutation volumes.

## 3. Feasibility Analysis

### On-Chain vs. Off-Chain

| Feature | On-Chain (Self-contained) | Off-Chain (External Service) |
| :--- | :--- | :--- |
| **Data Storage** | **Limited**: Only current state or small circular buffers. High cost for time-series. | **Unlimited**: Can store years of history in SQL/Time-series DB. |
| **Real-time** | **High Latency**: Depends on query calls (~200ms) or update calls (~2s). | **Variable**: Depends on polling frequency. |
| **Alerting** | **Passive**: Can only "alert" when user checks, unless using HTTP Outcalls (expensive). | **Active**: Can send Emails/SMS/Webhooks immediately upon detection. |
| **Trust** | **Trustless**: Verified by consensus. | **Trusted**: Relies on the integrity of the monitor service. |

### Recommendation: Hybrid Approach
*   **Primary (On-Chain)**: The "Apparatus" itself lives on-chain as a Module/Canister that exposes a `getSystemStatus()` query. It stores *only* the current snapshot and short-term history (e.g., last 24h).
*   **Secondary (Client/Off-Chain)**: The Frontend (Client) acts as the bridge. When open, it polls `getSystemStatus()` and visualizes it. For historical analysis or alerting, we can eventually integrate an "Observer Agent" (off-chain or TEE) that polls this endpoint.

## 4. Vision: "Cortex Admin" (Unified Command)
*In response to user request comparing to Linear.app*

The vision is a **Unified Command Center** that merges two distinct but complementary paths:
1.  **System Monitor (The Watchtower)**: Real-time, passive observability. High-level dashboard for "Is the system healthy?".
2.  **Cortex Admin (The Workshop)**: Active, stateful triage. Low-level inbox for "What do I need to fix?".

### Comparison of Paths

| Feature | System Monitor (Dashboard) | Cortex Admin (Triage) |
| :--- | :--- | :--- |
| **Goal** | Awareness & Health | Resolution & Governance |
| **Core Entity** | **Metric** (Gauge/Chart) | **Signal** (TriageItem/Ticket) |
| **Interaction** | Passive (View only) | Active (Resolve, Ack, Snooze) |
| **Linear Analogy** | "Insights" Tab | "Inbox" / "Issues" Tab |
| **Input Source** | System Polling | Agents, Error Logs, User Requests |

### Missing Features (Gap Analysis)
To match the utility of Linear for an AI OS, we need:
1.  **"The Inbox"**: A unified queue for everything requiring attention (Failed workflows, Low cycles, User access requests, Content flag).
2.  **Actionability**: Not just viewing a log, but *resolving* it (e.g., "Restart Workflow", "Grant Access", "Top up Canister").
3.  **Context Aware Details**: Like the right-side panel in Linear. If the item is a "Failed Workflow", show the Trace. If "Access Request", show the User Profile.

## 5. Architecture

### Backend: `MonitorModule`
*   **Metrics Engine**: Integrates with `SystemAPI` and `LogRegistry` for dashboards.
*   **Triage Engine**: Stores `TriageItems` (The stateful representation of alerts).
*   **Exporters**: Exposes `http_request` for Prometheus (Monitor path).

### Frontend: `Mission Control` UI
The UI will be a "Super-App" with two distinct modes:

*   **Mode A: Monitor (Dashboard)**
    *   **Heads-up Display**: Traffic lights, Cycles, Health %.
    *   **Visualizations**: D3.js Charts for memory/operations.
    
*   **Mode B: Admin (Triage)**
    *   **Inbox Reference**: "Linear-style" 3-pane layout (Nav, List, Context).
    *   **Action Deck**: Quick actions (J/K shortcuts) to resolve items.

## 5. Resolution with Existing Initiatives

*   **019-nostra-log-registry**: The Monitor is the *consumer* of the logs produced here. It adds the "Analytics" layer on top of the raw logs.
*   **031-production-foundation**: Monitor visualizes the security events (RBAC denials) and relies on the stable storage structures for its own config.
*   **013-nostra-workflow-engine**: Monitor tracks the "Queue Depth" of the workflow engine.
*   **028-a2ui-integration**: The User Interface for the Monitor could potentially be generated/enhanced by A2UI, allowing dynamic "Inspection Panels" for different system parts.

## 6. Frontend Observability & Testing
*User Question: How to manage browser errors and testing?*

### 6.1 Frontend Error Capturing (The "Log Drain")
The System Monitor must not only track the Backend. The Frontend is equally critical.
*   **Strategy**: The Frontend will have a `Logger` service that buffers client-side errors (React Error Boundaries, Store panics, Network failures) and "drains" them to the Backend `LogRegistry` via `submitLog`.
*   **Benefits**:
    *   **Unified View**: FE and BE errors appear side-by-side in the Monitor.
    *   **Triage**: A frontend crash becomes specific `TriageItem` in the Admin Inbox.
*   **Implementation**:
    *   `src/services/Logger.ts`: Captures `console.error` and `window.onerror`.
    *   `Auto-Context`: Automatically attaches `UserAgent`, `Route`, and `Wallet` info.

### 6.2 Browser Testing Strategy
Given the complexity of the Admin/Monitor apps, manual testing is insufficient.
*   **E2E Testing (Playwright)**:
    *   **Why**: Can simulate full user flows (Login -> Toggle Library -> Check Graph).
    *   **Integration**: Run against a local `dfx` replica in CI.
*   **"The Test Agent"**:
    *   Instead of just distinct scripts, we can use a dedicated "User Simulation Agent" that uses the actual frontend (via A2UI or headless driver) to perform smoke tests daily.
    *   **Monitor Integration**: The results of these tests (`Pass/Fail`) are sent to the Monitor as a `SystemHealth` metric.

## 7. Implementation Stages
1.  **Metric Exposure**: Add `getMetrics()` to backend.
2.  **Basic Dashboard**: UI Traffic lights for Memory/Cycles.
3.  **Logic Monitoring**: Integrate Workflow/Log counts.
4.  **Advance UI**: D3.js visualization of history (client-side cache).

## 8. Canister Topology & Management
*User Requirement: Multi-canister view and cycles management.*

To manage a *constellation* of canisters (Backend, Worker, Indexers, Linked Projects), the Monitor needs a **Topology View**.

### 8.1 The "Fleet" Registry
*   The Monitor will maintain a registry of `KnownCanisterId`.
*   **Capabilities**:
    *   **Auto-Discovery**: Detects canisters it interacts with (e.g., Worker) or manually added.
    *   **Aggregate Status**: "System Health" is the aggregate health of all known canisters.
*   **Visualization**:
    *   **Topology Graph**: A D3 node-link diagram showing the Backend, Workers, and External Services, with edges representing recent calls (derived from trace logs).

### 8.2 Cycles Manager
*   **Monitoring**: Tracking cycles for *all* fleet canisters, not just the backend.
*   **Management**:
    *   **"Top Up" Action** (Admin/Inbox): Trigger a cycles transfer to a starving canister.
    *   **Auto-Refuel Strategy** (Future): Agent that monitors and tops up automatically.

## 9. Cross-Initiative Resolution
*Analysis of overlaps and requirements from other research tracks.*

| Initiative | Cortex Monitor Requirement | Status |
| :--- | :--- | :--- |
| **013 Workflow Engine** | Monitor must track **Queue Depth** and **Stalled AsyncOps**. Agents failing to callback must trigger an alert. | **Covered** (in Phase 1 Metrics) |
| **017 AI Agent Roles** | Dashboard should group status by **Role** (e.g., "Gardener Status", "Analyst Load"). "Gardener Pruning" events should be visible. | **New Requirement**: `AgentStatus` registry. |
| **028 A2UI Integration** | The Dashboard *could* be an A2UI surface, but for high-density Triage, a custom UI is safer. **Decision**: Use Custom UI for Inbox, allow A2UI "Widgets" on Dashboard. | **Resolved** |
| **031 Prod Foundation** | **Security**: Must enforce RBAC (#admin only). **Storage**: Must use `StableBTreeMap` or RingBuffer, not Arrays. | **Critical Constraint** |
| **021 KIP Integration** | **Graph Health**: Monitor must report "Orphan Nodes" or "Schema Violations" detected by the Gardener. | **Added to Dashboard** |
| **030 Artifacts** | Editor health (e.g., Save failures) must be reported to Log Registry (Frontend Observability). | **Covered** by Frontend Log Drain |

## 10. Economic & Integrity Monitoring
*User Requirement: Adaptation of "Blockchain Monitoring" concepts (Risk, Analysis, Compliance) for Cortex.*

While Cortex is an OS, not just a DeFi protocol, it has an internal economy (Bounties, Libraries, Governance). We adapt standard blockchain monitoring features as follows:

| Blockchain Monitoring Feature | Cortex Equivalent | Relevance |
| :--- | :--- | :--- |
| **Transaction Analysis** | **Contribution Flow Analysis** | Track flow of Bounties/Pledges. Detect "Wash Trading" of contributions to farm reputation. |
| **Risk Scoring** | **Contributor Trust Score** | Assign confidence scores to Agents/Users based on successful interaction history. Flag low-trust actors. |
| **Illicit Activity** | **Spam/Vandalism Detection** | Detect "Graph Vandalism" (creating junk nodes) or "Cycle Draining" attacks. |
| **Ecosystem Monitoring** | **Library Economics** | Monitor "Library GDP" (usage/installs) to ensure creators are rewarded and libraries are sustainable. |

### 10.1 Feature Additions
*   **Trust Dashboard**: View top contributors and flagged "High Risk" actors (e.g., failed validators).
*   **Economy Gauge**: Total Value Locked (TVL) in Bounties/Escrow.

## 11. Composable Packaging Strategy
*User Requirement: Features must be packages that can be turned off (e.g., for different Space tiers).*

To support "Lite" vs "Pro" versions of Cortex Spaces, the Monitor will be modular.

### 11.1 Feature Flags
We will use the existing `userProfile.featureFlags` (from `031`) to control visibility:
*   `monitor:core` -> Basic Dashboard (Health/logs).
*   `monitor:fleet` -> Topology & Cycles Manager.
*   `monitor:economy` -> Trust Scores & TVL.
*   `monitor:admin` -> Triage Inbox.

### 11.2 "Space Editions"
This allows us to ship different configurations:
*   **Personal Space (Free)**: `monitor:core` only.
*   **Team Space (Pro)**: `monitor:core` + `monitor:admin` + `monitor:fleet`.
*   **DAO Space (Enterprise)**: All features including `monitor:economy`.

### 11.3 Implementation
*   **Frontend**: Components (`FleetGrid`, `EconomyGauge`) check flags before rendering.
*   **Backend**: `getSystemStatus` returns `null` for fields not authorized by the caller's tier.

## 12. System Map Visualization (The Cortex Atlas)
*User Requirement: Enhance D3.js "System Maps" of connected nodes, canisters, agents, and libraries.*

The **Cortex Atlas** is a dynamic, visual map of the entire operating system's topology. It is distinct from the Knowledge Graph (which stores content); the Atlas stores *structure*.

### 12.1 The "Meta-Graph" Schema
The Atlas visualizes a "Meta-Graph" where nodes represent system components:

| Node Type | Shape/Color | Represents | Data Source |
| :--- | :--- | :--- | :--- |
| **Canister** | ⬢ (Hexagon) / Blue | Physical infrastructure (Backend, Worker). | Fleet Registry |
| **Library** | 📦 (Box) / Amber | Installed Logic Packages (Nostra Core, Auth). | Library Manager |
| **Agent** | 🤖 (Bot) / Purple | Active AI Agents (Librarian, Gardener). | Agent Registry |
| **Workflow** | ⚡ (Bolt) / Green | Active Processes. | Workflow Engine |

### 12.2 Relationship Types (Edges)
| Edge Type | Style | Represents |
| :--- | :--- | :--- |
| **Host** | Solid Line | "Library A is installed on Canister B" |
| **Dependency** | Dotted Arrow | "Library A depends on Library B" |
| **Call** | Animated Pulse | "Agent X called Workflow Y" (Runtime Trace) |
| **Signal** | Dashed Line | "Workflow Y emitted Signal Z" |

### 12.3 Visualization Features (D3.js)
*   **Force-Directed Layout**: Nodes naturally cluster by type (Canisters form the "backbone", Libraries cluster around their host).
*   **"Live Pulse"**: When an inter-canister call or log event occurs, the corresponding edge "pulses" visually (real-time feedback).
*   **Semantic Zoom**:
    *   *Zoom Level 1*: Only Canisters (Physical Topology).
    *   *Zoom Level 2*: + Libraries & Agents (Logical Topology).
    *   *Zoom Level 3*: + Active Workflows (Runtime Topology).

### 12.4 Diagnostics Overlay
*   **Traffic Light Nodes**: If a canister is low on cycles, its node glows Red.
*   **Broken Links**: If a Library dependency is missing, the edge appears broken/red.
