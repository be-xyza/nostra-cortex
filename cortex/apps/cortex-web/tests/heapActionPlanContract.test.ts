import assert from "node:assert/strict";
import test from "node:test";

import {
  buildHeapActionPlan,
  type HeapActionId,
} from "../src/components/heap/heapActionPlan.ts";

function actionIds(
  plan: ReturnType<typeof buildHeapActionPlan>,
  zone: "page" | "selection" | "detail" | "detailHeader" | "cardMenu",
) {
  return plan[zone].map((action) => action.id);
}

function actionById(
  plan: ReturnType<typeof buildHeapActionPlan>,
  zone: "page" | "selection" | "detail" | "detailHeader" | "cardMenu",
  id: HeapActionId,
) {
  return plan[zone].find((action) => action.id === id) ?? null;
}

test("heap action plan keeps create in the page action bar when nothing is selected", () => {
  const plan = buildHeapActionPlan({
    selectionCount: 0,
    heapCreateFlowEnabled: true,
    heapParityEnabled: true,
  });

  assert.deepEqual(actionIds(plan, "page"), ["create"]);
  assert.deepEqual(actionIds(plan, "selection"), []);
});

test("heap action plan keeps create visible and enables single-selection actions when one block is selected", () => {
  const plan = buildHeapActionPlan({
    selectionCount: 1,
    heapCreateFlowEnabled: true,
    heapParityEnabled: true,
  });

  assert.deepEqual(actionIds(plan, "page"), ["create"]);
  assert.deepEqual(actionIds(plan, "selection"), [
    "regenerate",
    "refine_selection",
    "export",
    "history",
    "publish",
    "synthesize",
    "pin",
    "delete",
  ]);
  assert.equal(actionById(plan, "selection", "history")?.enabled, true);
  assert.equal(actionById(plan, "selection", "publish")?.enabled, true);
  assert.equal(actionById(plan, "selection", "synthesize")?.enabled, false);
});

test("heap action plan disables synthesize until at least three blocks are selected", () => {
  const twoSelected = buildHeapActionPlan({
    selectionCount: 2,
    heapCreateFlowEnabled: true,
    heapParityEnabled: true,
  });
  const threeSelected = buildHeapActionPlan({
    selectionCount: 3,
    heapCreateFlowEnabled: true,
    heapParityEnabled: true,
  });

  assert.equal(actionById(twoSelected, "selection", "synthesize")?.enabled, false);
  assert.match(
    actionById(twoSelected, "selection", "synthesize")?.disabledReason ?? "",
    /3 blocks/i,
  );
  assert.equal(actionById(threeSelected, "selection", "synthesize")?.enabled, true);
});

test("heap action plan hides create when the create flow is disabled", () => {
  const plan = buildHeapActionPlan({
    selectionCount: 0,
    heapCreateFlowEnabled: false,
    heapParityEnabled: true,
  });

  assert.deepEqual(actionIds(plan, "page"), []);
});

test("heap action plan disables parity-backed actions when heap parity is disabled", () => {
  const plan = buildHeapActionPlan({
    selectionCount: 1,
    heapCreateFlowEnabled: true,
    heapParityEnabled: false,
  });

  for (const action of plan.selection) {
    assert.equal(action.enabled, false);
    assert.match(action.disabledReason ?? "", /disabled/i);
  }
});

test("heap detail action plan reuses shared descriptors", () => {
  const plan = buildHeapActionPlan({
    selectionCount: 1,
    heapCreateFlowEnabled: true,
    heapParityEnabled: true,
  });

  assert.deepEqual(actionIds(plan, "detail"), [
    "discussion",
    "relation_edit",
    "regenerate",
  ]);
  assert.equal(actionById(plan, "detail", "relation_edit")?.enabled, true);
});

test("heap action plan exposes detail header and card menu zones for contextual actions", () => {
  const plan = buildHeapActionPlan({
    selectionCount: 1,
    heapCreateFlowEnabled: true,
    heapParityEnabled: true,
  });

  assert.deepEqual(actionIds(plan, "detailHeader"), ["discussion", "relation_edit", "regenerate"]);
  assert.deepEqual(actionIds(plan, "cardMenu"), ["discussion", "history", "pin", "delete"]);
});

test("heap action plan marks destructive delete actions with confirmation metadata", () => {
  const plan = buildHeapActionPlan({
    selectionCount: 1,
    heapCreateFlowEnabled: true,
    heapParityEnabled: true,
  });

  assert.equal(actionById(plan, "selection", "delete")?.confirmation?.required, true);
  assert.equal(actionById(plan, "selection", "delete")?.confirmation?.style, "danger");
  assert.equal(actionById(plan, "cardMenu", "delete")?.confirmation?.required, true);
  assert.equal(actionById(plan, "cardMenu", "delete")?.confirmation?.style, "danger");
});
