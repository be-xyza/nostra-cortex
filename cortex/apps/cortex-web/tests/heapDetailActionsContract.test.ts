import test from "node:test";
import assert from "node:assert/strict";

import { createHeapDetailActionHandlers } from "../src/components/heap/heapDetailActions.ts";
import type { ActionSelectionContext } from "../src/contracts.ts";

test("createHeapDetailActionHandlers routes regenerate through the parent selection callback", () => {
  const regenerateSelections: ActionSelectionContext[] = [];

  const handlers = createHeapDetailActionHandlers({
    artifactId: "artifact-live-1",
    blockType: "agent_solicitation",
    onClose: () => undefined,
    onToggleRelations: () => undefined,
    onViewDiscussion: () => undefined,
    onRegenerate: (selection) => {
      regenerateSelections.push(selection);
    },
  });

  handlers.onRegenerate?.();

  assert.deepEqual(regenerateSelections, [
    {
      selectedArtifactIds: ["artifact-live-1"],
      activeArtifactId: "artifact-live-1",
      selectedCount: 1,
      selectedBlockTypes: ["agent_solicitation"],
    },
  ]);
});

test("createHeapDetailActionHandlers routes relation edits separately from generic edit", () => {
  let relationEditCount = 0;
  let genericEditCount = 0;

  const handlers = createHeapDetailActionHandlers({
    artifactId: "artifact-live-2",
    blockType: "note",
    onClose: () => undefined,
    onToggleRelations: () => {
      relationEditCount += 1;
    },
    onViewDiscussion: () => undefined,
    onRegenerate: () => undefined,
  });

  handlers.onRelationEdit?.({
    selectedArtifactIds: ["artifact-live-2"],
    activeArtifactId: "artifact-live-2",
    selectedCount: 1,
    selectedBlockTypes: ["note"],
  });
  handlers.onEdit?.({
    selectedArtifactIds: ["artifact-live-2"],
    activeArtifactId: "artifact-live-2",
    selectedCount: 1,
    selectedBlockTypes: ["note"],
  });

  assert.equal(relationEditCount, 1);
  assert.equal(genericEditCount, 0);
});
