# Space Design A2UI Fixture Validation Evidence - 2026-04-27

## Context

This evidence records the next Initiative 120 implementation slice after the Space design next-session handoff. The work adds recommendation-only Cortex Web fixture validation for `SpaceDesignProfileV1` candidates without activating runtime profile selection, external design-system imports, token wiring, or Hermes approval authority.

## Changed Surfaces

- `scripts/check_ndl_design_profiles.py`
- `research/120-nostra-design-language/prototypes/space-design/LINT_CONTRACT.md`
- `research/120-nostra-design-language/prototypes/space-design/design-reality-snapshot.v1.json`
- `research/120-nostra-design-language/_archive/LINT_CONTRACT_2026-04-27_PRE_FIXTURE_VALIDATION.md`
- `research/120-nostra-design-language/_archive/design-reality-snapshot_2026-04-27_PRE_FIXTURE_VALIDATION.v1.json`

## Fixture Validation Added

`python3 scripts/check_ndl_design_profiles.py` now validates Space design profiles beside the effective A2UI theme/render truth:

- `shared/a2ui/themes/*`
- `shared/a2ui/fixtures/*`
- `cortex/libraries/cortex-domain/src/theme/policy.rs`
- `cortex/apps/cortex-eudaemon/src/services/theme_policy.rs`

The check confirms that themed A2UI render fixtures reference known theme names, supported runtime token versions, safe mode, accepted motion policies, and accepted contrast preferences. It also rejects Space design profile claims that would reuse runtime or fixture theme allowlist IDs, claim runtime A2UI token versions, claim runtime enforcement, or encode Tier 1 governance state in profile tokens.

## Validation Evidence

Preflight:

```text
PASS: preflight contract has required sections
PASS: agent preflight contract checks
PASS: dynamic source contract checks
PASS: no hardcoded workspace root paths in active gateway/scripts
```

Space design contract:

```text
PASS: Space design contract checks (1 profile(s), 1 import(s), 1 template pack(s), 2 A2UI theme(s), 2 themed fixture(s))
```

Python compile:

```text
python3 -m py_compile scripts/check_ndl_design_profiles.py
```

Touched-file semantic alignment:

```json
{
  "errors": 0,
  "issues": [],
  "scanned_files": 5,
  "warnings": 0
}
```

Terminology and diff hygiene:

```text
PASS: terminology checks passed
PASS: git diff --check
```

## Boundary Statement

This work remains recommendation-only. It does not add a runtime theme switcher, does not select Space profiles at runtime, does not import an external design system, does not wire profile tokens into Cortex Web rendering, and does not expand Hermes beyond read-only advisory analysis of bounded source packets.
