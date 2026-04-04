import type { ActionSelectionContext } from "../../contracts.ts";
import type { ActionHandlers } from "./actionExecutor.ts";

interface CreateHeapDetailActionHandlersOptions {
  artifactId: string;
  blockType: string;
  onClose: () => void;
  onViewDiscussion: (artifactId: string) => void;
  onToggleRelations: () => void;
  onRegenerate?: (selection: ActionSelectionContext) => void;
}

export function createHeapDetailSelection(
  artifactId: string,
  blockType: string,
): ActionSelectionContext {
  return {
    selectedArtifactIds: [artifactId],
    activeArtifactId: artifactId,
    selectedCount: 1,
    selectedBlockTypes: blockType ? [blockType] : [],
  };
}

export function createHeapDetailActionHandlers(
  options: CreateHeapDetailActionHandlersOptions,
): ActionHandlers {
  const selection = createHeapDetailSelection(
    options.artifactId,
    options.blockType,
  );

  return {
    onRegenerate: () => options.onRegenerate?.(selection),
    onDeselect: () => options.onClose(),
    onOpenDiscussion: () => options.onViewDiscussion(options.artifactId),
    onEdit: () => options.onToggleRelations(),
  };
}
