import assert from "node:assert/strict";
import test from "node:test";

import type {
  PlatformCapabilityCatalog,
  SpaceCapabilityGraph,
} from "../src/contracts.ts";
import {
  applyEditorOverride,
  buildCapabilityEditorGraph,
  buildPersistedCapabilityGraph,
} from "../src/components/schema-editor/schemaEditorModel.ts";

const catalog: PlatformCapabilityCatalog = {
  schemaVersion: "1.0.0",
  catalogVersion: "catalog-v1",
  catalogHash: "catalog-hash",
  nodes: [
    {
      id: "route:/heap",
      name: "Heap",
      description: "Primary heap route",
      intentType: "operate",
      routeId: "/heap",
      category: "core",
      requiredRole: "operator",
      surfacingHeuristic: "PrimaryCore",
      operationalFrequency: "Continuous",
    },
    {
      id: "route:/system",
      name: "System",
      description: "System route",
      intentType: "monitor",
      routeId: "/system",
      category: "system",
      requiredRole: "viewer",
      surfacingHeuristic: "ContextualDeep",
      operationalFrequency: "Daily",
    },
  ],
  edges: [
    {
      source: "route:/heap",
      target: "route:/system",
      relationship: "depends_on",
    },
  ],
};

const baseGraph: SpaceCapabilityGraph = {
  schemaVersion: "1.0.0",
  spaceId: "nostra-governance-v0",
  baseCatalogVersion: "catalog-v1",
  baseCatalogHash: "catalog-hash",
  nodes: [
    {
      capabilityId: "route:/heap",
      isActive: true,
    },
    {
      capabilityId: "route:/system",
      isActive: true,
      localAlias: "Systems Console",
      localRequiredRole: "steward",
      surfacingHeuristic: "Secondary",
      operationalFrequency: "Daily",
    },
  ],
  edges: catalog.edges,
  updatedAt: "2026-03-22T12:00:00Z",
  updatedBy: "system",
  lineageRef: "decision:130",
};

test("buildCapabilityEditorGraph merges catalog metadata with overlay overrides", () => {
  const graph = buildCapabilityEditorGraph(catalog, baseGraph);

  assert.equal(graph.nodes.length, 2);
  assert.equal(graph.nodes[1]?.data.title, "System");
  assert.equal(graph.nodes[1]?.data.localAlias, "Systems Console");
  assert.equal(graph.nodes[1]?.data.localRequiredRole, "steward");
  assert.equal(graph.nodes[1]?.data.isActive, true);
  assert.equal(catalog.nodes[1]?.name, "System");
});

test("applyEditorOverride updates only the targeted overlay node", () => {
  const updated = applyEditorOverride(baseGraph.nodes, "route:/system", {
    isActive: false,
    localAlias: "System Steward Console",
  });

  assert.equal(updated[0]?.isActive, true);
  assert.equal(updated[1]?.isActive, false);
  assert.equal(updated[1]?.localAlias, "System Steward Console");
  assert.equal(baseGraph.nodes[1]?.localAlias, "Systems Console");
});

test("buildPersistedCapabilityGraph preserves graph authority fields while refreshing steward metadata", () => {
  const overrides = applyEditorOverride(baseGraph.nodes, "route:/system", {
    surfacingHeuristic: "ContextualDeep",
  });

  const persisted = buildPersistedCapabilityGraph(baseGraph, overrides, {
    updatedAt: "2026-03-22T13:00:00Z",
    updatedBy: "steward.alex",
    lineageRef: "decision:130:overlay-update",
  });

  assert.equal(persisted.spaceId, "nostra-governance-v0");
  assert.equal(persisted.baseCatalogHash, "catalog-hash");
  assert.equal(persisted.updatedBy, "steward.alex");
  assert.equal(persisted.lineageRef, "decision:130:overlay-update");
  assert.equal(persisted.nodes[1]?.surfacingHeuristic, "ContextualDeep");
});
