---
id: ''
name: baml
title: BAML Experiments (Initiative 118)
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-02-15'
updated: '2026-02-15'
---

# BAML Experiments (Initiative 118)

Artifacts generated on 2026-02-15 for the BAML intake follow-up experiments:

- `rust_typed_function_contract_prototype.rs`
  - Rust-only prototype of typed function contracts with retry + fallback + round-robin primitives.
- `BAML_RETRY_FALLBACK_COMPARISON_2026-02-15.md`
  - Behavior comparison between BAML runtime strategies and current Cortex workflow behavior.
- `BAML_SIGNATURE_MAPPING_NOTE_2026-02-15.md`
  - Mapping from BAML-style function signatures to Nostra workflow/canister contract surfaces.

## Local Verification
Run:

```bash
rustc --edition 2021 --test \
  /Users/xaoj/ICP/research/118-cortex-runtime-extraction/experiments/baml/rust_typed_function_contract_prototype.rs \
  -o /tmp/baml_typed_contract_tests \
  && /tmp/baml_typed_contract_tests
```
