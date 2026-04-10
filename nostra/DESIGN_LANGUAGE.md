# DESIGN_LANGUAGE.md
Version: 0.1.0 (Foundational Constitutional Release)
Status: Draft – Proposed as Commons Seed
Scope: commons
Authority Mode: Constitutional Substrate
Applies To: Nostra (Platform), Cortex (Execution Runtime via Profile)

---

## 1. Purpose

The Nostra Design Language (NDL) defines the canonical interface grammar for Nostra Cortex.

**NDL is not a theme.**

NDL is the constitutional projection layer of:
*   Contributions
*   Governance
*   Time
*   Authority
*   Identity
*   Simulation
*   Accessibility

NDL ensures that every interface rendered within Nostra Cortex:
*   Accurately represents platform truth
*   Prevents spoofing of governance artifacts
*   Preserves lineage and historical integrity
*   Maintains universal accessibility
*   Constrains AI-generated UI to constitutional invariants

NDL is implemented as a Commons Institution and may be:
*   Adopted
*   Pinned
*   Forked
*   Upgraded via Proposal

### 1.1 Semantic Boundaries (Lexicon)

To ensure NDL applies correctly across the Nostra Platform and Cortex Execution environments, the following semantic terms must be strictly observed in all UI surfaces:

*   **Space** (Nostra): The sovereign, user-facing container for platforms, providing domains for communities, data isolation, and governance rules. Under no circumstances should this be called a "workspace" in the UI.
*   **Workspace** (Developer Layer): A strictly structural developer-side term defining canonical code boundaries (e.g. `nostra/` workspace, `cortex/` workspace). Never surface this term to users!
*   **Workbench** (Cortex): The execution environment and application shell (e.g., React/Vite/A2UI) where tools operate on data within a specific Space.

#### Fork / Merge / Lineage Semantics

Nostra adopts the DAG-based versioning pattern common to distributed version control but redefines its operations under constitutional governance. The following terms must be distinguished from their VCS analogues:

| Term | Nostra Definition | VCS Analogue | Key Difference |
| :--- | :--- | :--- | :--- |
| **Fork** | Constitutional right to diverge any Contribution (Idea, Project, Space, Institution, Commons). Creates a new entity with `fork_from` pointer. Governance-mediated — target Space may impose different rules. | Git/GitHub fork: mechanistic copy of a codebase at a point in time. | Nostra forks are **cross-entity** (not just code), **governance-aware**, and **lineage-preserving**. |
| **Merge** | Governance primitive. Requires Proposal → Steward Review → Approval. Combines two Contributions while preserving full lineage history. | Git merge: algorithmic combination of two branches. Conflict resolution is mechanical. | Nostra merges are **deliberative** (humans decide). Git merges are **mechanistic** (algorithms decide). |
| **Lineage** | Cryptographically-attested version chain via `previousVersionId` + `previousVersionChecksum` (DEC-085-006). Also expressed as `derives_from` graph edges between Institutions. | Git commit DAG: parent-pointer chain, content-addressable by SHA. | Nostra lineage is **tamper-evident** and **queryable across entity types**, not scoped to a single repository. |
| **Version Chain** | Ordered sequence of Contribution versions where each version structurally attests to its predecessor. Rendered as an immutable visual trail per §4.1. | Git log: chronological commit history within a branch. | Nostra version chains carry **confidence scores**, **contributor lists**, and **phase transitions** — not just diffs. |

> [!IMPORTANT]
> These terms appear in motion tokens (`fork_diverge`, `merge_converge`), in the Contribution grammar (§4.5), and in governance flows. Misusing git/GitHub semantics in Nostra surfaces risks constitutional misrepresentation — e.g., implying a merge is automatic when it requires steward approval.

---

## 2. Constitutional Alignment

NDL derives authority from the following invariants:

**From Nostra Platform Spec:**
*   Everything Is a Contribution
*   Time Is a Primitive
*   History Is Sacred
*   Spaces Are Sovereign
*   Execution Is First-Class

**From Shared Specifications:**
*   Universal Access
*   Technology Neutrality
*   Replayable Events

**From AGENTS Charter:**
*   Authority requires responsibility
*   Sensitive actions require escalation
*   Memory integrity must be preserved

NDL is subordinate to these constitutional doctrines.

---

## 3. Design Philosophy

NDL operates under six core doctrines:

### 3.1 Authority Is Visible
Exploratory, Deliberative, and Decisive artifacts must be visually distinguishable.

### 3.2 Time Is Visible
Durability, scheduling, dormancy, and lineage must have spatial and motion representation.

### 3.3 History Is Immutable
Version chains must be structurally represented and never hidden.

### 3.4 Accessibility Is Mandatory
No interface is legitimate if it violates Universal Access.

### 3.5 Forking Is Legitimate
Visual grammar must support divergence without chaos.

### 3.6 Simulation Is Valid Input
Interactive states are first-class citizens of the interface.

---

## 4. Canonical Contribution Grammar

NDL is Contribution-first, not component-first.

Each ContributionType has a mandatory representation schema.

### 4.1 Contribution Base Layout
Every contribution must render:
*   `[Type Indicator]`
*   `[Title]`
*   `[Authority Badge]`
*   `[Phase Badge]`
*   `[Metadata Block]`
*   `[Body Content]`
*   `[Version Chain]`
*   `[Interaction Layer]`

Mandatory Metadata Fields (if present):
*   Contributors
*   CreatedAt / UpdatedAt
*   Version
*   Confidence Score
*   Status
*   Space Context

### 4.2 Phase-Based Authority Differentiation

| Phase | Visual Treatment |
| :--- | :--- |
| Exploratory | Soft elevation, neutral accent |
| Deliberative | Structured border, elevated emphasis |
| Decisive | Immutable badge, high-contrast anchor |

Decisive artifacts must always include:
*   Governance indicator
*   Vote summary (if applicable)
*   Ratification marker

### 4.3 Decision Artifact (Anti-Spoofing Invariant)
A Decision must include:
*   Ratified badge (non-removable)
*   GovernanceHost source reference
*   Vote metrics
*   Strategy badge
*   Immutable accent background

These elements cannot be overridden by:
*   Profile
*   Agent rendering
*   Custom theme

### 4.4 Proposal Artifact
Must display:
*   Target action
*   Target resource
*   Voting state
*   Deadline
*   Quorum indicator (if configured)

Must visually differ from: Idea, Post, Issue.

### 4.5 Fork Representation
Forked contributions must include:
*   `fork_from` indicator
*   Divergence animation (first render only)
*   Lineage breadcrumb

Merges must include:
*   Merge source
*   Merge target
*   Convergence animation

---

## 5. Temporal Motion System

NDL defines Semantic Motion Tokens. Motion is not decorative. It encodes state transitions in time.

### 5.1 Core Tokens
*   `enter_exploratory`
*   `enter_deliberative`
*   `ratify`
*   `fork_diverge`
*   `merge_converge`
*   `dormant_state`
*   `workflow_rehydrate`
*   `time_passed`
*   `attention_critical`

### 5.2 Reduced Motion Mapping
If `prefers-reduced-motion = true`:
*   Replace motion with opacity transitions
*   Preserve temporal meaning via iconography
*   Never remove semantic state indicator

Motion must never be required for comprehension.

---

## 6. Accessibility Enforcement Layer

NDL enforces:

### 6.1 Contrast
*   WCAG AA minimum
*   Governance artifacts require AAA contrast where feasible

### 6.2 Keyboard Navigation
*   Full navigability
*   Visible focus rings
*   Deterministic tab order

### 6.3 A2UI Schema Compliance
All components must define:
*   Semantic role
*   Accessible label
*   Motion metadata
*   Interaction description

**Accessibility violations invalidate compliance.**

---

## 7. AI Rendering Contract

Agents rendering Nostra UI must:
1.  Use only NDL-approved components.
2.  Respect canonical Contribution grammar.
3.  Preserve governance artifacts.
4.  Preserve authority badges.
5.  Preserve version chain display.
6.  Preserve confidence indicator (if present).
7.  Not visually downgrade decisive artifacts.

Agents may:
*   Adjust density (profile)
*   Adjust spacing tokens
*   Adjust color within allowed profile constraints

Agents may not:
*   Hide lineage
*   Remove governance indicators
*   Collapse version history
*   Re-label contribution type visually

**Violation = non-compliant render.**

---

## 8. Profiles (Density Modes)

Profiles adjust density and layout — not constitutional grammar.

### 8.1 Publication (Default Nostra)
*   Readability optimized
*   Comfortable spacing
*   Balanced motion

### 8.2 Cockpit (Cortex Runtime)
*   High-density
*   Data-forward
*   Dark-biased

### 8.3 Labs
*   Experimental layout allowed
*   Must preserve authority grammar
*   Clearly marked “Labs Mode”

### 8.4 DAO
*   Governance metrics emphasized
*   Vote state visible in list view

### 8.5 Simulation
*   ECS-based states visible
*   Real-time state deltas represented

---

## 9. Token Architecture

Directory:
```
nostra/commons/design/
  ├── tokens/
  │     ├── color.json
  │     ├── typography.json
  │     ├── spacing.json
  │     ├── motion.json
  │     ├── sound.json
  │     └── haptics.json
  ├── schemas/
  ├── icons/
  └── assets/
```
Tokens must be JSON schema validated, versioned, and backwards-compatible where possible.

---

## 10. Governance Model

NDL is an Institution with:
`scope = ["commons"]`

Lifecycle: Emergent → Operational → Archived

Changes require Proposal, Vote, and Version increment.

Adoption Modes:
*   `adopted` (auto-upgrade minor)
*   `pinned` (manual upgrade)

Spaces may fork NDL. Forked NDL must retain Contribution grammar invariants and declare divergence explicitly.

---

## 11. Non-Negotiable Invariants

The following cannot be altered by profiles, themes, or forks:
1.  Decision visual integrity
2.  Governance indicators
3.  Version chain visibility
4.  Contribution type indicator
5.  Accessibility baseline
6.  Authority badges
7.  Space visibility indicators

---

## 12. Enforcement Strategy (Future)

To be implemented via:
*   A2UI schema validation
*   Token JSON schema validation
*   Governance component signature check
*   CI-level accessibility validation
*   Agent rendering compliance test suite

---

## 13. Versioning

Format: `MAJOR.MINOR.PATCH`
*   **MAJOR**: Breaking constitutional change
*   **MINOR**: Additive grammar extension
*   **PATCH**: Token adjustments / accessibility fixes

---

## 14. Open Questions (v0.1)
*   Visual watermarking for ratified Decisions?
*   Cryptographic UI badge for GovernanceHost?
*   Confidence visualization standard?
*   Cross-space design divergence limits?

*These require research before v1.*

---

## 16. Surface Boundary Extension (v0.1)

To protect the creative freedom of space-level apps, games, and simulations without sacrificing constitutional integrity, NDL implements **Surface Classifications**.

### 16.1 Surface Types

Every rendered UI surface MUST declare its type:
*   `constitutional`: Projects platform truth (Contributions, Version Chains, Governance artifacts). Strict adherence to Tier 1 NDL components is mandatory.
*   `execution`: Runtime environments (Games, Monitors, Labs). Delegates layout and styling to the app builder.
*   `hybrid`: A mix of both (e.g., a workflow dashboard with embedded ratification panels).

### 16.2 Execution Surface Containment Rule

An `execution` surface MUST be visually bounded by an NDL containment frame to prevent governance spoofing. It must render a non-spoofable header band indicating: "Execution Surface" and provide a clear exit path back to the constitutional layer.

### 16.3 Exchange I/O Doctrine

Interactive apps and games are valid inputs to the Nostra Graph, but they must communicate via **Exchange I/O**.
*   Apps return structured `Contribution` events or `Mutation` contracts.
*   Apps MAY NOT inject UI fragments into constitutional surfaces. Truth is derived from the graph, not the rendering client.

### 16.4 Governance Impersonation Prohibition

Execution surfaces are physically barred (via `NDL_JSON_SCHEMA` and `NdlValidator`) from rendering Tier 1 authority components, specifically `Decision` and `RatificationMarker`.

### 16.5 Final Statement on Boundaries

NDL governs truth. It does not govern gameplay layout or domain-specific tools. This boundary preserves creative sovereignty while permanently cementing visual governance integrity.

---

## 17. Final Doctrine

NDL is not brand styling.

NDL is: **The deterministic constitutional interface of the Nostra knowledge graph.**

If a UI violates NDL:
*   It is visually incorrect.
*   It is constitutionally misleading.
*   It is non-compliant.
