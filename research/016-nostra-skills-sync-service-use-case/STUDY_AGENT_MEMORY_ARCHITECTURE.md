# Study 16.3: Unified Agent Memory with Log References

**Date**: 2026-01-18
**Status**: DRAFT
**Dependencies**: 018 (Library Registry), 019 (Log Registry), 024 (Agent Tools Library)

---

## Executive Summary

> [!WARNING]
> **SUPERSEDED**
> This architectural proposal (`ObservationEntity` mapped to the Nostra Graph) has been superseded by Initiative `121-cortex-memory-fs`. Agent episodic traces and working memory are now routed to the Git-backed local filesystem rather than being forced into on-chain graph schemas. This document remains for historical context.

This study originally proposed storing agent memories (observations) as **Library Entities** with references to raw logs in the **Log Registry**. This creates a two-tier observability system: fast operational debugging + long-term knowledge persistence.

---

## 1. Architecture

### 1.1 Two-Tier Model

```
┌─────────────────────────────────────┐
│  LIBRARY (Agent Memory)             │  Long-term knowledge
│  Entities: Observations (typed)     │
│  References: logRefs[]              │───┐
└─────────────────────────────────────┘   │
                                          │ References
┌─────────────────────────────────────┐   │
│  LOG REGISTRY (Raw Events)          │◄──┘
│  Entries: ERROR/WARN/INFO logs      │  Short-term operational
└─────────────────────────────────────┘
```

### 1.2 Observation Entity Schema

```typescript
interface ObservationEntity {
  id: string;
  type: "discovery" | "decision" | "problem" | "solution" |
        "pattern" | "warning" | "success" | "refactor" | "bugfix" | "feature";
  summary: string;
  content: string;

  // Log References
  logRefs?: string[];  // Log Registry IDs

  // Knowledge Graph Links
  relationships: {
    relates_to?: string[];
    led_to?: string[];
    solved_by?: string[];
    blocked_by?: string[];
  };

  metadata: {
    confidence: number;
    files?: string[];
    workflowInstanceId?: string;
    agentId?: string;
    timestamp: number;
  };
}
```

---

## 2. Labs Feature Organization

### 2.1 Recommendation: Separate Feature with Dependency

| Approach | Pros | Cons |
|:---------|:-----|:-----|
| **A: Bundle into Agent Tools Library** | Single toggle | Bloated; confuses tools + memory |
| **B: Separate feature, requires Agent Tools** ✅ | Clear separation; opt-in | Two toggles |

**Recommended: Approach B** - Create `labs:agent-memory` that depends on `labs:agent-libraries`.

### 2.2 Labs Feature Hierarchy

```
Labs Features:
├── labs:agent-libraries     (018 - Agent Tools Library)
│   └── Mount tools from libraries
│
└── labs:agent-memory        (016 - NEW, requires agent-libraries)
    ├── Store observations in Personal Library
    ├── Reference logs via logRefs
    └── Mount memory query tools
```

### 2.3 Feature Flag Definition

```rust
// Labs flags in backend
LabsFeature::AgentLibraries => "agent-libraries"  // Must be enabled first
LabsFeature::AgentMemory => "agent-memory"        // Depends on above
```

```typescript
// Frontend Labs toggle
{
  id: "agent-memory",
  name: "Agent Memory Library",
  description: "Store agent observations in a Personal Library with log references",
  requires: ["agent-libraries"],  // Dependency
  status: "experimental"
}
```

---

## 3. Integration Pattern

### 3.1 How It Works

1. **Agent executes action** → Log emitted to Log Registry
2. **PostToolUse hook** → Observation created in Personal Library
3. **Observation includes** → `logRefs` pointing to relevant logs
4. **SessionStart** → Memory tools mounted + recent observations injected

### 3.2 Memory Tools (Mounted via Agent Tools Library)

```yaml
# Library: agent-memory-tools
tools:
  - name: "recall"
    description: "Search past observations by query"

  - name: "get_context"
    description: "Get observations related to current problem"

  - name: "get_logs"
    description: "Drill into raw logs for an observation"
```

---

## 4. Benefits Summary

| Capability | Value |
|:-----------|:------|
| **Semantic Search** | Find by observation type, not just text |
| **Log Drill-down** | Click observation → See raw error stack |
| **Knowledge Graph** | Observations linked by relationships |
| **Temporal Queries** | "What did I know when I made that decision?" |
| **Issue Management** | Auto-attach logs to problem observations |
| **Agent Onboarding** | Fork memory library → Transfer knowledge |

---

## 5. Implementation Phases

### Phase 1: Foundation
- [ ] Define `ObservationEntity` schema in backend
- [ ] Add `logRefs` field support
- [ ] Create Personal Library template for agent memory

### Phase 2: Labs Integration
- [ ] Add `labs:agent-memory` feature flag
- [ ] Implement dependency check on `labs:agent-libraries`
- [ ] Create frontend Labs toggle

### Phase 3: Memory Tools
- [ ] Create `agent-memory-tools` library
- [ ] Implement `recall`, `get_context`, `get_logs` tools
- [ ] Test tool mounting via Agent Tools Library

---

## 6. Related Research

| Initiative | Relationship |
|:-----------|:-------------|
| [016](./RESEARCH.md) | Skills Sync Service (parent) |
| [018](../018-nostra-library-registry/RESEARCH.md) | Library storage substrate |
| [019](../019-nostra-log-registry/RESEARCH.md) | Raw log storage |
| [024](../024-agent-tools-library/RESEARCH.md) | Tool mounting mechanism |
| [034](../034-nostra-labs/RESEARCH.md) | Labs feature framework |
