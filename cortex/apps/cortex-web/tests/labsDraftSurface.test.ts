import assert from "node:assert/strict";
import test from "node:test";

import {
  MOCK_UX_WORKBENCH_LABS,
  PLATFORM_CAPABILITY_CATALOG,
} from "../src/store/seedData.ts";
import {
  EXECUTION_CANVAS_ROUTE,
  SPACE_STUDIO_ROUTE
} from "../src/components/spaces/spaceStudioRoutes.ts";

test("Labs presents Space Studio as a draft surface rather than a live space setting", () => {
  const labsCapability = PLATFORM_CAPABILITY_CATALOG.nodes.find(
    (node) => node.id === "workbench-labs",
  );

  assert.equal(labsCapability?.name, "Labs");
  assert.match(labsCapability?.description ?? "", /draft/i);
  assert.equal(MOCK_UX_WORKBENCH_LABS.title, "Labs");
  assert.equal(MOCK_UX_WORKBENCH_LABS.components[2]?.props?.text, "Try ideas here before they become live spaces or templates.");
  assert.equal(MOCK_UX_WORKBENCH_LABS.components[3]?.props?.text, "Space Capability Overlay");
  assert.equal(MOCK_UX_WORKBENCH_LABS.components[7]?.props?.text, "Space Studio");
  assert.match(
    String(MOCK_UX_WORKBENCH_LABS.components[8]?.props?.text ?? ""),
    /draft a new space/i,
  );
  assert.equal(MOCK_UX_WORKBENCH_LABS.components[9]?.props?.text, "Execution Canvas");
  assert.match(
    String(MOCK_UX_WORKBENCH_LABS.components[10]?.props?.text ?? ""),
    /prototype execution flows/i,
  );
  assert.match(
    String(MOCK_UX_WORKBENCH_LABS.components[12]?.props?.text ?? ""),
    /Drafts stay in Labs/i,
  );
  assert.equal(MOCK_UX_WORKBENCH_LABS.components[14]?.props?.href, SPACE_STUDIO_ROUTE);
  assert.equal(
    MOCK_UX_WORKBENCH_LABS.components[15]?.props?.href,
    `${SPACE_STUDIO_ROUTE}?view=templates`,
  );
  assert.equal(
    MOCK_UX_WORKBENCH_LABS.components[16]?.props?.href,
    EXECUTION_CANVAS_ROUTE,
  );
});
