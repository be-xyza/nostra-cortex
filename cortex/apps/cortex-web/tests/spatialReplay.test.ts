import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { replaySpatialCommands } from "../src/components/a2ui/spatialReplay.ts";

function loadDeterministicFixture(): {
  input: { commands: unknown[] };
  expected_final_shapes: unknown[];
} {
  const fixturePath = path.resolve(
    process.cwd(),
    "../../..",
    "shared/a2ui/fixtures/spatial_plane_replay_deterministic_case.json"
  );
  return JSON.parse(readFileSync(fixturePath, "utf8")) as {
    input: { commands: unknown[] };
    expected_final_shapes: unknown[];
  };
}

function sortShapesById(shapes: Array<Record<string, unknown>>): Array<Record<string, unknown>> {
  return [...shapes].sort((a, b) => String(a.id).localeCompare(String(b.id)));
}

test("replaySpatialCommands matches deterministic fixture output", () => {
  const fixture = loadDeterministicFixture();
  const replayed = replaySpatialCommands(fixture.input.commands as any);
  const replayedShapes = sortShapesById(Array.from(replayed.values()) as Array<Record<string, unknown>>);
  const expected = sortShapesById(fixture.expected_final_shapes as Array<Record<string, unknown>>);
  assert.deepEqual(replayedShapes, expected);
});

test("replaySpatialCommands is deterministic for identical command stream", () => {
  const fixture = loadDeterministicFixture();
  const first = replaySpatialCommands(fixture.input.commands as any);
  const second = replaySpatialCommands(fixture.input.commands as any);

  const firstShapes = sortShapesById(Array.from(first.values()) as Array<Record<string, unknown>>);
  const secondShapes = sortShapesById(Array.from(second.values()) as Array<Record<string, unknown>>);
  assert.deepEqual(firstShapes, secondShapes);
});

