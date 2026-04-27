---
title: Space Design Cortex Web Preview Client
captured_at: 2026-04-27T18:21:12Z
initiative: 132-eudaemon-alpha-initiative
scope: Cortex Web typed client contract for Space design preview metadata
status: passed
---

# Space Design Cortex Web Preview Client

## Summary

Added a typed Cortex Web client path for the Space design preview fixture:

- `workbenchApi.getSpaceDesignProfilePreview()` reads `/api/system/ux/space-design-profiles`.
- `spaceDesignProfilePreviewContract.ts` now exposes JSON-free types and helper functions for metadata-only interpretation.
- Contract tests assert the preview response remains recommendation-only, metadata-only, and not usable as runtime theme input.

## Boundary

This is a client/contract path only. It does not add UI rendering, runtime Space profile selection, token application, theme switching, external design system imports, or Hermes approval authority.

## Validation

```text
bash scripts/check_agent_preflight_contract.sh
PASS

bash scripts/check_dynamic_config_contract.sh
PASS

python3 scripts/check_ndl_design_profiles.py
PASS

python3 scripts/test_ndl_design_profile_a2ui_fixture_validation.py
PASS

node --experimental-strip-types --test tests/previewFixtureCatalog.test.ts tests/heapApiContract.test.ts
PASS: 24 tests

npm run test:spatial
PASS: 65 tests

npm run check
PASS

npm run build
PASS

python3 scripts/check_semantic_alignment_surfaces.py --json cortex/apps/cortex-web/src/api.ts cortex/apps/cortex-web/src/store/spaceDesignProfilePreview.ts cortex/apps/cortex-web/src/store/spaceDesignProfilePreviewContract.ts cortex/apps/cortex-web/tests/previewFixtureCatalog.test.ts cortex/apps/cortex-web/tests/heapApiContract.test.ts
PASS: errors=0 warnings=0

bash scripts/check_nostra_cortex_terminology.sh
PASS

git diff --check
PASS
```
