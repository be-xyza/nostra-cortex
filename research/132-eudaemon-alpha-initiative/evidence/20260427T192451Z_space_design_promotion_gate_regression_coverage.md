---
title: Space Design Promotion Gate Regression Coverage
captured_at: 2026-04-27T19:24:51Z
initiative: 132-eudaemon-alpha-initiative
scope: Negative regression coverage for Space design promotion gates
status: passed
---

# Space Design Promotion Gate Regression Coverage

## Summary

Extended the Space design fixture regression harness with negative coverage for `DesignPromotionGateV1` drift.

The new cases assert that the checker rejects:

- promotion gates missing required evidence,
- promotion gates referencing unresolved evidence files,
- draft gates that record `approved_by` before approval,
- gates that enable runtime profile selection.

## Boundary

This keeps the promotion gate as evidence only. It does not approve a profile, activate runtime Space profile selection, apply profile tokens, switch themes, reuse runtime theme allowlists, or grant Hermes approval authority.

## Validation

```text
python3 scripts/test_ndl_design_profile_a2ui_fixture_validation.py
PASS: Space design A2UI fixture validation regression coverage

python3 scripts/check_ndl_design_profiles.py
PASS: Space design contract checks (1 profile(s), 1 import(s), 1 template pack(s), 1 promotion gate(s), 2 A2UI theme(s), 2 themed fixture(s))

python3 -m py_compile scripts/test_ndl_design_profile_a2ui_fixture_validation.py scripts/check_ndl_design_profiles.py
PASS
```
