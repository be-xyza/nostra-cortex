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
import { HEAP_BLOCK_CAPABILITY_INVENTORY_SNAPSHOT_ID } from "../src/store/heapBlockCapabilityInventoryContract.ts";
import { SHELL_SURFACE_INVENTORY_SNAPSHOT_ID } from "../src/store/shellSurfaceInventoryContract.ts";
import {
  isSpaceDesignProfilePreviewMetadataOnly,
  spaceDesignProfilePreviewThemeBindingState,
  type SpaceDesignProfilePreviewFixture,
  SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID,
} from "../src/store/spaceDesignProfilePreviewContract.ts";

test("preview fixture catalog recognizes seeded preview artifact ids", () => {
  assert.ok(PREVIEW_ARTIFACT_IDS.has("mock-solicitation-1"));
  assert.equal(isPreviewArtifactId("mock-solicitation-1"), true);
  assert.equal(isPreviewArtifactId("01KM4CDYTP8RD94Z52HJQQNTTD"), false);
  assert.ok(PREVIEW_SNAPSHOT_IDS.has(SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID));
  assert.ok(PREVIEW_SNAPSHOT_IDS.has(SHELL_SURFACE_INVENTORY_SNAPSHOT_ID));
  assert.ok(PREVIEW_SNAPSHOT_IDS.has(HEAP_BLOCK_CAPABILITY_INVENTORY_SNAPSHOT_ID));
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
  assert.equal(isSpaceDesignProfilePreviewMetadataOnly(fixture as SpaceDesignProfilePreviewFixture), true);
  assert.equal(spaceDesignProfilePreviewThemeBindingState(fixture as SpaceDesignProfilePreviewFixture), "metadata_only");
});

test("shell surface inventory fixture remains recommendation-only observability", () => {
  const fixturePath = fileURLToPath(
    new URL("../src/store/shellSurfaceInventory.fixture.json", import.meta.url),
  );
  const fixture = JSON.parse(readFileSync(fixturePath, "utf8"));

  assert.equal(fixture.schema_version, "CortexWebShellSurfaceInventoryV1");
  assert.equal(fixture.snapshot_id, SHELL_SURFACE_INVENTORY_SNAPSHOT_ID);
  assert.equal(fixture.authority_mode, "recommendation_only");
  assert.ok(fixture.scope_boundaries.excludes.includes("heap_block_capability_deep_validation"));

  const discovery = fixture.routes.find((route: any) => route.route === "/discovery");
  assert.equal(discovery.status, "under_construction");
  assert.equal(discovery.visible_in_nav, true);
  assert.equal(discovery.known_gap.severity, "high");

  const settings = fixture.routes.find((route: any) => route.route === "/settings");
  assert.equal(settings.class, "settings_gap");
  assert.equal(settings.status, "missing");

  const settingsCategories = fixture.settings_requirements.map((item: any) => item.category);
  assert.ok(settingsCategories.includes("personal_preferences"));
  assert.ok(settingsCategories.includes("space_settings"));
  assert.ok(settingsCategories.includes("workbench_settings"));
  assert.ok(settingsCategories.includes("operator_settings"));
  assert.ok(settingsCategories.includes("design_theme_governance"));
});

test("heap block capability inventory fixture remains recommendation-only observability", () => {
  const fixturePath = fileURLToPath(
    new URL("../src/store/heapBlockCapabilityInventory.fixture.json", import.meta.url),
  );
  const fixture = JSON.parse(readFileSync(fixturePath, "utf8"));

  assert.equal(fixture.schema_version, "CortexWebHeapBlockCapabilityInventoryV1");
  assert.equal(fixture.snapshot_id, HEAP_BLOCK_CAPABILITY_INVENTORY_SNAPSHOT_ID);
  assert.equal(fixture.authority_mode, "recommendation_only");
  assert.ok(fixture.scope_boundaries.includes.includes("heap_action_zones"));
  assert.ok(fixture.scope_boundaries.excludes.includes("runtime_mutation_authority"));

  const actions = new Set(fixture.actions.map((action: any) => action.id));
  assert.ok(actions.has("create"));
  assert.ok(actions.has("publish"));
  assert.ok(actions.has("delete"));
  assert.ok(actions.has("discussion"));
  assert.ok(actions.has("relation_edit"));

  const deleteAction = fixture.actions.find((action: any) => action.id === "delete");
  assert.equal(deleteAction.class, "destructive_write");
  assert.equal(deleteAction.confirmation_contract.required, true);
  assert.equal(deleteAction.confirmation_contract.style, "danger");
  assert.equal(deleteAction.confirmation_contract.fallback_enforced, true);

  const relationEditAction = fixture.actions.find((action: any) => action.id === "relation_edit");
  assert.equal(relationEditAction.class, "runtime_write");
  assert.equal(relationEditAction.status, "functional_relation_editor_toggle");

  const createModes = new Set(fixture.create_modes.map((mode: any) => mode.mode));
  assert.deepEqual(createModes, new Set(["create", "generate", "upload", "chat", "plan"]));

  const gapIds = new Set(fixture.known_gaps.map((gap: any) => gap.id));
  assert.equal(gapIds.has("heap.block.delete.confirmation"), false);
  assert.equal(gapIds.has("heap.block.edit.semantic_split"), false);
  assert.ok(gapIds.has("heap.block.comments.persistence"));
  assert.ok(gapIds.has("heap.block.overlay.layering"));
});
