import assert from "node:assert/strict";
import test from "node:test";

import { resolveRequestSpaceId } from "../src/serviceWorker/requestScope.ts";

test("resolveRequestSpaceId prefers canonical space_id when present", () => {
  const url = new URL("http://localhost:4173/api/cortex/studio/heap/blocks?space_id=space-live&spaceId=space-legacy");
  assert.equal(resolveRequestSpaceId(url, "fallback-space"), "space-live");
});

test("resolveRequestSpaceId falls back to legacy spaceId when needed", () => {
  const url = new URL("http://localhost:4173/api/cortex/studio/heap/blocks?spaceId=space-legacy");
  assert.equal(resolveRequestSpaceId(url, "fallback-space"), "space-legacy");
});

test("resolveRequestSpaceId returns the fallback for unscoped requests", () => {
  const url = new URL("http://localhost:4173/api/cortex/studio/heap/blocks");
  assert.equal(resolveRequestSpaceId(url, "fallback-space"), "fallback-space");
});
