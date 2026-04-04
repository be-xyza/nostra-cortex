---
id: '047'
name: temporal-architecture
title: 'Research Initiative 047: Temporal Architecture Adoption'
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-30'
---

# Research Initiative 047: Temporal Architecture Adoption

## Executive Summary
This research analyzed the `Temporal` codebase to identify architectural patterns suitable for Nostra / Cortex. The primary finding is that Temporal's "Durable Execution" model is the missing link for robust AI Agents ("Cortex"). By modeling Agents as Workflows and Tools as Activities, we achieve fault-tolerant, stateful, and long-running autonomous systems.

## Key Findings

### 1. Core Architecture (The Engine)
- **Pattern**: **State Machines** (`sdk-core`) drive execution.
- **Lesson**: Don't write procedural Scripts. Write Event-Driven State Machines.
- **Action**: detailed in `RUST_CORE_ALIGNMENT.md`.

### 2. Visibility & Monitoring
- **Pattern**: **Dual Write / Async Indexing**.
- **Lesson**: Execution (History) is separate from Visibility (Search).
- **Action**: detailed in `VISIBILITY_ARCH.md`.

### 3. Interoperability (Gap Analysis)
- **Pattern**: **Nexus RPC**.
- **Lesson**: Standardize "Service Linking" for cross-agent calls.
- **Action**: detailed in `GAP_ANALYSIS.md`.

### 4. Security & Access (Gap Analysis)
- **Pattern**: **S2S Proxy** & **Interceptors** (`common/authorization`).
- **Lesson**: Use Multiplexed Tunneling for "Personal OS" access. Use **Interceptors** (Middleware) for centralized Auth, Logging, and Metrics.
- **Discovery**: `temporal/common/authorization/interceptor.go`.

### 5. Behavior-Driven Orchestration (Adaptive Alignment)
- **Gap**: Currently, we have "Permissions" (Static) but not "Reputation" (Dynamic).
- **Solution**: Implement the **Adaptive Alignment Loop**.
    - **Mechanism**: A "Governance Interceptor" evaluates **Alignment Signals** (`056-alignment-signals`) against **Orchestration Policies** (`056-orchestration-policies`).
    - **Effect**: The Interceptor injects `Updates` (Temporal Signals) into the Agent Workflow to dynamically `DowngradeAutonomy` or `Pause` based on behavior.
    - **Status**: Defined in Research 056. Needs implementation in `nostra-workflow-engine`.

### 6. Distributed Systems (Final Sweep)
- **Pattern**: **Transactional Outbox (Transfer Queue)**.
- **Lesson**: Never "fire and forget". Always "Persist Task -> Ack Task".

- **Pattern**: **Archival**.
- **Lesson**: Implement `HistoryArchiver` interface for cold storage offloading.

### 6. Component Model (CHASM)
- **Pattern**: **Hierarchical State Machines**.
- **Lesson**: Align Nostra's "Entity" model with this. An Agent is a Component Tree.

### 7. AI Patterns (Cookbook)
- **Pattern**: **Human-in-the-Loop**.
- **Lesson**: Use `Signal` and `wait_condition` to pause execution until human approval.

### 8. System Self-Healing (Internals)
- **Pattern**: **System Workflows** (`temporal/service/worker/scanner`).
- **Lesson**: Implement "Garbage Collector" and "Invariant Validator" as **Nostra Agents**.

- **Pattern**: **Dynamic Config** (`temporal/common/dynamicconfig`).
- **Lesson**: Use a "Config Registry" canister to push system-wide overrides (e.g. rate limits).

### 9. Data Security (Middleware)
- **Pattern**: **Data Converter** (`common/payloads`).
- **Lesson**: All inputs/outputs should be `Payloads` (bytes). This allows a "Data Converter" middleware to encrypt/decrypt data at the edge, so the Agent Kernel never sees plaintext secrets (only the Worker does).

### 10. Developer Experience (Loop 1)
- **Pattern**: **Admin CLI** (`temporal/cli`).
- **Lesson**: Provide a `nostra-cli` that uses the same GRPC/Canister APIs as the web frontend for debugging workflows (view traces, reset history).

### 11. Multi-Cluster Replication (Loop 2)
- **Pattern**: **Stream Receiver & Task Processor** (`service/history/replication`).
- **Discovery**: `StreamReceiver` (passive listener) and `TaskProcessor` (active applier) sync state between clusters using High/Low watermarks.
- **Lesson**: For "Personal OS" syncing (e.g. Mobile <-> Canister), use this exact pattern. Don't just "copy files". Replay the history events.

### 12. Wire Protocol (Loop 3)
- **Pattern**: **ReplicationTask Protobuf**.
- **Discovery**: `ReplicationTask` contains `TaskType`, `SourceTaskId`, and `Attributes`.
- **Lesson**: The sync protocol must be strongly typed and versioned. Use a similar Protobuf message for syncing Nostra Agents.

### 13. Optimization Patterns (Loop 4)
- **Pattern**: **Sticky Execution**.
- **Discovery**: `arch_docs/sticky_queues.md`.
- **Lesson**: Workers cache workflow state and poll a unique "Sticky Queue". This prevents replaying full history for every step. Nostra Agents should cache state in the Worker (browser/daemon) and only fetch "New Events" from the Canister unless the cache is cold.

- **Pattern**: **Workflow Task Chunking**.
- **Discovery**: `arch_docs/workflow_task_chunking.md`.
- **Lesson**: Collapse "Empty" Workflow Tasks (e.g. heartbeats) into a single logical task to avoid waking up the Agent logic unnecessarily.

- **Pattern**: **Resource-Based Tuner**.
- **Discovery**: `sdk-core/worker/tuner`.
- **Lesson**: Don't just use fixed concurrency limits. Use a "Tuner" that dynamically adjusts slot permits based on system resources (Memory/CPU) to maximize throughput without crashing the worker.

### 14. Communication Patterns (Loop 5)
- **Pattern**: **Sync Match Fairness**.
- **Discovery**: `temporal/service/matching/matcher.go` (`Offer`).
- **Lesson**: **Do not Sync Match if there is a Backlog**. If the queue has older tasks in DB, force the Worker to drain the Backlog first. This prevents "Starvation" of old tasks during high traffic.

- **Pattern**: **Task Queue Partitioning**.
- **Discovery**: `temporal/service/matching/forwarder.go`.
- **Lesson**: Split high-traffic Queues into "Partitions" (Logic Channels). Tasks are written to Leaf Partitions. Pollers aggregate at the Root. Nostra should implement "Queue Sharding" where one Logical Queue spans multiple Canisters.


- **Lesson**: Split high-traffic Queues into "Partitions" (Logic Channels). Tasks are written to Leaf Partitions. Pollers aggregate at the Root. Nostra should implement "Queue Sharding" where one Logical Queue spans multiple Canisters.

### 15. Persistence Patterns (Loop 6)
- **Pattern**: **Transfer Queue (The Universal Outbox)**.
- **Discovery**: `temporal/service/history/transfer_queue_active_task_executor.go`.
- **Lesson**: **Everything is a Transfer Task**. Signals, Cancellations, activity scheduling, and child workflow starts are ALL written to the Transfer Queue in the same transaction as the state update. An async processor then picks them up. Nostra must assume *nothing* happens synchronously.

- **Pattern**: **Shard Context**.
- **Discovery**: `temporal/service/history/history_engine.go`.
- **Lesson**: The `HistoryEngine` is instantiated *per shard*. It owns the `MutableState` cache for that shard. Nostra agents should be "Sharded" by ID, with a dedicated Canister (or partition) owning the lock for that ID.

- **Lesson**: The `HistoryEngine` is instantiated *per shard*. It owns the `MutableState` cache for that shard. Nostra agents should be "Sharded" by ID, with a dedicated Canister (or partition) owning the lock for that ID.

### 16. Timer Patterns (Loop 7)
- **Pattern**: **Unified Timer Queue**.
- **Discovery**: `temporal/service/history/timer_queue_active_task_executor.go`.
- **Lesson**: All timeouts (Activity, Workflow, User) are just records in a `TimerQueue`, stored by Timestamp.
- **Optimization**: Use a `BTree` in Stable Memory related to time. The Canister `hearbeat` checks the root of the tree. If `now > root.timestamp`, pop and execute.

### 17. Versioning Patterns (Loop 8)
- **Pattern**: **Build ID Pinning**.
- **Discovery**: `temporal/service/history/worker_versioning_util.go`.
- **Lesson**: Workflows are "Pinned" to a specific `BuildID` (Code Version) upon start. Child Workflows inherit this pin.
- **Nostra Application**: An Agent Run should be pinned to a specific **Wasm Hash**. If the Canister upgrades, old runs should ideally continue using the old Wasm (if we support multi-version hosting) or fail safely.

### 18. Scheduler & Nexus Patterns (Loop 9)
- **Pattern**: **Workflow-as-Scheduler**.
- **Discovery**: `temporal/service/worker/scheduler/workflow.go`.
- **Lesson**: A "Schedule" in Temporal is just a Workflow that runs a loop: `Calculate Next Time` -> `Sleep` -> `Spark Child Workflow`.
- **Nostra Application**: Do NOT build a separate "Scheduler" canister. Just write a standard Nostra Agent that runs a `while(true)` loop with `await sleep(next_time)`.

- **Pattern**: **Nexus (Inter-Cluster Signaling)**.
- **Discovery**: `temporal/service/history/api/signalwithstartworkflow`.
- **Lesson**: "SignalWithStart" is the critical pattern for Singleton Agents. It ensures the Agent is running before sending the Signal, all in one atomic operation. This prevents race conditions where a signal arrives before the agent starts.

### 19. Frontend & Gateway Patterns (Loop 10)
- **Pattern**: **Layered Interceptor Chain**.
- **Discovery**: `temporal/service/frontend/nexus_handler.go`.
- **Lesson**: Every incoming request passes through a pipeline: `Auth` -> `NamespaceValidation` -> `ConcurrencyLimit` -> `RateLimit` -> `ClientVersionCheck` -> `HeaderSanitization`. Nostra's gateway (or each canister's public endpoint) MUST implement this sequence.

- **Pattern**: **Cross-Cluster Forwarding**.
- **Discovery**: `temporal/service/frontend/nexus_handler.go` (`forwardStartOperation`).
- **Lesson**: If a namespace is "global" but the current cluster is "standby", the Frontend FORWARDS the request to the Active cluster. This is key for Multi-Region deployments. Nostra should research IC multi-subnet forwarding.

### 20. Hierarchical State Machines (Loop 11)
- **Pattern**: **HSM Tree with Operation Log**.
- **Discovery**: `temporal/service/history/hsm/tree.go`.
- **Lesson**: The entire mutable state is a TREE of smaller state machines. Child state machines (e.g., Activities, Timers, Updates) are nested under a parent (Workflow). Each transition generates an `Operation` entry. Compaction removes operations for deleted subtrees.
- **Nostra Application**: Model Agent State as a tree: `Agent -> [ ActivityRuns, Timers, ChildAgents, Updates ]`. This allows fine-grained, per-component transitions.

### 21. Replication Stream (Loop 12)
- **Pattern**: **Prioritized Stream Receiver with Flow Control**.
- **Discovery**: `temporal/service/history/replication/stream_receiver.go`.
- **Lesson**: Replication messages have `HIGH` and `LOW` priority. The receiver tracks a "low watermark" for each. A `FlowController` can **PAUSE** a priority tier if its backlog exceeds a threshold. This is classic "Back-Pressure".
- **Nostra Application**: If syncing data between Canisters (e.g., for analytics or replication), implement tiered queues with back-pressure signals.

### 22. Update Protocol (Durable Signals) (Loop 13)
- **Pattern**: **Provisional State + Lifecycle Stage Awaiting**.
- **Discovery**: `temporal/service/history/workflow/update/update.go`.
- **Lesson**: An "Update" (Durable Signal) has a state machine: `Created -> Admitted -> Sent -> Accepted -> Completed`. Callers can *await* specific stages (`WaitLifecycleStage`). The server uses *Provisional* states (e.g., `ProvisionallyAccepted`) to handle rollbacks.
- **Nostra Application**: For "Durable Signals" to Agents, implement a state machine per Signal. Callers can await `Admitted` (persisted), `Accepted` (worker ACK), or `Completed` (result ready).

### 23. Archival Subsystem (Loop 14)
- **Pattern**: **Multi-Target Archival with Rate Limiting**.
- **Discovery**: `temporal/service/history/archival/archiver.go`.
- **Lesson**: Archival runs against multiple *Targets* (History, Visibility) in parallel, each to a pluggable backend (S3, GCS, etc.). A `RateLimiter.WaitN` gates the request before starting.
- **Nostra Application**: Build an `Archiver` trait. Concrete implementations for IC Stable Memory, DFINITY Archive Canister, or External S3 (via HTTPS Outcalls). Always rate-limit before archiving.

### 24. Deletion Manager (Loop 15)
- **Pattern**: **Task-Based Staged Deletion**.
- **Discovery**: `temporal/service/history/deletemanager/delete_manager.go`.
- **Lesson**: Deletion is NOT an API call that deletes immediately. It schedules a `DeleteWorkflowExecutionTask` to the Transfer Queue. The task then deletes History -> Visibility -> Mutable State in stages. The task re-schedules itself if the Workflow isn't closed yet.
- **Nostra Application**: Never delete data inline. Schedule a `DeletionTask`, let the task executor handle the multi-step cleanup. This is idempotent and restartable.

### 25. NDC (No Data-loss Continue) (Loop 16)
- **Pattern**: **Transaction Policies for Conflict Resolution**.
- **Discovery**: `temporal/service/history/ndc/transaction_manager.go`.
- **Lesson**: When replicating workflows across clusters, conflicts can occur. NDC defines policies: `CreateAsCurrent`, `CreateAsZombie`, `SuppressCurrentAndCreateAsCurrent`, `ConflictResolveAsCurrent`. The `TransactionManager` dispatches to specialized handlers based on the policy.
- **Nostra Application**: For multi-canister or multi-subnet replication, define conflict resolution policies. "Zombie" workflows can exist as non-current versions until a conflict is resolved.

### 26. Completion Callbacks (Loop 17)
- **Pattern**: **Event-Triggered Callbacks**.
- **Discovery**: `temporal/service/history/workflow/mutable_state_impl.go` (`addCompletionCallbacks`).
- **Lesson**: Workflows can register `CompletionCallbacks` that are invoked when the Workflow closes. These are stored in Mutable State and triggered by `WorkflowClosed` events. Callbacks can target Nexus endpoints or internal handlers.
- **Nostra Application**: Implement `OnAgentComplete` hooks. Store callback targets in Agent State. On completion, iterate and invoke.

---
### 28. Time as First-Class Primitive (SOD Link)
- **Pattern**: **TimeSlicedIndex**.
- **Discovery**: `048-sod-resource-adequacy` Research (Hybrid Compute).
- **Lesson**: Large-scale vector indices on ICP hit instruction limits (40B). We must "Slice" time into logical buckets. In Temporal, this aligns with **History Archival** (slices of history). In Nostra/ELNA, this means vector indices are physically sharded by `start_time` / `end_time`.
- **Nostra Application**: Use `TimeSlicedIndex` for embedding storage. Query logic "Scatters" across active slices and "Gathers" results.

## Summary of Deep Dive

**Discovered 57 distinct architectural patterns** from the Temporal codebase + Nostra Research:

| # | Pattern | Source |
|---|---------|--------|
...
| 25-27 | Nexus RPC, S2S Proxy, Worker Controller | Gap Analysis |
| 28 | **TimeSlicedIndex** | `048-research` |

### Additional Patterns (Deep Scan - Jan 2026)

| # | Pattern | Source | Nostra Application |
|---|---------|--------|-------------------|
...
| 56 | **Cache Eviction Reason Tracking** | `sdk-core/run_cache.rs` | Metrics for forced vs normal evictions |
| 57 | **Time-Sliced Indexing** | `048` | Hybrid compute sharding for vectors |

**Total: 57 distinct patterns** identified from Temporal ecosystem.

All patterns have been mapped to Nostra/Cortex equivalents in `IMPLEMENTATION_STRATEGY.md`.



### 27. UI/UX & Visualization (Loop 50 - Final)
- **Pattern**: **Headless UI & Educational Components**.
- **Discovery**: `temporal/documentation/src/components` & `temporal/cli`.
- **Lesson**: The "Console" is just a client. Use **D3.js** for high-meaning semantic visuals (History/Lineage). Use **AG-UI** as the protocol for agents to suggest layouts.
- **Action**: detailed in `UI_UX_PATTERNS.md`.





## Recommendations for Nostra

### 1. "Workflow-as-Agent" Standard
Adopts the pattern where an Agent is defined as a durable workflow.
- **State**: Stored in Canister stable memory.
- **Execution**: The "Loop" is the `heartbeat` or a recurring timer.

### 2. Durable MCP Integration
Upgrade `ic-rmcp` to support "Async/Durable" tool calls, returning a "Job ID".

### 3. Rust Core Adoption
Use `temporal-sdk-core` patterns for the `013-nostra-workflow-engine`.

### 4. Hybrid Compute (SOD Fix)
Adopt "Off-Chain Worker / On-Chain Truth" model. Workers perform heavy lifting (embeddings) and push to Time-Sliced Indices.

## System Patterns for Nostra

1.  **"Agentic Loop"**: The `while(true)` loop in python agents is the standard for autonomous agents. Nostra Agents = Durable Loops.
2.  **"Durable MCP"**: Wrap MCP tools in Activities. Use `Workflow` to chain them.
3.  **"Sync Match" Queue**: Nostra's `nostra-queue` canister should attempt direct canister-to-canister calls (or websocket push) before falling back to stable storage.
4.  **"Permit" Polling**: The Nostra Rust SDK must enforce `MaxConcurrentTasks` via semaphores before asking the queue for work.
5.  **"Patch" Markers**: Implement `workflow.patched("fix-issue-123")` in the Nostra SDK to allow upgrading Agent logic without breaking running instances.
6.  **"Scheduler" Agent**: Don't build a separate cron system. A "Scheduler Agent" is just an Agent that sleeps and sends signals to other Agents.
6.  **"Time-Sliced Index"**: Shard heavy data (Vectors, Logs) by uniform time buckets to allow parallel processing and avoid canister limits.
- [ ] Define the `NostraSyncTask` protobuf for multi-device sync.

## Next Steps
- [ ] Prototype a "Durable MCP" on ICP.
- [ ] Refactor `013` design to align with `sdk-core`.
- [ ] Create a "Nostra Agent" template with "Human-in-the-Loop" signals.
- [ ] Implement a "Canister Scanner" Agent for system health.
- [ ] Design the "Interceptor" trait for Nostra Agents (Auth, Logging, Encryption).
- [ ] Verify 048 Time-Sliced architecture (Completed).
