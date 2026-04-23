---
id: '001'
name: multi-project-architecture
title: Requirements & Tech Stack
type: architecture
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements & Tech Stack

## Overview
Technical requirements and technology stack for the Multi-Project Knowledge Graph Architecture.

---

## Tech Stack

### Backend (ICP Canisters)

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| **KG Data Canisters** | Motoko | Latest | Entity/relationship storage, schema validation |
| **Registry Canister** | Motoko | Latest | Project lifecycle, user management |
| **Schema Registry** | Motoko | Latest | Schema storage, versioning |
| **MCP Server** | Rust + ic-rmcp | v0.3.0+ | AI tool exposure via MCP protocol |
| **Cycles Wallet** | IC SDK | Latest | Platform-subsidized canister funding |

### Frontend

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| **UI Framework** | Dioxus | 0.5+ | Reactive web interface |
| **Styling** | Tailwind CSS | 3.x | Responsive design |
| **Graph Viz** | 3D Force Graph | Latest | Knowledge graph visualization |
| **Build Tool** | dx (Dioxus CLI) | Latest | WASM compilation |
| **ICP Integration** | @dfinity/agent | Latest | Canister communication |

### Infrastructure

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Runtime** | Internet Computer | Decentralized compute + storage |
| **Deployment** | dfx | Canister lifecycle management |
| **Local Dev** | dfx replica | Local ICP simulation |

---

## Functional Requirements

### FR-1: Multi-Project Management
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | Users can create up to 5 free projects | Must |
| FR-1.2 | Each project gets isolated canister | Must |
| FR-1.3 | Users can archive/delete projects | Should |
| FR-1.4 | Projects can be shared with collaborators | Should |

### FR-2: Dynamic Schema System
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | Schemas define entity types with properties | Must |
| FR-2.2 | Schemas define predicates with semantics | Must |
| FR-2.3 | Predicates support `inverseOf` relationships | Should |
| FR-2.4 | Predicates support transitivity types | Could |
| FR-2.5 | Inference rules can be defined per schema | Could |
| FR-2.6 | Schemas are versionable | Should |
| FR-2.7 | Public schemas can be shared/forked | Should |

### FR-3: Temporal Data Model
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | Entities track `eventTime` (when fact occurred) | Must |
| FR-3.2 | Entities track `ingestedAt` (when recorded) | Must |
| FR-3.3 | Entities have `validFrom`/`validTo` periods | Should |
| FR-3.4 | Historical queries ("as of date X") supported | Could |
| FR-3.5 | Entity version history is retained | Could |

### FR-4: MCP Integration
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | Expose KG tools via MCP protocol | Must |
| FR-4.2 | Support `query_entities` tool | Must |
| FR-4.3 | Support `add_entity` tool | Must |
| FR-4.4 | API key authentication for MCP endpoints | Must |
| FR-4.5 | Temporal query tools (`query_as_of`) | Should |
| FR-4.6 | Inference tools (`run_inference`) | Could |

### FR-5: ICP Canon
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1 | Provide read-only ICP ecosystem reference data | Must |
| FR-5.2 | Users can fork Canon to new projects | Should |
| FR-5.3 | Canon updates via DAP governance | Could |

---

## Non-Functional Requirements

### NFR-1: Performance
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1.1 | Entity query latency | < 500ms |
| NFR-1.2 | Project creation time | < 10s |
| NFR-1.3 | Graph visualization render | < 2s for 1000 nodes |

### NFR-2: Scalability
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-2.1 | Entities per project | 100,000+ |
| NFR-2.2 | Relationships per project | 500,000+ |
| NFR-2.3 | Concurrent users per project | 10+ |

### NFR-3: Security
| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-3.1 | Principal-based access control | Must |
| NFR-3.2 | Write operations require caller validation | Must |
| NFR-3.3 | MCP endpoints require API key | Must |
| NFR-3.4 | Rate limiting on project creation | Must |

### NFR-4: Reliability
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-4.1 | Data durability | ICP stable memory guarantees |
| NFR-4.2 | Canister upgrade safety | Zero data loss on upgrades |

---

## Dependencies

### Motoko Libraries
| Library | Purpose |
|---------|---------|
| `mo:base` | Standard library |
| `mo:json` | JSON parsing for AI responses |
| `mo:stable-structures` (if needed) | Scalable storage |

### Rust Crates (MCP Server)
| Crate | Version | Purpose |
|-------|---------|---------|
| `ic-rmcp` | 0.3.0+ | MCP server SDK for ICP |
| `ic-cdk` | Latest | Canister development kit |
| `serde` | Latest | Serialization |
| `candid` | Latest | ICP interface definition |

### Frontend Dependencies
| Package | Purpose |
|---------|---------|
| `dioxus` | UI framework |
| `dioxus-web` | Web target |
| `3d-force-graph` | Graph visualization |
| `@dfinity/agent` | ICP agent |

---

## Development Environment

### Required Tools
```bash
# ICP SDK
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# Rust (for MCP server)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Dioxus CLI
cargo install dioxus-cli

# Node.js (for frontend tooling)
# Use nvm or direct install
```

### Local Development
```bash
# Start local replica
dfx start --background

# Deploy all canisters
dfx deploy

# Frontend dev server
cd frontend && dx serve
```

---

## API Contracts

### Inter-Canister Interfaces

#### Registry → KG Template
```candid
service : {
  getProjectInfo : () -> (ProjectInfo) query;
  setCollaborators : (vec principal) -> ();
  getSchema : () -> (SchemaDefinition) query;
}
```

#### KG Template → Schema Registry
```candid
service : {
  getSchema : (text) -> (opt SchemaDefinition) query;
  validateEntity : (EntityInput, text) -> (ValidationResult);
}
```

#### MCP Server → KG Template
```candid
service : {
  queryEntities : (EntityFilter) -> (vec Entity) query;
  createEntity : (EntityInput) -> (Entity);
  getEntityAsOf : (text, int) -> (opt Entity) query;
}
```

---

## Constraints

| Constraint | Description |
|------------|-------------|
| No external DBs | All data stored in ICP stable memory |
| WASM size limits | Canisters must fit within ICP limits (~2MB) |
| Cycles costs | Platform must manage cycle economics |
| HTTP response size | MCP responses limited by ICP gateway |
| No Python/Java | Cannot run Graphiti/OpenSPG directly |

---

## References

- [PLAN.md](./PLAN.md) - Implementation phases
- [DECISIONS.md](./DECISIONS.md) - Architectural decisions
- [FRAMEWORK_STRATEGY.md](./FRAMEWORK_STRATEGY.md) - External framework analysis
- [ic-rmcp](https://github.com/ByteSmithLabs/ic-rmcp) - MCP SDK for ICP
