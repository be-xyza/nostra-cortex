import assert from "node:assert/strict";
import test from "node:test";

import { executeHeapAction } from "../src/components/heap/actionExecutor.ts";
import type { ActionSelectionContext, ToolbarActionDescriptor } from "../src/contracts.ts";

const selection: ActionSelectionContext = {
  selectedArtifactIds: ["artifact-1"],
  selectedCount: 1,
  selectedBlockTypes: ["note"],
};

function compiledAction(overrides: Partial<ToolbarActionDescriptor> = {}): ToolbarActionDescriptor {
  return {
    id: "action.heap.create",
    capabilityId: "cap.heap.create",
    zone: "heap_page_bar",
    label: "Create",
    kind: "mutation",
    action: "create_block",
    priority: 100,
    group: "primary",
    visible: true,
    enabled: true,
    selectionConstraints: {
      minSelected: 0,
    },
    ...overrides,
  };
}

test("executeHeapAction honors compiled backend create actions", async () => {
  let called = 0;

  await executeHeapAction(compiledAction(), selection, {
    onCreateBlock: () => {
      called += 1;
    },
  });

  assert.equal(called, 1);
});

test("executeHeapAction honors compiled backend delete actions", async () => {
  const originalFetch = globalThis.fetch;
  const calls: string[] = [];
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    calls.push(String(input));
    return new Response(
      JSON.stringify({ accepted: true }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
  }) as typeof fetch;

  try {
    await executeHeapAction(
      compiledAction({
        id: "action.heap.delete",
        capabilityId: "cap.heap.delete",
        zone: "heap_selection_bar",
        label: "Delete",
        kind: "destructive",
        action: "delete",
        group: "danger",
        selectionConstraints: {
          minSelected: 1,
        },
      }),
      selection,
      {},
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 1);
  assert.ok(calls[0]?.endsWith("/api/cortex/studio/heap/blocks/artifact-1/delete"));
});
