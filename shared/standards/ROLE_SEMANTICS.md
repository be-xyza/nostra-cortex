# Role Semantics Doctrine (Nostra / Cortex)

**Status**: Draft (Proposed Standard)
**Owner**: `research/097-nostra-cortex-alignment`
**Scope**: Naming of roles, personas, and user-facing titles across Nostra (platform/governance) and Cortex (execution).

This doctrine exists to prevent *semantic drift* where UI labels or agent personas accidentally imply illegitimate authority (e.g., civic metaphors), and to ensure titles remain valid as agency becomes increasingly non-human, procedural, auditable, and forkable.

---

## 1) Doctrine (Normative Rules)

1. **Roles Describe Scope, Not Status**
   A role names what can be done (bounded responsibilities), not who someone is (prestige, hierarchy, identity).

2. **Agent-Legible**
   Every role must plausibly be held by:
   - a human
   - an AI agent
   - a future non-human executor

   If “an AI <role>” sounds absurd, the role fails.

3. **Constraint Before Authority**
   Prefer titles that imply boundaries, duties, reversibility, and continuity over command and unilateral power.

4. **Fork-Safe Legitimacy**
   Titles must survive forking without contradiction:
   - no role implies exclusive legitimacy
   - no role implies “overthrow” dynamics

5. **Layer-Aligned Semantics (Nostra vs Cortex)**
   Role semantics must align to the constitutional layer that grants legitimacy:
   - **Nostra** defines what exists (spaces, contributions, permissions, governance).
   - **Cortex** defines how things run (workflows, workers, runtime services).

   Avoid cross-contamination unless explicitly specified by a contract.

6. **Self-Describing Without Lore**
   A new user should infer what the role does *and does not do* without requiring a long disclaimer paragraph.

---

## 2) Recommended Role Lexicon (By Layer)

Use these as the primary vocabulary when naming roles/personas.

### Nostra (Constitutional / Platform)
- `Steward`: accountable caretaker for space-scoped policy, intake, and continuity; proposes and escalates, does not decree.
- `Maintainer`: scope-bound authority for evolution of schemas/contracts/processes; fork-friendly OSS semantics.
- `Custodian`: preservation/guardianship of history, archives, and provenance (more protective than change-making).
- `Registrar`: responsible for canonical registration and indexing boundaries (naming, catalogs, registries).

### Nostra (Governance Procedures)
- `Reviewer`: evaluates proposals for correctness/compliance.
- `Signer`: authorizes irreversible or high-impact transitions (deploy, merge, ratify).
- `Delegate`: temporary scope-limited authority explicitly granted by a steward/governance policy.
- `Auditor`: post-hoc verification of compliance and invariants.

### Cortex (Execution / Runtime)
- `Operator`: runs workflows/tools under explicit constraints and approvals; execution-forward, not legitimacy-forward.
- `Worker`: performs scoped tasks (often ephemeral), usually as a sub-agent.
- `Monitor`: observes signals/health and emits events (no authority to mutate without escalation).

### Internal System Components (Not User-Visible)
- `Orchestrator`: internal coordination module (schedulers, dependency graphs).
- `Intention Compiler`: parser/translator from human intent to workflow + A2UI + proposal artifacts.

---

## 3) Deprecation Guidance (Anti-Patterns)

Avoid (or explicitly deprecate) civic/sovereign metaphors for agents and system roles:
- `Mayor`, `Governor`, `President`, `Leader`, `Ruler`

Rationale: these imply political agency, representation, discretionary power, and centralized legitimacy — all future-hostile in an agent-driven, procedural governance system.

Also avoid temporal identity-as-role titles:
- `Founder` (acceptable as a historical attribute, not an ongoing role)

---

## 4) Migration Note: “Mayor” → Layer-True Roles

Historical usage of **“Mayor”** (e.g., as a chat-first command center) MUST be treated as *deprecated terminology*.

When the old “Mayor” concept is present, map it explicitly:
- **User-facing interface**: `Steward Console` (Nostra-facing) and/or `Operator Console` (Cortex-facing)
- **Execution agent persona**: `agent-operator`
- **Governance intake persona**: `agent-steward`
- **Schema/process evolution persona**: `agent-maintainer`
- **Internal module name**: `intention_compiler` / `orchestrator` (not “mayor”)

Protocol identifiers and canonical docs SHOULD NOT introduce new `mayor`-named entities, types, or persona IDs.
