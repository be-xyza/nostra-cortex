import { useCallback, useEffect, useState } from "react";

import type {
  EmitHeapBlockRequest,
  HeapBlockListItem,
  SpaceCreateRequest,
  SpaceCreateResponse,
} from "../../contracts.ts";
import {
  resolveSpaceArchetypeProfile,
  resolveSpaceStudioTemplate,
  SPACE_STUDIO_TEMPLATES,
  type SpaceStudioTemplate,
} from "./spaceArchetypeProfiles.ts";
import { buildHeapArtifactHref } from "../heap/heapArtifactRouting.ts";

const STORAGE_KEY = "cortex.space-studio.drafts.v1";

export { SPACE_STUDIO_TEMPLATES, type SpaceStudioTemplate };

export type DraftSpaceSource = "blank" | "template" | "reference";
export type DraftSpaceStatus = "draft" | "submitted" | "promoted";
export type DraftSpaceGovernanceScope = "personal" | "private" | "public";
export type DraftReviewLane = "private_review" | "public_review";

export interface DraftSpaceV1 {
  schemaVersion: "1.0.0";
  draftId: string;
  title: string;
  purpose: string;
  owner: string;
  accessSummary: string;
  proposedSpaceId: string;
  sourceMode: DraftSpaceSource;
  governanceScope: DraftSpaceGovernanceScope;
  templateId?: string;
  referenceUri?: string;
  lineageNote?: string;
  archetype?: string;
  status: DraftSpaceStatus;
  savedAt?: string;
  savedArtifactId?: string;
  submittedAt?: string;
  submittedArtifactId?: string;
  reviewLane?: DraftReviewLane;
  promotedAt?: string;
  promotedSpaceId?: string;
  promotionReceiptArtifactId?: string;
  requestedByActorId?: string;
  requestedByRole?: string;
  createdAt: string;
  updatedAt: string;
}

export interface PromotionIssue {
  code: "missing_space_id" | "missing_template_id" | "missing_reference_uri" | "missing_owner";
  field: "proposedSpaceId" | "templateId" | "referenceUri" | "owner";
  message: string;
}

export interface PromotionValidationResult {
  request: SpaceCreateRequest | null;
  issues: PromotionIssue[];
  ready: boolean;
}

export interface DraftGovernanceRow {
  label: string;
  value: string;
  href?: string;
}

function nowIso(): string {
  return new Date().toISOString();
}

function draftId(): string {
  return `draft-space-${Date.now()}-${Math.floor(Math.random() * 10000)}`;
}

function slugify(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48);
}

function canUseStorage(): boolean {
  return typeof window !== "undefined" && !!window.localStorage;
}

function normalizeDraftStatus(value?: string): DraftSpaceStatus {
  if (value === "submitted" || value === "promoted") {
    return value;
  }
  return "draft";
}

function normalizeGovernanceScope(value?: string): DraftSpaceGovernanceScope {
  if (value === "private" || value === "public") {
    return value;
  }
  return "personal";
}

export function resolveDraftReviewLane(
  scope: DraftSpaceGovernanceScope,
): DraftReviewLane | undefined {
  if (scope === "public") {
    return "public_review";
  }
  if (scope === "private") {
    return "private_review";
  }
  return undefined;
}

function formatDraftReviewLane(lane?: DraftReviewLane): string {
  if (lane === "public_review") {
    return "public review";
  }
  if (lane === "private_review") {
    return "private review";
  }
  return "review";
}

export function resolveVisibilityStateForGovernanceScope(
  scope: DraftSpaceGovernanceScope,
): "owner_only" | "members_only" | "discoverable" {
  if (scope === "personal") {
    return "owner_only";
  }
  if (scope === "public") {
    return "discoverable";
  }
  return "members_only";
}

export function createDraftSpace(seed: Partial<DraftSpaceV1> = {}): DraftSpaceV1 {
  const timestamp = nowIso();
  const title = seed.title?.trim() || "Untitled draft";
  const normalizedArchetype = seed.archetype?.trim()
    || resolveSpaceStudioTemplate(seed.templateId)?.archetypeId
    || undefined;
  const archetypeLabel = normalizedArchetype
    ? resolveSpaceArchetypeProfile(normalizedArchetype).label
    : undefined;
  const proposedSpaceId =
    seed.proposedSpaceId?.trim() || slugify(title) || `space-${Date.now()}`;

  return {
    schemaVersion: "1.0.0",
    draftId: seed.draftId?.trim() || draftId(),
    title,
    purpose: seed.purpose?.trim() || "",
    owner: seed.owner?.trim() || "",
    accessSummary: seed.accessSummary?.trim() || "",
    proposedSpaceId,
    sourceMode: seed.sourceMode || "blank",
    governanceScope: normalizeGovernanceScope(seed.governanceScope),
    templateId: seed.templateId?.trim() || undefined,
    referenceUri: seed.referenceUri?.trim() || undefined,
    lineageNote: seed.lineageNote?.trim() || undefined,
    archetype: archetypeLabel,
    status: normalizeDraftStatus(seed.status),
    savedAt: seed.savedAt?.trim() || undefined,
    savedArtifactId: seed.savedArtifactId?.trim() || undefined,
    submittedAt: seed.submittedAt?.trim() || undefined,
    submittedArtifactId: seed.submittedArtifactId?.trim() || undefined,
    reviewLane: seed.reviewLane,
    promotedAt: seed.promotedAt?.trim() || undefined,
    promotedSpaceId: seed.promotedSpaceId?.trim() || undefined,
    promotionReceiptArtifactId: seed.promotionReceiptArtifactId?.trim() || undefined,
    requestedByActorId: seed.requestedByActorId?.trim() || undefined,
    requestedByRole: seed.requestedByRole?.trim() || undefined,
    createdAt: seed.createdAt || timestamp,
    updatedAt: seed.updatedAt || timestamp,
  };
}

export function createDraftFromTemplate(
  templateId: string,
  seed: Partial<DraftSpaceV1> = {},
): DraftSpaceV1 {
  const template = resolveSpaceStudioTemplate(templateId);
  if (!template) {
    return createDraftSpace({
      ...seed,
      sourceMode: "template",
      templateId,
      title: seed.title || "Template draft",
    });
  }

  const archetype = resolveSpaceArchetypeProfile(template.archetypeId).label;
  return createDraftSpace({
    ...seed,
    sourceMode: "template",
    archetype,
    templateId: template.templateId,
    title: seed.title || template.name,
    purpose: seed.purpose ?? template.purpose,
    accessSummary: seed.accessSummary ?? template.accessSummary,
    proposedSpaceId: seed.proposedSpaceId || slugify(template.name),
    lineageNote:
      seed.lineageNote || `Started from template ${template.templateId}.`,
  });
}

export function parseDraftSpaces(raw: string | null | undefined): DraftSpaceV1[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter((item) => item && typeof item === "object")
      .map((item) => createDraftSpace(item as Partial<DraftSpaceV1>))
      .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt));
  } catch {
    return [];
  }
}

export function serializeDraftSpaces(drafts: DraftSpaceV1[]): string {
  return JSON.stringify(drafts, null, 2);
}

export function buildPromotionValidationResult(
  draft: DraftSpaceV1,
): PromotionValidationResult {
  const spaceId = draft.proposedSpaceId.trim();
  const owner = draft.owner.trim();
  const templateId = draft.templateId?.trim() || undefined;
  const referenceUri = draft.referenceUri?.trim() || undefined;
  const governanceScope = normalizeGovernanceScope(draft.governanceScope);
  const issues: PromotionIssue[] = [];

  if (!spaceId) {
    issues.push({
      code: "missing_space_id",
      field: "proposedSpaceId",
      message: "Add a proposed live space id before submitting promotion.",
    });
  }

  if (draft.sourceMode === "template" && !templateId) {
    issues.push({
      code: "missing_template_id",
      field: "templateId",
      message: "Choose a template before creating a template-backed space.",
    });
  }

  if (draft.sourceMode === "reference" && !referenceUri) {
    issues.push({
      code: "missing_reference_uri",
      field: "referenceUri",
      message: "Provide a reference URI before importing a space draft.",
    });
  }

  if (governanceScope === "personal" && !owner) {
    issues.push({
      code: "missing_owner",
      field: "owner",
      message: "Personal spaces need an owner before they can be created.",
    });
  }

  if (issues.length > 0) {
    return { request: null, issues, ready: false };
  }

  const request: SpaceCreateRequest = {
    space_id: spaceId,
    creation_mode:
      draft.sourceMode === "reference"
        ? "import"
        : draft.sourceMode === "template"
          ? "template"
          : "blank",
    governance_scope: governanceScope,
    draft_id: draft.draftId,
    draft_source_mode: draft.sourceMode,
  };

  if (owner) {
    request.owner = owner;
  }
  if (draft.archetype?.trim()) {
    request.archetype = draft.archetype.trim();
  }
  if (draft.sourceMode === "template" && templateId) {
    request.template_id = templateId;
  }
  if (draft.sourceMode === "reference" && referenceUri) {
    request.reference_uri = referenceUri;
  }
  if (draft.lineageNote?.trim()) {
    request.lineage_note = draft.lineageNote.trim();
  }

  return { request, issues: [], ready: true };
}

export function buildPromotionRequestFromDraft(
  draft: DraftSpaceV1,
): SpaceCreateRequest | null {
  return buildPromotionValidationResult(draft).request;
}

export function canPromoteDraftToLive(
  role?: string | null,
  draft?: Pick<DraftSpaceV1, "governanceScope" | "owner"> | null,
  actorId?: string | null,
): boolean {
  const normalized = role?.trim().toLowerCase();
  if (normalized === "steward" || normalized === "admin") {
    return true;
  }
  if (
    draft?.governanceScope === "personal" &&
    normalized &&
    normalized !== "viewer" &&
    actorId?.trim() &&
    draft.owner.trim() === actorId.trim()
  ) {
    return true;
  }
  return false;
}

export function buildPromotionHandoffHeapBlock(
  draft: DraftSpaceV1,
  targetSpaceId: string,
  actorId: string,
  actorRole: string,
  emittedAt = nowIso(),
): EmitHeapBlockRequest {
  const summaryLines = [
    `Space promotion request: ${draft.title || draft.proposedSpaceId}`,
    ``,
    `Requested live space id: ${draft.proposedSpaceId}`,
    `Requested by: ${actorId} (${actorRole})`,
    `Source: ${draft.sourceMode}`,
    `Governance scope: ${draft.governanceScope}`,
  ];
  const reviewLane = resolveDraftReviewLane(draft.governanceScope);
  if (reviewLane) {
    summaryLines.push(`Review lane: ${formatDraftReviewLane(reviewLane)}`);
  }

  if (draft.owner) {
    summaryLines.push(`Intended owner: ${draft.owner}`);
  }
  if (draft.archetype) {
    summaryLines.push(`Archetype: ${draft.archetype}`);
  }
  if (draft.templateId) {
    summaryLines.push(`Template: ${draft.templateId}`);
  }
  if (draft.referenceUri) {
    summaryLines.push(`Reference: ${draft.referenceUri}`);
  }
  if (draft.lineageNote) {
    summaryLines.push(`Lineage note: ${draft.lineageNote}`);
  }

  return {
    schema_version: "1.0.0",
    mode: "heap",
    space_id: targetSpaceId,
    source: {
      agent_id: actorId,
      emitted_at: emittedAt,
    },
    block: {
      type: "space_promotion_request",
      title: `Space promotion request · ${draft.title || draft.proposedSpaceId}`,
      attributes: {
        draft_id: draft.draftId,
        source_mode: draft.sourceMode,
        governance_scope: draft.governanceScope,
        review_lane: reviewLane || "",
        proposed_space_id: draft.proposedSpaceId,
        intended_owner: draft.owner,
        archetype: draft.archetype || "",
        template_id: draft.templateId || "",
        reference_uri: draft.referenceUri || "",
        requested_by_actor_id: actorId,
        requested_by_role: actorRole,
      },
      behaviors: ["draft", "steward_review", "space_promotion_request"],
    },
    content: {
      payload_type: "rich_text",
      rich_text: {
        plain_text: summaryLines.join("\n"),
      },
    },
  };
}

export function buildPromotionReceiptHeapBlock(
  draft: DraftSpaceV1,
  created: Pick<SpaceCreateResponse, "space_id" | "status" | "message">,
  actorId: string,
  emittedAt = nowIso(),
): EmitHeapBlockRequest {
  const summaryLines = [
    `Space created from draft: ${draft.title || draft.proposedSpaceId}`,
    ``,
    `Live space id: ${created.space_id}`,
    `Created by: ${actorId}`,
    `Source: ${draft.sourceMode}`,
    `Governance scope: ${draft.governanceScope}`,
  ];

  if (draft.templateId) {
    summaryLines.push(`Template: ${draft.templateId}`);
  }
  if (draft.referenceUri) {
    summaryLines.push(`Reference: ${draft.referenceUri}`);
  }
  if (draft.lineageNote) {
    summaryLines.push(`Lineage note: ${draft.lineageNote}`);
  }
  if (created.message?.trim()) {
    summaryLines.push(`Result: ${created.message.trim()}`);
  }

  return {
    schema_version: "1.0.0",
    mode: "heap",
    space_id: created.space_id,
    source: {
      agent_id: actorId,
      emitted_at: emittedAt,
    },
    block: {
      type: "space_promotion_receipt",
      title: "Space created from draft",
      attributes: {
        draft_id: draft.draftId,
        source_mode: draft.sourceMode,
        governance_scope: draft.governanceScope,
        proposed_space_id: draft.proposedSpaceId,
        created_space_id: created.space_id,
        owner: draft.owner,
        archetype: draft.archetype || "",
        created_status: created.status,
      },
      behaviors: ["draft_lineage", "governance_receipt"],
    },
    content: {
      payload_type: "rich_text",
      rich_text: {
        plain_text: summaryLines.join("\n"),
      },
    },
  };
}

export function buildDraftSnapshotHeapBlock(
  draft: DraftSpaceV1,
  targetSpaceId: string,
  actorId: string,
  actorRole: string,
  emittedAt = nowIso(),
): EmitHeapBlockRequest {
  const snapshotDraft = createDraftSpace({
    ...draft,
    requestedByActorId: actorId,
    requestedByRole: actorRole,
    updatedAt: emittedAt,
  });

  return {
    schema_version: "1.0.0",
    mode: "heap",
    space_id: targetSpaceId,
    source: {
      agent_id: actorId,
      emitted_at: emittedAt,
    },
    block: {
      type: "space_draft_snapshot",
      title: `Saved draft · ${snapshotDraft.title || snapshotDraft.proposedSpaceId}`,
      attributes: {
        draft_id: snapshotDraft.draftId,
        source_mode: snapshotDraft.sourceMode,
        governance_scope: snapshotDraft.governanceScope,
        proposed_space_id: snapshotDraft.proposedSpaceId,
        requested_by_actor_id: actorId,
        requested_by_role: actorRole,
      },
      behaviors: ["draft", "draft_snapshot"],
    },
    content: {
      payload_type: "structured_data",
      structured_data: {
        draft: snapshotDraft,
      },
    },
  };
}

export function parseDraftSnapshotHeapBlock(
  item: HeapBlockListItem | null | undefined,
): DraftSpaceV1 | null {
  if (!item || item.projection.blockType !== "space_draft_snapshot") {
    return null;
  }

  const surfaceJson = item.surfaceJson as
    | {
        structured_data?: {
          draft?: Partial<DraftSpaceV1>;
        };
      }
    | undefined;
  const snapshotDraft = surfaceJson?.structured_data?.draft;
  if (!snapshotDraft || typeof snapshotDraft !== "object") {
    return null;
  }

  return createDraftSpace({
    ...snapshotDraft,
    savedAt: item.projection.emittedAt || item.projection.updatedAt,
    savedArtifactId: item.projection.artifactId,
  });
}

export function mergeDraftCollections(
  localDrafts: DraftSpaceV1[],
  incomingDrafts: DraftSpaceV1[],
): DraftSpaceV1[] {
  const merged = new Map<string, DraftSpaceV1>();

  for (const draft of [...localDrafts, ...incomingDrafts]) {
    const existing = merged.get(draft.draftId);
    if (!existing || draft.updatedAt.localeCompare(existing.updatedAt) > 0) {
      merged.set(draft.draftId, createDraftSpace(draft));
    }
  }

  return Array.from(merged.values()).sort((left, right) =>
    right.updatedAt.localeCompare(left.updatedAt),
  );
}

function formatDraftGovernanceDate(value?: string): string {
  if (!value) {
    return "recently";
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("en-US", {
    month: "long",
    day: "numeric",
    year: "numeric",
  }).format(parsed);
}

export function buildDraftGovernanceRows(draft: DraftSpaceV1): DraftGovernanceRow[] {
  const rows: DraftGovernanceRow[] = [];

  if (draft.savedAt && draft.savedArtifactId) {
    rows.push({
      label: "Saved for later",
      value: `Saved on ${formatDraftGovernanceDate(draft.savedAt)} so you can reopen it later.`,
      href: buildHeapArtifactHref(draft.savedArtifactId),
    });
  }

  if (draft.submittedAt && draft.submittedArtifactId) {
    rows.push({
      label: "Sent for review",
      value: `Sent for ${formatDraftReviewLane(draft.reviewLane)} on ${formatDraftGovernanceDate(draft.submittedAt)}.`,
      href: buildHeapArtifactHref(draft.submittedArtifactId),
    });
  }

  if (draft.promotedAt && draft.promotedSpaceId) {
    rows.push({
      label: "Live space created",
      value: `Created as ${draft.promotedSpaceId} on ${formatDraftGovernanceDate(draft.promotedAt)}.`,
      href: `/spaces/${encodeURIComponent(draft.promotedSpaceId)}`,
    });
  }

  return rows;
}

export function applyDraftSubmission(
  draft: DraftSpaceV1,
  artifactId: string,
  submittedAt = nowIso(),
  actorId?: string,
  actorRole?: string,
  reviewLane?: DraftReviewLane,
): DraftSpaceV1 {
  return createDraftSpace({
    ...draft,
    status: "submitted",
    submittedAt,
    submittedArtifactId: artifactId,
    reviewLane,
    requestedByActorId: actorId ?? draft.requestedByActorId,
    requestedByRole: actorRole ?? draft.requestedByRole,
    updatedAt: submittedAt,
  });
}

export function applyDraftSave(
  draft: DraftSpaceV1,
  artifactId: string,
  savedAt = nowIso(),
): DraftSpaceV1 {
  return createDraftSpace({
    ...draft,
    savedAt,
    savedArtifactId: artifactId,
    updatedAt: savedAt,
  });
}

export function applyDraftPromotion(
  draft: DraftSpaceV1,
  promotedSpaceId: string,
  receiptArtifactId?: string,
  promotedAt = nowIso(),
): DraftSpaceV1 {
  return createDraftSpace({
    ...draft,
    status: "promoted",
    promotedAt,
    promotedSpaceId,
    promotionReceiptArtifactId: receiptArtifactId?.trim() || undefined,
    updatedAt: promotedAt,
  });
}

export function applyDraftEdit(
  draft: DraftSpaceV1,
  patch: Partial<DraftSpaceV1>,
  editedAt = nowIso(),
): DraftSpaceV1 {
  const nextStatus = draft.status === "draft" ? draft.status : "draft";

  return createDraftSpace({
    ...draft,
    ...patch,
    status: nextStatus,
    submittedAt: nextStatus === "draft" ? undefined : draft.submittedAt,
    submittedArtifactId: nextStatus === "draft" ? undefined : draft.submittedArtifactId,
    promotedAt: nextStatus === "draft" ? undefined : draft.promotedAt,
    promotedSpaceId: nextStatus === "draft" ? undefined : draft.promotedSpaceId,
    promotionReceiptArtifactId:
      nextStatus === "draft" ? undefined : draft.promotionReceiptArtifactId,
    updatedAt: editedAt,
  });
}

function readDraftsFromStorage(): DraftSpaceV1[] {
  if (!canUseStorage()) return [];
  try {
    return parseDraftSpaces(window.localStorage.getItem(STORAGE_KEY));
  } catch {
    return [];
  }
}

function writeDraftsToStorage(drafts: DraftSpaceV1[]): void {
  if (!canUseStorage()) return;
  try {
    window.localStorage.setItem(STORAGE_KEY, serializeDraftSpaces(drafts));
  } catch {
    // ignore storage failures
  }
}

export function useSpaceStudioDrafts() {
  const [drafts, setDrafts] = useState<DraftSpaceV1[]>(() => readDraftsFromStorage());

  useEffect(() => {
    writeDraftsToStorage(drafts);
  }, [drafts]);

  const createDraft = useCallback((seed: Partial<DraftSpaceV1> = {}) => {
    const next = createDraftSpace(seed);
    setDrafts((current) => [next, ...current]);
    return next;
  }, []);

  const createTemplateDraft = useCallback((
    templateId: string,
    seed: Partial<DraftSpaceV1> = {},
  ) => {
    const next = createDraftFromTemplate(templateId, seed);
    setDrafts((current) => [next, ...current]);
    return next;
  }, []);

  const updateDraft = useCallback((draftIdValue: string, patch: Partial<DraftSpaceV1>) => {
    setDrafts((current) =>
      current.map((draft) =>
        draft.draftId === draftIdValue
          ? applyDraftEdit(
              createDraftSpace({
                ...draft,
                draftId: draft.draftId,
                createdAt: draft.createdAt,
              }),
              patch,
              nowIso(),
            )
          : draft,
      ),
    );
  }, []);

  const markDraftSubmitted = useCallback((
    draftIdValue: string,
    artifactId: string,
    actorId?: string,
    actorRole?: string,
    reviewLane?: DraftReviewLane,
  ) => {
    const submittedAt = nowIso();
    setDrafts((current) =>
      current.map((draft) =>
        draft.draftId === draftIdValue
          ? applyDraftSubmission(draft, artifactId, submittedAt, actorId, actorRole, reviewLane)
          : draft,
      ),
    );
  }, []);

  const markDraftSaved = useCallback((draftIdValue: string, artifactId: string) => {
    const savedAt = nowIso();
    setDrafts((current) =>
      current.map((draft) =>
        draft.draftId === draftIdValue ? applyDraftSave(draft, artifactId, savedAt) : draft,
      ),
    );
  }, []);

  const markDraftPromoted = useCallback((
    draftIdValue: string,
    promotedSpaceId: string,
    receiptArtifactId?: string,
  ) => {
    const promotedAt = nowIso();
    setDrafts((current) =>
      current.map((draft) =>
        draft.draftId === draftIdValue
          ? applyDraftPromotion(draft, promotedSpaceId, receiptArtifactId, promotedAt)
          : draft,
      ),
    );
  }, []);

  const removeDraft = useCallback((draftIdValue: string) => {
    setDrafts((current) => current.filter((draft) => draft.draftId !== draftIdValue));
  }, []);

  const mergeDrafts = useCallback((incomingDrafts: DraftSpaceV1[]) => {
    if (incomingDrafts.length === 0) return;
    setDrafts((current) => mergeDraftCollections(current, incomingDrafts));
  }, []);

  return {
    drafts,
    createDraft,
    createTemplateDraft,
    updateDraft,
    markDraftSaved,
    markDraftSubmitted,
    markDraftPromoted,
    mergeDrafts,
    removeDraft,
  };
}
