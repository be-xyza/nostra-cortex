# Eudaemon Alpha — Agent Workspace Analysis

> Should `agent:eudaemon-alpha-01` have a dedicated workspace? What primitives enable scratchpads, simulations, and charts for agent discovery?

---

## 1. What Exists Today

Nostra already has several primitives relevant to agent workspaces:

| Primitive | What It Is | Relevance |
|-----------|-----------|-----------|
| **Space** | Sovereign collaboration domain with configurable contribution types, visibility, governance | Could host a dedicated agent research workspace |
| **Space Archetypes** | Pre-configured space types: Research (Ideas/Essays/Reviews/Reports), DAO, Product, Community | "Research space" archetype is purpose-built for this |
| **Artifact** | Archival contribution type: `Knowledge asset (File, Doc, Media)` | Container for scratchpad outputs, charts, simulation results |
| **Labs** | Experimental sandbox environments for testing new UI/UX paradigms | Conceptual match for agent experimentation |
| **SpatialPlane** | Spatial experiment persistence via JSONL + gateway APIs (Research 123) | Precedent for agent-generated spatial/visual data |
| **Simulation / Godot Bridge** | ECS-mapped game states → ContributionGraph; interactive states as first-class data | Framework for simulations that feed knowledge back |
| **AGUI Canvas** | Research Canvas, Planner Canvas patterns in reference materials | Agent-native workspace UI paradigm |

**No "heap board" or "universal block" exists as a named primitive.** But the components to build one already exist.

---

## 2. Three Placement Options

### Option A: Dedicated Research Space

Create a Nostra space specifically for Eudaemon:

```
Space: "Cortex Stewardship Research"
  archetype: Research
  visibility: public (or member-only)
  enabled_types: [Idea, Essay, Reflection, Report, Issue, Artifact, Proposal]
  governance: Cortex Stewardship Institute
  members: [agent:eudaemon-alpha-01, human-steward]
```

**How it works**: The agent's contributions live in this space. Scratchpads are `Artifact` contributions with `status: draft`. Charts/visualizations are Artifacts with media content. Notes are Reflections with `phase: Exploratory`. The space's activity stream shows the agent's complete research timeline.

**Strengths**:
- Uses existing primitives — zero new abstractions needed
- Space sovereignty gives the institution governance over its own research
- Activity stream is a natural "feed" of agent work
- Graph edges connect agent contributions to contributions in other spaces
- Commons adoption enables other spaces to opt into research observation
- Fully searchable, versionable, and graph-connected

**Weaknesses**:
- Spaces are designed for collaboration between *multiple* agents/humans. A single-agent space underutilizes the social features
- No native concept of "ephemeral" or "working" content — everything is a contribution with full versioning overhead
- Contribution types are structured; freeform scratchpad content doesn't naturally fit the type taxonomy

---

### Option B: Heap Board (New Primitive)

Introduce a new structured container — a "Heap Board" — that acts as an unstructured working memory surface within a space.

```
HeapBoard {
  id: "heap:eudaemon-workbench"
  spaceId: "cortex-stewardship-research"
  owner: "agent:eudaemon-alpha-01"
  items: [
    { type: "note", content: "Workflow patterns converging with simulation state...", pinned: true },
    { type: "chart", data: { ... }, renderer: "d3" },
    { type: "sketch", content: { excalidraw_elements: [...] } },
    { type: "snapshot", graphQuery: "related_to:cortex-runtime", timestamp: "..." }
  ]
  visibility: "space-members"
}
```

**How it works**: A freeform canvas-like surface where the agent dumps intermediate thoughts, charts, graph snapshots, and visual sketches. Items are unversioned, mutable, and ephemeral by default. High-value items get "promoted" to full contributions.

**Strengths**:
- Matches the cognitive model of research work: scattered notes → organized insight
- Enables visual/spatial reasoning (charts, sketches, graph snapshots)
- Lower overhead than full contributions for intermediate work
- Natural promotion path: heap item → draft Artifact → published contribution
- Agent-to-human communication surface: steward can browse the heap to see what the agent is "thinking about"

**Weaknesses**:
- **New primitive** — breaks "Everything Is a Contribution" principle
- No native versioning or lineage (by design, but violates Nostra's time-as-primitive philosophy)
- Requires new API surface, storage model, and UI rendering
- Creates a second-class knowledge tier that may accumulate without governance

---

### Option C: Augmented Contribution Model (Universal Block)

Instead of a new primitive, extend the existing `Artifact` contribution type with agent-aware subtypes:

```
Contribution {
  type: Artifact
  subtype: "scratchpad" | "chart" | "snapshot" | "simulation-result" | "note"
  phase: Exploratory
  status: draft
  confidence: 0.3
  contributors: ["institution:cortex-stewardship-institute"]
  metadata: {
    agent_id: "agent:eudaemon-alpha-01",
    ephemeral: true,           // may be garbage-collected if not promoted
    promote_threshold: 0.7     // auto-promote to permanent when confidence exceeds
  }
  content: { ... }             // type-specific payload
}
```

**How it works**: Agent working content is modeled as `Artifact` contributions in `Exploratory` phase with `draft` status and `ephemeral: true`. They live inside the dedicated research space but are structurally distinguished from polished outputs. When confidence or quality exceeds a threshold, the agent promotes them to permanent contributions.

**Strengths**:
- Preserves "Everything Is a Contribution" — no new primitives
- Ephemeral flag solves the storage/governance concern
- Confidence-based promotion creates a natural quality gate
- Full graph connectivity from day one — even scratchpads can link to other contributions
- Versioning still tracks the evolution of working notes into finished essays

**Weaknesses**:
- `Artifact` becomes a very broad category (files, docs, media, scratchpads, charts, simulation results)
- The subtype field is informal — not enforced by the contribution model
- Ephemeral contributions still consume graph storage until garbage-collected

---

## 3. Analysis of the User's Assumption

> *"My assumption is that this would allow for more powerful discovery capabilities and communication for agents."*

### ✅ Validated: Workspace Enhances Discovery

A dedicated workspace **does** enhance discovery because:

1. **Intermediate reasoning becomes visible.** Without a workspace, the agent's thought process is invisible between contributions. With one, the steward (and future agents in Stage 2) can observe the agent's working state — what it's investigating, what patterns it's tracking, what hypotheses it's forming.

2. **Graph connectivity amplifies pattern detection.** If scratchpad notes link to contributions across spaces, the agent builds a *working graph* of connections that may not yet warrant formal contributions. This working graph is itself a discovery mechanism.

3. **Simulation feeds knowledge.** The Nostra spec already treats simulation as first-class: *"Game states, simulations, and interactive labs feed data back into the knowledge graph just like human comments do"* ([spec.md L27](file:///Users/xaoj/ICP/nostra/spec.md#L27)). If the agent can run lightweight simulations (e.g., graph projections, "what if" analyses) and store results, those become inputs for future cycles.

### ⚠️ Challenge: Communication Between Agents

The assumption about "communication for agents" is forward-looking (Stage 2 multi-agent). For this to work:

1. **Agents need a shared read surface.** If Agent A writes a scratchpad note and Agent B needs to see it, both need access to the same space. This is solved by space membership.

2. **Agents need structured hand-off.** Freeform notes don't enable reliable agent-to-agent communication. The receiving agent needs structured data (contribution type, graph edges, confidence). This argues for Option C (augmented contributions) over Option B (freeform heap).

3. **The graph IS the communication medium.** In the Nostra model, agents communicate by writing contributions and reading the graph. A dedicated workspace space simply scopes this communication to a research context.

---

## 4. Recommendation

**Use a layered approach combining Option A + Option C:**

### Layer 1: Dedicated Research Space (Option A)
```
Space: "Cortex Stewardship Research"
  archetype: Research
  visibility: public
  governance: institution:cortex-stewardship-institute
```

This gives the institution a sovereign domain for all research activity, with its own activity stream, graph, and governance.

### Layer 2: Ephemeral Artifacts for Working Content (Option C)
Within that space, agent working content uses the `Artifact` type with subtypes:

| Subtype | Purpose | Example |
|---------|---------|---------|
| `note` | Working observation or hypothesis | "Noticing convergence in workflow patterns..." |
| `chart` | Data visualization | Token usage histogram, contribution frequency |
| `snapshot` | Graph projection at a point in time | Subgraph of runtime-related contributions |
| `simulation` | "What if" analysis result | Projected impact of WASM migration |
| `scratchpad` | Free-form working document | Collected evidence for an emerging essay |

All carry `phase: Exploratory`, `status: draft`, `ephemeral: true`.

### Layer 3: Promotion Gate
When working content matures:
```
Artifact (draft, ephemeral) → Reflection/Essay/Proposal (published, permanent)
```

The agent promotes content when confidence exceeds threshold. Steward can also promote manually.

### Why Not the Heap Board?

Option B (heap board) is conceptually appealing but architecturally problematic because:

1. **It breaks "Everything Is a Contribution."** Nostra's power comes from uniform treatment of all knowledge. A second-class content type fragments the model.
2. **It creates ungoverned territory.** Heap items without versioning or lineage are invisible to governance mechanisms. This violates Principle 2 (Lineage) of the Eudaemon Constitution.
3. **The SpatialPlane precedent shows it's not needed.** Research 123 solved spatial experimentation by persisting events through gateway APIs and JSONL — still structured, still auditable, still within the system. No new content class was required.
4. **Agent-to-agent communication needs structure.** Freeform heaps are readable by humans but not by other agents. Typed contributions with graph edges are machine-navigable.

The *feeling* of a heap board (freeform, low-friction, visual) can be achieved through the UI/UX layer without changing the data model. A research space with a "working board" view that shows ephemeral Artifacts in a canvas layout gives the same experience while preserving architectural integrity.

---

## 5. What This Unlocks for Eudaemon Alpha

| Capability | How |
|-----------|-----|
| **Scratchpad** | `Artifact(subtype: scratchpad, ephemeral: true)` in research space |
| **Notes** | `Artifact(subtype: note)` — observations that may become reflections |
| **Charts** | `Artifact(subtype: chart)` with D3/visualization payload |
| **Graph Snapshots** | `Artifact(subtype: snapshot)` — frozen subgraph projections |
| **Simulations** | `Artifact(subtype: simulation)` via Godot bridge or graph projection |
| **Agent Communication** | Other agents read the research space's graph — structured, typed, navigable |
| **Discovery** | Graph edges from ephemeral artifacts to contributions across spaces create a discovery web |
| **Steward Visibility** | Activity stream shows what the agent is working on, not just what it's published |

---

## 6. Summary

| Approach | Alignment | New Primitives | Discovery Power | Agent Communication |
|----------|-----------|---------------|----------------|-------------------|
| **A: Dedicated Space** | ✅ Strong | None | Medium — no working content | Medium — formal contributions only |
| **B: Heap Board** | ❌ Breaks core principle | Yes — new entity | High — freeform visual | Low — unstructured |
| **A+C: Space + Ephemeral Artifacts** | ✅ Strong | Subtype extension only | **High — working + published** | **High — typed, graph-connected** |

> [!IMPORTANT]
> The dedicated space + ephemeral Artifact model gives Eudaemon a powerful research workspace without introducing new primitives. It preserves "Everything Is a Contribution," maintains lineage for all working content, enables graph-connected discovery, and provides a natural promotion path from working notes to published insight. The "heap board" UX can be delivered as a *view* of this data, not a new data model.
