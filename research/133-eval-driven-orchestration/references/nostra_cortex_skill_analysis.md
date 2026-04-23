# Skill-Creator Analysis: `nostra-cortex-dev-core`

## Structural Validation

| Check | Result |
|-------|--------|
| `quick_validate.py` | ✅ Pass |
| Frontmatter `name` | ✅ Present |
| Frontmatter `description` | ✅ Present |
| Body instructions | ✅ Present (50 lines) |
| Referenced scripts exist | ✅ All 3 exist in `/ICP/scripts/` |
| Referenced docs exist | ✅ `standards.md`, `nostra-cortex-boundary.md` |
| Bundled references | ✅ `failure_modes.md`, `preflight_contract.md` |
| Bundled agents | ✅ `openai.yaml` |

**Verdict**: Structurally sound. No validation errors.

---

## Coverage Analysis

The skill's **actual scope** is narrow: it enforces governance preflight (3 scripts), FM-009 regression prevention (hardcoded values), and closeout evidence formatting. It does **not** provide any domain knowledge about the platform it governs.

### What the Skill Covers

| Domain | Coverage |
|--------|----------|
| FM-009 governance bypass detection | ✅ Full |
| Preflight/post-edit script workflow | ✅ Full |
| Closeout evidence formatting | ✅ Full |
| Nostra vs Cortex boundary definition | ⚠️ Mentioned, not explained |

### What the Skill Does NOT Cover

The `nostra/spec.md` (837 lines) defines the entire platform. The skill provides **zero** procedural knowledge for any of the following:

| Domain | Lines in Spec | Skill Coverage |
|--------|--------------|----------------|
| Unified Contribution Model (26 types) | 95 lines | ❌ None |
| Canister Architecture (8 modules) | 30 lines | ❌ None |
| Frontend Stack (Dioxus/Lit/D3/A2UI) | 15 lines | ❌ None |
| Temporal Workers / Durable Execution | 20 lines | ❌ None |
| Gaming & Simulation (Godot/ECS) | 20 lines | ❌ None |
| Vector Infrastructure / Embeddings | 12 lines | ❌ None |
| Space Configuration & Archetypes | 20 lines | ❌ None |
| Activity Stream / Graph Layer | 30 lines | ❌ None |
| CRUD patterns (Ideas/Projects/Issues/etc.) | 400+ lines | ❌ None |
| UI Component patterns (Dioxus signals) | 100+ lines | ❌ None |
| Architecture standards (14 enforcement hooks) | 40 lines | ⚠️ References but doesn't explain |

### Description Trigger Analysis

Current description:
> "Mandatory preflight and governance-aligned coding workflow for Nostra and Cortex development, including FM-009 dynamic source checks before and after edits."

**Issues:**
1. **Too narrow trigger**: Only activates for governance/preflight tasks. An agent working on `frontend/src/components/` or `worker/src/workflows/` would not trigger this skill.
2. **Missing "when to use"**: Doesn't mention frontend development, backend canister work, A2UI rendering, workflow authoring, or graph operations.
3. **Missing negative triggers**: Doesn't specify when NOT to use the skill (e.g., research-only tasks, documentation-only changes).

---

## Recommendations

### Option A: Enrich the Existing Skill (Quick Win)

Add references that provide domain context the agent currently lacks:

```
nostra-cortex-dev-core/
├── SKILL.md                          (expand description triggers)
├── references/
│   ├── failure_modes.md              (existing)
│   ├── preflight_contract.md         (existing)
│   ├── contribution_model.md         (NEW - extract from spec.md)
│   ├── canister_architecture.md      (NEW - extract from spec.md)
│   ├── frontend_patterns.md          (NEW - Dioxus/A2UI/Lit patterns)
│   └── boundary_contract.md          (NEW - copy boundary doc)
└── agents/
    └── openai.yaml                   (existing)
```

**Pros**: Minimal disruption, one skill to maintain.
**Cons**: Context window bloat risk — a single skill tries to cover everything.

### Option B: Decompose into Domain-Specific Skills (Recommended)

Following the skill-creator's **Progressive Disclosure** principle, split into focused skills:

| Skill | Trigger | Contents |
|-------|---------|----------|
| `nostra-cortex-dev-core` | Any Nostra/Cortex code change | Governance preflight only (current) |
| `nostra-platform-spec` | Backend canister work, contribution CRUD, data models | Contribution model, canister arch, graph edges |
| `cortex-frontend-dev` | Frontend components, UI patterns, Dioxus signals | Dioxus patterns, A2UI rendering, design language |
| `cortex-worker-dev` | Temporal workers, workflows, durable execution | Worker split, saga patterns, workflow DSL |

**Pros**: Each skill stays under 500 lines, triggers precisely, loads only relevant context.
**Cons**: More files to maintain, requires careful description optimization.

### Option C: Hybrid (Pragmatic)

Keep `nostra-cortex-dev-core` as the governance gate, but add a single new companion skill `nostra-platform-knowledge` that acts as the "big reference guide" with progressive disclosure:

```
nostra-platform-knowledge/
├── SKILL.md                          (navigation + when to read each file)
└── references/
    ├── contribution_model.md
    ├── canister_architecture.md
    ├── frontend_patterns.md
    ├── temporal_workers.md
    └── space_configuration.md
```

**Pros**: Governance stays lean, domain knowledge is discoverable, progressive disclosure keeps context efficient.
**Cons**: Two skills must be maintained in sync.

---

## Test Cases (Draft)

These would validate whichever option is chosen:

| ID | Prompt | Should Trigger | Assertion |
|----|--------|---------------|-----------|
| 1 | "Add a new contribution type called 'Workshop' to the backend canister" | ✅ | Agent references contribution model, creates proper type with all required fields |
| 2 | "Fix the issue creation form — it's not passing spaceId to the backend" | ✅ | Agent checks Dioxus signal patterns, validates backend createIssue contract |
| 3 | "Write a temporal workflow that sends weekly space digests" | ✅ | Agent references worker split, uses durable execution patterns |
| 4 | "Update the README with a badge for CI status" | ❌ | Skill should NOT trigger for simple doc edits |
| 5 | "Refactor the similarity engine to use cosine distance instead of overlap" | ✅ | Agent references vector infrastructure and similarity scoring |

---

## Next Steps

1. Choose Option A, B, or C
2. Run description optimization loop (`scripts/run_loop.py`) on chosen skill(s)
3. Execute test cases with A/B baseline comparison
4. Review in eval-viewer and iterate
