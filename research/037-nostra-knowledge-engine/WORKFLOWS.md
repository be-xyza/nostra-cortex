---
id: '037'
name: nostra-knowledge-engine
title: Knowledge Engine Workflows
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-30'
updated: '2026-01-30'
---

# Knowledge Engine Workflows

This document defines the **Standard Operating Procedures (SOPs)** for interacting with the Nostra Knowledge Engine. These workflows orchestrate the [037 Extraction Pipeline](../../nostra/extraction/) and define the collaboration between **Users**, **Librarians**, and **Gardeners**.

## 1. `ingest-knowledge` (Core Pipeline)

**Trigger**:
- 👤 User uploads file in Artifacts Editor
- 🤖 Librarian Agent scans a repository
- ⏱️ Scheduled Crawler

**Actors**: `Librarian` (Agent), `User` (Human Reviewer)

```yaml
id: "nostra.workflows.knowledge.ingest"
name: "Ingest Knowledge Asset"
version: "1.0.0"

steps:
  # 1. Pipeline Execution (Automated)
  - id: "run_pipeline"
    type: "SystemOp"
    service: "nostra-extraction"
    method: "run_pipeline"
    inputs:
      document: "${trigger.document}"
      config:
        stages: ["Ingest", "Extract", "Reflect", "Classify"]

  # 2. Confidence Check
  - id: "check_confidence"
    type: "Gate"
    condition: "${run_pipeline.metadata.confidence_score > 0.85}"
    if_true: "commit_graph"
    if_false: "human_review"

  # 3A. Human-in-the-Loop (If low confidence)
  - id: "human_review"
    type: "UserTask"
    role: "ContentReviewer"
    title: "Review Low-Confidence Extraction"
    description: "The Librarian is unsure about ${run_pipeline.metadata.flagged_entities}. Please verify."
    ui_component: "nostra-knowledge-graph.ReviewCanvas"
    inputs:
      extraction_result: "${run_pipeline.result}"

  # 4. Commit (Write to Graph)
  - id: "commit_graph"
    type: "SystemOp"
    service: "nostra-graph"
    method: "merge_batch"
    inputs:
      entities: "${run_pipeline.entities}"
      provenance: "${trigger.source}"
```

---

## 2. `start-reflexion` (On-Demand Critique)

**Trigger**:
- 👤 User clicks "Critique Draft" in Artifacts Editor
- 🤖 Analyst Agent requests feedback

**Actors**: `Analyst` (Agent)

**Context**: This bypasses the full extraction loop and uses the **Reflect Stage** (Graphiti Pattern) as a standalone reasoning tool.

```yaml
id: "nostra.workflows.knowledge.reflexion"
name: "Reflexion Loop"

steps:
  - id: "read_draft"
    type: "SystemOp"
    service: "artifact-store"
    method: "get_content"

  - id: "reflect"
    type: "SystemOp"
    service: "nostra-extraction"
    method: "run_stage"
    inputs:
      stage: "Reflect"
      content: "${read_draft.content}"
      prompt: "Identify logical gaps and missing citations."

  - id: "create_comment"
    type: "SystemOp"
    service: "artifact-store"
    method: "add_annotation"
    inputs:
      target_id: "${trigger.artifact_id}"
      content: "${reflect.output}"
      type: "critique"
```

---

## 3. `prune-orphans` (Garden Maintenance)

**Trigger**:
- ⏱️ Weekly Schedule (every Monday 00:00 UTC)
- ⚠️ "High Fragmentation" Alert

**Actors**: `Gardener` (Agent)

```yaml
id: "nostra.workflows.knowledge.prune"
name: "Prune Global Graph"

steps:
  - id: "scan_orphans"
    type: "SystemOp"
    service: "nostra-graph"
    method: "find_disconnected_subgraphs"
    inputs:
      min_size: 1 # Single standalone nodes

  - id: "categorize_orphans"
    type: "AgentTask"
    role: "Gardener"
    instructions: "Classify these 50 orphans: Archive (trash), Link (find parent), or Incubate (keep)."

  - id: "execute_prune"
    type: "SystemOp"
    service: "nostra-graph"
    method: "batch_update_status"
    inputs:
      updates: "${categorize_orphans.decisions}"
```
