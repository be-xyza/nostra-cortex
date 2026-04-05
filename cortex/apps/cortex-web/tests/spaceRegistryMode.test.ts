import assert from "node:assert/strict";
import test from "node:test";

import {
  PREVIEW_SPACES,
  getRegistryBootstrapSpaces,
  getRegistryFallbackSpaces,
  partitionSpacesBySource,
  resolveCanonicalActiveSpaceIds,
  resolveRegistryFailureSpaces,
  resolveSpaceRegistryMode,
} from "../src/store/spacesRegistry.ts";

test("preview mode keeps preview spaces available", () => {
  const spaces = getRegistryFallbackSpaces("preview");

  assert.equal(resolveSpaceRegistryMode("preview"), "preview");
  assert.equal(spaces[0]?.id, "meta");
  assert.equal(spaces[1]?.id, "01ARZ3NDEKTSV4RRFFQ69G5FAV");
  assert.match(spaces[1]?.name ?? "", /demo/i);
  assert.equal(spaces.length, PREVIEW_SPACES.length + 1);
});

test("auto mode bootstraps neutral while fallback behavior stays mode-aware", () => {
  assert.equal(resolveSpaceRegistryMode(undefined), "auto");
  assert.deepEqual(
    getRegistryBootstrapSpaces("auto").map((space) => space.id),
    ["meta"],
  );
  assert.deepEqual(
    getRegistryBootstrapSpaces("live").map((space) => space.id),
    ["meta"],
  );
  assert.deepEqual(
    getRegistryFallbackSpaces("auto").map((space) => space.id),
    ["meta"],
  );
  assert.deepEqual(
    getRegistryFallbackSpaces("live").map((space) => space.id),
    ["meta"],
  );
  assert.deepEqual(
    getRegistryBootstrapSpaces("preview").map((space) => space.id),
    ["meta", ...PREVIEW_SPACES.map((space) => space.id)],
  );
});

test("stale active space ids fall back to the first discovered live space", () => {
  const activeSpaceIds = ["01ARZ3NDEKTSV4RRFFQ69G5FAV"];
  const availableSpaces = [
    { id: "meta", name: "Platform Overview", type: "global" as const },
    { id: "01KM4C04QY37V9RV9H2HH9J1NM", name: "Research · 01KM4C04", type: "user" as const },
  ];

  assert.deepEqual(
    resolveCanonicalActiveSpaceIds(activeSpaceIds, availableSpaces),
    ["01KM4C04QY37V9RV9H2HH9J1NM"],
  );
});

test("active selections keep any still-valid spaces and drop only stale ones", () => {
  const activeSpaceIds = ["research", "stale-space", "meta"];
  const availableSpaces = [
    { id: "meta", name: "Platform Overview", type: "global" as const },
    { id: "research", name: "Research Demo", type: "user" as const },
    { id: "system", name: "System Demo", type: "system" as const },
  ];

  assert.deepEqual(
    resolveCanonicalActiveSpaceIds(activeSpaceIds, availableSpaces),
    ["research", "meta"],
  );
});

test("unresolved auto fallback does not overwrite a live active space selection", () => {
  const activeSpaceIds = ["01KM4C04QY37V9RV9H2HH9J1NM"];
  const availableSpaces = [
    { id: "meta", name: "Platform Overview", type: "global" as const },
    { id: "01ARZ3NDEKTSV4RRFFQ69G5FAV", name: "Nostra Intro Demo", type: "user" as const },
  ];

  assert.deepEqual(
    resolveCanonicalActiveSpaceIds(activeSpaceIds, availableSpaces, { deferInvalidation: true }),
    ["01KM4C04QY37V9RV9H2HH9J1NM"],
  );
});

test("registry failures preserve the last known live spaces when available", () => {
  const currentSpaces = [
    { id: "meta", name: "Platform Overview", type: "global" as const },
    { id: "01KM4C04QY37V9RV9H2HH9J1NM", name: "Research · 01KM4C04", type: "user" as const, sourceMode: "registered" as const },
    { id: "nostra-governance-v0", name: "Observed Governance", type: "system" as const, sourceMode: "observed" as const },
  ];

  assert.deepEqual(
    resolveRegistryFailureSpaces("live", currentSpaces).map((space) => space.id),
    ["meta", "01KM4C04QY37V9RV9H2HH9J1NM", "nostra-governance-v0"],
  );
});

test("spaces are partitioned into distinct source buckets for the UI", () => {
  const grouped = partitionSpacesBySource([
    { id: "registered-space", name: "Registered", type: "user" as const, sourceMode: "registered" as const },
    { id: "observed-space", name: "Observed", type: "user" as const, sourceMode: "observed" as const },
    { id: "preview-space", name: "Preview", type: "user" as const, sourceMode: "preview" as const },
    { id: "draft-space", name: "Draft", type: "user" as const, sourceMode: "draft" as const },
  ]);

  assert.deepEqual(grouped.registered.map((space) => space.id), ["registered-space"]);
  assert.deepEqual(grouped.observed.map((space) => space.id), ["observed-space"]);
  assert.deepEqual(grouped.preview.map((space) => space.id), ["preview-space"]);
  assert.deepEqual(grouped.draft.map((space) => space.id), ["draft-space"]);
});
