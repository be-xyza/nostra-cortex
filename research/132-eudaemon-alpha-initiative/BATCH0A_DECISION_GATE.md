# Initiative 132 Batch 0A Decision Gate

**Date**: 2026-03-31
**Outcome**: `Hold in Batch 0A`

## Decision

Batch 0A reduced contract drift on the targeted heap, provider-runtime, lifecycle, and ACP boundaries, but Initiative 132 should not advance into Batch 1 provider-runtime extraction yet.

## Why

1. The touched-surface verification passed:
   - heap emission now normalizes the public `space_id` boundary while preserving `workspace_id` as an intentional compatibility alias at the web edge
   - provider-runtime SSE parsing now uses typed response envelopes for completed outputs and text deltas instead of depending on ad hoc raw `Value` matching alone
   - lifecycle payload expectations are now explicitly tested for `camelCase` serialization
   - ACP request params now reject silent legacy snake_case drift instead of ignoring unknown aliases
2. The broader extraction gate is still red:
   - the governed `gateway_parity` suite was already failing before Batch 0A because the inventory fixtures and parity cases in the worktree are out of sync
   - Initiative 132's parity policy says no extraction phase should advance while that suite is failing
3. The current evidence says the remaining parity debt is pre-existing and unrelated to the Batch 0A edits, but it still blocks the next extraction batch as a governance matter.

## Answers Required By The Gate

### Is provider runtime ready for extraction?

Not yet.

The boundary is clearer than it was at the start of Batch 0A, especially around typed SSE envelopes, but extraction should wait until the parity gate is green again and the app-layer startup wiring can be evaluated without outstanding baseline noise.

### Did ACP contract work uncover a broader execution-control boundary that should move next?

Yes, but not urgently enough to skip the parity gate.

The ACP hardening confirmed that silent parameter alias drift was real maintenance risk. That strengthens the case for the ACP / terminal execution-control seam as a later extraction target, but it does not justify starting that extraction in the same stage.

### Did any workbench or heap projection issues surface that must be solved before Batch 1?

Yes.

Heap emission and heap projection still carry compatibility baggage across `space_id`, `workspace_id`, and `workspaceId`. Batch 0A normalized the network edge, but the presence of those aliases means workbench and heap projection callers still need a narrower compatibility cleanup before extraction work starts.

## Evidence

### Baseline

- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --test gateway_parity`
  - failed before Batch 0A with fixture-count and replay-count mismatches
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --test acp_gateway_integration`
  - passed with the integration test ignored in this environment
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --test acp_staging_operationalization`
  - passed with the staging drill ignored in this environment
- `bash /Users/xaoj/ICP/scripts/check_vps_runtime_authority.sh --repo-contract`
  - passed

### Batch 0A Targeted Verification

- `node --experimental-strip-types --test /Users/xaoj/ICP/cortex/apps/cortex-web/tests/heapRelationEditorContract.test.ts /Users/xaoj/ICP/cortex/apps/cortex-web/tests/heapApiContract.test.ts`
  - passed
- `node --experimental-strip-types --test /Users/xaoj/ICP/cortex/apps/cortex-web/tests/chatSocketContract.test.ts`
  - passed
- `./node_modules/.bin/tsc --noEmit`
  - passed
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --lib emits_lifecycle_jsonl`
  - passed
- `cargo test --manifest-path /Users/xaoj/ICP/cortex/Cargo.toml -p cortex-eudaemon --lib session_prompt_and_load_preserve_ordering`
  - passed
- targeted unit filters passed for:
  - provider-runtime typed SSE parsing
  - lifecycle `camelCase` serialization
  - ACP snake_case alias rejection
  - heap `space_id` alias coverage on both mapper and gateway tests

## Next Step For This Stage

Stay in Batch 0A until the `gateway_parity` fixture inventory debt is repaired and rerun cleanly. Once that gate is green, reassess Batch 1 starting with provider-runtime extraction only.
