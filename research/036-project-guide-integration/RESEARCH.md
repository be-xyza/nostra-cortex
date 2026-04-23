---
id: '036'
name: project-guide-integration
title: 'Research Initiative: Project Guide Integration (036)'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Research Initiative: Project Guide Integration (036)

**Status**: Active
**Type**: Meta-Analysis
**Created**: 2026-01-18

## 1. Overview
This initiative serves as the parent container for adapting the patterns found in the [Project Guide](https://github.com/sojohnnysaid/project-guide) repository into the Nostra ecosystem.

The research has been split into **three distinct paths**, each with its own dedicated study and implementation plan.

## 2. Path 1: Local Repository Auditability
**Goal**: Turn the local `/ICP` repo into a self-documenting, auditable entity using "Shadow Knowledge Bases".

-   📄 **[Study: Local Audit Patterns](./STUDY_LOCAL_AUDIT.md)**
-   🛠️ **[Plan: Local Audit Implementation](./PLAN_LOCAL_AUDIT.md)**

**Key Resolving Initiatives**:
-   `019-nostra-log-registry` (Storing audit diffs)
-   `008-nostra-contribution-types` (Reflections)

## 3. Path 2: SKILL.md Conversion & Sync
**Goal**: Package the analysis logic as a portable `auto-document` Skill that can be distributed via the Skills Sync Service.

-   📄 **[Study: Skill Conversion](./STUDY_SKILL_CONVERSION.md)**
-   🛠️ **[Plan: Skill Packaging](./PLAN_SKILL_CONVERSION.md)**

**Key Resolving Initiatives**:
-   `016-nostra-skills-sync-service-use-case` (Distribution)
-   `014-ai-agents-llms-on-icp` (Agent Capabilities)

## 4. Path 3: Nostra Workflow & A2UI Visualization
**Goal**: Contribute to the broader "Space Dashboard" in `013`, visualizing the analysis results within the Orchestration Layer.

-   📄 **[Study: Space Dashboard (013)](../013-nostra-workflow-engine/STUDY_SPACE_DASHBOARD.md)**
-   🛠️ **[Plan: Visual Workflow](./PLAN_WORKFLOW_A2UI.md)**

**Key Resolving Initiatives**:
-   `013-nostra-workflow-engine` (Home of the Dashboard)
-   `007-nostra-spaces-concept` (The User Interface)

## 5. Original Analysis
*Below is the original analysis of the project-guide repository.*

### Repository Findings
- **Repo**: `https://github.com/sojohnnysaid/project-guide`
- **Core Logic**: `guide.py` uses recursive file scanning + LLM summarization.
- **Output**: Generates `initial-summaries.txt` (raw) and `structure.json` (hierarchical).
- **Utility**: Highly effective for "bootstrapping" knowledge about legacy codebases.
