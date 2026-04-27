import assert from "node:assert/strict";
import test from "node:test";

import {
  HEAP_ARTIFACT_QUERY_KEY,
  buildHeapArtifactHref,
  readHeapArtifactIdFromSearchParams,
} from "../src/components/heap/heapArtifactRouting.ts";

test("buildHeapArtifactHref creates an explore deep link for a heap artifact", () => {
  assert.equal(
    buildHeapArtifactHref("artifact-123"),
    `/explore?${HEAP_ARTIFACT_QUERY_KEY}=artifact-123`,
  );
});

test("buildHeapArtifactHref preserves the active space context when provided", () => {
  assert.equal(
    buildHeapArtifactHref("artifact-123", "01KM4C04QY37V9RV9H2HH9J1NM"),
    `/explore?space_id=01KM4C04QY37V9RV9H2HH9J1NM&${HEAP_ARTIFACT_QUERY_KEY}=artifact-123`,
  );
});

test("readHeapArtifactIdFromSearchParams reads the explicit artifact query seam", () => {
  const params = new URLSearchParams(`${HEAP_ARTIFACT_QUERY_KEY}=artifact-456`);
  assert.equal(readHeapArtifactIdFromSearchParams(params), "artifact-456");
  assert.equal(readHeapArtifactIdFromSearchParams(new URLSearchParams("")), null);
});
