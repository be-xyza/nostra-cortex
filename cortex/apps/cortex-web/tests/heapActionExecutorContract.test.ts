import assert from "node:assert/strict";
import test from "node:test";

import { executeHeapAction } from "../src/components/heap/actionExecutor.ts";
import type { ActionSelectionContext, ToolbarActionDescriptor } from "../src/contracts.ts";

function action(overrides: Partial<ToolbarActionDescriptor>): ToolbarActionDescriptor {
  return {
    id: "test-action",
    capabilityId: "cap.heap.test",
    zone: "heap_selection_bar",
    label: "Test",
    icon: "test",
    kind: "command",
    action: "test",
    priority: 0,
    group: "secondary",
    visible: true,
    enabled: true,
    ...overrides,
  };
}

const selection: ActionSelectionContext = {
  selectedArtifactIds: ["artifact-1"],
  activeArtifactId: "artifact-1",
  selectedCount: 1,
  selectedBlockTypes: ["note"],
};

test("executeHeapAction blocks destructive actions when confirmation is rejected", async () => {
  const originalFetch = globalThis.fetch;
  let fetchCalls = 0;
  globalThis.fetch = (async () => {
    fetchCalls += 1;
    return new Response("{}", { status: 200, headers: { "Content-Type": "application/json" } });
  }) as typeof fetch;

  try {
    await executeHeapAction(
      action({
        id: "fallback.heap_selection_bar.delete",
        capabilityId: "cap.heap.delete",
        kind: "destructive",
        action: "delete",
        group: "danger",
        confirmation: {
          required: true,
          style: "danger",
          title: "Delete selected blocks?",
          message: "Delete selected blocks?",
        },
      }),
      selection,
      { confirmAction: () => false },
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(fetchCalls, 0);
});

test("executeHeapAction blocks destructive actions that lack confirmation metadata", async () => {
  const originalFetch = globalThis.fetch;
  let fetchCalls = 0;
  globalThis.fetch = (async () => {
    fetchCalls += 1;
    return new Response("{}", { status: 200, headers: { "Content-Type": "application/json" } });
  }) as typeof fetch;

  try {
    await executeHeapAction(
      action({
        id: "remote.heap_selection_bar.delete",
        capabilityId: "cap.heap.delete",
        kind: "destructive",
        action: "delete",
        group: "danger",
      }),
      selection,
      {},
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(fetchCalls, 0);
});

test("executeHeapAction allows destructive actions after confirmation approval", async () => {
  const originalFetch = globalThis.fetch;
  const calls: string[] = [];
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    calls.push(String(input));
    return new Response(
      JSON.stringify({
        accepted: true,
        artifactId: "artifact-1",
        action: "deleted",
        updatedAt: "2026-04-29T00:00:00Z",
      }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
  }) as typeof fetch;

  try {
    await executeHeapAction(
      action({
        id: "fallback.heap_selection_bar.delete",
        capabilityId: "cap.heap.delete",
        kind: "destructive",
        action: "delete",
        group: "danger",
        confirmation: {
          required: true,
          style: "danger",
          title: "Delete selected blocks?",
          message: "Delete selected blocks?",
        },
      }),
      selection,
      { confirmAction: () => true },
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.equal(calls.length, 1);
  assert.match(calls[0] ?? "", /\/api\/cortex\/studio\/heap\/blocks\/artifact-1\/delete$/);
});

test("executeHeapAction routes relation_edit separately from generic edit", async () => {
  let relationEditCount = 0;
  let genericEditCount = 0;

  await executeHeapAction(
    action({
      id: "fallback.heap_detail_header.relation_edit",
      capabilityId: "cap.heap.relation_edit",
      zone: "heap_detail_header",
      kind: "panel_toggle",
      action: "relation_edit",
    }),
    selection,
    {
      onRelationEdit: () => {
        relationEditCount += 1;
      },
      onEdit: () => {
        genericEditCount += 1;
      },
    },
  );

  assert.equal(relationEditCount, 1);
  assert.equal(genericEditCount, 0);
});
