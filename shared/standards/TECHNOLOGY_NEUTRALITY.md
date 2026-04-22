# Technology Neutrality & Adapter Doctrine

**Type**: Constitutional Standard
**Status**: DRAFT
**Context**: Defines the separation between Constitutional Invariants (Meaning) and Implementational Adapters (Execution).

---

## 1. The Core Doctrine

> **"Meaning is Sovereign. Execution is Interchangeable."**

Nostra is defined by its semantic data models, governance rules, and identity invariants. It is *hosted* by technologies like the Internet Computer, Temporal, or LLMs, but it is *not defined* by them.

To preserve long-term evolvability and forkability, all external technologies must be treated as **Adapters**, not **Foundations**.

## 2. The Constitutional Boundary

We explicitly categorize all systems into two planes:

| Plane | Definition | Properties | Examples |
| :--- | :--- | :--- | :--- |
| **Constitutional (Invariant)** | The "Soul" of the network. Defines what things *are* and how they *relate*. | Durable, Platform-Agnostic, Semantic. | User Identity (Abstract), Governance Rules, Contribution Graph (Data). |
| **Implementational (Adapter)** | The "Body" of the network. Defines how things *run* or *look*. | Replaceable, Optimization-focused, Ephemeral. | ICP Canisters, Temporal Workflows, D3 Visualizations, Vector DBs. |

### 2.1 The Adapter Rule
**Rule**: No Constitutional Invariant may depend directly on an Implementational technology. It must bind through an **Adapter Interface**.

*   **Bad**: `User.id` is an ICP Principal. (Locks identity to ICP).
*   **Good**: `User.id` is a DID/UUID. `User.credentials` contains an ICP Principal Adapter.

---

## 3. Technology Classifications

### 3.1 Execution Substrate (Current: Internet Computer)
*   **Role**: Provides compute, consensus, and storage.
*   **Constitutional Status**: **ADAPTER**.
*   **Invariant**: The Ledger of Truth (History) serves the graph.
*   **Requirement**: The system must be capable of migrating execution to another substrate (e.g., bare metal, another chain) without changing the semantic history of the Contribution Graph.

### 3.2 Orchestration Engine (Current: Temporal)
*   **Role**: Manages long-running processes and retries.
*   **Constitutional Status**: **ADAPTER**.
*   **Invariant**: Workflows are deterministic state transitions defined by the Governance system.
*   **Requirement**: "Workflow Contracts" define input/output and rollback logic. The engine is merely a runner.

### 3.3 Intelligence (Current: LLMs/Agents)
*   **Role**: Proposes changes, summarizes data, automates tasks.
*   **Constitutional Status**: **ADAPTER**.
*   **Invariant**: Agents are **Non-Authoritative Actors**.
*   **Requirement**: AI never defines truth. It proposes mutations that a Human or Governance Ruleset must commit.

### 3.4 Visualization (Current: D3, CVOS)
*   **Role**: Renders the graph for human interaction.
*   **Constitutional Status**: **ADAPTER**.
*   **Invariant**: The **Contribution Graph** is the canonical form.
*   **Requirement**: Visual graphs are "Lenses" (Projections). They must not store state that is not reflected in the canonical graph.

---

## 4. Implementation Guidelines

### 4.1 Identity Layering
*   **Level 1: Actor** (Constitutional). Abstract ID (UUID/DID). Use this for all Graph relations.
*   **Level 2: Credential** (Adapter). Methods of proving control (ICP Principal, Metamask Sig, WebAuthn).
*   **Level 3: Role** (Context). Permissions granted to an Actor in a specific Space.

### 4.2 Capability-First Interfaces
Define what the system *needs*, not what opensource tool provides it.

*   `IQueryCapability` (NOT "SQL Interface")
*   `IMutationCapability` (NOT "Canister Update")
*   `IWorkflowCapability` (NOT "Temporal Activity")

### 4.3 The Forkability Stress Test
For any proposed dependency, ask:
*   "If this vendor/project disappears tomorrow, does the Contribution Graph retain its meaning?"
*   If **No** → The dependency has leaked into the Constitution. Refactor immediately.
