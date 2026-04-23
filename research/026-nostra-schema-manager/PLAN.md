---
status: active
portfolio_role: satellite
stewardship:
  layer: "Architectural"
  primary_steward: "Systems Steward"
  domain: "Knowledge Graphs"
---
# Plan: Nostra Schema Manager

**Status**: PROPOSED
**Version**: 1.0
**Context**: Implementation roadmap for the Schema Manager feature enabling user-driven schema evolution.

> [!NOTE]
> This plan implements concepts from [RESEARCH.md](./RESEARCH.md) and integrates with [013-nostra-workflow-engine](../013-nostra-workflow-engine/workflow-builder.md).

---

## Overview

Build a **Schema Manager** feature that allows users to:
1. **View** their Space/Library schema (types, predicates, domains)
2. **Manage** schema definitions with appropriate governance
3. **Steer** schema evolution as AI extraction and contributions add new types

---

## Resolved Decisions

> [!NOTE]
> **Architectural Decisions** (Resolved 2026-01-17):
> 1. ✅ **Dual-Mode UI**: Embedded in Workflow Builder for initialization; separate view for maintenance/management
> 2. ✅ **Log-Only for Personal**: No approval workflows, but ChronicleEvents logged for history/timeline
> 3. ✅ **MVP = View-First**: Phase 1 priority is Schema Explorer (visibility)
> 4. ✅ **Intelligent Guidance**: Personal Libraries include contextual hints and onboarding

---

## Phase 0: Foundation (Prerequisites)

### 0.1 Verify KIP DESCRIBE Commands

**Objective**: Ensure KIP can return schema metadata.

**Tasks**:
- [ ] Test `DESCRIBE CONCEPT TYPES` returns all `$ConceptType` definitions
- [ ] Test `DESCRIBE PROPOSITION TYPES` returns all `$PropositionType` definitions
- [ ] Add `instance_count` aggregation to DESCRIBE output
- [ ] Add `usage_count` for predicates (relationship count)

**Verification**:
```bash
dfx canister call nostra_backend execute_kip '("DESCRIBE CONCEPT TYPES IN DOMAIN \"NostraCore\"", vec {})'
# Expected: List of types with counts
```

---

### 0.2 Create Schema Status Field

#### [MODIFY] [nostra/libraries/nostra-core/types/Contribution.kip](file:///Users/xaoj/ICP/nostra/libraries/nostra-core/types/Contribution.kip)

Add `schema_status` to `$ConceptType` definitions:

```prolog
SET ATTRIBUTES {
  // ... existing attributes ...
  schema_status: {
    type: "string",
    enum: ["draft", "proposed", "approved", "archived"],
    default: "approved"
  }
}
```

**Tasks**:
- [x] Define `schema_status` attribute pattern — implemented as `SchemaStatus` variant in `types.mo`
- [x] Update existing types with `schema_status: "approved"` — all 18 system schemas bootstrap as `#approved`
- [x] Implement status filtering in DESCRIBE — `getSchemasByStatus` query added to Schema Registry

---

## Phase 1: Schema Explorer (View)

**Focus**: Read-only visibility into schema structure.

### 1.1 Backend API

#### [NEW] [nostra/backend/modules/schema.mo](file:///Users/xaoj/ICP/nostra/backend/modules/schema.mo)

```motoko
module {
    public type SchemaTypeInfo = {
        id : Text;
        name : Text;
        domain : Text;
        description : Text;
        instanceCount : Nat;
        status : Text;
        attributes : [(Text, Text)];
    };

    public type PredicateInfo = {
        name : Text;
        usageCount : Nat;
        sourceTypes : [Text];
        targetTypes : [Text];
    };

    public func getSchemaTypes(domain : ?Text) : async [SchemaTypeInfo];
    public func getPredicates() : async [PredicateInfo];
    public func getSchemaStats() : async SchemaStats;
};
```

**Tasks**:
- [ ] Create `schema.mo` module with type definitions
- [ ] Implement `getSchemaTypes()` query
- [ ] Implement `getPredicates()` query
- [ ] Implement `getSchemaStats()` for dashboard metrics

---

### 1.2 Frontend: Schema Explorer Component

#### [NEW] [nostra/frontend/src/components/schema_explorer.rs](file:///Users/xaoj/ICP/nostra/frontend/src/components/schema_explorer.rs)

**UI Layout**:
```
┌─────────────────────────────────────────────────────────────────────────┐
│ SCHEMA EXPLORER                                            [🔍 Search] │
├───────────────────────┬─────────────────────────────────────────────────┤
│ DOMAINS               │ TYPE DETAILS                                    │
│ ├── NostraCore (12)   │ ┌─────────────────────────────────────────────┐ │
│ │   ├── Idea (42)     │ │ 💡 Idea                        [approved]  │ │
│ │   ├── Project (18)  │ │ Domain: NostraCore                          │ │
│ │   ├── Decision (7)  │ │ Instances: 42                               │ │
│ │   └── ...           │ │                                             │ │
│ └── ICPKnowledge (3)  │ │ Attributes:                                 │ │
│     ├── Protocol (5)  │ │  - title: string (required)                 │ │
│     ├── Component (8) │ │  - description: string                      │ │
│     └── Infra (3)     │ │  - tags: array[string]                      │ │
│                       │ │                                             │ │
├───────────────────────┤ │ Predicates (outgoing):                      │ │
│ PREDICATES            │ │  - relates_to (15 usages)                   │ │
│ ├── relates_to (120)  │ │  - spawns (3 usages)                        │ │
│ ├── implements (45)   │ └─────────────────────────────────────────────┘ │
│ ├── spawns (28)       │                                                 │
│ └── belongs_to (89)   │                                                 │
└───────────────────────┴─────────────────────────────────────────────────┘
```

**Tasks**:
- [ ] Create `SchemaExplorer` component with domain tree
- [ ] Implement type detail view (attributes, predicates, stats)
- [ ] Add search functionality
- [ ] Style with existing design system

---

## Phase 2: Schema Governance (Manage)

**Focus**: Workflow-based approval for schema changes.

### 2.1 Schema Workflow Templates

#### [NEW] [nostra/libraries/nostra-core/workflows/](file:///Users/xaoj/ICP/nostra/libraries/nostra-core/workflows/)

Create workflow definitions for schema operations:

```yaml
# schema-type-approval.yaml
id: "schema-type-approval"
intention: "Approve a new entity type for the schema"
steps:
  - id: "validate"
    type: "AsyncExternalOp"
    agent: "Architect"
    action: "validate_schema_type"

  - id: "review"
    type: "UserTask"
    role: "SchemaStew"
    action: "approve_or_reject"

  - id: "install"
    type: "SystemOp"
    condition: "review.approved == true"
    action: "execute_kip_mutation"
```

**Tasks**:
- [ ] Create `schema-type-approval.yaml` workflow
- [ ] Create `schema-archive-proposal.yaml` workflow
- [ ] Create `schema-merge-workflow.yaml` workflow
- [ ] Register workflows in `n-lib.json`

---

### 2.2 Type Editor Component

#### [NEW] [nostra/frontend/src/components/type_editor.rs](file:///Users/xaoj/ICP/nostra/frontend/src/components/type_editor.rs)

**Capabilities**:
- Create new `$ConceptType` via form UI
- Edit existing type attributes
- Preview KIP capsule output
- Submit to governance workflow

**Tasks**:
- [ ] Create `TypeEditor` form component
- [ ] Implement attribute builder (add/remove fields)
- [ ] Implement KIP preview panel
- [ ] Connect "Submit" to workflow trigger

---

## Phase 3: Evolution Dashboard (Steer)

**Focus**: Monitor schema changes and hygiene alerts.

### 3.1 Dashboard Component

#### [NEW] [nostra/frontend/src/components/evolution_dashboard.rs](file:///Users/xaoj/ICP/nostra/frontend/src/components/evolution_dashboard.rs)

**UI Sections**:
1. **Pending Proposals**: Types awaiting approval
2. **AI Suggestions**: Extraction-proposed types
3. **Hygiene Alerts**: Orphans, duplicates, low-confidence
4. **Growth Metrics**: Sparklines, counts over time

**Tasks**:
- [ ] Create `EvolutionDashboard` component
- [ ] Implement proposal list with approve/reject actions
- [ ] Implement hygiene alert cards
- [ ] Add growth metric visualizations

---

### 3.2 AI Extraction Integration

#### [MODIFY] [nostra/worker/src/skills/oneke_extractor.rs](file:///Users/xaoj/ICP/nostra/worker/src/skills/oneke_extractor.rs)

When extraction identifies a new type:

```rust
if !schema_contains_type(&proposed_type) {
    // Create draft type proposal
    let proposal = SchemaProposal {
        type_name: proposed_type.name,
        attributes: inferred_attributes,
        source: "ai-extraction",
        confidence: extraction_confidence,
    };

    // Trigger governance workflow
    workflow_client.start("schema-type-approval", proposal).await?;
}
```

**Tasks**:
- [ ] Add schema lookup before creating new types
- [ ] Implement `SchemaProposal` struct
- [ ] Connect to workflow trigger
- [ ] Add confidence threshold for auto-proposals

---

### 3.3 Gardener Integration

#### [MODIFY] [nostra/worker/src/skills/gardener.rs](file:///Users/xaoj/ICP/nostra/worker/src/skills/gardener.rs)

Add schema hygiene tasks:

```rust
pub async fn schema_hygiene(&self) -> Result<Vec<HygieneAlert>> {
    let mut alerts = vec![];

    // Find types with 0 instances
    let orphan_types = self.find_orphan_types().await?;
    for t in orphan_types {
        alerts.push(HygieneAlert::UnusedType(t));
    }

    // Find potential duplicates
    let duplicates = self.find_similar_types().await?;
    for (t1, t2, similarity) in duplicates {
        if similarity > 0.8 {
            alerts.push(HygieneAlert::PossibleDuplicate(t1, t2));
        }
    }

    Ok(alerts)
}
```

**Tasks**:
- [ ] Implement `find_orphan_types()`
- [ ] Implement `find_similar_types()` with embedding similarity
- [ ] Create `SleepTask` for each hygiene alert
- [ ] Connect to Evolution Dashboard display

---

## Phase 4: Workflow Builder Integration

**Focus**: Schema Manager as a core feature of the Workflow Builder.

### 4.1 Schema Node in Canvas

Add schema operations as draggable nodes:

| Node Type | Action |
|:---|:---|
| `CreateType` | Propose new `$ConceptType` |
| `ValidateSchema` | Run Architect validation |
| `ApplySchema` | Execute KIP mutation |
| `ArchiveType` | Mark type as archived |

**Tasks**:
- [ ] Add schema nodes to workflow builder palette
- [ ] Implement node configuration panels
- [ ] Connect to schema.mo backend methods

---

### 4.2 Schema Context in Trace Mode

When viewing a workflow instance:
- Show which types are being created/modified
- Highlight schema dependencies
- Display validation results

**Tasks**:
- [ ] Add schema context panel to trace view
- [ ] Show affected types in workflow visualization

---

## Verification Plan

### Automated Tests

```bash
# Unit: Schema module queries
dfx canister call nostra_backend getSchemaTypes '(opt "NostraCore")'

# Integration: Type proposal workflow
cargo test --package nostra-worker schema_proposal_workflow

# E2E: Create type via UI, verify appears in schema
# (Browser test using browser_subagent)
```

### Manual Verification

**Test Case 1: Schema Explorer**
1. Navigate to Cortex → Schema (new menu item)
2. Verify domain tree shows NostraCore, ICPKnowledge
3. Click "Idea" type → verify attributes displayed
4. Search for "Protocol" → verify results

**Test Case 2: Type Proposal**
1. Click "Propose New Type" in Schema Explorer
2. Enter: Name="Tutorial", Domain="NostraCore"
3. Add attributes: title, difficulty, content
4. Click "Submit for Review"
5. Verify workflow instance created
6. Approve in workflow UI
7. Verify type appears in Schema Explorer with status="approved"

**Test Case 3: Hygiene Alerts**
1. Create type with 0 instances (or wait for test data)
2. Run Gardener hygiene cycle
3. Verify alert appears in Evolution Dashboard
4. Click "Archive" → confirm → verify type status changes

---

## Success Metrics

| Metric | Target | Measurement |
|:---|:---|:---|
| Schema visibility | 100% types browsable | All types appear in explorer |
| Proposal workflow | <5 min end-to-end | Workflow logs |
| Hygiene detection | 95%+ orphan detection | Compare auto vs manual |
| User satisfaction | 4/5 usefulness | Survey |

---

## Cross-Research Dependencies

| Phase | Depends On |
|:---|:---|
| Phase 0 | 021-kip-integration (DESCRIBE commands) |
| Phase 1 | 021-kip-integration (KQL queries) |
| Phase 2 | 013-nostra-workflow-engine (templates) |
| Phase 3 | 025-oneke-integration, 017-ai-agent-role-patterns |
| Phase 4 | 013-workflow-builder (canvas integration) |

---

## Next Immediate Actions

### Pending
- [ ] User approval of research direction
- [ ] Confirm Phase 1 priority (View-first MVP)
- [ ] Create REQUIREMENTS.md with detailed specs

## Alignment Addendum (Constitution + System Standards)

- Labs Constitution: Default to Production patterns unless explicitly labeled as Labs; experiments remain fork-safe and documented.
- Knowledge Integrity & Memory: Preserve lineage, log decisions, and avoid rewriting history; summaries are additive, not replacements.
- Spaces Constitution: All authority and data are space-scoped; cross-space effects are explicit links, not merges.
- Stewardship & Roles: Identify accountable roles per change; unclear authority defaults to recommendation-only.
- Contribution Lifecycle: Renames, merges, archives, and scope changes require explicit rationale and approval.
- Agent Behavior & Authority: Agents operate in observe/recommend/simulate unless execution is explicitly approved.
- Security & Privacy: Least authority, explicit consent, and scoped access; minimize sensitive data exposure.
- Governance & Escalation: Disputes and irreversible actions follow escalation pathways and steward review.
- UI/UX Manifesto: Interfaces must surface provenance, time, and agency; avoid dark patterns.
- Modularity: Strict interfaces, no hardcoded canister IDs, and clean boundary contracts.
- Composability: Actions are workflow-compatible and emit standard events.
- Data Confidence & Integrity: Confidence/reliability metadata is required where applicable.
- Portability: Data must be exportable and WASM-safe; avoid OS-specific dependencies in core logic.
- Durable Execution: State is persisted via stable memory; workflows are replayable.
- Visibility Decoupling: Index/search are async from source of truth.
- Outbox Pattern: External calls are queued with idempotency and retry semantics.
- Verification: Each initiative includes verification steps and records results.
