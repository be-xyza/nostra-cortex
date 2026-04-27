---
title: Space Design Cortex Web Preview Fixture
captured_at: 2026-04-27T18:04:58Z
initiative: 132-eudaemon-alpha-initiative
scope: Cortex Web preview fixture validation for Space design profile metadata
status: passed
---

# Space Design Cortex Web Preview Fixture

## Summary

Added a Cortex Web preview fixture that exposes the Research Observatory `SpaceDesignProfileV1` as advisory metadata only. The fixture is reachable through preview fixtures at `/api/system/ux/space-design-profiles` and is registered in the preview fixture catalog as `system:ux:space-design-profiles`.

The fixture intentionally does not carry `design_tokens`, does not apply tokens to Cortex Web, does not select a runtime theme, and keeps `runtime_binding` set to `none`.

## Guardrails Added

- `scripts/check_ndl_design_profiles.py` now validates the Cortex Web Space design preview fixture against the canonical Space design profile and A2UI runtime truth.
- `scripts/test_ndl_design_profile_a2ui_fixture_validation.py` now includes negative coverage for Cortex Web preview fixture runtime binding and token-carrying regressions.
- `cortex/apps/cortex-web/tests/previewFixtureCatalog.test.ts` now asserts the preview fixture remains metadata-only and recommendation-only.

## Validation

```text
bash scripts/check_agent_preflight_contract.sh
PASS: preflight contract has required sections
PASS: agent preflight contract checks

bash scripts/check_dynamic_config_contract.sh
PASS: dynamic source contract checks
PASS: no hardcoded workspace root paths in active gateway/scripts

python3 scripts/check_ndl_design_profiles.py
PASS: Space design contract checks (1 profile(s), 1 import(s), 1 template pack(s), 2 A2UI theme(s), 2 themed fixture(s))

python3 scripts/test_ndl_design_profile_a2ui_fixture_validation.py
PASS: Space design A2UI fixture validation regression coverage

python3 -m py_compile scripts/check_ndl_design_profiles.py scripts/test_ndl_design_profile_a2ui_fixture_validation.py
PASS

node --experimental-strip-types --test tests/previewFixtureCatalog.test.ts
PASS: 3 tests

npm run test:spatial
PASS: 64 tests

npm run check
PASS

npm run build
PASS

python3 scripts/check_semantic_alignment_surfaces.py --json cortex/apps/cortex-web/src/store/spaceDesignProfilePreview.fixture.json cortex/apps/cortex-web/src/store/spaceDesignProfilePreview.ts cortex/apps/cortex-web/src/store/spaceDesignProfilePreviewContract.ts cortex/apps/cortex-web/src/store/previewFixtureCatalog.ts cortex/apps/cortex-web/src/sw.ts cortex/apps/cortex-web/tests/previewFixtureCatalog.test.ts cortex/apps/cortex-web/package.json scripts/check_ndl_design_profiles.py scripts/test_ndl_design_profile_a2ui_fixture_validation.py
PASS: errors=0 warnings=0

bash scripts/check_nostra_cortex_terminology.sh
PASS

git diff --check
PASS
```

## Boundary

This remains fixture and validation work only. It does not introduce runtime Space profile selection, direct token wiring, theme switching, external design system imports, or Hermes approval authority.
