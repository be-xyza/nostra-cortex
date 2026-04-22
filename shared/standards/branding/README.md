# Branding Standards

This directory contains the governed branding contracts and the runtime policy/state data used by Nostra Cortex branding surfaces.

## Canonical Artifacts

- `brand_policy.schema.json`
- `brand_policy_v1.json`
- `brand_visual_state_cases_v1.json`

## Contract Roles

1. `brand_policy.schema.json` defines the validation shape for branding policy.
2. `brand_policy_v1.json` is the governed policy payload consumed by brand-aware renderers and runtime surfaces.
3. `brand_visual_state_cases_v1.json` captures the expected rendering cases used to validate policy interpretation.

## Discovery Notes

- Schema files are validation contracts.
- Versioned JSON files are governed artifacts, not schemas.
- This directory should remain additive and backward compatible.

## References

- `shared/standards/README.md`
- `shared/standards/branding/brand_policy.schema.json`
- `shared/standards/branding/brand_policy_v1.json`
- `shared/standards/branding/brand_visual_state_cases_v1.json`
