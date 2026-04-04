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
