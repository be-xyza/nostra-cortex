# Space Design A2UI Fixture Negative Coverage Evidence - 2026-04-27

## Context

This evidence records the regression-coverage slice after PR #63 added recommendation-only Cortex Web fixture validation for `SpaceDesignProfileV1` candidates.

The work keeps the same boundary: it validates draft profile compatibility with effective A2UI theme/render truth, but does not activate runtime Space profile selection, import an external design system, wire profile tokens into Cortex Web rendering, or expand Hermes beyond read-only advisory analysis.

## Changed Surfaces

- `scripts/test_ndl_design_profile_a2ui_fixture_validation.py`
- `.github/workflows/test-suite.yml`

## Coverage Added

The regression harness creates temporary profile/theme/fixture variants and asserts expected failures for:

- profile reuse of runtime theme allowlist id `host-default`,
- profile reuse of render fixture allowlist id `trusted-core`,
- runtime theme truth incorrectly claiming `ndl-token-v1` as a supported A2UI token version,
- profile token values that imply Tier 1 governance authority,
- A2UI render fixture metadata referencing an unknown theme,
- A2UI render fixture metadata disabling safe mode.

The canonical Initiative 120 profile/import/template fixtures remain valid and unchanged.

The active GitHub Actions workflow display name was also normalized from slash-form wording to `Nostra Cortex Test Suite` while the workflow was in scope. After CI exposed repeatable `arduino/setup-protoc@v3` lookup failures before repo checks ran, the affected workflow jobs were updated to install `protobuf-compiler` from the Ubuntu runner package index and print `protoc --version`.

## Validation Evidence

```text
PASS: preflight contract has required sections
PASS: agent preflight contract checks
PASS: dynamic source contract checks
PASS: no hardcoded workspace root paths in active gateway/scripts
PASS: Space design A2UI fixture validation regression coverage
PASS: Space design contract checks (1 profile(s), 1 import(s), 1 template pack(s), 2 A2UI theme(s), 2 themed fixture(s))
PASS: python3 -m py_compile scripts/test_ndl_design_profile_a2ui_fixture_validation.py scripts/check_ndl_design_profiles.py
PASS: terminology checks passed
PASS: git diff --check
```

Touched-file semantic alignment:

```json
{
  "errors": 0,
  "issues": [],
  "scanned_files": 2,
  "warnings": 0
}
```

## Boundary Statement

This work remains recommendation-only. It adds regression coverage for validation boundaries only; it does not add runtime profile selection, runtime theme switching, external design-system import, Cortex Web token wiring, or Hermes approval authority.
