---
id: '026'
name: nostra-schema-manager
title: 'Requirements: Nostra Schema Manager'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Nostra Schema Manager

**Version**: 1.0
**Date**: 2026-01-17
**Status**: DRAFT

---

## Functional Requirements

### FR-1: Schema Visibility

| ID | Requirement | Priority |
|:---|:---|:---|
| FR-1.1 | System shall display all `$ConceptType` definitions in a browsable tree | MUST |
| FR-1.2 | System shall display all `$PropositionType` definitions with usage counts | MUST |
| FR-1.3 | System shall show instance count for each entity type | MUST |
| FR-1.4 | System shall group types by Domain | MUST |
| FR-1.5 | System shall provide search across type names and descriptions | SHOULD |
| FR-1.6 | System shall show type attributes with data types and constraints | MUST |

---

### FR-2: Schema Management

| ID | Requirement | Priority |
|:---|:---|:---|
| FR-2.1 | User shall be able to create new entity type definitions | MUST |
| FR-2.2 | User shall be able to edit type attributes (add/remove/modify) | MUST |
| FR-2.3 | User shall be able to define predicate constraints (valid source/target) | SHOULD |
| FR-2.4 | System shall enforce governance workflow for Curated Library changes | MUST |
| FR-2.5 | Personal Library changes shall apply immediately without workflow | MUST |
| FR-2.6 | Shared Library changes shall require proposal + community vote | SHOULD |
| FR-2.7 | User shall be able to archive (soft-delete) unused types | MUST |

---

### FR-3: Schema Evolution

| ID | Requirement | Priority |
|:---|:---|:---|
| FR-3.1 | System shall display pending schema proposals | MUST |
| FR-3.2 | System shall display AI-suggested types from extraction | MUST |
| FR-3.3 | System shall display hygiene alerts (orphans, duplicates) | SHOULD |
| FR-3.4 | System shall track schema changes via ChronicleEvents | MUST |
| FR-3.5 | User shall be able to view schema history for any type | SHOULD |
| FR-3.6 | System shall show growth metrics (types over time) | COULD |

---

### FR-4: AI Integration

| ID | Requirement | Priority |
|:---|:---|:---|
| FR-4.1 | AI extraction shall check existing schema before creating types | MUST |
| FR-4.2 | AI-proposed types shall trigger governance workflow | MUST |
| FR-4.3 | AI proposals shall include confidence score | SHOULD |
| FR-4.4 | Gardener agent shall identify unused types | SHOULD |
| FR-4.5 | Gardener agent shall identify potential duplicates | COULD |

---

### FR-5: Workflow Integration

| ID | Requirement | Priority |
|:---|:---|:---|
| FR-5.1 | Schema type approval shall use `013` workflow primitives | MUST |
| FR-5.2 | Schema archival shall use governance workflow | SHOULD |
| FR-5.3 | Schema merge shall use governance workflow | COULD |
| FR-5.4 | Workflow Builder shall include schema-related nodes | COULD |

---

## Non-Functional Requirements

### NFR-1: Performance

| ID | Requirement | Target |
|:---|:---|:---|
| NFR-1.1 | Schema Explorer load time | < 2 seconds |
| NFR-1.2 | Type detail query | < 500ms |
| NFR-1.3 | Search response time | < 1 second |

---

### NFR-2: Usability

| ID | Requirement |
|:---|:---|
| NFR-2.1 | Non-technical users shall understand type definitions without code |
| NFR-2.2 | Type editor shall provide inline validation feedback |
| NFR-2.3 | Evolution Dashboard shall use visual indicators (badges, colors) |

---

### NFR-3: Security

| ID | Requirement |
|:---|:---|
| NFR-3.1 | Only authorized users can modify Curated Library schema |
| NFR-3.2 | Personal Library schema only modifiable by owner |
| NFR-3.3 | AI proposals require human approval before activation |

---

## Data Requirements

### DR-1: Schema Type Metadata

```motoko
type SchemaTypeInfo = {
    id : Text;                    // Unique identifier
    name : Text;                  // Human-readable name
    domain : Text;                // Parent domain
    description : Text;           // Purpose/usage
    status : SchemaStatus;        // draft | proposed | approved | archived
    instanceCount : Nat;          // Number of entities of this type
    attributes : [AttributeDef];  // Field definitions
    createdAt : Int;              // Timestamp
    updatedAt : Int;              // Timestamp
    createdBy : Principal;        // Author
    version : Nat;                // Schema version
};
```

### DR-2: Chronicle Event Types

| Event | Payload |
|:---|:---|
| `SchemaTypeCreated` | `{typeId, name, domain, author}` |
| `SchemaTypeModified` | `{typeId, changes, author}` |
| `SchemaTypeArchived` | `{typeId, reason, author}` |
| `SchemaTypeMerged` | `{sourceIds, targetId, author}` |
| `SchemaProposalCreated` | `{proposalId, typeDefinition, author}` |
| `SchemaProposalApproved` | `{proposalId, approver}` |
| `SchemaProposalRejected` | `{proposalId, reason, rejector}` |

---

## Interface Requirements

### IR-1: Backend API

```candid
service : {
    // Query
    getSchemaTypes : (domain: opt text) -> (vec SchemaTypeInfo) query;
    getPredicates : () -> (vec PredicateInfo) query;
    getSchemaHistory : (typeId: text) -> (vec ChronicleEvent) query;
    getSchemaStats : () -> (SchemaStats) query;

    // Mutation (governance-aware)
    proposeSchemaType : (definition: text) -> (variant { #ok: text; #err: text });
    approveSchemaProposal : (proposalId: text) -> (variant { #ok; #err: text });
    archiveSchemaType : (typeId: text, reason: text) -> (variant { #ok; #err: text });
};
```

### IR-2: Frontend Components

| Component | Inputs | Outputs |
|:---|:---|:---|
| `SchemaExplorer` | None | Renders type tree + detail view |
| `TypeEditor` | `?typeId` (edit mode) | `SchemaTypeDefinition` on submit |
| `EvolutionDashboard` | None | Renders proposals, alerts, metrics |
| `SchemaTimeline` | `typeId` | Renders ChronicleEvents |

---

## Acceptance Criteria

### AC-1: Schema Explorer

**Given** a user navigates to Schema Manager
**When** the page loads
**Then** all Domains and their types are displayed in a tree structure
**And** clicking a type shows its attributes and instance count

### AC-2: Type Proposal

**Given** a user creates a new type definition
**When** the user clicks "Submit for Review"
**Then** a governance workflow instance is created
**And** the proposal appears in the Evolution Dashboard
**And** a ChronicleEvent is logged

### AC-3: AI Extraction

**Given** AI extraction identifies a new entity type
**When** the type does not exist in the schema
**Then** a governance proposal is automatically created
**And** the proposal includes the confidence score
**And** the proposal appears in "AI Suggestions" section

### AC-4: Hygiene Alerts

**Given** a type has 0 instances for > 30 days
**When** the Gardener runs hygiene check
**Then** an "Unused Type" alert appears in Evolution Dashboard
**And** the user can click "Archive" to trigger archival workflow
