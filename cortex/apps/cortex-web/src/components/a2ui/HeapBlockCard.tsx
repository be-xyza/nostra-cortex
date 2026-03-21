import React from "react";
import { A2UIComponentProps } from "./WidgetRegistry";

export function HeapBlockCard({ componentProperties }: A2UIComponentProps) {
    const props = componentProperties["HeapBlockCard"] as Record<string, unknown> || {};

    // Legacy support
    const blockType = String(props.blockType || "");
    const author = String(props.author || "");

    // New A2UI dynamic object rendering support
    const title = String(props.title || blockType || "Details");
    const attributes = (props.attributes as Record<string, unknown>) || {};

    return (
        <div className="a2ui-heap-summary-card flex flex-col gap-2 p-4 border border-gray-700 bg-gray-800 rounded-lg shadow-sm">
            <div className="a2ui-heap-summary-card__header flex justify-between items-center text-xs text-gray-400 border-b border-gray-700/50 pb-2 mb-2">
                <span className="a2ui-heap-summary-card__badge uppercase tracking-wider font-semibold text-blue-300">{title}</span>
                {author && <span>By {author}</span>}
            </div>

            <div className="a2ui-heap-summary-card__body">
                <div className="flex flex-col gap-1.5">
                    {Object.entries(attributes).map(([k, v]) => (
                        <div key={k} className="a2ui-heap-summary-card__attr flex justify-between items-start bg-gray-900/30 p-2 rounded border border-gray-700/30">
                            <span className="a2ui-heap-summary-card__attr-key text-xs font-medium text-gray-400 w-1/3 truncate" title={k}>{k}:</span>
                            <span className="a2ui-heap-summary-card__attr-value text-xs text-gray-200 w-2/3 break-all text-right">{String(v)}</span>
                        </div>
                    ))}
                    {Object.keys(attributes).length === 0 && (
                        <div className="text-xs text-gray-500 italic p-2">No attributes</div>
                    )}
                </div>
            </div>
        </div>
    );
}
