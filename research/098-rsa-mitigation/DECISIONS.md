---
id: "098-rsa-mitigation-decisions"
name: "rsa-mitigation-decisions"
title: "Decision Log: RSA Risk Mitigation"
type: "decision"
project: "nostra"
status: active
authors:
  - "User"
  - "Codex"
tags: [security, crypto, remediation]
created: "2026-02-04"
updated: "2026-02-05"
---

# Decision Log: RSA Risk Mitigation

Track architectural decisions with rationale for future reference.

---

## DEC-001: Initiate RSA Risk Mitigation
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Accept risk without remediation
2. Start a dedicated mitigation initiative

**Decision**: Start a dedicated mitigation initiative (`research/098-rsa-mitigation`).

**Rationale**: The vulnerability has no upstream fix; migration requires coordinated design and rollout.

**Implications**: All crypto migration planning and decisions are tracked in this initiative.

---

## DEC-002: Target Replacement Primitive
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Ed25519
2. P‑256 (NIST)
3. Secp256k1

**Decision**: Use HPKE with X25519 + HKDF‑SHA256 + ChaCha20‑Poly1305 for asymmetric encryption of stored keys.

**Rationale**: HPKE provides modern, standardized public‑key encryption suitable for small payloads (API keys), avoids RSA timing risks, and is efficient in WASM. The chosen suite balances compatibility and performance.

**Implications**: Drives the crypto abstraction API, backend key metadata (`alg`/`enc_version`), and migration logic. Requires verifying crate compatibility with `wasm32-unknown-unknown`.

---

## DEC-003: Migration Strategy & Timeline
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Direct cutover (break compatibility)
2. Dual‑stack with lazy re‑encryption

**Decision**: Dual‑stack with lazy re‑encryption and a 30‑day RSA deprecation window.

**Rationale**: Preserves user workflows, avoids mass re‑encryption, and allows gradual migration while maintaining service continuity.

**Implications**:
- Worker must decrypt both RSA and HPKE during the window.
- On successful RSA decrypt, worker re‑encrypts with HPKE and updates backend.
- Frontend encrypts with HPKE by default and falls back to RSA if HPKE key unavailable.

---

## DEC-004: HPKE Envelope Metadata + Worker Update API
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Store only ciphertext (no metadata)
2. Store ciphertext + `alg` + `enc_version` + `key_id` + `ephemeral_pub_key`

**Decision**: Store full HPKE envelope metadata and add a worker‑only `updateKeyV2ForUser` API.

**Rationale**: Enables algorithm agility, prevents downgrade ambiguity, and allows lazy re‑encryption without duplicating keys.

**Implications**: Backend schema and Candid contracts require updates; worker performs in‑place updates after RSA decrypt.

---

## DEC-005: Legacy Usage Telemetry + RSA Decrypt Kill Switch
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Rely only on worker logs (no backend telemetry)
2. Add explicit backend logging and an operator‑controlled RSA disable flag

**Decision**: Add a worker‑only backend method to record legacy RSA usage, and introduce an env‑flag (`NOSTRA_DISABLE_RSA_DECRYPT`) to disable RSA decrypt on the worker when ready.

**Rationale**: We need observability to know when RSA is truly unused, and a safe, reversible control to disable legacy decryption at cutover without code changes.

**Implications**: Backend logs include legacy‑usage events; worker behavior can be switched to HPKE‑only by configuration.

---

## DEC-006: Frontend RSA Encrypt Kill Switch
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Remove RSA encryption immediately
2. Add a frontend toggle to block RSA encryption when ready

**Decision**: Add `NOSTRA_DISABLE_RSA_ENCRYPT=1` to require HPKE in the frontend (no RSA fallback).

**Rationale**: Provides a safe, reversible cutover control while telemetry is still being collected.

**Implications**: Operators can enforce HPKE‑only encryption without code changes; RSA remains available until full deprecation.

---

## DEC-007: Remove RSA Dependencies and Code Paths
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Keep RSA dual‑stack indefinitely
2. Remove RSA dependencies and enforce HPKE‑only

**Decision**: Remove RSA dependencies and all RSA encrypt/decrypt code paths in frontend and worker.

**Rationale**: The vulnerability has no upstream fix; HPKE‑only reduces risk and simplifies the crypto surface.

**Implications**: Legacy RSA‑encrypted keys can no longer be decrypted; rollback requires code revert or re‑introducing RSA support.

---

## DEC-008: Replace `dotenv` With `dotenvy` in Worker
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Keep `dotenv` (unmaintained warning)
2. Replace with `dotenvy`

**Decision**: Replace `dotenv` with `dotenvy` in the worker.

**Rationale**: Removes an unmaintained dependency warning with minimal behavioral change.

**Implications**: Worker binaries use `dotenvy::dotenv()` for local env loading; lockfile updated.

---

## DEC-009: Upgrade `reqwest` to 0.12 in Worker + Ingest + Desktop
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Leave `reqwest` at 0.11
2. Upgrade to 0.12 where directly depended

**Decision**: Upgrade `reqwest` to 0.12 in `cortex_worker`, `ingest_gaia`, and `cortex-desktop`.

**Rationale**: Aligns on a newer `reqwest` release and reduces exposure to older dependency chains.

**Implications**: Audit warnings tied to `reqwest 0.11` remain via `ic-agent 0.34`; resolving fully requires upgrading `ic-agent`.

---

## DEC-010: Upgrade `ic-agent` to 0.45 Across Workspace
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Keep older `ic-agent` versions
2. Upgrade to a single newer version across workspace crates

**Decision**: Upgrade `ic-agent` to 0.45 for `frontend`, `cortex_worker`, and `nostra-test-kit`.

**Rationale**: Aligns dependency versions and removes the older `reqwest 0.11` chain while keeping API changes manageable.

**Implications**: Audit warnings reduced (notably `rustls-pemfile` removed); remaining warnings are tied to `backoff`, `instant`, `serde_cbor`, and GTK3 bindings.

---

## DEC-011: Upgrade `rfd` to 0.17 in Cortex Desktop
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Keep `rfd` 0.12 (GTK3 dependency chain)
2. Upgrade to `rfd` 0.17 (newer defaults, fewer GTK3 deps)

**Decision**: Upgrade `rfd` to 0.17 in `cortex-desktop`.

**Rationale**: Removes GTK3 binding warnings from the audit surface.

**Implications**: Audit warning count drops; `cortex-desktop` build should be revalidated for any UI API regressions.

---

## DEC-012: Replace `bincode` With `postcard` in Workflow Engine Storage
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Keep `bincode` (unmaintained)
2. Replace with `postcard` (maintained, serde‑compatible)

**Decision**: Replace `bincode` serialization with `postcard` for `StorableWorkflow` and `VfsFile`.

**Rationale**: Removes an unmaintained dependency warning while keeping serialization lightweight and deterministic.

**Implications**: Stored stable‑memory payloads now include a magic header and `postcard` format. Existing `bincode` data is not readable without migration and will trap on access; operators should migrate or reinitialize stable memory before deploying this change. `postcard` is configured without default features to avoid the `atomic-polyfill` warning.

---

## DEC-013: Accept Remaining Upstream Unmaintained Warnings (Temporary)
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Attempt to fork/patch upstream crates immediately
2. Accept warnings temporarily and monitor upstream updates

**Decision**: Accept remaining unmaintained warnings for `backoff`, `instant`, `paste`, and `serde_cbor` as upstream constraints, and track for future upgrades.

**Rationale**: These warnings are transitive from `ic-agent` and `candid`/`pocket-ic`. Replacing them safely would require upstream alignment or broader dependency changes beyond the RSA remediation scope.

**Implications**: No functional changes now; we will monitor upstream releases and reassess if vulnerabilities appear or if the dependencies become blocking.

---

## DEC-014: Require Stable Memory Migration Before Postcard Upgrade
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Deploy postcard changes directly (risking traps on legacy data)
2. Require migration or reinstall for stateful environments

**Decision**: Require a migration (or reinstall where safe) before deploying the postcard storage format to stateful environments.

**Rationale**: Legacy `bincode` data cannot be decoded after the switch and would cause runtime traps.

**Implications**: Operational playbook must include a migration step or planned reinitialization before rollout.

---

## DEC-015: Provide Legacy Migration Build Flag + Entry Point
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Manual rebuild using ad‑hoc scripts
2. Explicit feature‑flagged migration build with a canister method

**Decision**: Add a `legacy-migration` feature and `migrate_legacy_storage` update method to convert `bincode` data to `postcard`.

**Rationale**: Provides a repeatable, auditable migration path without exposing legacy decode paths in normal builds.

**Implications**: Operators must use a migration build for stateful upgrades and then deploy the standard build after conversion. The migration build reintroduces `bincode` as an optional dependency.

---

## DEC-016: Accept `bincode` Warning as Migration‑Only Dependency
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Avoid `bincode` entirely (no in‑place migration path)
2. Use `bincode` behind a feature for legacy conversion

**Decision**: Keep `bincode` as an optional dependency under `legacy-migration` to enable safe conversion.

**Rationale**: A migration path is required for stateful environments; `bincode` is only used in the temporary migration build.

**Implications**: `cargo-audit` will still surface the `bincode` warning due to lockfile inclusion; treat it as a migration‑only warning and remove after all stateful environments migrate.

---

## DEC-017: Add Chunked Migration Entry Point
**Date**: 2026-02-04
**Status**: ✅ Decided

**Options Considered**:
1. Keep a single `migrate_legacy_storage` call for all data
2. Add a chunked migration method with a per‑call limit

**Decision**: Add `migrate_legacy_storage_chunk(max_items)` and make `migrate_legacy_storage` run a conservative default chunk.

**Rationale**: The unbounded migration call can hang or time out in non‑interactive or constrained environments. Chunking limits per‑call work and allows operators to iterate safely.

**Implications**: Operators must repeat the chunk call until it returns `(0, 0)`; runbooks and checklists are updated to reflect the chunked workflow.

---

## DEC-018: Use Named Fields for Migration Counts
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Return positional tuples `(u64, u64)`
2. Return a named record `{ workflows: nat64; vfs_files: nat64 }`

**Decision**: Return a named record for migration counts.

**Rationale**: Named fields make dfx output readable and prevent ambiguous type-only output when the interface changes.

**Implications**: Candid interfaces and generated bindings must be updated; operators will see `workflows` and `vfs_files` in migration output.

---

## DEC-019: Add Canister ID Helper Script
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Manual lookup in `.dfx/local/canister_ids.json`
2. Add a small helper script to resolve IDs

**Decision**: Add `nostra/scripts/print_canister_id.sh` to resolve local canister IDs using `dfx` or the `.dfx` file.

**Rationale**: Avoids manual ID hunting and reduces operator errors during migrations.

**Implications**: Operators can run `scripts/print_canister_id.sh workflow-engine` to get the active local ID.

---

## DEC-020: Add `canister_ids.json` Template for Non‑Local Networks
**Date**: 2026-02-05
**Status**: ✅ Decided

**Options Considered**:
1. Keep IDs only in local `.dfx` data
2. Provide a project‑level `canister_ids.json` for staging/ic

**Decision**: Add `nostra/canister_ids.json` with placeholders for staging/ic.

**Rationale**: Centralizes non‑local canister IDs and enables helper scripts to resolve IDs without manual lookup.

**Implications**: Operators must populate staging/ic values before running non‑local migrations.

---

## DEC-021: Remove Legacy Migration Feature for V1 Greenfield
**Date**: 2026-02-05
**Status**: ✅ Decided

**Supersedes**: DEC-014, DEC-015, DEC-016, DEC-017, DEC-018 (legacy migration requirements and bincode feature).

**Options Considered**:
1. Keep the legacy migration feature and runbooks in case future data appears.
2. Remove the legacy migration feature for V1 and document a reintroduction path if needed.

**Decision**: Remove the `legacy-migration` feature, migration entry points, and `bincode` dependency for the greenfield V1. Archive migration runbooks and templates.

**Rationale**: There is no legacy data to migrate for V1, and keeping migration code adds risk, maintenance overhead, and audit noise without benefit.

**Implications**:
- The workflow-engine interface no longer exposes migration methods.
- Operators should use canister reinstall for clean-state resets in disposable environments.
- If legacy data is introduced later, a new initiative must reintroduce a feature-flagged migration build and update runbooks.
