# Nostra KIP Entity Tags Standard

This document defines the standardized tagging taxonomy for all KIP entity types in the Nostra ecosystem.

## Tag Categories

### 1. Phase Tags (Lifecycle)
Indicates which phase of the contribution lifecycle an entity belongs to:

| Tag | Description |
|-----|-------------|
| `exploratory` | Discovery, ideation, and open-ended thinking |
| `deliberative` | Discussion, debate, and consensus-building |
| `decisive` | Formal choices and commitments |
| `executable` | Action, implementation, and delivery |
| `archival` | Preservation, documentation, and retrospection |

### 2. Type Tags (Classification)
Indicates the structural role of an entity:

| Tag | Description |
|-----|-------------|
| `contribution` | A durable record created by human/agent action |
| `predicate` | A relationship type between entities |
| `meta` | System-level or schema definition |
| `governance` | Related to decision-making and policies |
| `system` | Internal Nostra platform infrastructure |

### 3. Domain Tags (Context)
Indicates the functional domain:

| Tag | Description |
|-----|-------------|
| `core` | Foundational Nostra concepts |
| `workflow` | Process and automation related |
| `licensing` | Access control and pricing |
| `agent` | AI agent and automation related |

### 4. Behavior Tags (Characteristics)
Indicates behavioral properties:

| Tag | Description |
|-----|-------------|
| `abstract` | Cannot be instantiated directly (base types) |
| `temporal` | Has time-based aspects |
| `incentivized` | Involves rewards or bounties |

---

## Standard Tags by Entity Type

### Contribution Types
| Entity | Tags |
|--------|------|
| Contribution (base) | `core`, `contribution`, `abstract` |
| Post | `core`, `contribution`, `exploratory` |
| Comment | `core`, `contribution`, `exploratory` |
| Idea | `core`, `contribution`, `exploratory` |
| Question | `core`, `contribution`, `exploratory` |
| Issue | `core`, `contribution`, `deliberative` |
| Poll | `core`, `contribution`, `deliberative` |
| Review | `core`, `contribution`, `deliberative` |
| Proposal | `core`, `contribution`, `deliberative` |
| Decision | `core`, `contribution`, `decisive` |
| Project | `core`, `contribution`, `executable` |
| Deliverable | `core`, `contribution`, `executable` |
| Milestone | `core`, `contribution`, `executable`, `temporal` |
| Bounty | `core`, `contribution`, `executable`, `incentivized` |
| Implementation | `core`, `contribution`, `executable` |
| Artifact | `core`, `contribution`, `archival` |
| Report | `core`, `contribution`, `archival` |
| Reflection | `core`, `contribution`, `archival` |
| Essay | `core`, `contribution`, `archival` |

### Organizational Types
| Entity | Tags |
|--------|------|
| Space | `core`, `meta`, `governance` |
| ViewSummary | `core`, `system` |

### Predicates (Relationship Types)
| Entity | Tags |
|--------|------|
| relates_to | `core`, `predicate` |
| spawns | `core`, `predicate` |
| implements | `core`, `predicate` |
| blocks | `core`, `predicate` |
| resolves | `core`, `predicate` |
| supersedes | `core`, `predicate` |
| belongs_to_space | `core`, `predicate` |
| authored_by | `core`, `predicate` |
| part_of | `core`, `predicate` |
| reviews | `core`, `predicate` |
| rewards | `core`, `predicate` |

### System Types
| Entity | Tags |
|--------|------|
| SleepTask | `core`, `system`, `agent`, `temporal` |
| SchemaLicense | `core`, `licensing`, `governance` |
| ChronicleEvent | `core`, `system`, `temporal` |
| ContributionPhase | `core`, `meta` |

---

## Usage in KIP Files

```kip
UPSERT {
    CONCEPT ?entity_type {
        {type: "$ConceptType", name: "EntityName"}
        SET ATTRIBUTES {
            description: "...",
            tags: ["core", "contribution", "executable"],
            ...
        }
    }
}
```

> **Note**: Tags are parsed as an array of strings inside square brackets.
