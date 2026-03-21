import assert from "node:assert/strict";
import test from "node:test";

import {
  buildSpaceStudioRoute,
  isSpaceStudioPath,
  SPACE_STUDIO_ROUTE,
} from "../src/components/spaces/spaceStudioRoutes.ts";

test("space studio routes stay under labs and keep template view explicit", () => {
  assert.equal(SPACE_STUDIO_ROUTE, "/labs/space-studio");
  assert.equal(buildSpaceStudioRoute(), "/labs/space-studio");
  assert.equal(buildSpaceStudioRoute("templates"), "/labs/space-studio?view=templates");
  assert.equal(isSpaceStudioPath("/labs/space-studio"), true);
  assert.equal(isSpaceStudioPath("/labs/space-studio/drafts"), true);
  assert.equal(isSpaceStudioPath("/labs"), false);
  assert.equal(isSpaceStudioPath("/spaces"), false);
});
