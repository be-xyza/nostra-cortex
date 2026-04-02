import assert from "node:assert/strict";
import test from "node:test";

import { A2UI_EVENT_TYPES } from "../src/components/a2ui/spatialEventContract.ts";

test("A2UI event contract is locked to approved SpatialPlane event types", () => {
  assert.deepEqual(A2UI_EVENT_TYPES, [
    "button_click",
    "approval",
    "spatial_shape_click",
    "spatial_shape_move",
    "spatial_edge_connect",
    "spatial_adapter_loaded",
    "spatial_adapter_fallback",
    "spatial_adapter_replay",
    "spatial_adapter_replay_failed"
  ]);
});
