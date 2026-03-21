import assert from "node:assert/strict";
import test from "node:test";

import {
  PREVIEW_SPACES,
  getRegistryBootstrapSpaces,
  getRegistryFallbackSpaces,
  resolveCanonicalActiveSpaceIds,
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
    ["meta", ...PREVIEW_SPACES.map((space) => space.id)],
  );
  assert.deepEqual(
    getRegistryFallbackSpaces("live").map((space) => space.id),
    ["meta"],
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
