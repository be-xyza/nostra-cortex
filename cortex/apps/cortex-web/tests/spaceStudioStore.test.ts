import assert from "node:assert/strict";
import test from "node:test";

import {
  applyDraftEdit,
  applyDraftPromotion,
  applyDraftSave,
  applyDraftSubmission,
  buildDraftGovernanceRows,
  buildDraftSnapshotHeapBlock,
  buildPromotionHandoffHeapBlock,
  buildPromotionReceiptHeapBlock,
  buildPromotionRequestFromDraft,
  buildPromotionValidationResult,
  canPromoteDraftToLive,
  createDraftFromTemplate,
  createDraftSpace,
  mergeDraftCollections,
  parseDraftSnapshotHeapBlock,
  parseDraftSpaces,
  serializeDraftSpaces,
} from "../src/components/spaces/spaceStudioStore.ts";

test("createDraftSpace builds a local draft with stable defaults", () => {
  const draft = createDraftSpace({ title: "Research Home" });

  assert.equal(draft.schemaVersion, "1.0.0");
  assert.equal(draft.status, "draft");
  assert.equal(draft.owner, "");
  assert.equal(draft.sourceMode, "blank");
  assert.equal(draft.governanceScope, "personal");
  assert.equal(draft.proposedSpaceId, "research-home");
});

test("createDraftSpace preserves session-derived owner and archetype defaults", () => {
  const draft = createDraftSpace({
    title: "Governance Home",
    owner: "steward.alex",
    archetype: "Governance",
    requestedByActorId: "steward.alex",
    requestedByRole: "steward",
  });

  assert.equal(draft.owner, "steward.alex");
  assert.equal(draft.archetype, "Governance");
  assert.equal(draft.requestedByActorId, "steward.alex");
  assert.equal(draft.requestedByRole, "steward");
});

test("createDraftFromTemplate carries template lineage into the draft", () => {
  const draft = createDraftFromTemplate("research-starter");

  assert.equal(draft.sourceMode, "template");
  assert.equal(draft.templateId, "tpl_research_starter_v1");
  assert.match(draft.lineageNote ?? "", /tpl_research_starter_v1/);
});

test("parseDraftSpaces normalizes stored data back into draft objects", () => {
  const serialized = serializeDraftSpaces([
    createDraftSpace({ title: "Alpha Space", proposedSpaceId: "alpha-space" }),
  ]);
  const drafts = parseDraftSpaces(serialized);

  assert.equal(drafts.length, 1);
  assert.equal(drafts[0]?.title, "Alpha Space");
  assert.equal(drafts[0]?.proposedSpaceId, "alpha-space");
});

test("buildPromotionRequestFromDraft maps source modes into live creation requests", () => {
  const blank = buildPromotionRequestFromDraft(
    createDraftSpace({ proposedSpaceId: "alpha-space", governanceScope: "private" }),
  );
  const template = buildPromotionRequestFromDraft(
    createDraftSpace({
      proposedSpaceId: "beta-space",
      governanceScope: "private",
      sourceMode: "template",
      templateId: "tpl_beta",
    }),
  );
  const reference = buildPromotionRequestFromDraft(
    createDraftSpace({
      proposedSpaceId: "gamma-space",
      governanceScope: "private",
      sourceMode: "reference",
      referenceUri: "nostra://reference/research",
    }),
  );

  assert.deepEqual(blank, {
    space_id: "alpha-space",
    creation_mode: "blank",
    governance_scope: "private",
    draft_id: blank?.draft_id,
    draft_source_mode: "blank",
  });
  assert.deepEqual(template, {
    space_id: "beta-space",
    creation_mode: "template",
    governance_scope: "private",
    template_id: "tpl_beta",
    draft_id: template?.draft_id,
    draft_source_mode: "template",
  });
  assert.deepEqual(reference, {
    space_id: "gamma-space",
    creation_mode: "import",
    governance_scope: "private",
    reference_uri: "nostra://reference/research",
    draft_id: reference?.draft_id,
    draft_source_mode: "reference",
  });
});

test("buildPromotionRequestFromDraft omits blank owner and returns null when the draft is invalid", () => {
  const blankOwner = buildPromotionRequestFromDraft(
    createDraftSpace({
      proposedSpaceId: "alpha-space",
      governanceScope: "private",
      owner: "   ",
    }),
  );
  const invalidTemplate = buildPromotionRequestFromDraft(
    createDraftSpace({
      proposedSpaceId: "beta-space",
      sourceMode: "template",
      templateId: "   ",
    }),
  );
  const invalidReference = buildPromotionRequestFromDraft(
    createDraftSpace({
      proposedSpaceId: "gamma-space",
      sourceMode: "reference",
      referenceUri: "   ",
    }),
  );

  assert.deepEqual(blankOwner, {
    space_id: "alpha-space",
    creation_mode: "blank",
    governance_scope: "private",
    draft_id: blankOwner?.draft_id,
    draft_source_mode: "blank",
  });
  assert.equal(invalidTemplate, null);
  assert.equal(invalidReference, null);
});

test("buildPromotionRequestFromDraft carries lineage fields into the live create request", () => {
  const request = buildPromotionRequestFromDraft(
    createDraftSpace({
      draftId: "draft-space-9",
      proposedSpaceId: "governance-home",
      sourceMode: "template",
      templateId: "tpl_governance",
      governanceScope: "private",
      archetype: "Governance",
      lineageNote: "Started from the governance starter.",
    }),
  );

  assert.deepEqual(request, {
    space_id: "governance-home",
    creation_mode: "template",
    template_id: "tpl_governance",
    archetype: "Governance",
    governance_scope: "private",
    draft_id: "draft-space-9",
    draft_source_mode: "template",
    lineage_note: "Started from the governance starter.",
  });
});

test("personal spaces require an owner before live creation", () => {
  const invalidPersonal = buildPromotionValidationResult(
    createDraftSpace({
      proposedSpaceId: "solo-space",
      governanceScope: "personal",
      owner: "   ",
    }),
  );

  assert.equal(invalidPersonal.ready, false);
  assert.equal(invalidPersonal.request, null);
  assert.match(invalidPersonal.issues[0]?.message ?? "", /owner/i);
});

test("buildPromotionValidationResult exposes issues for incomplete promotion drafts", () => {
  const invalidTemplate = buildPromotionValidationResult(
    createDraftSpace({
      proposedSpaceId: "beta-space",
      sourceMode: "template",
      templateId: "   ",
    }),
  );
  const invalidReference = buildPromotionValidationResult(
    createDraftSpace({
      proposedSpaceId: "gamma-space",
      sourceMode: "reference",
      referenceUri: "   ",
    }),
  );

  assert.equal(invalidTemplate.ready, false);
  assert.equal(invalidTemplate.request, null);
  assert.match(invalidTemplate.issues[0]?.message ?? "", /template/i);
  assert.equal(invalidReference.ready, false);
  assert.equal(invalidReference.request, null);
  assert.match(invalidReference.issues[0]?.message ?? "", /reference/i);
});

test("canPromoteDraftToLive only allows steward-capable roles", () => {
  assert.equal(canPromoteDraftToLive("viewer"), false);
  assert.equal(
    canPromoteDraftToLive(
      "operator",
      createDraftSpace({ governanceScope: "personal", owner: "operator.alex" }),
      "operator.alex",
    ),
    true,
  );
  assert.equal(
    canPromoteDraftToLive(
      "operator",
      createDraftSpace({ governanceScope: "private", owner: "operator.alex" }),
      "operator.alex",
    ),
    false,
  );
  assert.equal(canPromoteDraftToLive("steward"), true);
  assert.equal(canPromoteDraftToLive("admin"), true);
});

test("buildPromotionReceiptHeapBlock creates a visible lineage artifact in the new live space", () => {
  const draft = createDraftSpace({
    draftId: "draft-space-1",
    title: "Research Home",
    proposedSpaceId: "research-home",
    sourceMode: "template",
    templateId: "tpl_research_starter_v1",
    lineageNote: "Started from the research starter.",
  });

  const receipt = buildPromotionReceiptHeapBlock(
    draft,
    { space_id: "01LIVE123", status: "created", message: "created" },
    "systems-steward",
    "2026-03-20T12:00:00Z",
  );

  assert.equal(receipt.space_id, "01LIVE123");
  assert.equal(receipt.source.agent_id, "systems-steward");
  assert.equal(receipt.block.type, "space_promotion_receipt");
  assert.equal(receipt.block.attributes?.draft_id, "draft-space-1");
  assert.equal(receipt.block.attributes?.source_mode, "template");
  assert.equal(receipt.content.payload_type, "rich_text");
  assert.match(receipt.content.rich_text?.plain_text ?? "", /Live space id: 01LIVE123/);
  assert.match(receipt.content.rich_text?.plain_text ?? "", /Template: tpl_research_starter_v1/);
});

test("buildPromotionHandoffHeapBlock targets the current working space with steward review metadata", () => {
  const draft = createDraftSpace({
    draftId: "draft-space-2",
    title: "Research Home",
    proposedSpaceId: "research-home",
    governanceScope: "private",
    sourceMode: "template",
    templateId: "tpl_research_starter_v1",
    archetype: "Research",
    owner: "steward.alex",
    requestedByActorId: "operator.jo",
    requestedByRole: "operator",
  });

  const handoff = buildPromotionHandoffHeapBlock(
    draft,
    "research",
    "operator.jo",
    "operator",
    "2026-03-20T13:00:00Z",
  );

  assert.equal(handoff.space_id, "research");
  assert.equal(handoff.block.type, "space_promotion_request");
  assert.equal(handoff.block.attributes?.draft_id, "draft-space-2");
  assert.equal(handoff.block.attributes?.requested_by_actor_id, "operator.jo");
  assert.equal(handoff.block.attributes?.requested_by_role, "operator");
  assert.equal(handoff.block.attributes?.archetype, "Research");
  assert.equal(handoff.block.attributes?.review_lane, "private_review");
  assert.match(handoff.content.rich_text?.plain_text ?? "", /private review/i);
  assert.ok(handoff.block.behaviors?.includes("draft"));
  assert.ok(handoff.block.behaviors?.includes("steward_review"));
});

test("draft lifecycle helpers preserve lineage when drafts are submitted or promoted", () => {
  const draft = createDraftSpace({
    draftId: "draft-space-3",
    proposedSpaceId: "alpha-space",
  });

  const submitted = applyDraftSubmission(
    draft,
    "artifact-handoff-1",
    "2026-03-20T13:10:00Z",
    "operator.jo",
    "operator",
    "private_review",
  );
  const promoted = applyDraftPromotion(
    submitted,
    "01LIVE999",
    "artifact-receipt-1",
    "2026-03-20T13:11:00Z",
  );

  assert.equal(submitted.status, "submitted");
  assert.equal(submitted.submittedArtifactId, "artifact-handoff-1");
  assert.equal(submitted.reviewLane, "private_review");
  assert.equal(submitted.requestedByActorId, "operator.jo");
  assert.equal(promoted.status, "promoted");
  assert.equal(promoted.promotedSpaceId, "01LIVE999");
  assert.equal(promoted.promotionReceiptArtifactId, "artifact-receipt-1");
  assert.equal(promoted.submittedArtifactId, "artifact-handoff-1");
});

test("applyDraftSave preserves the current status while recording the saved snapshot", () => {
  const submitted = applyDraftSubmission(
    createDraftSpace({
      draftId: "draft-space-12",
      proposedSpaceId: "alpha-space",
    }),
    "artifact-handoff-3",
    "2026-03-20T13:15:00Z",
    "operator.jo",
    "operator",
    "private_review",
  );

  const saved = applyDraftSave(
    submitted,
    "artifact-snapshot-3",
    "2026-03-20T13:20:00Z",
  );

  assert.equal(saved.status, "submitted");
  assert.equal(saved.savedArtifactId, "artifact-snapshot-3");
  assert.equal(saved.savedAt, "2026-03-20T13:20:00Z");
});

test("buildDraftSnapshotHeapBlock captures a resumable remote draft snapshot", () => {
  const draft = createDraftSpace({
    draftId: "draft-space-10",
    title: "Research Home",
    proposedSpaceId: "research-home",
    purpose: "Track research work in one place.",
    requestedByActorId: "steward.alex",
    requestedByRole: "steward",
  });

  const snapshot = buildDraftSnapshotHeapBlock(
    draft,
    "01LIVE123",
    "steward.alex",
    "steward",
    "2026-03-20T15:00:00Z",
  );

  assert.equal(snapshot.space_id, "01LIVE123");
  assert.equal(snapshot.block.type, "space_draft_snapshot");
  assert.equal(snapshot.block.attributes?.draft_id, "draft-space-10");
  assert.equal(snapshot.block.attributes?.requested_by_actor_id, "steward.alex");
  assert.equal(snapshot.content.payload_type, "structured_data");
});

test("parseDraftSnapshotHeapBlock restores a draft from a heap snapshot", () => {
  const restored = parseDraftSnapshotHeapBlock({
    projection: {
      artifactId: "artifact-snapshot-1",
      title: "Research Home",
      blockType: "space_draft_snapshot",
      emittedAt: "2026-03-20T15:00:00Z",
      updatedAt: "2026-03-20T15:00:00Z",
    },
    surfaceJson: {
      payload_type: "structured_data",
      structured_data: {
        draft: {
          schemaVersion: "1.0.0",
          draftId: "draft-space-10",
          title: "Research Home",
          purpose: "Track research work in one place.",
          owner: "steward.alex",
          accessSummary: "",
          proposedSpaceId: "research-home",
          sourceMode: "blank",
          status: "draft",
          requestedByActorId: "steward.alex",
          requestedByRole: "steward",
          createdAt: "2026-03-20T15:00:00Z",
          updatedAt: "2026-03-20T15:00:00Z",
        },
      },
    },
  } as never);

  assert.equal(restored?.draftId, "draft-space-10");
  assert.equal(restored?.requestedByActorId, "steward.alex");
  assert.equal(restored?.savedArtifactId, "artifact-snapshot-1");
  assert.equal(restored?.savedAt, "2026-03-20T15:00:00Z");
});

test("mergeDraftCollections keeps the newest version of each draft", () => {
  const older = createDraftSpace({
    draftId: "draft-space-11",
    title: "Research Home",
    updatedAt: "2026-03-20T14:00:00Z",
  });
  const newer = createDraftSpace({
    draftId: "draft-space-11",
    title: "Research Home Revised",
    updatedAt: "2026-03-20T15:00:00Z",
  });

  const merged = mergeDraftCollections([older], [newer]);
  assert.equal(merged.length, 1);
  assert.equal(merged[0]?.title, "Research Home Revised");
});

test("editing a submitted draft reopens it as a fresh local draft", () => {
  const submitted = applyDraftSubmission(
    createDraftSpace({
      draftId: "draft-space-4",
      proposedSpaceId: "alpha-space",
    }),
    "artifact-handoff-2",
    "2026-03-20T13:15:00Z",
    "operator.jo",
    "operator",
    "private_review",
  );

  const reopened = applyDraftEdit(submitted, {
    title: "Alpha Space Revised",
  }, "2026-03-20T13:16:00Z");

  assert.equal(reopened.status, "draft");
  assert.equal(reopened.submittedArtifactId, undefined);
  assert.equal(reopened.promotedSpaceId, undefined);
  assert.equal(reopened.title, "Alpha Space Revised");
});

test("buildDraftGovernanceRows keeps draft history simple and actionable", () => {
  const draft = createDraftSpace({
    draftId: "draft-space-13",
    proposedSpaceId: "alpha-space",
    savedAt: "2026-03-20T13:05:00Z",
    savedArtifactId: "artifact-snapshot-4",
    submittedAt: "2026-03-20T13:10:00Z",
    submittedArtifactId: "artifact-handoff-4",
    reviewLane: "public_review",
    promotedAt: "2026-03-20T13:15:00Z",
    promotedSpaceId: "01LIVE123",
    promotionReceiptArtifactId: "artifact-receipt-4",
    status: "promoted",
  });

  const rows = buildDraftGovernanceRows(draft);
  assert.deepEqual(
    rows.map((row) => row.label),
    ["Saved for later", "Sent for review", "Live space created"],
  );
  assert.equal(rows[0]?.href, "/explore?artifact_id=artifact-snapshot-4");
  assert.equal(rows[1]?.href, "/explore?artifact_id=artifact-handoff-4");
  assert.match(rows[1]?.value ?? "", /public review/i);
  assert.equal(rows[2]?.href, "/spaces/01LIVE123");
});
