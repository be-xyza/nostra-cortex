# NDL Space Design Lint Contract

## Purpose

This prototype defines the lint shape for NDL-compatible Space design profiles inspired by the upstream `design.md` reference. The contract is intentionally recommendation-only until a steward approves runtime enforcement.

## Input Surfaces

- `SPACE_DESIGN.md`: human-readable design profile with upstream-compatible token front matter.
- `SPACE_DESIGN.ndl-profile.v1.json`: canonical Nostra wrapper for authority metadata, Space lineage, surface scope, NDL tier policy, and A2UI theme policy.
- `NdlDesignProfileV1.schema.json`: structural schema for the wrapper.

## Local Command

```bash
python3 scripts/check_ndl_design_profiles.py
```

The command defaults to all Initiative 120 prototype profiles and can also receive explicit profile JSON paths.

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

## Hermes Use

Hermes may consume this profile only through a bounded source packet. Its output should be a source-linked advisory finding set plus one synthesis artifact. Hermes must not mutate profile files, approve a profile, change the Space capability graph, or mark any theme as runtime-enforced.

## Promotion Gate

A Space design profile can move from draft to steward-approved only after:

1. upstream-compatible lint has no errors,
2. Nostra-specific lint has no errors,
3. accessibility checks cover contrast, focus visibility, keyboard reachability, reduced motion, and text fit,
4. the Design Systems Steward records approval lineage,
5. Cortex Web renders the profile through a fixture or preview without allowing Tier 1 spoofing.
