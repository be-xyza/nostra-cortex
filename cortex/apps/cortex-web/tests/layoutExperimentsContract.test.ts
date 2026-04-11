import test from "node:test";
import assert from "node:assert/strict";
import {
  EXPERIMENTAL_LAYOUT_DESCRIPTIONS,
  EXPERIMENTAL_LAYOUT_FAMILIES,
  EXPERIMENTAL_LAYOUT_LABELS,
  LAYOUT_MATRIX_SAMPLE_BLOCKS,
} from "../src/components/a2ui/catalogue/layoutMatrixCatalogueModel.ts";

test("layout matrix catalogue model keeps the normalized experimental topology vocabulary local", () => {
  assert.deepEqual(EXPERIMENTAL_LAYOUT_FAMILIES, [
    "lane_board",
    "spatial_bsp",
    "force_graph",
    "time_indexed",
  ]);
  for (const family of EXPERIMENTAL_LAYOUT_FAMILIES) {
    assert.ok(EXPERIMENTAL_LAYOUT_LABELS[family].length > 0);
    assert.ok(EXPERIMENTAL_LAYOUT_DESCRIPTIONS[family].length > 0);
  }
});

test("layout matrix sample blocks stay aligned to the Nodepad-inspired reference vocabulary", () => {
  assert.equal(LAYOUT_MATRIX_SAMPLE_BLOCKS.length, 7);
  assert.equal(LAYOUT_MATRIX_SAMPLE_BLOCKS[0]?.type, "claim");
  assert.equal(LAYOUT_MATRIX_SAMPLE_BLOCKS[6]?.type, "thesis");
  assert.match(
    LAYOUT_MATRIX_SAMPLE_BLOCKS[6]?.title ?? "",
    /topology contracts/i
  );
});
