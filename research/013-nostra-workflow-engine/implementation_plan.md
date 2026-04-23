# Implementation Plan - On-Chain Workflow Engine

We will implement the Nostra Workflow Engine directly within the `nostra_backend` canister using Motoko. This replaces the deprecated off-chain Rust prototype.

## User Review Required

> [!NOTE]
> **Persistence Strategy**: We will use Motoko's `OrderedMap` stored in `stable var` (consistent with `state.entities` in `main.mo`). This provides upgrade safety but shares the 4GB heap limit. Migration to `ic-stable-structures` (64GB) is a future infrastructure task.

## Proposed Changes

### `nostra/backend/modules`

#### [NEW] [WorkflowTypes.mo](file:///Users/xaoj/ICP/nostra/backend/modules/WorkflowTypes.mo)
- Define `WorkflowInstance`, `WorkflowStatus` (Running/Suspended/Completed), `ExecutionEvent`.
- Define `ControlFlow` types (`Switch`, `Parallel`).

#### [NEW] [WorkflowStore.mo](file:///Users/xaoj/ICP/nostra/backend/modules/WorkflowStore.mo)
- Implement `WorkflowStore` class to encapsulate `OrderedMap` logic.
- Methods: `put`, `get`, `updateStatus`.

#### [NEW] [WorkflowEngine.mo](file:///Users/xaoj/ICP/nostra/backend/modules/WorkflowEngine.mo)
- Core FSM (Finite State Machine).
- `tick(instanceId)`: advances state.
- `start(defId)`: creates instance.
- `signal(instanceId)`: resumes suspended instance.

#### [NEW] [A2UIGenerator.mo](file:///Users/xaoj/ICP/nostra/backend/modules/A2UIGenerator.mo)
- Helper to generate JSON schemas for `UserTasks`.

### `nostra/backend`

#### [MODIFY] [main.mo](file:///Users/xaoj/ICP/nostra/backend/main.mo)
- Import new modules.
- Add `workflowInstances` to `StableState`.
- Expose `WorkflowService` public actor methods:
    - `start_workflow`
    - `send_signal`
    - `get_workflow_state`

## Verification Plan

### Automated Tests
We will add a new test suite in `nostra/backend/modules/WorkflowTest.mo` (if test runner permits) or run integration tests via script.

1.  **Deploy Canister**: `dfx deploy nostra_backend`
2.  **Register Workflow**: Call `engine.register(testDef)` (via a new script or existing seed).
3.  **Start Workflow**: Call `start_workflow`. Assert `instanceId` returned.
4.  **Check State**: Call `get_workflow_state`. Assert `status: #active`.
5.  **Signal**: Call `send_signal`. Assert state transition.

### Manual Verification
1.  **CLI Interaction**:
    ```bash
    dfx canister call nostra_backend start_workflow '("test-flow", "{}")'
    dfx canister call nostra_backend get_pending_tasks '("principal-id")'
    ```
