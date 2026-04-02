import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import {
  applySpatialLayout,
  replaySpatialCommands,
  replaySpatialPayload,
  validateSpatialPayload,
} from "../src/components/a2ui/spatialReplay.ts";

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

test("validateSpatialPayload accepts execution canvas v2 nodes, edges, groups, and annotations", () => {
  const validation = validateSpatialPayload({
    plane_id: "execution-plane-1",
    surface_class: "execution",
    commands: [
      {
        op: "create_shape",
        shape: {
          id: "node-input",
          kind: "node",
          node_class: "input",
          status: "idle",
          x: 100,
          y: 80,
          w: 200,
          h: 120,
          text: "Prompt",
          ports: [
            { id: "out", side: "right", direction: "out" }
          ]
        }
      },
      {
        op: "create_shape",
        shape: {
          id: "node-tool",
          kind: "node",
          node_class: "tool",
          status: "running",
          x: 420,
          y: 90,
          w: 220,
          h: 140,
          text: "Model",
          ports: [
            { id: "in", side: "left", direction: "in" },
            { id: "out", side: "right", direction: "out" }
          ]
        }
      },
      {
        op: "create_shape",
        shape: {
          id: "group-1",
          kind: "group",
          x: 60,
          y: 40,
          w: 640,
          h: 260,
          label: "Execution lane",
          member_ids: ["node-input", "node-tool"],
          collapsed: false
        }
      },
      {
        op: "create_shape",
        shape: {
          id: "annotation-1",
          kind: "annotation",
          x: 120,
          y: 250,
          w: 180,
          h: 80,
          text: "Operator note"
        }
      },
      {
        op: "create_shape",
        shape: {
          id: "edge-1",
          kind: "edge",
          edge_class: "data",
          x: 300,
          y: 140,
          from_shape_id: "node-input",
          to_shape_id: "node-tool",
          from_port_id: "out",
          to_port_id: "in"
        }
      },
      {
        op: "set_selection",
        shape_ids: ["node-tool"]
      },
      {
        op: "set_view_state",
        view_state: { zoom: 1.2, pan_x: 12, pan_y: -4 }
      }
    ]
  } as any);

  assert.equal(validation.errors.length, 0);
});

test("validateSpatialPayload rejects missing edge references and duplicate node ports", () => {
  const validation = validateSpatialPayload({
    plane_id: "execution-plane-2",
    surface_class: "execution",
    commands: [
      {
        op: "create_shape",
        shape: {
          id: "node-bad",
          kind: "node",
          node_class: "tool",
          status: "idle",
          x: 100,
          y: 80,
          ports: [
            { id: "dup", side: "left", direction: "in" },
            { id: "dup", side: "right", direction: "out" }
          ]
        }
      },
      {
        op: "create_shape",
        shape: {
          id: "edge-bad",
          kind: "edge",
          edge_class: "control",
          x: 220,
          y: 140,
          from_shape_id: "node-bad",
          to_shape_id: "missing-node"
        }
      }
    ]
  } as any);

  assert.equal(validation.errors.length, 2);
  assert.ok(validation.errors.some((error) => String(error.reason).includes("duplicate port")));
  assert.ok(validation.errors.some((error) => String(error.reason).includes("unknown node reference")));
});

test("replaySpatialPayload tracks selection and view state for execution canvas surfaces", () => {
  const replayed = replaySpatialPayload({
    plane_id: "execution-plane-3",
    surface_class: "execution",
    focus_bounds: { x: 0, y: 0, w: 800, h: 500 },
    commands: [
      {
        op: "create_shape",
        shape: {
          id: "node-output",
          kind: "node",
          node_class: "output",
          status: "done",
          x: 500,
          y: 100,
          ports: [{ id: "in", side: "left", direction: "in" }]
        }
      },
      {
        op: "set_selection",
        shape_ids: ["node-output"]
      },
      {
        op: "set_view_state",
        view_state: { zoom: 1.4, pan_x: 40, pan_y: 20 }
      }
    ]
  } as any);

  assert.deepEqual(replayed.selection, ["node-output"]);
  assert.equal(replayed.viewState?.zoom, 1.4);
  assert.equal(replayed.shapes.get("node-output")?.kind, "node");
});

test("applySpatialLayout updates known shapes only and reports stale layout refs", () => {
  const replayed = replaySpatialPayload({
    plane_id: "execution-plane-4",
    surface_class: "execution",
    commands: [
      {
        op: "create_shape",
        shape: {
          id: "node-layout",
          kind: "node",
          node_class: "procedure",
          status: "blocked",
          x: 80,
          y: 60
        }
      }
    ]
  } as any);

  const result = applySpatialLayout(replayed, {
    schema_version: "1.0.0",
    plane_id: "execution-plane-4",
    view_spec_id: "viewspec:layout",
    space_id: "meta",
    revision: 2,
    layout: {
      shape_positions: {
        "node-layout": { x: 300, y: 220 },
        "missing-node": { x: 999, y: 999 }
      },
      collapsed_groups: {
        "missing-group": true
      },
      view_state: {
        zoom: 1.25,
        pan_x: 18,
        pan_y: -12,
      },
      selected_shape_ids: ["node-layout", "missing-node"]
    },
    lineage: {
      updated_by: "tester",
      updated_at: "2026-04-01T00:00:00Z"
    }
  } as any);

  assert.equal(result.shapes.get("node-layout")?.x, 300);
  assert.equal(result.shapes.get("node-layout")?.y, 220);
  assert.equal(result.viewState?.zoom, 1.25);
  assert.deepEqual(result.selection, ["node-layout"]);
  assert.deepEqual(result.warnings, ["missing-node", "missing-group", "missing-node"]);
});
