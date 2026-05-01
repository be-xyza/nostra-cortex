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
import { GRAPH_SURFACE_INVENTORY_SNAPSHOT_ID } from "../src/store/graphSurfaceInventoryContract.ts";
import { HEAP_BLOCK_CAPABILITY_INVENTORY_SNAPSHOT_ID } from "../src/store/heapBlockCapabilityInventoryContract.ts";
import { OVERLAY_SURFACE_INVENTORY_SNAPSHOT_ID } from "../src/store/overlaySurfaceInventoryContract.ts";
import { ROUTE_IA_INVENTORY_SNAPSHOT_ID } from "../src/store/routeIaInventoryContract.ts";
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
  assert.ok(PREVIEW_SNAPSHOT_IDS.has(GRAPH_SURFACE_INVENTORY_SNAPSHOT_ID));
  assert.ok(PREVIEW_SNAPSHOT_IDS.has(OVERLAY_SURFACE_INVENTORY_SNAPSHOT_ID));
  assert.ok(PREVIEW_SNAPSHOT_IDS.has(ROUTE_IA_INVENTORY_SNAPSHOT_ID));
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

test("graph surface inventory fixture remains recommendation-only observability", () => {
  const fixturePath = fileURLToPath(
    new URL("../src/store/graphSurfaceInventory.fixture.json", import.meta.url),
  );
  const fixture = JSON.parse(readFileSync(fixturePath, "utf8"));

  assert.equal(fixture.schema_version, "CortexWebGraphSurfaceInventoryV1");
  assert.equal(fixture.snapshot_id, GRAPH_SURFACE_INVENTORY_SNAPSHOT_ID);
  assert.equal(fixture.authority_mode, "recommendation_only");
  assert.ok(fixture.scope_boundaries.excludes.includes("runtime_mutation_authority"));
  assert.ok(fixture.scope_boundaries.excludes.includes("graph_data_correctness"));

  const surfaces = new Map(fixture.graph_surfaces.map((surface: any) => [surface.surface_id, surface]));
  assert.ok(surfaces.has("graph.contributions.full"));
  assert.ok(surfaces.has("graph.capability.system"));
  assert.ok(surfaces.has("graph.workflow.flow_graph"));
  assert.ok(surfaces.has("graph.execution_canvas.spatial_plane"));
  assert.ok(surfaces.has("graph.heap.detail_relations"));

  const flowGraph = surfaces.get("graph.workflow.flow_graph") as any;
  assert.equal(flowGraph.layout_engine, "json");
  assert.equal(flowGraph.render_status, "functional_json_only");

  const gapIds = new Set(fixture.known_gaps.map((gap: any) => gap.id));
  assert.ok(gapIds.has("graph.surface.shared_health_contract"));
  assert.ok(gapIds.has("graph.execution_canvas.topology_drift"));
  assert.ok(gapIds.has("graph.capability.renderer_visibility"));
});

test("overlay surface inventory fixture remains recommendation-only observability", () => {
  const fixturePath = fileURLToPath(
    new URL("../src/store/overlaySurfaceInventory.fixture.json", import.meta.url),
  );
  const fixture = JSON.parse(readFileSync(fixturePath, "utf8"));

  assert.equal(fixture.schema_version, "CortexWebOverlaySurfaceInventoryV1");
  assert.equal(fixture.snapshot_id, OVERLAY_SURFACE_INVENTORY_SNAPSHOT_ID);
  assert.equal(fixture.authority_mode, "recommendation_only");
  assert.ok(fixture.scope_boundaries.excludes.includes("runtime_mutation_authority"));
  assert.ok(fixture.scope_boundaries.excludes.includes("product_behavior_changes"));

  const surfaces = new Map(fixture.overlay_surfaces.map((surface: any) => [surface.surface_id, surface]));
  assert.ok(surfaces.has("overlay.heap.detail_modal"));
  assert.ok(surfaces.has("overlay.heap.chat_panel"));
  assert.ok(surfaces.has("overlay.heap.comment_sidebar"));
  assert.ok(surfaces.has("overlay.system.provider_detail_sheet"));
  assert.ok(surfaces.has("overlay.artifacts.workflow_inspector"));
  assert.ok(surfaces.has("overlay.shared.confirmation"));

  const chatPanel = surfaces.get("overlay.heap.chat_panel") as any;
  assert.equal(chatPanel.authority_class, "runtime_write");
  assert.match(chatPanel.known_collision, /create controls/);

  const providerSheet = surfaces.get("overlay.system.provider_detail_sheet") as any;
  assert.equal(providerSheet.authority_class, "operator_only");

  const gapIds = new Set(fixture.known_gaps.map((gap: any) => gap.id));
  assert.ok(gapIds.has("overlay.heap.chat_create_collision"));
  assert.ok(gapIds.has("overlay.system.provider_provenance"));
  assert.ok(gapIds.has("overlay.artifacts.console_only_action"));
});

test("route IA inventory fixture keeps settings absent and candidates explicit", () => {
  const fixturePath = fileURLToPath(
    new URL("../src/store/routeIaInventory.fixture.json", import.meta.url),
  );
  const fixture = JSON.parse(readFileSync(fixturePath, "utf8"));

  assert.equal(fixture.schema_version, "CortexWebRouteIaInventoryV1");
  assert.equal(fixture.snapshot_id, ROUTE_IA_INVENTORY_SNAPSHOT_ID);
  assert.equal(fixture.authority_mode, "recommendation_only");
  assert.equal(fixture.settings_absence_contract.route_id, "/settings");
  assert.equal(fixture.settings_absence_contract.global_settings_page_allowed_this_stage, false);
  assert.ok(fixture.scope_boundaries.excludes.includes("new_settings_page"));

  const routes = new Map(fixture.routes.map((route: any) => [route.route_id, route]));
  assert.equal(routes.get("/settings")?.readiness_status, "missing");
  assert.equal(routes.get("/settings")?.route_class, "settings_absence_contract");
  assert.equal(routes.get("/system/providers")?.operator_boundary, "operator_only");
  assert.equal(routes.get("/discovery")?.visible_in_nav, true);
  assert.equal(routes.get("/discovery")?.readiness_status, "under_construction");
  assert.equal(routes.get("/studio")?.a2ui_fallback_allowed, true);
  assert.equal(routes.get("/spaces/:id?tab=overview")?.detail_tabs.includes("routing"), true);

  const gapIds = new Set(fixture.known_gaps.map((gap: any) => gap.id));
  assert.ok(gapIds.has("route.settings.absent"));
  assert.ok(gapIds.has("route.discovery.visible_placeholder"));
  assert.ok(gapIds.has("route.a2ui.seeded_candidates_untyped"));
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

  const commentSidebar = fixture.overlay_surfaces.find((surface: any) => surface.id === "comment_sidebar");
  assert.equal(commentSidebar.authority_contract.persistence, "local_ui_state");
  assert.equal(commentSidebar.authority_contract.durable_evidence, false);
  assert.equal(commentSidebar.authority_contract.governed_heap_record, false);
  assert.equal(commentSidebar.authority_contract.exportable_as_evidence, false);

  const gapIds = new Set(fixture.known_gaps.map((gap: any) => gap.id));
  assert.equal(gapIds.has("heap.block.delete.confirmation"), false);
  assert.equal(gapIds.has("heap.block.edit.semantic_split"), false);
  assert.equal(gapIds.has("heap.block.comments.persistence"), false);
  assert.ok(gapIds.has("heap.block.overlay.layering"));
});
