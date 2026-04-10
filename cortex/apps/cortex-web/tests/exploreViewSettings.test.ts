import assert from "node:assert/strict";
import test from "node:test";

import {
  buildExploreSpaceDefaults,
  resolveExploreViewSettings,
} from "../src/components/heap/exploreViewSettings.ts";
import {
  EXPLORE_VIEW_SETTING_SECTIONS,
  resolveExploreSettingVisualKind,
} from "../src/components/heap/exploreViewSettingsRegistry.ts";
import { resolveExploreSurfacePolicy } from "../src/components/heap/exploreSurfacePolicy.ts";

test("buildExploreSpaceDefaults maps research spaces to dense compact defaults", () => {
  const defaults = buildExploreSpaceDefaults({
    spaceArchetype: "Research",
    ambientGraphVariant: "2d",
  });

  assert.equal(defaults.projectionIntent, "density");
  assert.equal(defaults.layoutMode, "compact");
  assert.equal(defaults.cardDepth, "summary");
  assert.equal(defaults.aggregationMode, "all");
  assert.equal(defaults.aggregationDensity, "tight");
  assert.equal(defaults.showGroupDescriptions, false);
  assert.equal(defaults.visualizationMode, "2d");
});

test("explicit projection intent overrides archetype-driven policy resolution", () => {
  const policy = resolveExploreSurfacePolicy({
    spaceArchetype: "Research",
    projectionIntent: "lineage",
  });

  assert.equal(policy.policyId, "explore.list.lineage.v1");
  assert.equal(policy.projectionIntent, "lineage");
});

test("resolveExploreViewSettings layers space defaults, user overrides, and session overrides safely", () => {
  const resolved = resolveExploreViewSettings({
    defaults: {
      projectionIntent: "overview",
      layoutMode: "balanced",
      cardDepth: "summary",
      aggregationMode: "all",
      aggregationDensity: "balanced",
      showGroupDescriptions: false,
      visualizationMode: "off",
    },
    userOverrides: {
      layoutMode: "open",
      cardDepth: "full",
      showGroupDescriptions: true,
      visualizationMode: "3d",
    },
    sessionOverrides: {
      projectionIntent: "lineage",
      aggregationMode: "steward_feedback",
    },
    isMobile: true,
    reduceMotion: true,
  });

  assert.equal(resolved.effective.projectionIntent, "lineage");
  assert.equal(resolved.effective.layoutMode, "balanced");
  assert.equal(resolved.effective.cardDepth, "full");
  assert.equal(resolved.effective.aggregationMode, "steward_feedback");
  assert.equal(resolved.effective.aggregationDensity, "balanced");
  assert.equal(resolved.effective.showGroupDescriptions, false);
  assert.equal(resolved.effective.visualizationMode, "2d");
  assert.equal(resolved.derived.laneCap, 3);
  assert.equal(resolved.derived.groupPreviewCount, 2);
  assert.equal(resolved.provenance.projectionIntent, "session");
  assert.equal(resolved.provenance.layoutMode, "user");
  assert.equal(resolved.provenance.cardDepth, "user");
  assert.equal(resolved.provenance.aggregationMode, "session");
  assert.equal(resolved.provenance.aggregationDensity, "space");
  assert.equal(resolved.provenance.showGroupDescriptions, "user");
  assert.equal(resolved.provenance.visualizationMode, "user");
});

test("explore setting registry resolves toggle, slider, segmented, and chips presentations", () => {
  const surfaceSection = EXPLORE_VIEW_SETTING_SECTIONS.find((section) => section.id === "surface");
  const aggregateSection = EXPLORE_VIEW_SETTING_SECTIONS.find((section) => section.id === "aggregate");

  assert.ok(surfaceSection);
  assert.ok(aggregateSection);

  const projection = surfaceSection?.controls.find((control) => control.key === "projectionIntent");
  const layout = surfaceSection?.controls.find((control) => control.key === "layoutMode");
  const groupDescriptions = aggregateSection?.controls.find((control) => control.key === "showGroupDescriptions");
  const previewDepth = aggregateSection?.controls.find((control) => control.key === "aggregationDensity");

  assert.equal(resolveExploreSettingVisualKind(projection!, { isCompactPanel: true }), "segmented");
  assert.equal(resolveExploreSettingVisualKind(layout!, { isCompactPanel: true }), "chips");
  assert.equal(resolveExploreSettingVisualKind(groupDescriptions!, { isCompactPanel: false }), "toggle");
  assert.equal(resolveExploreSettingVisualKind(previewDepth!, { isCompactPanel: false }), "slider");
});
