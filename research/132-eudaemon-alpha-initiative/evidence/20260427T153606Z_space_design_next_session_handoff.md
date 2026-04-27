# Space Design Standards Handoff for Fresh Cortex Session - 2026-04-27

## Purpose

This handoff gives the next Cortex session a clean continuation point for Initiative 120 Space-level design standards after the `design.md` reference intake, Space design contract primitives, skill pickup, Hermes advisory validation, accessibility lint hardening, and promotion-gate hardening.

## Current Mainline State

The main branch has the following merged work:

- PR #54 `b3e911148`: `design.md` reference intake and initial Initiative 120 Space design prototype.
- PR #55 `1f106561`: semantic alignment surface scanner.
- PR #56 `ee947f3d`: removed active `_bmad` runtime defaults from Cortex/Nostra script surfaces.
- PR #57 `e0b2c1e8`: promoted Space design contract primitives:
  - `SpaceDesignProfileV1`
  - `DesignElementImportV1`
  - `SpaceTemplatePackV1`
  - profile/import/template validation in `scripts/check_ndl_design_profiles.py`
- PR #58 `edcf9a27`: wired Space design contract pickup into `nostra-cortex-dev-core`, `frontend-design`, and the repo-managed skill registry.
- PR #59 `500b8d0a`: recorded bounded Hermes PASS evidence for post-skill-pickup Initiative 120 validation.
- PR #60 `8b9f0af6`: expanded Space design accessibility lint.
- PR #61 `7e287143`: hardened Space design import/template promotion gates.

## Canonical Sources to Read First

Read these before changing anything:

1. `AGENTS.md`
2. `docs/architecture/standards.md`
3. `docs/architecture/nostra-cortex-boundary.md`
4. `research/120-nostra-design-language/PLAN.md`
5. `research/120-nostra-design-language/prototypes/space-design/LINT_CONTRACT.md`
6. `research/132-eudaemon-alpha-initiative/evidence/20260427T080023Z_initiative_120_space_design_post_skill_pickup.md`
7. `nostra/commons/skills/nostra-cortex-dev-core/SKILL.md`
8. `nostra/commons/skills/frontend-design/SKILL.md`

## Current Contract Shape

Space-level design work is a recommendation-only standards-hardening layer. It does not authorize runtime profile selection, external design-system import, Cortex Web theme enforcement, or Hermes approval authority.

The active primitives are:

- `SpaceDesignProfileV1`: Nostra-owned wrapper for Space design profile authority metadata, lineage, surface scope, NDL tier policy, A2UI theme policy, tokens, and design sections.
- `DesignElementImportV1`: candidate import record for established design materials. It remains `recommendation_only` and must carry provenance, lineage, accessibility, brand, NDL, A2UI, and Space capability checks.
- `SpaceTemplatePackV1`: candidate Space archetype bundle that references profile defaults and design imports behind a `needs_steward_review` promotion gate.
- `DesignAuditUnitV1`: bounded source packet for local/Hermes advisory analysis.
- `DesignRealitySnapshotV1`: locked design reality manifest.

## Validation Commands

Run these before and after edits:

```bash
bash scripts/check_agent_preflight_contract.sh
bash scripts/check_dynamic_config_contract.sh
python3 scripts/check_ndl_design_profiles.py
python3 -m py_compile scripts/check_ndl_design_profiles.py
python3 scripts/check_semantic_alignment_surfaces.py --json <touched-files>
bash scripts/check_nostra_cortex_terminology.sh
git diff --check
```

When skill governance surfaces are touched, also run:

```bash
bash scripts/check_skill_registry_integrity.sh
bash scripts/check_skill_policy.sh
```

## Known Residuals and Boundaries

- Full semantic alignment scan is still observe-mode and reports unrelated Initiative 132 slash-form findings. Do not treat those as blockers for Initiative 120 unless the next task explicitly targets Initiative 132 terminology cleanup.
- `docs/architecture/a2ui-theme-policy.md` is still absent. Effective theme-policy truth remains:
  - `shared/a2ui/themes/*`
  - `shared/a2ui/fixtures/*`
  - `cortex/libraries/cortex-domain/src/theme/policy.rs`
  - `cortex/apps/cortex-eudaemon/src/services/theme_policy.rs`
- Hermes remains read-only/advisory. It may review bounded packets but cannot approve, mutate, import, enforce, install skills, or activate runtime behavior.
- User-facing container terminology is `Space`; never surface `workspace` as the product concept.
- Nostra platform defines what exists; Cortex runtime defines how work runs.

## Recommended Next PR

The next implementation slice should be **Cortex Web fixture validation for Space design profiles**, because accessibility lint and promotion-gate hardening are now complete.

Recommended scope:

1. Add a small fixture or validation command that checks a `SpaceDesignProfileV1` candidate against the effective A2UI theme/render truth.
2. Keep the check recommendation-only and non-runtime.
3. Assert that profile tokens do not claim theme allowlist authority, runtime enforcement, or Tier 1 governance state.
4. Prefer validating compatibility with existing `shared/a2ui/themes` and `shared/a2ui/fixtures` rather than adding a runtime theme switcher.
5. Update Initiative 120 lint contract and evidence only after the fixture check exists.

Suggested starting files:

- `scripts/check_ndl_design_profiles.py`
- `research/120-nostra-design-language/prototypes/space-design/LINT_CONTRACT.md`
- `research/120-nostra-design-language/prototypes/space-design/design-reality-snapshot.v1.json`
- `shared/a2ui/themes/*`
- `shared/a2ui/fixtures/*`
- `cortex/libraries/cortex-domain/src/theme/policy.rs`
- `cortex/apps/cortex-eudaemon/src/services/theme_policy.rs`

## Do Not Do Next

- Do not implement runtime Space profile selection yet.
- Do not import an external design system.
- Do not wire profile tokens directly into Cortex Web rendering.
- Do not expand Hermes authority.
- Do not rename Space/Workspace semantics.
- Do not remove research/reference historical mentions of `_bmad`; only active runtime defaults were removed.

## Closeout Expectation

For the next PR, include:

- Preflight Evidence
- Dynamic Source Evidence
- Space design contract lint evidence
- touched-file semantic alignment evidence
- explicit statement that the work remains recommendation-only
