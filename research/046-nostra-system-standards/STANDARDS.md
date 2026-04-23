# Nostra Core Standards

**Version**: 1.0.0
**Status**: ACTIVE
**Authority**: `research/046-nostra-system-standards`

> The immutable primitives and protocols that define the Nostra Ecosystem.

---

## 1. The Core 5 Primitives

These 5 interfaces are the atomic units of the system. All valid Nostra objects MUST implement one of these.

### 1.1 `Entity` (The Atom)
Represents any addressable object in the system.
```typescript
interface Entity {
  id: string;              // UUID or DID
  owner: Principal;        // ICP Principal
  created_at: Timestamp;
  updated_at: Timestamp;

  // 046 Data Integrity Standard
  provenance: {
    creator: Principal;    // Agent or User ID
    method: string;        // "user_input", "import", "inference"
    source_uri?: string;   // Original URL/File
  };

  // 046 Confidence Standard
  trust_score: float;      // 0.0 to 1.0
  validation_proof?: string; // CID or Hash of reasoning trace

  // Semantic Web Standard
  "@context"?: string;     // JSON-LD Context URI (e.g. "https://schema.org")
  "@type"?: string;        // Semantic Type (e.g. "Person")
}
```

### 1.2 `Link` (The Tissue)
Represents a directional relationship between two Entities.
```typescript
interface Link {
  source: string;          // Entity ID
  target: string;          // Entity ID
  relation: string;        // Predicate (e.g. "authored", "depends_on")
  weight: float;           // Strength of connection (0.0 to 1.0)
  bidirectional: boolean;
}
```

### 1.3 `Agent` (The Actor)
Represents an autonomous actor (AI or Service).
**Alignment**: Follows the [Apify Actor Standard](https://docs.apify.com/academy/deploying-actors/input-schema).
```typescript
interface Agent {
  id: Principal;
  name: string;
  description: string;

  // Input/Output Definitions (JSON Schema)
  input_schema: object;
  output_schema: object;

  // Capabilities
  permissions: string[];   // internal scopes
  pricing: PricingModel;
}
```

### 1.4 `Event` (The Timeline)
Represents an immutable point in time.
```typescript
interface Event {
  id: string;
  type: string;            // "EntityCreated", "WorkflowStarted"
  source: Principal;       // Who caused it
  timestamp: Timestamp;
  payload: Blob;           // The data diff
  signature?: Blob;        // Cryptographic proof
}
```

### 1.5 `Locus` (The Context)
Represents a physical or jurisdictional boundary (Contextual Sovereignty).
```typescript
interface Locus {
  id: string;
  source: string;          // e.g. "GPS", "User-Declared"
  confidence: float;
  coordinates?: string;    // GeoJSON or Lat/Long
  geohash?: string;
  jurisdiction?: string;   // Reference to a Scope
}
```

---

## 2. Interoperability Protocols

### 2.1 The Adapter Pattern
Nostra does not force the world to change. We adapt to it.
*   **Definition**: `nostra.core.Adapter`
*   **Function**: Maps external API types (OpenAPI/GraphQL) -> `nostra.core` types.
*   **Rule**: Never pollute core logic with external vendor types. Always wrap them in an Adapter.

### 2.2 Semantic Compatibility (JSON-LD)
*   **Rule**: Where possible, align field names with [Schema.org](https://schema.org).
*   **Benefit**: Allows "zero-config" import of web data and better LLM understanding.

### 2.3 Structural Typing (Duck Typing)
*   **Rule**: The system uses "Shape Matching" to validate types.
*   **Mechanism**: If `MyObject` matches the `TaskBehavior` interface (has `status`, `title`), it IS a Task. No explicit inheritance required.

---

## 2.5 Temporal Reliability Pillars

> From [047-temporal-architecture](../047-temporal-architecture/RESEARCH.md)

| # | Pillar | Principle |
|---|--------|----------|
| 5 | Durable Execution | "If it's not in History, it didn't happen" |
| 6 | Visibility Decoupling | "Don't ask the Worker, ask the Index" |
| 7 | Outbox Pattern | "Persist before you Send" |

Full specification: [docs/architecture/standards.md](file:///Users/xaoj/ICP/docs/architecture/standards.md)


## 2.6 Proof Capability Standard

> From [061-zk-proof-integration](../061-zk-proof-integration/RESEARCH.md)

| Level | Mechanism | Cost | Use Case |
|---|---|---|---|
| 0 | Signature | Minimal | Identity, Authorship |
| 1 | Hash | Low | Data Integrity |
| 2 | Attestation | Medium | Compliance Assertions |
| 3 | ZK Proof | High | Non-inspectable Guarantees |


## 3. Governance
*   **Changes**: Modifying the "Core 4" requires a KIP (Knowledge Improvement Proposal) and 80% Stakeholder Consensus.
*   **Extensions**: Creating new Traits/Behaviors is permissionless.

## 4. UI/UX Standards

### 4.1 Iconography
*   **Rule**: Do NOT use inline SVGs for standard UI icons. Use the `<Icon />` component from `crate::components::icons`.
*   **Purpose**: Ensures consistency, theme support, and centralized management of assets.
*   **Exception**: Brand logos or specialized, one-off illustrations may use inline SVG or `img` tags.
