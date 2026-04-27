# NDL Space Design Lint Contract

## Purpose

This prototype defines the lint shape for NDL-compatible Space design profiles inspired by the upstream `design.md` reference. The contract is intentionally recommendation-only until a steward approves runtime enforcement.

## Input Surfaces

- `SPACE_DESIGN.md`: human-readable design profile with upstream-compatible token front matter.
- `SPACE_DESIGN.space-profile.v1.json`: canonical Nostra wrapper for authority metadata, Space lineage, surface scope, NDL tier policy, and A2UI theme policy.
- `SpaceDesignProfileV1.schema.json`: structural schema for the wrapper.
- `*.design-import.v1.json`: recommendation-only candidate material records for palettes, recipes, layout rules, token packs, or other established design elements.
- `*.template-pack.v1.json`: recommendation-only Space archetype bundles that reference profile defaults and import candidates.
- `*.promotion-gate.v1.json`: recommendation-only evidence requirements before a Space profile can request steward approval.

## Local Command

```bash
python3 scripts/check_ndl_design_profiles.py
```

The command defaults to all Initiative 120 prototype profiles, design imports, and template packs. It can also receive explicit profile JSON paths plus optional `--imports` and `--templates` lists.

By default, the command also validates profile compatibility against the effective A2UI theme/render truth:

- `shared/a2ui/themes/*`
- `shared/a2ui/fixtures/*`
- `cortex/libraries/cortex-domain/src/theme/policy.rs`
- `cortex/apps/cortex-eudaemon/src/services/theme_policy.rs`

This fixture validation is recommendation-only. It checks that a draft Space profile can be evaluated beside Cortex Web render fixtures without claiming runtime theme allowlist authority, runtime token-version authority, runtime enforcement, or Tier 1 governance state.

## Upstream-Compatible Checks

These can be delegated to an upstream-style `design.md` lint pass:

- broken token references
- missing primary color
- missing typography tokens
- missing spacing or rounding scale
- component text/background contrast
- orphaned color tokens
- canonical section order
- token summary

## Nostra-Specific Checks

These must be enforced by a Nostra-owned lint pass before runtime adoption:

- `surface_scope` must not include constitutional surfaces.
- `tier1_components_allowed` must remain false for Space profile drafts.
- `verified_projection_required` must remain true.
- prohibited governance claims must include `ratified`, `approved`, `constitutional`, `verified_identity`, and `steward_authorized`.
- `authority_mode` must stay `recommendation_only` until steward approval is recorded.
- `approved_by` must be empty unless `authority_mode` is at least `steward_approved`.
- `lineage_ref` must resolve to the source profile.
- `a2ui_theme_policy` must include token version, safe mode, allowlist id, motion policy, and contrast preference.
- all governance, identity, and approval states must be rendered through verified projection rather than component tokens.
- candidate imports must include provenance and `license_or_lineage` checks.
- template packs must remain `recommendation_only`, resolve profile defaults, resolve included imports, and exclude constitutional surfaces.
- imports must carry the full promotion-gate check set: locked reality snapshot, brand policy, accessibility, NDL surface boundary, A2UI theme policy, Space capability alignment, and license or lineage.
- imports must not claim `steward_reviewed`, approved, runtime, or runtime-enforced status before a steward promotion gate.
- template packs must require `needs_steward_review` as their draft promotion recommendation.
- template pack promotion gates must include locked reality snapshot, profile validation, import validation, brand policy, accessibility, A2UI theme policy, Space capability alignment, and Hermes advisory-only checks.
- template packs may include only recommendation-only imports whose adoption status remains candidate, adapt-only, or needs-steward-review.
- promotion gates must reference an existing recommendation-only profile, resolve source evidence refs, require the full promotion evidence set, keep steward approval unclaimed until recorded, and leave runtime activation disabled.
- A2UI fixture validation must prove that existing themed render fixtures reference known A2UI themes, supported runtime token versions, safe mode, accepted motion policy, and accepted contrast preference.
- Space design profile tokens must not reuse runtime or fixture theme allowlist IDs, claim runtime A2UI token versions, claim runtime enforcement, or encode Tier 1 governance state.

## Accessibility Checks

These checks bind Space profile drafts to `shared/standards/ACCESSIBILITY.md` without approving runtime use:

- component text/background pairs must meet WCAG AA 4.5:1 contrast,
- status colors must have at least 3.0:1 non-text contrast against surface or neutral backgrounds,
- state-bearing components such as evidence, warning, and boundary affordances must include typography and text color so color is not the only state channel,
- recommendation-only profiles must keep `safe_mode = true` and must not use `motion_policy = full`,
- typography tokens must keep readable font sizes, non-negative letter spacing, and adequate line height,
- `spacing.measure` must stay at or below the readable 80-character text-fit bound,
- `lint_contract.required_local_checks` must explicitly include contrast, reduced motion, focus visibility, keyboard reachability, text fit, and color-channel checks.

## Hermes Use

Hermes may consume profiles, imports, and template packs only through a bounded source packet. Its output should be a source-linked advisory finding set plus one synthesis artifact. Hermes must not mutate profile files, approve a profile, import design systems, change the Space capability graph, or mark any theme as runtime-enforced.

## Promotion Gate

A Space design profile can move from draft to steward-approved only after:

1. upstream-compatible lint has no errors,
2. Nostra-specific lint has no errors,
3. candidate imports and template packs validate as recommendation-only inputs,
4. accessibility checks cover contrast, focus visibility, keyboard reachability, reduced motion, text fit, and non-color state communication,
5. the Design Systems Steward records approval lineage,
6. Cortex Web renders the profile through a fixture or preview without allowing Tier 1 spoofing.

The current A2UI fixture validation satisfies only the draft compatibility evidence for item 6. It does not activate runtime profile selection, import an external design system, or wire profile tokens into Cortex Web rendering.

`DesignPromotionGateV1` records these requirements as evidence, not authority. A draft gate cannot mark a profile approved, reuse a runtime theme allowlist, enable profile selection, apply profile tokens, switch themes, or treat Hermes findings as steward approval.
