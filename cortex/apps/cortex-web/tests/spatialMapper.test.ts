import assert from "node:assert/strict";
import test from "node:test";

import { mapShapesToTldraw } from "../src/components/a2ui/spatialMapper.ts";

test("mapShapesToTldraw maps valid note and arrow shapes", () => {
  const result = mapShapesToTldraw([
    {
      id: "note-1",
      kind: "note",
      x: 100,
      y: 120,
      w: 220,
      h: 120,
      text: "Operator review"
    },
    {
      id: "arrow-1",
      kind: "arrow",
      x: 320,
      y: 180,
      to_x: 500,
      to_y: 260
    }
  ]);

  assert.equal(result.errors.length, 0);
  assert.equal(result.mapped.length, 2);
  assert.equal(result.mapped[0]?.id, "shape:note-1");
  assert.equal(result.mapped[1]?.id, "shape:arrow-1");
});

test("mapShapesToTldraw maps execution canvas v2 shapes", () => {
  const result = mapShapesToTldraw([
    {
      id: "node-1",
      kind: "node",
      node_class: "tool",
      status: "running",
      x: 120,
      y: 80,
      w: 240,
      h: 140,
      text: "Agent tool",
      ports: [
        { id: "in", side: "left", direction: "in" },
        { id: "out", side: "right", direction: "out" }
      ]
    },
    {
      id: "group-1",
      kind: "group",
      x: 80,
      y: 40,
      w: 420,
      h: 260,
      label: "Execution lane",
      member_ids: ["node-1"],
      collapsed: false
    },
    {
      id: "annotation-1",
      kind: "annotation",
      x: 540,
      y: 70,
      w: 180,
      h: 90,
      text: "Operator note"
    },
    {
      id: "edge-1",
      kind: "edge",
      edge_class: "data",
      x: 360,
      y: 150,
      from_shape_id: "node-1",
      to_shape_id: "node-1",
      from_port_id: "out",
      to_port_id: "in"
    }
  ] as any);

  assert.equal(result.errors.length, 0);
  assert.equal(result.mapped.length, 4);
  assert.equal(result.mapped[0]?.type, "geo");
  assert.equal(result.mapped[1]?.type, "geo");
  assert.equal(result.mapped[2]?.type, "geo");
  assert.equal(result.mapped[3]?.type, "arrow");
  assert.equal((result.mapped[3]?.meta as Record<string, unknown>)?.edgeClass, "data");
  assert.equal((result.mapped[3]?.props as Record<string, unknown>)?.dash, "solid");
});

test("mapShapesToTldraw rejects invalid payloads with contract_invalid classification", () => {
  const result = mapShapesToTldraw([
    {
      id: "",
      kind: "note",
      x: 100,
      y: 100
    },
    {
      id: "note-bad-dim",
      kind: "note",
      x: 100,
      y: 100,
      w: 0,
      h: 20
    },
    {
      id: "arrow-missing-end",
      kind: "arrow",
      x: 10,
      y: 10
    },
    {
      id: "note-valid",
      kind: "note",
      x: 10,
      y: 20,
      w: 160,
      h: 80
    }
  ]);

  assert.equal(result.mapped.length, 1);
  assert.equal(result.mapped[0]?.id, "shape:note-valid");
  assert.equal(result.errors.length, 3);
  assert.ok(result.errors.every((error) => error.errorClass === "contract_invalid"));
});

test("mapShapesToTldraw rejects invalid execution canvas v2 payloads", () => {
  const result = mapShapesToTldraw([
    {
      id: "node-bad",
      kind: "node",
      node_class: "tool",
      status: "idle",
      x: 100,
      y: 100,
      ports: [
        { id: "dup", side: "left", direction: "in" },
        { id: "dup", side: "right", direction: "out" }
      ]
    },
    {
      id: "edge-bad",
      kind: "edge",
      edge_class: "data",
      x: 20,
      y: 20,
      from_shape_id: "node-a",
      to_shape_id: ""
    }
  ] as any);

  assert.equal(result.mapped.length, 0);
  assert.equal(result.errors.length, 2);
});

test("mapShapesToTldraw preserves edge class styling hints", () => {
  const result = mapShapesToTldraw([
    {
      id: "node-a",
      kind: "node",
      node_class: "input",
      status: "idle",
      x: 100,
      y: 100,
      ports: [{ id: "out", side: "right", direction: "out" }]
    },
    {
      id: "node-b",
      kind: "node",
      node_class: "output",
      status: "done",
      x: 320,
      y: 100,
      ports: [{ id: "in", side: "left", direction: "in" }]
    },
    {
      id: "edge-control",
      kind: "edge",
      edge_class: "control",
      x: 200,
      y: 140,
      from_shape_id: "node-a",
      to_shape_id: "node-b",
      from_port_id: "out",
      to_port_id: "in"
    },
    {
      id: "edge-branch",
      kind: "edge",
      edge_class: "branch",
      x: 200,
      y: 180,
      from_shape_id: "node-a",
      to_shape_id: "node-b",
      from_port_id: "out",
      to_port_id: "in"
    }
  ] as any);

  assert.equal(result.errors.length, 0);
  assert.equal((result.mapped[2]?.props as Record<string, unknown>)?.dash, "dashed");
  assert.equal((result.mapped[3]?.props as Record<string, unknown>)?.dash, "dotted");
});
