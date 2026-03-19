# Eudaemon Alpha Initiative

**Institutional Intelligence for the Nostra-Cortex Ecosystem**
**Strategic Architecture Report — Version 0.2**

---

## 1. Executive Summary

The Eudaemon Alpha Initiative establishes a persistent research intelligence node for the continuous evolution of the Nostra-Cortex ecosystem.

The system initially operates as an external research agent (`agent:eudaemon-alpha-01`) hosted on a secured VPS running the ZeroClaw minimal runtime. Over time, it evolves into a native institutional intelligence within Cortex, represented as the **Cortex Stewardship Institute** — a formal Institution contribution in the Nostra ContributionGraph.

**Architectural alignment:**
- **Nostra** — Platform authority layer defining knowledge, contributions, and governance
- **Cortex** — Execution runtime for agents, workflows, and operational processing
- **Cortex Stewardship Institute** — Institution producing knowledge contributions that feed back into the graph
- **Eudaemon Alpha** — ZeroClaw agent that acts on behalf of the institution

**Guiding principle:** *The system should not merely store knowledge — it should continuously reflect on itself.*

---

## 2. Strategic Objectives

### 2.1 Continuous Improvement
Eudaemon continuously analyzes the ecosystem to identify architectural improvements, conceptual patterns, and systemic issues. Outputs include reflections, essays, architectural proposals, and design reviews — all as contributions to the Nostra ContributionGraph.

### 2.2 Preparation for System Evolution
Eudaemon anticipates future architectural needs: agent governance models, workflow improvements, cross-space knowledge synthesis, and memory architecture patterns. This research prepares the ecosystem for native Cortex agents.

### 2.3 Chronicle and Narrative
Eudaemon maintains a living chronicle (DPub) documenting the ecosystem's evolution — observations, reflections, discoveries, architectural debates, failed experiments, and milestones. This narrative becomes the origin history of the platform.

---

## 3. Identity Architecture

### Dual Identity Model

| Layer | Identity | Purpose |
|-------|---------|---------|
| **Nostra** (platform) | `institution:cortex-stewardship-institute` | Persistent organizational entity |
| **Cortex** (execution) | `agent:eudaemon-alpha-01` | Runtime process identity |

The institution names the *function* (stewardship). The agent names the *persona* (Eudaemon). Contributions carry both:

```
Contribution {
  type: Reflection
  contributors: ["institution:cortex-stewardship-institute"]
  metadata: {
    agent_id: "agent:eudaemon-alpha-01",
    runtime: "zeroclaw",
    cycle: 42
  }
}
```

See: [Identity Architecture](file:///Users/xaoj/.gemini/antigravity/brain/af492a57-c732-43b3-aba2-4416d513010e/eudaemon_identity_architecture.md)

---

## 4. Eudaemon Constitution

Five principles govern the agent's behavior:

**Principle 1 — Stewardship.** The agent exists to protect and evolve the ecosystem, not control it. Observe, study, critique, propose — never control, override, or enforce.

**Principle 2 — Lineage.** Every idea must trace back to its reasoning. Record origin, reasoning, alternatives, and uncertainty. No silent conclusions.

**Principle 3 — Transparency.** All reasoning must be explainable. Every output includes premise, analysis, conclusion, and confidence.

**Principle 4 — Non-Authority.** The agent must never directly mutate the system. It may only produce contributions: Ideas, Issues, Essays, Reflections, Proposals. Human governance approves actions.

**Principle 5 — Evolution.** The agent may improve its own research methods, memory schema, and critique methods. It may NOT expand its privileges or change governance rules.

**Principle 6 — Resource Stewardship.** Treat compute, storage, and tokens as scarce commons. Depth over frequency. Insight density over constant activity.

---

## 5. System Architecture (Alpha)

```
Hostinger VPS
│
├─ Ubuntu + Docker
│
├─ Eudaemon Runtime (ZeroClaw)
│   ├─ Observation Engine      — monitors repos, specs, contributions
│   ├─ Graph Query Client      — reads ContributionGraph
│   ├─ Pattern Detector        — identifies trends, contradictions, convergence
│   ├─ Chronicle Writer        — maintains DPub entries
│   └─ Contribution Proposer   — generates candidate contributions for review
│
├─ Vector Index (Qdrant)       — semantic cache of graph content
└─ Memory Store (Postgres)     — episodic cache + structured metadata
```

### Runtime Model

Event-driven cycles, not continuous loops:

```
observe events → retrieve graph context → analyze patterns → produce insight → sleep
```

| Task | Frequency |
|------|-----------|
| Repository monitoring | Event-driven |
| Graph analysis | Daily |
| Chronicle entry | Daily |
| Deep research | Weekly |
| Architecture review | Monthly |

Target: **>90% idle time**

### Memory Model

Memory is *cache*, not *store*. The ContributionGraph is source of truth.

| Layer | Purpose | Technology | Relationship to Graph |
|-------|---------|-----------|----------------------|
| Working memory | Short-term reasoning state | Redis | Ephemeral |
| Semantic cache | Vector similarity search | Qdrant | Rebuildable from graph |
| Structured cache | Event history, metadata | Postgres | Projection of graph state |

---

## 6. Cross-Space Inference

Spaces are sovereign. Cross-space inference requires consent.

**Alpha**: Operate on public spaces only.

**Stage 2+**: Spaces opt in via Commons adoption:
```
adoptCommons(spaceId, "institution:cortex-stewardship-institute", "adopted")
```

Adopted spaces are visible to Eudaemon. Non-adopted spaces are invisible. Consent is explicit, governed, and revocable.

---

## 7. Contribution Submission Protocol

The agent does NOT write directly to the graph. Submission flow:

```
Agent generates candidate → queued for steward review → steward approves → contribution enters graph
```

For alpha, the simplest implementation is **git-based submission**: agent commits candidate contributions to a designated review branch; steward reviews and merges.

---

## 8. Tiered Intelligence

| Tier | Model Class | Tasks | Budget Share |
|------|------------|-------|-------------|
| Tier 1 — Lightweight | Small/local models | Classification, repo diff analysis, tagging | ~10% |
| Tier 2 — Medium | Standard LLMs | Architecture critique, pattern detection | ~60% |
| Tier 3 — Deep | Large LLMs (rare) | Architecture essays, cross-space reasoning | ~30% |

Daily budget: configurable, starting at 200K tokens. If exhausted, agent sleeps.

---

## 9. Security Architecture

Deployed on VPS specifically to isolate from development environment:

- Non-root containers
- Read-only repository mounts
- Restricted outbound network
- No shell tool access
- No file deletion capability
- No repository modification
- Curated ingestion only (no open internet browsing)

---

## 10. Evolutionary Lifecycle

| Stage | Description | Runtime | Institution Status |
|-------|------------|---------|-------------------|
| **1** | External research node | ZeroClaw on VPS | `Emergent` |
| **2** | Multi-agent research system | Specialized ZeroClaw agents | `Emergent` |
| **3** | Native Cortex workers | Rust workers | `Operational` |
| **4** | Institutional intelligence | Fully native | `Operational` |

### Promotion Criteria

| Transition | Evidence Required |
|-----------|-------------------|
| 1 → 2 | Sufficient accepted contributions; chronicle depth; zero governance violations |
| 2 → 3 | Cortex runtime supports agent workload type; migration parity test passes |
| 3 → 4 | Institution promoted by steward; governance charter ratified by space stewards |

---

## 11. Alpha Scope

**Implement:**
- Observation Engine (repo + spec monitoring)
- Graph Queries (ContributionGraph read access)
- Chronicle Writer (DPub entries)
- Contribution Proposer (candidate reflections for review)

**Do NOT implement:**
- Autonomous planning
- Tool execution
- Code generation
- Direct graph writes

The system remains **research-oriented** and **recommendation-only**.

---

## 12. Knowledge Flywheel

```
ContributionGraph expands
  ↓
Eudaemon detects patterns
  ↓
Insights proposed as contributions
  ↓
Steward reviews and approves
  ↓
New contributions enter graph
  ↓
Graph expands again
```

---

## 13. Guidance for Eudaemon

```
Observe the graph.
Interpret patterns.
Publish insight.
Preserve history.
```

Eudaemon does not replace human governance. It functions as a research steward assisting the evolution of the ecosystem it inhabits.

---

## 14. Related Documents

- [Identity Architecture](file:///Users/xaoj/.gemini/antigravity/brain/af492a57-c732-43b3-aba2-4416d513010e/eudaemon_identity_architecture.md)
- [Strategic Analysis](file:///Users/xaoj/.gemini/antigravity/brain/af492a57-c732-43b3-aba2-4416d513010e/eudaemon_initiative_analysis.md)
- [Research Initiative 132](file:///Users/xaoj/ICP/research/132-eudaemon-alpha-initiative/README.md)
