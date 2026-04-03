import assert from "node:assert/strict";
import test from "node:test";

import {
  buildExecutionCanvasRoute,
  buildSpaceStudioRoute,
  EXECUTION_CANVAS_ROUTE,
  isExecutionCanvasPath,
  isSpaceStudioPath,
  SPACE_STUDIO_ROUTE,
} from "../src/components/spaces/spaceStudioRoutes.ts";

test("space studio routes stay under labs and keep template view explicit", () => {
  assert.equal(SPACE_STUDIO_ROUTE, "/labs/space-studio");
  assert.equal(EXECUTION_CANVAS_ROUTE, "/labs/execution-canvas");
  assert.equal(buildSpaceStudioRoute(), "/labs/space-studio");
  assert.equal(buildSpaceStudioRoute("templates"), "/labs/space-studio?view=templates");
  assert.equal(buildExecutionCanvasRoute(), "/labs/execution-canvas");
  assert.equal(isSpaceStudioPath("/labs/space-studio"), true);
  assert.equal(isSpaceStudioPath("/labs/space-studio/drafts"), true);
  assert.equal(isExecutionCanvasPath("/labs/execution-canvas"), true);
  assert.equal(isExecutionCanvasPath("/labs/execution-canvas/session"), true);
  assert.equal(isSpaceStudioPath("/labs"), false);
  assert.equal(isExecutionCanvasPath("/labs"), false);
  assert.equal(isSpaceStudioPath("/spaces"), false);
  assert.equal(isExecutionCanvasPath("/spaces"), false);
});
