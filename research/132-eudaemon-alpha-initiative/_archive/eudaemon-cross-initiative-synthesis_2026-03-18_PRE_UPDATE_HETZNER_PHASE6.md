# Eudaemon Alpha — Cross-Initiative Synthesis

> How the Eudaemon Alpha architecture integrates with the broader Nostra-Cortex strategic roadmap. Validated against 8 related active initiatives.

---

## The Big Picture: Convergence

Eudaemon Alpha is not an isolated agent running alongside the system. It is the first institutional pioneer to combine **eight parallel architectural initiatives** into a single functional unit.

The previous workspace analysis missed that many "future" or "hypothetical" primitives are already actively implemented or locked in recent architectural decisions.

Here is the definitive synthesis of how Eudaemon utilizes the broader Cortex-Nostra ecosystem:

---

## 1. The Workspace: Initiative 124 (AGUI Heap Mode)

**Previous Assumption**: Eudaemon needs a new "heap board" primitive or customized Artifacts.
**Correction**: The **Heap Board** already exists as a canonical Cortex feature.

Eudaemon's workspace is a standard open Space using the `HeapWorkspaceView`.
- **Outputs**: Eudaemon emits polymorphic blocks (charts, A2UI, text, pointers) directly to the heap via `POST /api/cortex/studio/heap/emit`.
- **Input Context**: Instead of manually parsing directories, Eudaemon calls `POST /api/cortex/studio/heap/blocks/context` to package selected blocks into an `AgentContextBundle`.
- **User Iteration**: Nostra users interact with Eudaemon's heap blocks (commenting, tagging) providing immediate improvement signals to the system. No custom interaction surfaces are needed.

## 2. Agent Governance & Lifecycle: Initiative 126 (Agent Harness)

**Previous Assumption**: Eudaemon follows its own "Constitution."
**Correction**: The Constitution provides behavioral guidelines, but **Initiative 126** provides the hard architectural contract for *all* agents.

- **Authority Level**: Eudaemon Alpha must be explicitly configured to run at **Authority L1 (Suggestion-only)** or **L2 (Limited write to space-scoped sandbox)**. Direct Nostra writes are blocked by the gateway.
- **Lifecycle Auditing**: Every Eudaemon cycle must emit an `AgentExecutionRecord` to the `GlobalEvent` stream. This payload includes `execution_id`, `input_snapshot_hash`, `output_snapshot_hash`, and `authority_scope`.
- **Evaluation Gate**: Before any Eudaemon proposal is promoted by a steward, it passes through the 126 Evaluation Loop interface.

## 3. The Runtime Model: Initiative 047 (Temporal Architecture)

**Previous Assumption**: ZeroClaw is a standalone script on a VPS.
**Correction**: **047** defines the "Workflow-as-Agent" standard. Agents are durable state machines.

- ZeroClaw (the runtime) must implement or map to Temporal's Execution Model: explicit separation of Workflow logic (deterministic) from Activities (tool calls/LLM inference).
- Sleep cycles (Phase 2 core loop) aren't `time.sleep()`; they are durable workflow timers (`await sleep(next_time)`) that can survive cluster upgrades.

## 4. The Chronicle: Initiative 080 (DPub Standard)

**Previous Assumption**: The chronicle is a flat markdown folder.
**Correction**: The chronicle is a **DPub** — a first-class Contribution type.

- The Eudaemon DPub is a Root Node in the ContributionGraph defining the Table of Contents via traversal paths.
- **Living Layer**: Daily operations update the `HEAD` (Draft) version of the DPub chapters.
- **Editions**: Monthly, the steward triggers a `Publish Edition` workflow, capturing an immutable Merkle-dag root hash of the chronicle state. This provides the stable historical citation required by the Constitution.

## 5. UI Synthesis & Collaboration: Initiatives 113 & 115

**Initiative 113 (CRDT Collaboration)**:
- All Heap blocks emitted by Eudaemon are persistent, deterministic CRDT structures. If a human modifies an agent's block, it converges gracefully without overwriting.

**Initiative 115 (ViewSpec Governed UI)**:
- If Eudaemon generates interactive UI components (e.g., a data visualization for the Heap), it must output a compliant `ViewSpecV1` contract.
- The UI is compiled deterministically (`ViewSpec -> RenderSurface`).

## 6. Space Permissions: Initiative 130 (Space Capability Graph)

**Previous Assumption**: Spaces inherently support all contribution types.
**Correction**: **130** requires space capabilities to be explicitly activated.

- Eudaemon's Home Space must have a `SpaceCapabilityGraph` defining the activation of Heap Mode, the DPub Chronicle, and Proposal contribution types.

## 7. The Sandbox Ingestion Boundary: Initiative 127 (Cortex-Native Repo Ingestion)

**Previous Assumption**: Eudaemon reads the live Nostra codebase repository directly using tools.
**Correction**: **127** establishes a strict Sandboxed Context FS for ingestion.

- Eudaemon must not run tools against the raw `~/ICP/nostra` directory. It must use the `cortex-memory-fs/sandboxes/` pattern established by 127, where code is fetched cleanly into an isolated execution ring.

## 8. The Runtime Strategy: Initiative 122 (Cortex Agent Runtime Kernel)

**Previous Assumption**: Eudaemon's Python ZeroClaw loop is the final runtime model.
**Correction**: **122** explicitly defines the Nostra end-state: a Minimal Viable Kernel (MVK) built in Rust running directly inside a Cortex Temporal Worker.

- **Resolution**: Eudaemon Alpha (`132`) is the *prototype* simulating the 122 MVK. By using ZeroClaw on a VPS, we can iterate rapidly on the *behavior* and *prompts* while the Rust constraints of 122 are being built in parallel. Once 122 is ready, Eudaemon Beta will migrate from the VPS Python script into the Rust Cortex runtime.

## 9. Model Constitutions: Initiative 062

**Previous Assumption**: Simply telling the model it is Eudaemon is enough.
**Correction**: **062** warns that base models (Claude, GPT-4o) have implicit, upstream safety/alignment constitutions that conflict with Nostra's Knowledge Integrity exploration values.

- Eudaemon must explicitly implement the **`AgentDisclosurePattern`** defined in 062. If it encounters a topic where its base model refuses to explore hypotheticals due to upstream safety training, it must explicitly annotate this as a `ModelBiasAnnotation` in the Heap, rather than presenting the refusal as a Nostra policy decision.

---

## Summary of Alignments

| Domain | Eudaemon Implementation | Governing Initiative |
|--------|------------------------|----------------------|
| **Workspace** | Heap Board & Polymorphic Blocks | 124 (AGUI Heap Mode) |
| **Authority** | L1 / L2 Escalation Ladder | 126 (Agent Harness) |
| **Auditing** | `AgentExecutionRecord` via GlobalEvent | 126 (Agent Harness) |
| **Runtime Model** | "Workflow-as-Agent" Durable Loop | 047 (Temporal Arch) |
| **Runtime Target** | ZeroClaw simulating MVK | 122 (Agent Runtime Kernel) |
| **Memory** | Git-Backed Local Ephemeral Traces | 121 (Cortex Memory FS) |
| **Ingestion** | Target `cortex-memory-fs/sandboxes/` | 127 (Cortex Repo Ingestion) |
| **Chronicle** | `Contribution<DPub>` with Editions | 080 (DPub Standard) |
| **Model Bias** | Implement `AgentDisclosurePattern` | 062 (Model Constitutions) |
| **State Sync** | Deterministic CRDT Convergence | 113 (Cortex CRDT) |
| **UI Gen** | `ViewSpecV1` Declarative Contract | 115 (ViewSpec UI) |
| **Space Rules** | `SpaceCapabilityGraph` Overrides | 130 (Space Capability) |

## Next Steps

1. Update the **132 Implementation Plan** to reflect the integration of 124 (Heap APIs) and 126 (AgentExecutionRecord).
2. Refine the **v0.2 Strategic Report** to cement these alignments as architectural requirements.
3. Validate that the chosen **ZeroClaw** runtime can interface flexibly enough to handle 126 lifecycle reporting and 124 API payloads.
