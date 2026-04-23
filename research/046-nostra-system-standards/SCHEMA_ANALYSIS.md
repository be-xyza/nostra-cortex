# Analysis: Nostra Schema Standards

**Date**: 2026-01-21
**Source**: `research/040-nostra-schema-standards`
**Context**: Reviewing the current schema strategy for long-term viability and alignment with `046` System Standards.

---

## 1. Overview of Current Strategy (040)
The current strategy proposes a "Graduation Process" for schemas, moving from personal customization to universal standards.

*   **The Index**: A centralized "Graph of Standards" defining core types (`nostra.core`, `nostra.social`, etc.).
*   **The Lifecycle**:
    1.  **Personal**: User defines ad-hoc types.
    2.  **Candidate**: Usage metrics (>50 users) trigger review.
    3.  **Standard**: Governance approval leads to a "Gold Standard" definition.
*   **Bootstrapping**: New workflows query the Index to map existing user data (e.g., "PersonalContact") to required standard inputs (`Person`), enabling zero-friction setup.

## 2. Feedback & Critique

### Strengths
*   **Organic Growth**: Allows users to start messy ("Personal") and evolve to structured ("Standard") without blocking creativity.
*   **Interoperability Focus**: The mapping strategy handles the "N+1" integration problem effectively.
*   **Governance Clarity**: Clear separation between "Candidate" and "Standard" prevents pollution of the core namespace.

### Weaknesses (Long-Term Risks)
*   **Versioning Hell**: Managing mappings between `Personal:v1` -> `Standard:v2` -> `Standard:v3` could become exponentially complex.
*   **Governance Bottleneck**: Relying on "Stewards" for all standard approvals might slow down innovation as the ecosystem scales.
*   **Semantic Drift**: A "Task" in a "Coding Workflow" might differ significantly from a "Task" in a "Grocery Workflow", making a single `nostra.core.Task` too rigid.

## 3. Recommendations for Long-Term Success

### 3.1 Adopt "Structural Typing" for Candidates
*   **Suggestion**: Instead of binary "Personal vs. Standard", use structural compatibility (Duck Typing).
*   **Mechanism**: If a Personal Schema has `{ title: Text, due: Nat }`, it *automatically* implements `nostra.core.Task` interface without formal graduation, provided it passes a validator.
*   **Benefit**: Reduces the need for formal "Graduation" events; automated compliance.

### 3.2 Implements vs. Extends
*   **Suggestion**: Enforce Composition over Inheritance.
*   **Rule**: `MyProject` should **implement** `nostra.core.Entity` (via a trait/interface system) rather than **extend** a base class. This avoids fragile inheritance chains.

### 3.3 The "Core 5" Definition
We recommend locking down the following **Core 5** primitives immediately (Immutable for 12 months) to build a stable foundation:

1.  **`nostra.core.Entity`**: `{ id: UUID, owner: Principal, created_at: Timestamp, confidence: Float }`
    *   *Note: Added `confidence` per 046.*
2.  **`nostra.core.Link`**: `{ source: ID, target: ID, relationship: RelationType, weight: Float }`
3.  **`nostra.core.Agent`**: `{ id: Principal, capabilities: [Skill], reliability_score: Float }`
4.  **`nostra.core.Event`**: `{ type: String, payload: Blob, source: ID, timestamp: Timestamp }`
5.  **`nostra.core.Locus`**: `{ coordinates: Text, geohash: Text, jurisdiction: Ref, source: Text, confidence: Float }`

### 3.4 Automated Depreciation
*   **Tooling**: Build a "Linter" into the Schema Manager that warns users when they are using fields that have been deprecated in the Standard Index, offering one-click migrations.

---

## 4. Integration Report (2026-01-21)
**Reviewer**: Antigravity

The recommendations above have been **Accepted and Integrated** into `research/040-nostra-schema-standards`.

### Key Changes Validated:
1.  **Strategy Shift**: `040/RESEARCH.md` updated to prioritize "Structural Interoperability" over the "Graduation Process".
2.  **Core Interfaces**: `040/PLAN.md` updated to focus on defining Trails (Behaviors) rather than Class Hierarchies.
3.  **Core 4 Integrity**: The `Entity` primitive now strictly includes `provenance` and `trust_score` (mapping to 046 confidence) as mandatory fields.
4.  **Runtime Casting**: The implementation plan now includes a "Schema Validator" and "Cast-on-Read" mechanism for the UI.

This effectively resolves the "Governance Bottleneck" risk by automating compatibility checks.

## 5. External Alignment (2026-01-21)
**Analysis**: `research/046-nostra-system-standards/API_MARKETPLACE_ANALYSIS.md`

We analyzed public API marketplaces (RapidAPI, Apify) and governance standards (Snapshot, Tally) to benchmark our approach.

### Key Decisions for Interoperability:
1.  **Agents as Actors**: Adopt Apify's strict `input_schema` / `output_schema` pattern (JSON Schema) for all Nostra Agents to enable auto-generated UIs.
2.  **Wrappers over Types**: Instead of forcing external APIs to conform to Nostra, we will introduce `nostra.core.Adapter` as a primitive to map external OpenAPI/GraphQL definitions to internal types.
3.  **Semantic Web**: Align core type definitions with `schema.org` via JSON-LD contexts to ensuring "web-wide" compatibility beyond just the crypto ecosystem.

## 6. Final Standards (2026-01-21)
The core primitives have been formalized in **[STANDARDS.md](STANDARDS.md)**.
This document supersedes previous definitions and should be used as the reference for all new development.
