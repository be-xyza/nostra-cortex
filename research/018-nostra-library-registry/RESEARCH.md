---
id: "018"
name: "nostra-library-system"
title: "Research: Nostra Library System"
type: "research"
project: "nostra"
status: active
authors:
  - "Nostra Team"
tags: ["library", "registry", "living-library", "agent-tools", "protocol"]
created: "2026-01-15"
updated: "2026-01-17"
related:
  - "021-kip-integration"
  - "013-nostra-workflow-engine"
  - "017-ai-agent-role-patterns"
  - "026-nostra-schema-manager"
  - "040-nostra-schema-standards"
  - "041-nostra-vector-store"
---

# Research: Nostra Library System

**Date**: 2026-01-17
**Status**: ACTIVE
**Context**: Unified research covering library registry, governance, living library concepts, and agent tools integration.

> [!NOTE]
> This initiative consolidates:
> - **018-nostra-library-registry** (original)
> - **020-living-library-concept** (merged)
> - **024-agent-tools-library** (merged)

---

## 1. Executive Summary

Nostra Libraries are **living knowledge ecosystems** that:
1. **Grow with use** — not static packages
2. **Capture temporal history** — via Chronicle events
3. **Enable agent capabilities** — via agent tools
4. **Connect to external services** — via the N-Lib Protocol

This research establishes the conceptual framework, registry architecture, and protocol standard for Nostra Libraries.

---

## 2. The Living Library Paradigm

### 2.1 Static vs. Living Systems

| Dimension | Static Library | Living Library |
|:---|:---|:---|
| **Metaphor** | A book collection you read | A garden you cultivate |
| **Growth** | Updated by maintainers only | Grows organically through use |
| **Ownership** | Published by vendor | Owned/Stewarded by user/community |
| **Time** | Snapshot (v1.0, v2.0) | Continuous stream (timeline) |
| **Narrative** | Documentation | Visual story via graph evolution |

### 2.2 The Three Library Archetypes

| Archetype | Description | Governance |
|:---|:---|:---|
| **Curated** | Publisher-maintained canonical library | Version releases, community vote for upgrades |
| **Personal** | User-owned knowledge garden | Immediate changes, no approval needed |
| **Shared** | Community-grown collaborative space | Proposal + voting for changes |

### 2.3 Growth Through Interaction

Every user action leaves a trace in the graph:

| User Action | Graph Effect |
|:---|:---|
| Reading an entity | Creates implicit "interest" edge (optional) |
| Creating a note | Adds node + links to current context |
| Connecting two ideas | Explicit relationship formed |
| Running a workflow | Workflow outputs become entities |
| Making a decision | Decision node records choice + alternatives |

---

## 3. Temporal Dimension: The Chronicle

The Chronicle is the event log that captures how a library evolved:

```motoko
type ChronicleEventType = {
    #EntityCreated;
    #EntityUpdated;
    #EntityArchived;
    #RelationshipFormed;
    #RelationshipRemoved;
    #WorkflowStarted;
    #WorkflowCompleted;
    #LibrarySeeded;
    #LibraryMerged;
    #LibraryForked;
};

type ChronicleEvent = {
    id : Text;
    libraryId : Text;
    timestamp : Int;
    eventType : ChronicleEventType;
    actorPrincipal : Principal;
    affectedEntities : [Text];
    metadata : [(Text, Text)];
};
```

### Temporal Queries

| Query | Purpose |
|:---|:---|
| `getGraphAtTime(timestamp)` | "Show me my graph as it was 30 days ago" |
| `getChronicle(since, until)` | "What happened in the last week?" |
| `getGrowthRate(period)` | "How fast is this library growing?" |

---

## 4. N-Lib Protocol Standard

> [!IMPORTANT]
> See [STUDY_NLIB_PROTOCOL.md](./STUDY_NLIB_PROTOCOL.md) for full specification.
> JSON Schema: [`nostra/spec/n-lib.schema.json`](file:///Users/xaoj/ICP/nostra/spec/n-lib.schema.json)

### 4.1 Manifest Structure (n-lib.json)

```json
{
  "$nlib_version": "1.0.0",
  "id": "nostra.core",
  "name": "Nostra Core Library",
  "version": "0.1.0",
  "archetype": "curated",
  "license": "MIT",

  "capsules": ["types/*.kip", "seed/*.kip"],

  "exports": {
    "workflows": ["workflows/*.yaml"],
    "types": ["types/*.kip"],
    "tools": ["tools/*.yaml"]
  },

  "agent_tools": {
    "enabled": true,
    "manifest_path": "tools/manifest.yaml"
  },

  "external_connections": {
    "endpoints": [{
      "id": "get_data",
      "canister_id": "abc123-cai",
      "method": "getData"
    }]
  }
}
```

### 4.2 Key Fields

| Field | Purpose |
|:---|:---|
| `archetype` | Governance model (curated/personal/shared) |
| `capsules` | KIP type definitions |
| `exports` | Explicit workflow/type/tool exports |
| `agent_tools` | Agent capability configuration |
| `external_connections` | External canister/MCP/webhook connections |
| `fork_policy` | Fork permissions and attribution |

---

## 5. Agent Tools Library

Libraries can export tools that agents can "mount" for enhanced capabilities.

### 5.1 Tool Manifest

```yaml
# tools/manifest.yaml
id: "nostra:research/std/v1"
tools:
  - name: "search_graph"
    description: "Search the knowledge graph"
    parameters:
      query: { type: string, required: true }
  - name: "create_entity"
    description: "Create a new entity"
```

### 5.2 Runtime Integration

1. **Resolve**: Fetch library by ID
2. **Mount**: Register tools with LLM tool-use API
3. **Execute**: Route calls to library implementations

### 5.3 A2UI Integration

Tools can return A2UI surfaces for interactive responses:
- See [A2UI_TOOL_RESPONSE_STD.md](./A2UI_TOOL_RESPONSE_STD.md)
- **Constraint**: Tool surfaces MUST declare `surface_type = "execution"` per the NDL Surface Boundary Doctrine and communicate via Exchange I/O. They may not impersonate constitutional Tier 1 components.

---

## 6. External Connections Protocol

### 6.1 Connection Types

| Type | Use Case |
|:---|:---|
| **Canister Endpoints** | Cross-canister queries |
| **MCP Servers** | AI tool exposure |
| **Webhooks** | Off-chain integrations |

### 6.2 Developer Workflow

```
1. DEVELOP → Create your canister + tool manifest
2. PACKAGE → Create n-lib.json with external_connections
3. PUBLISH → Submit to registry
4. INTEGRATE → Users install, Cortex discovers endpoints
```

---

## 7. Registry & Governance

### 7.1 Registry Functions

- **Discover**: Browse/search available libraries
- **Install**: Add library to a Space
- **Upgrade**: Governance-approved updates for curated libraries
- **Fork**: Create personal copy of a library

### 7.2 Governance Workflows

See [GOVERNANCE_WORKFLOWS.md](./GOVERNANCE_WORKFLOWS.md):
- Library Upgrade Workflow
- Fork Library Workflow

---

## 8. Libraries as the "Brain"

| Brain Function | Library Implementation |
|:---|:---|
| **Memory** | Entity/Relationship storage |
| **Temporal History** | Chronicle events |
| **Pattern Recognition** | Cluster/Hub detection (planned) |
| **Learning** | Living growth with use |
| **Skills** | Agent tools library |

**Conclusion**: Libraries serve as Nostra's unified knowledge + capability substrate.

---

## 9. Cross-Research Integration

| Initiative | Integration |
|:---|:---|
| [021-kip-integration](../021-kip-integration/PLAN.md) | Libraries = KIP Domains |
| [013-nostra-workflow-engine](../013-nostra-workflow-engine/PLAN.md) | Workflows emit Chronicle events |
| [017-ai-agent-role-patterns](../017-ai-agent-role-patterns/PLAN.md) | Agents mount library tools |
| [026-nostra-schema-manager](../026-nostra-schema-manager/PLAN.md) | Schema governance UI |
| [040-nostra-schema-standards](../040-nostra-schema-standards/PLAN.md) | Standard schemas & lifecycle |
| [041-nostra-vector-store](../041-nostra-vector-store/PLAN.md) | Vector embedding storage |
| [003-nostra-library-economics](../003-nostra-library-economics/PLAN.md) | Monetization layer |

---

## 10. Related Documents

- [PLAN.md](./PLAN.md) — Implementation roadmap
- [REQUIREMENTS.md](./REQUIREMENTS.md) — Functional/non-functional requirements
- [DECISIONS.md](./DECISIONS.md) — Architectural decisions
- [PACKAGE_SCHEMA.md](./PACKAGE_SCHEMA.md) — n-lib.json field definitions
- [STUDY_NLIB_PROTOCOL.md](./STUDY_NLIB_PROTOCOL.md) — Protocol formalization study
- [GOVERNANCE_WORKFLOWS.md](./GOVERNANCE_WORKFLOWS.md) — Governance processes
