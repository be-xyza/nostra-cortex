# ICP Best Practices

This note captures the baseline best-practice guidance currently relied on by `AGENTS.md`.

## Safety

1. Use bounded iteration in canisters and workflow loops.
2. Avoid hardcoded canister IDs; prefer environment or dynamic lookup.
3. Prefer recommendation-only mode when authority is unclear.
4. Keep destructive actions behind explicit approval.

## Cycles

1. Set conservative per-call cycle limits.
2. Use `freezing_threshold` defensively for production canisters.
3. Prefer micro-batching for expensive indexing or graph work.

## Contracts

1. Treat Candid `.did` files as public interface truth.
2. Keep Motoko and Rust bindings aligned with contract changes.
3. Prefer structured logs and durable evidence for production diagnostics.

## Reliability

1. Preserve replayable state and deterministic event identities where possible.
2. Archive governance and research files before updating them.
3. Fix broken internal references when discovered instead of documenting around them.
