import test from "node:test";
import assert from "node:assert/strict";
import {
  DEFAULT_DATA_TABLE_ROW_HREF_FIELD,
  DEFAULT_DATA_TABLE_ROW_KEY_FIELD,
  projectDataTable,
} from "../src/components/a2ui/dataTable.ts";

test("projectDataTable hides metadata columns and preserves row href fields", () => {
  const projection = projectDataTable({
    columns: ["Run ID", "Status"],
    rows: [
      {
        _row_id: "run-001",
        _href: "/agents?node_id=agent_run:run-001",
        "Run ID": "run-001",
        Status: "ready",
      },
    ],
  });

  assert.deepEqual(projection.columns, ["Run ID", "Status"]);
  assert.equal(projection.rowHrefField, DEFAULT_DATA_TABLE_ROW_HREF_FIELD);
  assert.equal(projection.rowKeyField, DEFAULT_DATA_TABLE_ROW_KEY_FIELD);
  assert.equal(projection.rows[0]?._row_id, "run-001");
  assert.equal(
    projection.rows[0]?._href,
    "/agents?node_id=agent_run:run-001"
  );
  assert.ok(!projection.columns.includes("_row_id"));
  assert.ok(!projection.columns.includes("_href"));
});
