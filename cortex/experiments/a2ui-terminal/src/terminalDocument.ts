import { A2UINode } from "./types.js";

export const TERMINAL_DOCUMENT_VERSION = "1.0.0";
export const TERMINAL_WIDGETS = new Set(["Container", "Box", "Text", "Spacer", "SelectList"]);
export const WEB_ONLY_WIDGETS = new Set([
    "TldrawCanvas",
    "CapabilityMatrixMap",
    "CapabilityMapV2",
    "WorkflowInstanceTimeline",
    "WorkflowProjectionPreview",
    "SchemaNodeEditor",
    "ContributionGraphViewer",
    "EvaluationDAGViewer",
]);

export type TerminalDocumentValidation = {
    valid: boolean;
    errors: string[];
};

export function validateTerminalDocument(node: unknown): TerminalDocumentValidation {
    const errors: string[] = [];
    validateNode(node, "$", errors);
    return {
        valid: errors.length === 0,
        errors,
    };
}

export function isTerminalDocument(node: unknown): node is A2UINode {
    return validateTerminalDocument(node).valid;
}

export function summarizeTerminalDocumentFailure(node: unknown): string {
    const validation = validateTerminalDocument(node);
    if (validation.valid) {
        return "Terminal document validated successfully.";
    }

    const webOnlyWidget = findFirstWidget(node, (type) => WEB_ONLY_WIDGETS.has(type));
    if (webOnlyWidget) {
        return `Widget '${webOnlyWidget}' is explicitly web-scoped in the current workbench surface matrix.`;
    }

    const unsupportedWidget = findFirstWidget(node, (type) => !TERMINAL_WIDGETS.has(type));
    if (unsupportedWidget) {
        return `Widget '${unsupportedWidget}' is not implemented in the terminal adapter yet.`;
    }

    return `Terminal document validation failed: ${validation.errors[0]}`;
}

function validateNode(value: unknown, path: string, errors: string[]): void {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
        errors.push(`${path} must be an object terminal node`);
        return;
    }

    const node = value as Record<string, unknown>;
    if (typeof node.type !== "string" || node.type.trim().length === 0) {
        errors.push(`${path}.type must be a non-empty string`);
    } else if (!TERMINAL_WIDGETS.has(node.type)) {
        errors.push(`${path}.type '${node.type}' is not part of terminal_document_v1`);
    }

    if (node.id !== undefined && typeof node.id !== "string") {
        errors.push(`${path}.id must be a string when present`);
    }

    if (node.componentProperties !== undefined) {
        if (!node.componentProperties || typeof node.componentProperties !== "object" || Array.isArray(node.componentProperties)) {
            errors.push(`${path}.componentProperties must be an object when present`);
        }
    }

    if (node.children === undefined) {
        return;
    }

    if (!node.children || typeof node.children !== "object" || Array.isArray(node.children)) {
        errors.push(`${path}.children must be an object when present`);
        return;
    }

    const children = node.children as Record<string, unknown>;
    if (children.explicitList === undefined) {
        errors.push(`${path}.children.explicitList must be present when children exists`);
        return;
    }

    if (!Array.isArray(children.explicitList)) {
        errors.push(`${path}.children.explicitList must be an array`);
        return;
    }

    children.explicitList.forEach((child, index) => {
        validateNode(child, `${path}.children.explicitList[${index}]`, errors);
    });
}

function findFirstWidget(value: unknown, predicate: (type: string) => boolean): string | null {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
        return null;
    }

    const queue: Record<string, unknown>[] = [value as Record<string, unknown>];
    while (queue.length > 0) {
        const current = queue.shift()!;
        const type = typeof current.type === "string" ? current.type : null;
        if (type && predicate(type)) {
            return type;
        }

        const children = current.children;
        if (children && typeof children === "object" && !Array.isArray(children)) {
            const explicitList = (children as Record<string, unknown>).explicitList;
            if (Array.isArray(explicitList)) {
                for (const child of explicitList) {
                    if (child && typeof child === "object" && !Array.isArray(child)) {
                        queue.push(child as Record<string, unknown>);
                    }
                }
            }
        }
    }

    return null;
}
