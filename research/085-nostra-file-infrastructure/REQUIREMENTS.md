---
id: 085
name: nostra-file-infrastructure
title: 'REQUIREMENTS: Nostra File & Media Infrastructure'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# REQUIREMENTS: Nostra File & Media Infrastructure

**Initiative**: 085-nostra-file-infrastructure
**Status**: RESEARCH COMPLETE
**Created**: 2026-01-28
**Updated**: 2026-01-28

---

## Functional Requirements

### FR-001: Artifact Upload
- System SHALL accept file uploads up to configured maximum size
- System SHALL validate MIME type against allowed list
- System SHALL compute SHA-256 checksum during upload
- System SHALL assign appropriate storage tier based on file size
- System SHALL detect duplicate content via hash matching

### FR-002: Draft Management
- System SHALL store drafts as mutable, private artifacts
- System SHALL allow draft replacement without versioning
- System SHALL allow draft deletion by artifact owner
- System SHALL restrict draft visibility to author only
- System SHALL enforce configurable draft retention period (1-90 days)

### FR-003: Publishing Workflow
- System SHALL transition drafts to published state via `publish()` call
- System SHALL create immutable version record upon publish
- System SHALL anchor checksum on-chain at publish time
- System SHALL record `publishedAt` timestamp
- System SHALL enforce role-based publishing permissions
- System SHALL support optional moderator approval workflow

### FR-004: Version Management
- System SHALL maintain ordered list of versions per artifact with explicit chaining
- System SHALL store `previousVersionId` and `previousChecksum` per version
- System SHALL allow retrieval of any historical version
- System SHALL support version comparison (for text documents)
- System SHALL migrate old versions to cold tier after configurable period
- System SHALL verify version chain integrity on request

### FR-005: Storage Quota
- System SHALL track storage usage per space
- System SHALL enforce upload blocking when quota exceeded
- System SHALL notify space owner at 80% and 100% quota usage
- System SHALL support quota upgrades via subscription or cycles payment
- System SHALL support quota transfer between canisters with approval

### FR-006: Retrieval & Streaming
- System SHALL serve small files via direct download
- System SHALL serve large files via chunked streaming
- System SHALL support range requests for video/audio (HLS/DASH compatible)
- System SHALL verify checksum on download for off-chain files
- System SHALL support CDN origin integration for hot-tier content

### FR-007: Deduplication & Ownership
- System SHALL deduplicate identical files across spaces via content addressing
- System SHALL maintain separate Storage Owner and Content Owner records
- System SHALL support multiple content claims per blob
- System SHALL notify all claimants when duplicate detected
- System SHALL provide dispute resolution workflow with escalation paths
- System SHALL support court ruling submission and verification
- System SHALL version all ownership changes (never delete)

### FR-008: Cycles-Based Payment
- System SHALL accept direct cycle deposits for storage funding
- System SHALL apply network fee (0.5%) on top-ups
- System SHALL track cycle balance per space
- System SHALL burn cycles daily based on storage usage
- System SHALL provide 7-day grace period after cycle exhaustion

### FR-009: Cross-Space Sharing
- System SHALL allow artifacts to be referenced (not copied) by other spaces
- System SHALL maintain ownership in origin space
- System SHALL create graph edge for cross-space references
- System SHALL enforce origin space visibility rules on references

### FR-010: Agent Access
- System SHALL allow Cortex agents to create/modify artifacts
- System SHALL require active workflow context for agent operations
- System SHALL enforce explicit permission scopes per agent
- System SHALL default agent-created artifacts to draft state
- System SHALL log all agent operations with `source: "agent"`
- System SHALL enforce per-agent daily operation limits
- System SHALL allow 24h rollback for agent-created artifacts

### FR-011: Streaming DRM (Phase 6)
- System SHALL support segment-based video encryption
- System SHALL gate key release on AccessProof validation
- System SHALL support access tiers: Preview, Pay-Per-View, Subscription, NFT, Licensed
- System SHALL integrate with ICP ledger for payment verification
- System SHALL log all key requests for audit trail
- System SHALL enforce rate limiting on key requests

### FR-012: CDN Integration (Phase 6)
- System SHALL support optional CDN tier per space
- System SHALL provide certified asset headers for CDN origin validation
- System SHALL support configurable cache TTL per artifact type
- System SHALL provide access analytics for CDN-enabled content

### FR-013: Compression
- System SHALL auto-detect compressible MIME types
- System SHALL compress eligible files before cold-tier migration
- System SHALL store compression algorithm in artifact metadata
- System SHALL decompress transparently on retrieval

---

## Non-Functional Requirements

### NFR-001: Performance
- Upload throughput: ≥1MB/s for files <100MB
- Download latency: <500ms for first byte (hot tier)
- Version listing: <100ms for <1000 versions
- Deduplication check: <50ms per upload

### NFR-002: Scalability
- Support 10GB storage per space (free tier + upgrades)
- Support 10,000 artifacts per space
- Support 100 concurrent uploads per canister
- Support 1M+ deduplicated blobs system-wide

### NFR-003: Integrity
- All published artifacts SHALL have on-chain checksum
- Off-chain files SHALL be verifiable against on-chain hash
- No data loss during canister upgrade
- Version chain SHALL be cryptographically verifiable

### NFR-004: Privacy
- Draft artifacts SHALL be accessible only to author
- Published artifacts SHALL respect space visibility settings
- Deleted artifacts SHALL be archived, not destroyed
- Deduplication SHALL NOT leak existence information across spaces

### NFR-005: Economics
- Storage cost SHALL be within 2x of baseline ICP stable memory rates
- Cycle consumption SHALL be predictable and documented
- Free tier SHALL be sustainable without subsidy
- Network fees SHALL cover operational costs

### NFR-006: Ownership & Disputes
- Duplicate detection latency: <5 seconds
- Dispute workflow response time: <24 hours (notification)
- Evidence submission window: 30 days minimum
- Ownership change propagation: <1 minute

---

## Constraints

### C-001: ICP Limits
- Canister heap: 4GB
- Canister stable memory: 400GB
- Message size: 2MB
- Instruction limit: 40B per call (with DTS)

### C-002: Constitutional Alignment
- "History Is Sacred": All published versions must be preserved; ownership changes versioned
- "Modular Persistence": Storage concerns isolated from contribution logic
- "Spaces Are Sovereign": Quotas and visibility per-space

### C-003: Integration Points
- MUST integrate with existing Artifact contribution type (008)
- MUST support knowledge graph relationships (037)
- MUST align with video streaming patterns (011)
- MUST integrate with version chaining in Contribution base type (spec.md)

### C-004: External Dependencies
- vetKeys API (beta) required for DRM features
- CDN provider integration (Cloudflare/Fastly) for CDN tier
- External rights databases for verification (ISRC, ISBN, DOI)

---

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| AC-001 | Upload 10MB file, verify checksum | Integration test |
| AC-002 | Create draft, edit, publish, verify version history with chaining | Integration test |
| AC-003 | Exceed quota, verify upload blocked | Integration test |
| AC-004 | Download file, verify checksum match | Integration test |
| AC-005 | Stream video with range requests | Manual + load test |
| AC-006 | Upgrade canister, verify no data loss | Upgrade test |
| AC-007 | Upload duplicate file, verify deduplication and notification | Integration test |
| AC-008 | Submit ownership dispute, verify workflow triggers | Workflow test |
| AC-009 | Transfer quota between spaces, verify safeguards | Integration test |
| AC-010 | Agent creates artifact, verify draft-only and audit log | Integration test |
| AC-011 | Pay-per-view purchase, verify access granted and expires | Integration test |
| AC-012 | CDN-enabled space, verify origin headers | Manual test |
| AC-013 | Cold-tier migration, verify compression applied | Integration test |
