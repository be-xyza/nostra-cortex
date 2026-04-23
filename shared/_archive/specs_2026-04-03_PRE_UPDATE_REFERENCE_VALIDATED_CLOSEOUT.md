# Shared Ecosystem Specifications

**Version**: 0.2.0 (Neutrality Refactor)
**Context**: Shared interfaces for [Nostra](../nostra/spec.md).
**Standard**: Compliant with [TECHNOLOGY_NEUTRALITY](./standards/TECHNOLOGY_NEUTRALITY.md) and [ACCESSIBILITY](./standards/ACCESSIBILITY.md).

---

## 1. Constitutional vs Implementational

| System | Constitutional Invariant (Meaning) | Implementational Adapter (Execution) |
| :--- | :--- | :--- |
| **Identity** | `ActorID` (UUID/DID) | `ICP Principal`, `WebAuthn`, `Eth Address` |
| **Governance** | `Proposal`, `Rule` | `Governance Host Canister`, `Snapshot Strategy` |
| **Execution** | `Mutation`, `Query` | `Internet Computer`, `EVM`, `Serverless` |
| **Graph** | `Contribution`, `Relation` | `Motoko Map`, `Neo4j`, `D3 Projection` |
| **Interaction** | `Universal Access` | `A2UI Renderer`, `Screen Reader`, `Haptic Interface` |

---

## 2. User Identity Standard (Constitutional)

**Invariant**: Identity is abstract. Credentials are keys fitting into the identity lock.

### 2.1 Actor Profile
```motoko
type ActorID = Text; // UUID or DID (e.g. "did:nostra:123...")

type UserProfile = {
    id: ActorID;              // The Sovereign Identity
    handle: Text;             // Global handle (e.g. "alice")
    displayName: Text;
    avatarUrl: ?Text;
    bio: ?Text;

    // Credentials (Adapters)
    credentials: [Credential];

    // Metadata
    createdAt: Int;
    updatedAt: Int;
};

type Credential = {
    #ICP : Principal;
    #EVM : Text; // 0x...
    #WebAuthn : Blob;
};
```

### 2.2 Host Interface (ICP Adapter)
**Host**: `kg-registry` canister.

```candid
service : {
    // Identity - Maps Credentials to ActorIDs
    registerUser : (handle: text, displayName: text) -> (result UserProfile Error);
    authenticate : (credential: blob) -> (result ActorID Error);

    // Profile Management
    getProfile : (id: text) -> (opt UserProfile) query; // By ActorID
    getProfileByCredential : (credential: principal) -> (opt UserProfile) query; // Convenience
};
```

---

## 3. Governance Standard (Constitutional)

**Invariant**: Governance manages *Actions* on *Targets*. The Host runs the vote.

### 3.1 Concepts
- **Target**: A resolvable resource that accepts a payload (e.g. `ic://canister/method` or `http://webhook`).
- **Proposal**: A declarative request to mutate the Target.
- **Workflow**: A sequence of steps (Adapter: Temporal/Step Functions) required to ratify.

### 3.2 Host Interface (ICP Adapter)
**Host**: `governance-host` canister.

```candid
type ActionTarget = record {
    protocol: text; // "ic", "http", "evm"
    address: text;  // "canister-id", "url", "contract-addr"
    method: text;
    payload: blob;
};

service : {
    submitProposal : (title: text, desc: text, action: ActionTarget) -> (ProposalId);
    // ... standard voting methods
};
```

---

## 4. Discovery & Event Standard (Canonical)

**Host**: `discovery-index` canister (Adapter).

### 4.1 Resource Reference (`ResourceRef`)
Universal URI format. Protocol indicates the Adapter.

- `ResourceRef` is a canonical identifier for *meaning* and *authority*, not necessarily a navigable route.
- Hosts should only auto-route explicit navigation intents using `nostra://nav/<route>` (e.g. `nostra://nav/system`).

- `nostra://profile/alice` (Canonical)
- `nostra://capability?id=route:%2Fsystem` (Canonical capability identifier; not auto-routed)
- `nostra://contribution?id=118` (Canonical contribution identifier; not auto-routed)
- `nostra://nav/system` (Canonical navigation intent; safe to auto-route)
- `ic://<canister>/profile/alice` (Sovereign Location)
- `ipfs://<cid>` (Content Address)

### 4.2 Global Event Log
Events define the history. They must be replayable on any substrate.

```motoko
type EventSource = {
    #System: Text;  // "System Monitor"
    #Actor: ActorID; // "did:nostra:alice"
    #Agent: ActorID; // "did:nostra:agent-smith"
};

type GlobalEvent = {
    id: Text; // UUID
    source: EventSource;
    type: EventType;
    resource: Text; // ResourceRef
    payload: Blob;
    timestamp: Int;
};
```

### 4.3 ACP Automation Event Extension
Worker-native ACP automation uses the canonical `GlobalEvent` envelope with standardized `type` values:

1. `AcpSloWindowSampleCaptured`
2. `AcpSloEvaluationComputed`
3. `AcpPromotionGateDecisionRequested`
4. `AgentExecutionLifecycle`

Minimum payload keys for this stream:
- `workflow_id`
- `event_type`
- `timestamp`
- `acp.slo.result` (if available)
- `acp.slo.reason` (if available)
- `confidence_score` (evidence publication stage)
- `source_reliability` (evidence publication stage)
- `validation_proof` (command/script artifact reference)

Minimum payload keys for `AgentExecutionLifecycle`:
- `schema_version`
- `execution_id`
- `attempt_id`
- `agent_id`
- `workflow_id`
- `phase`
- `status`
- `authority_scope`
- `input_snapshot_hash`
- `output_snapshot_hash`
- `timestamp`

---

## 5. System Observability (Adapter)

**Host**: `log-registry` (Module).

This system provides a *view* into the execution layer. It does not define truth.

```motoko
type LogLevel = { #Info; #Warn; #Error; #Critical };

service : {
    submitLog : (source: Text, level: LogLevel, message: Text) -> ();
};
```

---

## 6. Accessibility Standard (Constitutional)

**Invariant**: System Authority is contingent on Universal Access.

### 6.1 The 4 Rights
See [ACCESSIBILITY.md](./standards/ACCESSIBILITY.md) for full doctrine.

1.  **Perceivability** (Right to Know)
2.  **Operability** (Right to Act)
3.  **Comprehensibility** (Right to Understand)
4.  **Adaptability** (Right to Fork the View)

### 6.2 Host Interface (A2UI Adapter)
Accessibility is enforced via the **A2UI Schema**, which mandates semantic roles and motion tokens.

```

---

## 7. Knowledge Graph Commons Standard (Constitutional)

**Invariant**: Knowledge graph contracts are portable manifests and read-only query shapes. Cortex may execute them, but Nostra defines the semantic vocabulary.

### 7.1 Provenance Scope

`ProvenanceScope` is the graph-contract naming for the same provenance partitioning used by `GlobalEvent`.

```motoko
type ProvenanceScope = {
    #System : Text;  // System-owned scope label
    #Actor : ActorID; // User-owned scope label
    #Agent : ActorID; // Worker/agent-owned scope label
};
```

### 7.2 Retrieval Policy

Knowledge bundles carry an explicit retrieval policy so agents can reason about traversal cost, authority, and citation requirements before asking for graph data.

```motoko
type RetrievalAuthorityWeight = {
    scope: Text;   // "system" | "actor" | "agent"
    weight: Float; // 0.0 .. 1.0
};

type RetrievalPolicy = {
    maxHops: Nat;
    authorityWeights: [RetrievalAuthorityWeight];
    freshnessWindowSeconds: ?Nat;
    allowedPredicates: [Text];
    namedGraphScopes: [Text]; // "system" | "actor" | "agent"
    citationRequired: Bool;
};
```

### 7.3 Knowledge Bundle

The Space Knowledge Bundle is a portable manifest. It references, rather than embeds, the ontology, graph snapshot, embeddings manifest, and provenance root.

```motoko
type BundleCompatibility = {
    ontologyVersion: Text;
    bundleVersion: Text;
    breakingChangePolicy: Text;
};

type KnowledgeBundleTransport = {
    interactiveProtocol: Text; // "json"
    bulkProtocol: Text; // "msgpack"
};

type KnowledgeBundle = {
    bundleId: Text;
    bundleKind: Text; // "space_knowledge_bundle"
    spaceId: Text;
    ontologyRef: Text; // ResourceRef
    graphSnapshotRef: Text; // ResourceRef
    embeddingsManifestRef: ?Text; // ResourceRef
    provenanceRoot: Text; // ResourceRef
    retrievalPolicy: RetrievalPolicy;
    compatibility: BundleCompatibility;
    transport: KnowledgeBundleTransport;
    generatedAt: Int;
    notes: ?Text;
};
```

### 7.4 Triple Query Facade

The triple facade is read-only, JSON-first, and deterministic. It supports S-P-O filtering plus named graph scoping via `GlobalEvent` provenance partitions.

```motoko
type TripleQueryScope = {
    namedGraphScope: Text; // "system" | "actor" | "agent" | "any"
    scopeRef: ?Text; // ResourceRef for actor/agent scopes
};

type TripleQueryFilters = {
    subject: ?Text;
    predicate: ?Text;
    object: ?Text;
    includeProvenance: Bool;
    limit: Nat;
    offset: Nat;
};

type TripleQueryRequest = {
    queryId: Text;
    orderingStrategy: Text; // "canonical"
    scope: TripleQueryScope;
    filters: TripleQueryFilters;
};

type TripleMatch = {
    ordinal: Nat;
    subject: Text;
    predicate: Text;
    object: Text;
    graphScope: Text;
    provenanceScope: Text;
    sourceRef: ?Text;
    confidence: ?Float;
};

type TripleQueryResponse = {
    queryId: Text;
    orderingStrategy: Text; // "canonical"
    scope: TripleQueryScope;
    resultCount: Nat;
    triples: [TripleMatch];
};
```
