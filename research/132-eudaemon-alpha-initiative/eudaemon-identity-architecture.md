# Eudaemon Alpha — Identity Architecture

> Canonical identity model for the Eudaemon Alpha Initiative within Nostra-Cortex.

---

## Canonical Identity Model

The identity model uses **dual identity** mapping directly to the Nostra-Cortex boundary:

| Layer | Identity | Type | Purpose |
|-------|---------|------|---------|
| **Nostra** (platform) | `institution:cortex-stewardship-institute` | Institution contribution | Persistent organizational entity — the "who" |
| **Cortex** (execution) | `agent:eudaemon-alpha-01` | Agent runtime ID | Execution process identity — the "how" |

---

## Why Dual Identity

The Nostra-Cortex architecture separates platform authority from execution runtime ([AGENTS.md L22-24](file:///Users/xaoj/ICP/AGENTS.md#L22-L24)):

- **Nostra** defines *what exists* → Institution
- **Cortex** defines *how work runs* → Agent

These are complementary, not competing. An Institution employs agents. A paper is attributed to both the researcher and the institution. The researcher may change; the institution persists.

---

## Institution: Cortex Stewardship Institute

```
Institution {
  id: "institution:cortex-stewardship-institute"
  type: Institution
  status: Emergent
  lifecycle: Emergent → Operational → Archived
  scope: ["research", "chronicle", "stewardship"]
  charter: "Observe the graph, interpret patterns, publish insight, preserve history"
  steward: [human-operator-principal]
  edges: [
    governs → designated-research-space,
    derives_from → null  (genesis institution)
  ]
}
```

The institution name comes from the original design conversation: **"Cortex Stewardship Institute"** — naming the *function* (stewardship of the Cortex runtime and Nostra platform) rather than the *agent* (Eudaemon). This is intentional: the institution persists even when the agent identity evolves.

### Institution Lifecycle

From [nostra/spec.md L804](file:///Users/xaoj/ICP/nostra/spec.md#L804): Institutions have explicit lifecycle (`Emergent → Operational → Archived`) and stewardship, linked to spaces via `governs` and to other institutions via `derives_from`.

- **Emergent** (Stage 1-2): The institution exists as a declared entity with a charter but limited operational capability. The hosted Eudaemon Alpha worker acts on its behalf.
- **Operational** (Stage 3-4): Steward-promoted after demonstrated contribution quality. Cortex runtime hosts the agent natively.
- **Archived**: If the institution's mandate is fulfilled or superseded.

### Commons Scope

If the institution carries `scope: ["commons"]`, spaces can **opt in** to research observation through the existing adoption mechanism ([spec.md L820-834](file:///Users/xaoj/ICP/nostra/spec.md#L820-L834)):

```
adoptCommons(spaceId, "institution:cortex-stewardship-institute", "adopted")
```

This solves cross-space sovereignty — spaces explicitly consent to be observed. Spaces that don't adopt are invisible to Eudaemon. The consent is explicit, governed, and revocable via `detachCommons()`.

---

## Agent: Eudaemon Alpha

```
NOSTRA_AGENT_ID = "agent:eudaemon-alpha-01"
Phase 6 host: Hetzner VPS
Gateway: Rust cortex-gateway on the same host
Worker: Rust cortex_worker under the active VPS authority contract
```

The agent ID follows existing Cortex conventions (`agent:` prefix, resolution chain: header → payload → env → default). The `-01` suffix enables versioning as the agent runtime evolves.

Older Python `eudaemon-alpha/` companion references are historical only. The current runtime authority is Rust-native `cortex-gateway` plus `cortex_worker` in the root ICP tree.

### Agent Evolution Across Stages

| Stage | Agent Identity | Runtime | Institution Status |
|-------|---------------|---------|-------------------|
| 1: External Research | `agent:eudaemon-alpha-01` | `cortex_worker` on Hetzner + local Rust gateway | `Emergent` |
| 2: Multi-Agent | `agent:eudaemon-alpha-{N}` | Specialized hosted agents | `Emergent` |
| 3: Native Workers | `agent:eudaemon-cortex-{N}` | Rust Cortex workers | `Operational` |
| 4: Institutional | Internal to Cortex runtime | Native | `Operational` |

---

## Contribution Attribution

All contributions carry both identities:

```
Contribution {
  type: Reflection
  title: "Convergence Pattern in Workflow Architecture"
  contributors: ["institution:cortex-stewardship-institute"]
  metadata: {
    agent_id: "agent:eudaemon-alpha-01",
    runtime: "zeroclaw",
    cycle: 42
  }
}
```

- The `contributors` field links to the **institution** (persistent, graph-visible)
- The `metadata.agent_id` records the **agent** (execution detail, auditable)

### Production Identity Enforcement

For Phase 6, the runtime identity is no longer just a naming convention. It is a deployment requirement:

- `x-cortex-agent-id` must resolve to `agent:eudaemon-alpha-01`
- the gateway must validate that identity against the actor registry when enforcement is enabled
- the target Space must include the agent in `members`
- the target Space should declare the intended `archetype` for steward review and operational clarity

### DPub Attribution

The Chronicle is authored *by* `Cortex Stewardship Institute` (institutional voice) and *produced by* `agent:eudaemon-alpha-01` (execution detail). When the agent upgrades, the authorial voice continues; the production credits change.

---

## Stage Promotion Criteria

| Transition | Required Evidence |
|-----------|-------------------|
| 1 → 2 | ≥ N contributions accepted by steward; chronicle has ≥ M entries; zero governance violations |
| 2 → 3 | Cortex runtime supports agent workload type; migration prototype passes parity test |
| 3 → 4 | Institution promoted to `Operational` by steward decision; governance charter ratified |

---

## Bootstrap Paradox Resolution

The institution exists *before* the platform formally supports Institution nodes at runtime. This is resolved by temporal ordering:

1. **Stage 1**: Eudaemon exists as an external agent producing contributions. It is *described as* an institution in the DPub but not yet *instantiated* as an Institution node in the live graph.
2. **Stage 3-4**: The Institution contribution is instantiated in the graph; historical contributions are linked retroactively.

The `Emergent` lifecycle status explicitly supports institutions that don't yet have full operational capability. The DPub documents formation during this emergent period — creating a self-referential origin story.
