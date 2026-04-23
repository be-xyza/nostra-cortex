---
id: 018
name: nostra-library-registry
title: 'Requirements: Living Libraries'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Living Libraries

> [!NOTE]
> Requirements derived from [RESEARCH.md](./RESEARCH.md) and informing [PLAN.md](./PLAN.md).

---

## Functional Requirements

### FR-1: Library Archetypes

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | System SHALL support three library archetypes: Curated, Personal, Shared | P0 |
| FR-1.2 | Curated Libraries SHALL be read-only for non-publishers | P0 |
| FR-1.3 | Personal Libraries SHALL be writable only by their owner | P0 |
| FR-1.4 | Shared Libraries SHALL support multi-contributor access via governance | P1 |
| FR-1.5 | Users SHALL be able to fork a Curated Library into a Personal one | P1 |

### FR-2: Living System Properties

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | Libraries SHALL track `createdAt` and `lastActivityAt` timestamps | P0 |
| FR-2.2 | Libraries SHALL expose `entityCount` and `relationshipCount` metrics | P0 |
| FR-2.3 | System SHALL automatically update `lastActivityAt` on any mutation | P0 |
| FR-2.4 | Libraries SHALL calculate and expose `growthRate` (entities per period) | P1 |

### FR-3: Chronicle (Event Log)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | System SHALL log events for: EntityCreated, EntityUpdated, EntityArchived | P0 |
| FR-3.2 | System SHALL log events for: RelationshipFormed, RelationshipRemoved | P0 |
| FR-3.3 | System SHALL log events for: WorkflowStarted, WorkflowCompleted | P1 |
| FR-3.4 | System SHALL log events for: LibrarySeeded, LibraryForked, LibraryMerged | P1 |
| FR-3.5 | Chronicle events SHALL include: timestamp, actor, affected entities, metadata | P0 |
| FR-3.6 | System SHALL support querying Chronicle by time range | P0 |
| FR-3.7 | System SHALL support reconstructing graph state at a historical timestamp | P1 |

### FR-4: Personal Library Lifecycle

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | Authenticated user SHALL be able to create a Personal Library | P0 |
| FR-4.2 | User SHALL be able to add entities to their Personal Library | P0 |
| FR-4.3 | User SHALL be able to create relationships within their Personal Library | P0 |
| FR-4.4 | User SHALL be able to link their entities to Curated Library entities | P1 |
| FR-4.5 | User's actions SHALL automatically generate Chronicle events | P0 |

### FR-5: Visual Storytelling

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1 | UI SHALL display a Timeline Scrubber for temporal navigation | P1 |
| FR-5.2 | Graph view SHALL update to reflect state at selected timestamp | P1 |
| FR-5.3 | UI SHALL display health indicators (activity, growth, connectivity) | P1 |
| FR-5.4 | System SHALL detect and highlight graph patterns (clusters, hubs, orphans) | P2 |
| FR-5.5 | System SHOULD suggest names for detected clusters using AI | P2 |

---

## Non-Functional Requirements

### NFR-1: Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1.1 | Chronicle event logging SHALL complete in < 100ms | P0 |
| NFR-1.2 | Temporal graph query SHALL return in < 2s for graphs up to 10k nodes | P1 |
| NFR-1.3 | Timeline Scrubber SHALL update graph in < 500ms | P1 |

### NFR-2: Storage

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-2.1 | Chronicle MAY implement retention policies for high-volume events | P2 |
| NFR-2.2 | System SHOULD create periodic snapshots to optimize temporal queries | P1 |
| NFR-2.3 | Personal Libraries SHALL have configurable storage quotas | P2 |

### NFR-3: Security

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-3.1 | Personal Library access SHALL be restricted to owner principal | P0 |
| NFR-3.2 | Chronicle events SHALL NOT be deletable (append-only) | P0 |
| NFR-3.3 | Fork operation SHALL create independent chronicle (no cross-contamination) | P1 |

---

## Data Requirements

### DR-1: Library Schema Extensions

```motoko
public type LibraryArchetype = {
    #Curated;
    #Personal;
    #Shared;
};

public type Library = {
    // Existing fields...
    archetype : LibraryArchetype;
    owner : Principal;
    createdAt : Int;
    lastActivityAt : Int;
    entityCount : Nat;
    relationshipCount : Nat;
    chronicleEnabled : Bool;
    chronicleRef : ?Text;
};
```

### DR-2: Chronicle Event Schema

```motoko
public type ChronicleEventType = {
    #EntityCreated;
    #EntityUpdated;
    #EntityArchived;
    #RelationshipFormed;
    #RelationshipRemoved;
    #WorkflowStarted;
    #WorkflowCompleted;
    #LibrarySeeded;
    #LibraryForked;
    #LibraryMerged;
};

public type ChronicleEvent = {
    id : Text;
    libraryId : Text;
    timestamp : Int;
    eventType : ChronicleEventType;
    actorPrincipal : Principal;
    affectedEntities : [Text];
    metadata : [(Text, Text)];
};
```

### DR-3: Graph Snapshot Schema

```motoko
public type EntitySummary = {
    id : Text;
    label : Text;
    entityType : Text;
};

public type RelationshipSummary = {
    sourceId : Text;
    targetId : Text;
    relationType : Text;
};

public type GraphSnapshot = {
    asOf : Int;
    libraryId : Text;
    entities : [EntitySummary];
    relationships : [RelationshipSummary];
};
```

---

## API Requirements

### AR-1: Library Management

```candid
service : {
    // Create a Personal Library
    createPersonalLibrary : (name: text, description: text) -> (result Library Error);

    // Get caller's Personal Library
    getMyLibrary : () -> (opt Library) query;

    // Fork a library
    forkLibrary : (sourceId: text, newName: text) -> (result Library Error);

    // Get library stats
    getLibraryStats : (libraryId: text) -> (LibraryStats) query;
};
```

### AR-2: Chronicle Access

```candid
service : {
    // Get chronicle events
    getChronicle : (libraryId: text, since: int, until: int) -> (vec ChronicleEvent) query;

    // Get graph at point in time
    getGraphAtTime : (libraryId: text, timestamp: int) -> (GraphSnapshot) query;
};
```

---

## Acceptance Criteria

### AC-1: Personal Library Creation
- [ ] User can create a Personal Library with name and description
- [ ] Library is assigned `#Personal` archetype
- [ ] Library owner is set to caller's principal
- [ ] `createdAt` and `lastActivityAt` are set to current time
- [ ] Chronicle is initialized with `#LibrarySeeded` event

### AC-2: Entity Contribution
- [ ] User can add an entity to their Personal Library
- [ ] `entityCount` is incremented
- [ ] `lastActivityAt` is updated
- [ ] Chronicle records `#EntityCreated` event

### AC-3: Temporal Query
- [ ] `getGraphAtTime` returns accurate snapshot for past timestamp
- [ ] Snapshot includes only entities that existed at that time
- [ ] Relationships reflect state at that time

### AC-4: Timeline Scrubber
- [ ] UI displays timeline with event markers
- [ ] Dragging slider updates graph view
- [ ] Play button animates graph evolution
