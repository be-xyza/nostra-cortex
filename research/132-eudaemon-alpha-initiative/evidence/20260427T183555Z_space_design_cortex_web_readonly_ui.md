---
title: Space Design Cortex Web Read-Only UI
captured_at: 2026-04-27T18:35:55Z
initiative: 132-eudaemon-alpha-initiative
scope: Read-only Cortex Web surface for Space design preview metadata
status: passed
---

# Space Design Cortex Web Read-Only UI

## Summary

Added a compact read-only Space design preview panel to the Cortex Web Spaces page. The panel reads the metadata-only preview fixture through `workbenchApi.getSpaceDesignProfilePreview()` and displays the draft recommendation boundary without applying profile tokens.

The UI shows:

- profile id and version
- draft review status
- allowed surface scope
- metadata-only boundary copy

## Guardrails

- The render model reports `metadata_only` only when runtime binding is `none`, token application is false, and runtime theme selection is false.
- Runtime-style fixture drift is modeled as `blocked`.
- The panel does not read or render `design_tokens`.

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

node --experimental-strip-types --test tests/spaceDesignProfilePreviewPanel.test.ts
PASS: 2 tests

npm run test:spatial
PASS: 67 tests

npm run check
PASS

npm run build
PASS

python3 scripts/check_semantic_alignment_surfaces.py --json cortex/apps/cortex-web/package.json cortex/apps/cortex-web/src/components/spaces/SpacesPage.tsx cortex/apps/cortex-web/src/components/spaces/SpaceDesignProfilePreviewPanel.tsx cortex/apps/cortex-web/src/components/spaces/spaceDesignProfilePreviewModel.ts cortex/apps/cortex-web/tests/spaceDesignProfilePreviewPanel.test.ts
PASS: errors=0 warnings=0

bash scripts/check_nostra_cortex_terminology.sh
PASS

git diff --check
PASS
```

## Boundary

This is a read-only metadata display. It does not add runtime Space profile selection, direct token wiring, theme switching, external design system imports, or Hermes approval authority.
