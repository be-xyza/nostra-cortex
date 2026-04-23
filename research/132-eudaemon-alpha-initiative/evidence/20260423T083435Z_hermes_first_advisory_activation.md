# Hermes First Advisory Activation - 2026-04-23

## Context

This evidence records the first bounded Hermes advisory activation pass for Initiative 132 after the Hermes readiness language was promoted to root `ICP` and pushed on `main`.

Root authority commit:

- `5c118c8d Document Hermes advisory activation readiness`

Activation workspace:

- `/Users/xaoj/hermes`

Runtime posture:

- Hermes was run from the activation workspace with `HERMES_HOME=/Users/xaoj/hermes/runtime-home`.
- `/Users/xaoj/hermes/.hermes.md` was treated as the active guardrail surface.
- `~/.hermes` SOUL/profiles/config remained non-authoritative convenience state.

## Inputs

The pass used only explicit local planning inputs:

- `/Users/xaoj/hermes/manifests/initiative-132-authority-source-manifest.v1.json`
- `/Users/xaoj/hermes/audit-units/initiative-132-hermes-activation-gap-review.v1.json`

The manifest resolved the governed root `ICP` source set with no request-worktree authority paths.

## Operator Validation Context

The pass was bounded by the following current validation results:

- Root `ICP` clean and pushed at `5c118c8d`.
- Research portfolio consistency: PASS.
- Nostra/Cortex terminology: PASS.
- VPS runtime authority: PASS.
- Dynamic config contract: PASS.
- Gateway parity inventory sync: PASS (`inventory=240`, `fixtures=240`, `exemptions=0`).
- Hermes source manifest/source bundle parity: PASS (`10` root `ICP` sources, `0` worktree refs).
- `HermesObserverSessionV1` exact-shape validation: PASS.

## Outputs

Hermes produced exactly two local activation artifacts:

- `/Users/xaoj/hermes/sessions/initiative-132-first-advisory-pass.session.json`
- `/Users/xaoj/hermes/artifacts/synthesis/initiative-132-first-advisory-pass.md`

The synthesis result was PASS after local wording cleanup to avoid implying a pre-authorized broader execution-surface follow-on.

## Findings

Hermes found no unresolved contradictions across the active Initiative 132 authority sources, source manifest, audit unit, and operator validation context.

The only finding was managed historical lineage drift: older March 2026 decision text referenced a Python Phase 6 worker, while the current 2026-04-23 Initiative 132 authority confirms the active VPS contract is `cortex-gateway` plus Rust `cortex_worker`. This drift is already bounded by the latest README and DECISIONS entries and does not block activation.

## Guardrails Confirmed

The activation pass did not authorize:

- provider batch submission or polling
- queue runners
- execution adapters
- external action loops
- repository mutation by Hermes
- runtime mutation by Hermes
- broader execution-surface extraction

Hermes remains a local advisory meta-observer. Its outputs are recommendation-only and require steward promotion through governed Initiative 132 surfaces before they can influence durable authority.
