---
id: '015-taxonomy-validation'
name: taxonomy-validation
title: Taxonomy Validation (Semantic Stress Test)
type: research
project: nostra
status: draft
authors:
- User
- Antigravity
tags: ["008", "015", "taxonomy", "ast", "tldraw"]
created: '2026-02-25'
updated: '2026-02-25'
---

# Taxonomy Validation: The Semantic Stress Test

## 1. Objective
To formally prove whether the existing Nostra Contribution Types (`008` Graph Entity Taxonomy) provides sufficient granularity to support the Open Source Library (`015`) Feasibility Check.

**Hypothesis**: The existing `[Person, Organization, Library]` referent nodes structurally force semantic precision loss during AST ingestion, causing the Knowledge Graph to fail multi-component feasibility resolution.

---

## 2. Step 1: The Control Subject (`tldraw`)
We select the `tldraw` codebase, a complex whiteboarding tool. It is an ideal subject because it exports distinct, highly modular architectural capabilities (UI components, generic state syncing, rendering engines).

## 3. Step 2: Simulated Ingestion (AST Extraction)
If we run `agent-task-ast-parse` on `tldraw`, the agent identifies the following 5 distinct capabilities:
1.  **Capability A**: `TldrawEditor` (React Component for the canvas UI).
2.  **Capability B**: `SyncStore` (Yjs/CRDT state synchronization adapter).
3.  **Capability C**: `AssetManager` (Blob storage handler for images).
4.  **Capability D**: `ShapeUtil` (Mathematical intersection algorithms).
5.  **Capability E**: `BaseBoxShape` (A specific rendering class).

## 4. Step 3: The Graph Construction Test (Using Current `008` Rules)
Under current `008` rules, we must map these to either `EntityType: Library` or `ContributionType: Idea`. We cannot use `Module`, `Component`, or `Function`.

**Attempted JSON-LD Mapping**:
```json
{
  "@context": "nostra.kip.v1",
  "type": "Library",
  "name": "tldraw",
  "implements": [
    { "type": "Idea", "name": "Canvas UI" },
    { "type": "Idea", "name": "CRDT Sync" },
    { "type": "Idea", "name": "Image Storage" },
    { "type": "Idea", "name": "Intersection Math" }
  ]
}
```

*Result:* **Immediate Precision Loss**. The physical AST objects (`SyncStore`, `AssetManager`) are collapsed into abstract `Idea` strings attached to the monolithic `Library` node. We lose the boundaries between *what* the library is and *how* it is structured internally.

---

## 5. Step 4: The Query Feasibility Test
A user executes the `015` Feasibility Check workflow.
**User Query**: *"I am building a text editor. I need a CRDT state synchronization adapter. Can I use something existing?"*

**Expected Agent Behavior (Optimal)**:
"Yes. You can import the `SyncStore` component independently from the `tldraw` library, skipping the canvas UI."

**Actual Agent Behavior (Current `008` Graph)**:
"Yes. The `tldraw` library implements the `CRDT Sync` idea."

*Failure Analysis*: The LLM recommends bringing in the entire `tldraw` monolithic library (2MB+ bundle) just to get the CRDT adapter, because the graph cannot differentiate the `SyncStore` component from the `TldrawEditor` component. The graph only mathematically knows that "tldraw" does "CRDT Sync".

## 6. Conclusion
The Semantic Stress Test **FAILED**.

The current `008` Graph Entity Taxonomy is inadequate for AST-level semantic mapping. Collapsing physical structural boundaries (Classes, Modules, Functions) into abstract `Idea` edges destroys the modularity required for composition.

### Formal Proposal
We must formally request an extension to the `008-nostra-contribution-types` initiative.

**Proposed Addition to `EntityType` Enum**:
*   `Library` (The packaged namespace, e.g., `tldraw`)
*   `Module` / `Package` (e.g., `@tldraw/sync`)
*   `Component` (e.g., `SyncStore`)
*   `Function` (e.g., `intersectBoundingBox`)
*   `Concept` (A canonical algorithm standard, e.g., `CRDT_Protocol`)
