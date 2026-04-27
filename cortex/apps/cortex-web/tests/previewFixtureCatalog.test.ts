import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  PREVIEW_ARTIFACT_IDS,
  PREVIEW_SNAPSHOT_IDS,
  filterPreviewDeletedBlocks,
  filterPreviewHeapBlocks,
  isPreviewArtifactId,
} from "../src/store/previewFixtureCatalog.ts";
import { SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID } from "../src/store/spaceDesignProfilePreviewContract.ts";

test("preview fixture catalog recognizes seeded preview artifact ids", () => {
  assert.ok(PREVIEW_ARTIFACT_IDS.has("mock-solicitation-1"));
  assert.equal(isPreviewArtifactId("mock-solicitation-1"), true);
  assert.equal(isPreviewArtifactId("01KM4CDYTP8RD94Z52HJQQNTTD"), false);
  assert.ok(PREVIEW_SNAPSHOT_IDS.has(SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID));
});

test("preview fixture filters remove seeded mock blocks from heap responses", () => {
  const blocks = [
    {
      projection: {
        artifactId: "mock-solicitation-1",
        title: "Preview block",
        blockType: "agent_solicitation",
        updatedAt: "2026-03-20T01:05:00Z",
        tags: [],
        mentionsInline: [],
      },
    },
    {
      projection: {
        artifactId: "01KM4CDYTP8RD94Z52HJQQNTTD",
        title: "Live block",
        blockType: "agent_solicitation",
        updatedAt: "2026-03-20T01:06:00Z",
        tags: [],
        mentionsInline: [],
      },
    },
  ] as any;

  const deleted = [
    { artifactId: "mock-solicitation-1", deletedAt: "2026-03-20T01:07:00Z" },
    { artifactId: "01KM4CDYTP8RD94Z52HJQQNTTD", deletedAt: "2026-03-20T01:08:00Z" },
  ];

  assert.deepEqual(
    filterPreviewHeapBlocks(blocks).map((block) => block.projection.artifactId),
    ["01KM4CDYTP8RD94Z52HJQQNTTD"],
  );
  assert.deepEqual(
    filterPreviewDeletedBlocks(deleted).map((block) => block.artifactId),
    ["01KM4CDYTP8RD94Z52HJQQNTTD"],
  );
});

test("space design profile preview fixture remains advisory metadata only", () => {
  const fixturePath = fileURLToPath(
    new URL("../src/store/spaceDesignProfilePreview.fixture.json", import.meta.url),
  );
  const fixture = JSON.parse(readFileSync(fixturePath, "utf8"));

  assert.equal(fixture.schema_version, "CortexWebSpaceDesignProfilePreviewFixtureV1");
  assert.equal(fixture.snapshot_id, SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID);
  assert.equal(fixture.source_mode, "fixture");
  assert.equal(fixture.runtime_binding, "none");
  assert.deepEqual(fixture.runtime_policy, {
    recommendation_only: true,
    applies_tokens_to_cortex_web: false,
    runtime_theme_selection: false,
    requires_verified_projection_for_governance: true,
  });

  const [profile] = fixture.profiles;
  assert.equal(profile.authority_mode, "recommendation_only");
  assert.equal(profile.review_status, "draft");
  assert.equal(profile.approved_by_count, 0);
  assert.equal(profile.preview_status, "metadata_only");
  assert.equal(profile.a2ui_theme_policy.safe_mode, true);
  assert.equal(profile.a2ui_theme_policy.token_version, "ndl-token-v1");
  assert.equal(profile.a2ui_theme_policy.theme_allowlist_id, "ndl-space-profile-draft");
  assert.equal("design_tokens" in profile, false);
});
