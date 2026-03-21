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

