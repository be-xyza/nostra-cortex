# Decisions: Protected Resources and Secret Egress

## DEC-138-001: Treat AI-visible secret exposure as compromise

**Date**: 2026-05-01

**Decision**: Any live provider key, credential, token, private key, or sensitive personal identifier exposed through terminal output, AI-visible transcript, CI log, screenshot, generated artifact, or runtime diagnostic is considered compromised and must be rotated or revoked.

**Rationale**: AI/tool transcripts and generated evidence can be copied, indexed, retained, or replayed beyond the operator's intended trust boundary. Prompt instructions are not a reliable containment boundary.

**Implications**:
- Debugging must use redacted inspection tools.
- Incident response must prioritize rotation and audit before deeper architecture work.
- Runtime and CI surfaces must prefer fingerprints and redacted previews over raw values.

## DEC-138-002: Egress controls precede vault construction

**Date**: 2026-05-01

**Decision**: The first implementation slice will focus on redacted diagnostics and secret egress gates before building a full protected-resource vault.

**Rationale**: The immediate risk is accidental disclosure through existing operational surfaces. A vault reduces storage risk, but it does not by itself stop secrets from leaking through terminal, logs, CI artifacts, or upstream error envelopes.

**Implications**:
- Phase 1 delivers safe operator inspection.
- Phase 2 delivers scanning and CI protection.
- Nostra protected-resource primitives follow after current egress surfaces are controlled.

