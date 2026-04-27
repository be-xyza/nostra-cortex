---
title: Space Design Promotion Gate Contract
captured_at: 2026-04-27T19:10:40Z
initiative: 132-eudaemon-alpha-initiative
scope: Recommendation-only promotion gate contract for Space design profiles
status: passed
---

# Space Design Promotion Gate Contract

## Summary

Added `DesignPromotionGateV1` as the next Space design contract primitive. The gate records the evidence required before a `SpaceDesignProfileV1` can request steward approval, while keeping profile selection, token application, theme switching, and allowlist reuse disabled.

The slice includes:

- `DesignPromotionGateV1.schema.json`,
- a Research Observatory draft promotion gate prototype,
- validation coverage in `scripts/check_ndl_design_profiles.py`,
- lint contract and plan updates that make the primitive discoverable.

## Boundary

This is an evidence contract, not a runtime promotion. The draft gate:

- remains `recommendation_only`,
- targets an existing recommendation-only profile,
- resolves prior source evidence refs,
- requires the full promotion evidence set,
- leaves steward approval unclaimed,
- leaves runtime activation disabled.

It does not approve the profile, apply `design_tokens`, enable runtime Space profile selection, switch Cortex Web themes, reuse runtime theme allowlists, or grant Hermes approval authority.

## Validation

```text
python3 scripts/check_ndl_design_profiles.py
PASS: Space design contract checks (1 profile(s), 1 import(s), 1 template pack(s), 1 promotion gate(s), 2 A2UI theme(s), 2 themed fixture(s))

python3 scripts/check_semantic_alignment_surfaces.py --json research/120-nostra-design-language/schemas/DesignPromotionGateV1.schema.json research/120-nostra-design-language/prototypes/space-design/research-observatory.promotion-gate.v1.json
PASS: errors=0 warnings=0

bash scripts/check_nostra_cortex_terminology.sh
PASS

git diff --check
PASS
```
