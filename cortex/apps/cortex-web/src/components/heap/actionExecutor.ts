import type { ToolbarActionDescriptor, ActionSelectionContext } from "../../contracts.ts";
import { workbenchApi } from "../../api.ts";

export function evaluateActionConstraints(
    action: ToolbarActionDescriptor,
    selection: ActionSelectionContext
): { enabled: boolean; disabledReason?: string } {
    if (!action.enabled) {
        return { enabled: false, disabledReason: action.disabledReason || "Action disabled" };
    }

    if (action.selectionConstraints) {
        const { minSelected, maxSelected, requireSingleSelection } = action.selectionConstraints;
        const count = selection.selectedCount;

        if (requireSingleSelection && count !== 1) {
            return { enabled: false, disabledReason: "Requires exactly one selected block." };
        }

        if (typeof minSelected === "number" && count < minSelected) {
            return {
                enabled: false,
                disabledReason: `Requires at least ${minSelected} selected block${minSelected > 1 ? "s" : ""}.`,
            };
        }

        if (typeof maxSelected === "number" && count > maxSelected) {
            return { enabled: false, disabledReason: `Maximum ${maxSelected} selected blocks allowed.` };
        }
    }

    return { enabled: true };
}

export interface ActionHandlers {
    onDeselect?: (selection: ActionSelectionContext) => void;
    onPinToggled?: () => void;
    onDeleted?: () => void;
    onRegenerate?: (selection: ActionSelectionContext) => void;
    onContextBundle?: (selection: ActionSelectionContext) => void;
    onExport?: (selection: ActionSelectionContext) => void;
    onHistory?: (selection: ActionSelectionContext) => void;
    onPublish?: (selection: ActionSelectionContext) => void;
    onSynthesize?: (selection: ActionSelectionContext) => void;
    onCreateBlock?: (selection: ActionSelectionContext) => void;
    onOpenDiscussion?: (selection: ActionSelectionContext) => void;
    onRelationEdit?: (selection: ActionSelectionContext) => void;
    onEdit?: (selection: ActionSelectionContext) => void;
    confirmAction?: (
        action: ToolbarActionDescriptor,
        selection: ActionSelectionContext,
    ) => boolean | Promise<boolean>;
}

function normalizeActionToken(action: ToolbarActionDescriptor): string {
    return action.action.trim().toLowerCase();
}

async function confirmIfRequired(
    action: ToolbarActionDescriptor,
    selection: ActionSelectionContext,
    handlers: ActionHandlers,
): Promise<boolean> {
    if (!action.confirmation?.required) {
        return action.kind !== "destructive";
    }

    if (handlers.confirmAction) {
        return await handlers.confirmAction(action, selection);
    }

    if (typeof window !== "undefined" && typeof window.confirm === "function") {
        return window.confirm(action.confirmation.message || action.confirmation.title || "Confirm action?");
    }

    return false;
}

export async function executeHeapAction(
    action: ToolbarActionDescriptor,
    selection: ActionSelectionContext,
    handlers: ActionHandlers
) {
    const { enabled, disabledReason } = evaluateActionConstraints(action, selection);

    if (!enabled) return;
    if (!(await confirmIfRequired(action, selection, handlers))) return;

    const ids = selection.selectedArtifactIds;
    const actionToken = normalizeActionToken(action);

    switch (actionToken) {
        case "create":
        case "create_block":
            handlers.onCreateBlock?.(selection);
            break;
        case "pin":
            try {
                await Promise.all(ids.map((id) => workbenchApi.pinHeapBlock(id)));
                handlers.onPinToggled?.();
            } catch (e) {
                console.error("Failed to pin block:", e);
            }
            break;
        case "delete":
            try {
                await Promise.all(ids.map((id) => workbenchApi.deleteHeapBlock(id)));
                handlers.onDeleted?.();
            } catch (e) {
                console.error("Failed to delete block:", e);
            }
            break;
        case "regenerate":
            handlers.onRegenerate?.(selection);
            break;
        case "refine":
        case "refine_selection":
        case "refine_context":
            handlers.onContextBundle?.(selection);
            break;
        case "export":
            handlers.onExport?.(selection);
            break;
        case "history":
            handlers.onHistory?.(selection);
            break;
        case "publish":
            handlers.onPublish?.(selection);
            break;
        case "synthesize":
            handlers.onSynthesize?.(selection);
            break;
        case "discussion":
        case "view_discussion":
            handlers.onOpenDiscussion?.(selection);
            break;
        case "relation_edit":
        case "edit_relations":
            handlers.onRelationEdit?.(selection);
            break;
        case "edit":
            handlers.onEdit?.(selection);
            break;
        default:
            console.warn(`Unrecognized action: capability=${action.capabilityId} action=${action.action}`);
            break;
    }
}
