import assert from "node:assert/strict";
import test from "node:test";

import {
  MOCK_LAYOUT_SPEC,
  MOCK_UX_WORKBENCH_STUDIO,
  PLATFORM_CAPABILITY_CATALOG,
} from "../src/store/seedData.ts";

test("Flow Studio is user-facing while the existing studio route stays stable", () => {
  const studioRoute = MOCK_LAYOUT_SPEC.navigationGraph.entries.find(
    (route) => route.routeId === "/studio",
  );
  const studioCapability = PLATFORM_CAPABILITY_CATALOG.nodes.find(
    (node) => node.routeId === "/studio",
  );

  assert.equal(studioRoute?.label, "Flow Studio");
  assert.equal(studioCapability?.name, "Flow Studio");
  assert.equal(studioCapability?.routeId, "/studio");
  assert.equal(MOCK_UX_WORKBENCH_STUDIO.title, "Flow Studio");
  assert.equal(MOCK_UX_WORKBENCH_STUDIO.components[1]?.props?.text, "Flow Studio");
  assert.match(
    String(MOCK_UX_WORKBENCH_STUDIO.components[4]?.props?.content ?? ""),
    /Welcome to Flow Studio/,
  );
});
