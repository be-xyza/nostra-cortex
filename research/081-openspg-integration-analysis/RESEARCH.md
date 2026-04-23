---
id: 081
name: openspg-integration-analysis
title: 'Research Findings: OpenSPG Architecture'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-30'
---

# Research Findings: OpenSPG Architecture

## 1. Executive Summary
OpenSPG is a comprehensive "Semantic-enhanced Programmable Graph" engine. Its core value lies in its **Semantic Schema** (Event/Entity distinction) and **KGDSL** (Logic Rules), rather than its implementation (heavy Java/Spark stack).
**Recommendation**: Adoption of its **Concepts** (Schema definitions, Pattern-Match-Rule structure) into Nostra, but implementation via Rust/WASM (replacing their Java Builder).

## 2. SPG-Schema Analysis

### 2.1 Core Concepts (`BaseSPGType.java`)
OpenSPG strictly types its graph nodes into:
-   `EntityType`: Standard things (Person, Company).
-   `EventType`: Hyper-edges with temporal properties (`occur_time`, `subject`, `object`).
-   `ConceptType`: Semantic taxonomy (e.g., Medical Classifications).

### 2.2 Relevance to Nostra
We should adopt the `EventType` primitive. Currently, Nostra treats "DPub Publication" as a process, but explicitly modeling it as an `EpidemicEvent` style node (Subject: Author, Object: DPub, Time: Now) allows for richer queries ("Find all publications by X in 2024").

## 3. KGDSL Analysis

### 3.1 Grammar (`KGDSL.g4`)
KGDSL is a superset of ISO GQL/Cypher. It introduces a powerful **Pattern-Logic-Action** block structure:
```
GraphStructure {
    MATCH (s)-[p]->(o)
}
Rule {
    R1: s.age > 18
}
Action {
    createNode(...)
}
```
### 3.2 Adaptation
We should use this structure for the **Nostra Workflow Engine** (`013`). Instead of imperative code, Users/Agents define workflows as "Graph Patterns" (Trigger) + "Rules" (Condition) + "Actions" (Result).

## 4. Builder & Operator Analysis

### 4.1 "Hollow" Java Frame
The `OperatorLinking` and `OperatorFusing` classes reveal that the Java engine is largely a scheduler for **Python Scripts** (`PythonOperatorFactory`). The actual intelligence (OneKE, Linking) happens in Python.

### 4.2 Recommendation
Since the "Brain" is already Python, we can discard the heavy Java "Body" and build a lightweight **Rust/WASM Body** (Nostra Worker) to orchestrate the same Python logic (or Rust native logic where performance matters).

## 5. Adaptation Recommendations

| Component | OpenSPG Approach | Nostra Adaptation |
| :--- | :--- | :--- |
| **Schema** | `SPGSchema` (Java Class) | Define `EventType` in `040-schema` (Rust types). |
| **Logic** | `KGDSL` (Antlr Parsed) | Adopt `Pattern-Rule-Action` block structure for Workflows. |
| **Builder** | Java Scheduler -> Pemja -> Python | Rust Worker -> Python Microservice (or WASM). |
| **Storage** | Cloudext (MySQL/HBase) | ICP Canisters (`elna` + `nostra_graph`). |
