---
id: 085
name: nostra-file-infrastructure
title: 'Decision Log: Nostra File & Media Infrastructure'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Decision Log: Nostra File & Media Infrastructure

**Initiative**: 085-nostra-file-infrastructure
**Created**: 2026-01-28
**Updated**: 2026-01-28

Track architectural decisions regarding file storage and management.

---

## Decision Summary

| ID | Topic | Status |
|----|-------|--------|
| DEC-085-001 | Primary storage pattern (Tiered Hybrid) | 🟡 Proposed |
| DEC-085-002 | Tier boundaries (size thresholds) | 🟡 Proposed |
| DEC-085-003 | Versioning strategy per file type | 🟡 Proposed |
| DEC-085-004 | Draft→Publish permissions | 🟡 Proposed |
| DEC-085-005 | Quota and pricing tiers | 🟡 Proposed |
| DEC-085-006 | Explicit version chaining pattern | ✅ Decided |
| DEC-085-007 | CDN integration | ✅ Decided |
| DEC-085-008 | Deduplication & ownership attribution | ✅ Decided |
| DEC-085-009 | Quota transfer protocol | ✅ Decided |
| DEC-085-010 | Garbage collection policy | ✅ Decided |
| DEC-085-011 | Streaming DRM (vetKeys) | ✅ Decided |
| DEC-085-012 | Compression strategy | ✅ Decided |
| DEC-085-013 | Agent artifact access | ✅ Decided |

---

## DEC-085-001: Primary Storage Pattern

**Date**: 2026-01-28
**Status**: 🟡 Proposed

**Options Considered**:
1. **Single Storage Canister** — All files in one canister's stable memory
2. **Dedicated Storage Canister per Space** — Each space gets own canister
3. **Tiered Hybrid** — Hot (Asset Canister), Warm (Storage Canister), Cold (Off-chain + hash)
4. **Canister-per-Artifact** — Each file is sovereign (like ic-video-storage)

**Proposed Decision**: Option 3 — Tiered Hybrid

**Rationale**:
- Balances cost (avoid paying $0.26 per tiny file) with sovereignty (large artifacts can be isolated)
- Aligns with 011-video-streaming research conclusions
- Enables future migration between tiers based on access patterns
- Supports free tier economics

**Awaiting**: Formal approval to convert to ✅ Decided

---

## DEC-085-002: Tier Boundaries

**Date**: 2026-01-28
**Status**: 🟡 Proposed

**Decision**: Size-based tier assignment

| Tier | Size Range | Storage Location | Access Pattern |
|------|------------|------------------|----------------|
| **Hot** | < 10MB | Stable Memory / Asset Canister | Frequent |
| **Warm** | 10MB - 100MB | Dedicated Storage Canister | Periodic |
| **Cold** | > 100MB | Off-chain + Hash Anchor | Infrequent/Archival |

**Rationale**: Optimizes cost-to-access tradeoff per RESEARCH.md analysis

---

## DEC-085-003: Versioning Strategy

**Date**: 2026-01-28
**Status**: 🟡 Proposed

**Decision**: Hybrid versioning by file type

| File Type | Strategy | Rationale |
|-----------|----------|-----------|
| Text/Markdown/Code | Incremental diffs | High compression, frequent edits |
| Binary (images, PDF) | Full snapshots | Diffs not meaningful |
| Video/Audio | Full snapshots, cold-tier after 30 days | Large size, infrequent version access |

---

## DEC-085-004: Draft→Publish Permissions

**Date**: 2026-01-28
**Status**: 🟡 Proposed

**Decision**: Role-based publishing with optional approval

| Role | Can Create Draft | Can Publish | Can Approve Others |
|------|------------------|-------------|-------------------|
| Author | ✅ | ✅ (own) | ❌ |
| Editor | ✅ | ✅ | ❌ |
| Moderator | ✅ | ✅ | ✅ |
| Space Owner | ✅ | ✅ | ✅ |

**Optional Workflow**: Space can require moderator approval for publish

---

## DEC-085-005: Quota and Pricing Tiers

**Date**: 2026-01-28
**Status**: 🟡 Proposed

**Decision**: Tiered pricing with cycles-native option

| Tier | Storage | Max File | Price |
|------|---------|----------|-------|
| Free | 100MB | 10MB | $0 |
| Creator | 1GB | 50MB | ~$5/year |
| Team | 10GB | 200MB | ~$40/year |
| Enterprise | 100GB | 1GB | ~$300/year |

**Cycles Option**: Direct cycle deposits with 0.5% network fee on top-ups

---

## DEC-085-006: Explicit Version Chaining Pattern

**Date**: 2026-01-28
**Status**: ✅ Decided

**Context**: Version ordering was implicitly inferred from `versionNumber: Nat` and `createdAt: Time`, but not structurally enforced.

**Decision**: All versioned entities SHALL include explicit chain references:
- `previousVersionId: ?Text` — ID of prior version (null for v1)
- `previousChecksum: ?Text` — Checksum of prior version (null for v1)

**Rationale**:
- Makes ordering **provable**, not merely implied
- Enables tamper detection (cannot insert versions mid-chain)
- Each version cryptographically attests to its predecessor
- Aligns with "History Is Sacred" constitutional principle

**Implementation**: Applied to Contribution base type in spec.md

---

## DEC-085-007: CDN Integration

**Date**: 2026-01-28
**Status**: ✅ Decided

**Decision**: Implement optional premium CDN tier

**Implementation Phases**:
1. Integration with Cloudflare/Fastly via certified asset headers
2. BYO-CDN option for enterprise
3. Edge compute integration (transcoding, image optimization)

**Pricing**:
| Tier | Features | Cost |
|------|----------|------|
| Basic | Global caching, DDoS | +$5/space/mo |
| Pro | + Analytics, custom TTL | +$15/space/mo |
| Enterprise | + BYO-CDN, SLA | Custom |

---

## DEC-085-008: Deduplication & Ownership Attribution

**Date**: 2026-01-28
**Status**: ✅ Decided

**Decision**: Content-addressed deduplication with Split-Proportional quota model

**Key Elements**:
- Storage Owner ≠ Content Owner (separate concepts)
- Quota cost split proportionally across all referencing spaces
- Ownership disputes resolved via facilitated workflow (Nostra facilitates, does not adjudicate)
- Court rulings can be submitted and verified
- Ownership changes versioned (History Is Sacred)

**Schemas**: `BlobOwnership`, `ContentClaim`, `EvidenceRef`, `LegalRuling`, `OwnershipChange`

---

## DEC-085-009: Quota Transfer Protocol

**Date**: 2026-01-28
**Status**: ✅ Decided

**Decision**: Allow quota transfer between ANY canisters with security safeguards

**Safeguards**:
- Both controllers must approve (2-phase)
- Rate limit: 1 transfer/canister/day
- Max 50% of source quota per transfer
- 24h cooldown after receiving
- 0.1% network fee
- Full audit trail

**Schemas**: `TransferQuotaRequest`, `TransferQuotaResponse`

---

## DEC-085-010: Garbage Collection Policy

**Date**: 2026-01-28
**Status**: ✅ Decided

**Decision**: Space-configurable retention with governance hard limits

| Setting | Range | Default |
|---------|-------|---------|
| Draft retention | 1-90 days | 30 days |
| Archive retention | 30-365 days | 90 days |

**Constitutional Override**: Published versions NEVER hard-deleted

---

## DEC-085-011: Streaming DRM

**Date**: 2026-01-28
**Status**: ✅ Decided

**Decision**: Implement vetKeys-based segment encryption for premium content

**Architecture**:
- Each video segment encrypted with unique derived key
- Key release gated by AccessProof (payment, subscription, NFT, license)
- Access tiers: Preview, Ads, Pay-Per-View, Subscription, NFT, Licensed

**Implementation Phases**:
1. Access gating (Q2)
2. Segment encryption with vetKeys (Q3)
3. Pay-per-view integration (Q3)
4. Watermarking + anti-piracy (Q4)
5. Offline viewing (Q4+)

**Dependency**: vetKeys API stability (currently beta)

---

## DEC-085-012: Compression Strategy

**Date**: 2026-01-28
**Status**: ✅ Decided

**Decision**: Selective compression by MIME type before cold-tier migration

| File Type | Compress? | Algorithm |
|-----------|-----------|-----------|
| Text/JSON/Markdown | ✅ | zstd |
| Source code | ✅ | zstd |
| PNG | ⚠️ Optional | lossless recompress |
| JPEG/WebP | ❌ | Already compressed |
| Video/Audio | ❌ | Already compressed |

**Metadata**: Store `compressionAlgorithm` in artifact metadata; decompress transparently on retrieval

---

## DEC-085-013: Agent Artifact Access

**Date**: 2026-01-28
**Status**: ✅ Decided

**Decision**: Agents CAN create/modify artifacts with workflow safeguards

**Safeguards**:
1. Workflow-gated: Require active workflow context
2. Permission scopes: Explicit `artifact:create`, `artifact:modify` permissions
3. Draft-only default: Publishing requires human approval
4. Audit trail: All agent operations logged with `source: "agent"`
5. Rate limits: Per-agent daily limits configurable
6. Rollback: 24h revert window without version history pollution

**Schema**: `AgentArtifactPermission`

---

## Cross-Initiative References

| Topic | Related Research |
|-------|------------------|
| Video patterns | [011-tech-stack-video-streaming](../011-tech-stack-video-streaming/RESEARCH.md) |
| Artifact taxonomy | [008-nostra-contribution-types](../008-nostra-contribution-types/DECISIONS.md) |
| Embedding storage | [041-nostra-vector-store](../041-nostra-vector-store/RESEARCH.md) |
| Knowledge integration | [037-nostra-knowledge-engine](../037-nostra-knowledge-engine/RESEARCH.md) |
| Version chaining (spec.md) | [002-nostra-v2-architecture](../002-nostra-v2-architecture/DECISIONS.md) |
