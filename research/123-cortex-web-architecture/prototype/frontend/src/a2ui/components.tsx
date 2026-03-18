import React from 'react';
import type { A2uiSurfaceState } from './useA2uiProcessor';

interface ComponentProps {
    id: string;
    surface: A2uiSurfaceState;
    onAction?: (actionName: string, context?: any) => void;
}

// Basic lookup helper
function resolveComponentNode(surface: A2uiSurfaceState, id: string) {
    const node = surface.components.get(id);
    if (!node || !node.component) return null;
    // The first key is the type (e.g. "Column", "Text")
    const type = Object.keys(node.component)[0];
    const props = node.component[type];
    return { type, props };
}

// Recursive Dispatcher
export const A2uiNode: React.FC<ComponentProps> = ({ id, surface, onAction }) => {
    const node = resolveComponentNode(surface, id);
    if (!node) return null;

    switch (node.type) {
        case 'Column':
            return <A2uiColumn {...node.props} surface={surface} onAction={onAction} />;
        case 'Row':
            return <A2uiRow {...node.props} surface={surface} onAction={onAction} />;
        case 'Text':
            return <A2uiText {...node.props} />;
        case 'Card':
            return <A2uiCard {...node.props} surface={surface} onAction={onAction} />;
        case 'Button':
            return <A2uiButton {...node.props} surface={surface} onAction={onAction} />;
        default:
            console.warn(`[A2uiNode] Unhandled component type: ${node.type}`);
            return <div className="p-2 border border-red-500 text-red-500">Unknown Type: {node.type}</div>;
    }
};

// --- PRIMITIVES --- 

const A2uiColumn = ({ children, alignment, crossAlignment, surface, onAction }: any) => {
    const childIds = children?.explicitList || [];

    // Maps literal alignments to Tailwind
    const alignClass = alignment === 'center' ? 'justify-center' : alignment === 'end' ? 'justify-end' : 'justify-start';
    const crossClass = crossAlignment === 'center' ? 'items-center' : crossAlignment === 'end' ? 'items-end' : 'items-start';

    return (
        <div className={`flex flex-col gap-4 w-full h-full ${alignClass} ${crossClass}`}>
            {childIds.map((childId: string) => (
                <A2uiNode key={childId} id={childId} surface={surface} onAction={onAction} />
            ))}
        </div>
    );
};

const A2uiRow = ({ children, alignment, crossAlignment, surface, onAction }: any) => {
    const childIds = children?.explicitList || [];

    const alignClass = alignment === 'center' ? 'justify-center' : alignment === 'end' ? 'justify-end' : 'justify-start';
    const crossClass = crossAlignment === 'center' ? 'items-center' : crossAlignment === 'end' ? 'items-end' : 'items-start';

    return (
        <div className={`flex flex-row gap-4 w-full h-full ${alignClass} ${crossClass}`}>
            {childIds.map((childId: string) => (
                <A2uiNode key={childId} id={childId} surface={surface} onAction={onAction} />
            ))}
        </div>
    );
};

const A2uiText = ({ text, usageHint }: any) => {
    // We extract the literal string from the A2UI Data format
    const content = text?.literalString || "";

    switch (usageHint) {
        case 'h1':
            return <h1 className="text-3xl font-bold tracking-tight text-gray-900 mb-2">{content}</h1>;
        case 'h2':
            return <h2 className="text-2xl font-bold tracking-tight text-gray-900 mb-1">{content}</h2>;
        case 'caption':
            return <p className="text-sm text-gray-500">{content}</p>;
        default:
            return <p className="text-base text-gray-700">{content}</p>;
    }
};

const A2uiCard = ({ child, surface, onAction }: any) => {
    return (
        <div className="bg-white hover:bg-gray-50 transition-colors border border-gray-200 rounded-xl shadow-sm p-6 overflow-hidden">
            {child && <A2uiNode id={child} surface={surface} onAction={onAction} />}
        </div>
    );
};

const A2uiButton = ({ label, child, action, variant, surface, onAction }: any) => {
    const buttonLabel = label?.literalString || "Button";

    const handleClick = (e: React.MouseEvent) => {
        e.preventDefault();
        if (onAction && action) {
            onAction(action.name, action.context);
        }
    };

    const variantClass = variant === 'secondary'
        ? 'bg-white text-gray-900 ring-1 ring-inset ring-gray-300 hover:bg-gray-50'
        : 'bg-blue-600 text-white hover:bg-blue-500 shadow-sm';

    return (
        <button
            onClick={handleClick}
            className={`rounded-md px-3.5 py-2.5 text-sm font-semibold focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 ${variantClass}`}
        >
            {child ? <A2uiNode id={child} surface={surface} onAction={onAction} /> : buttonLabel}
        </button>
    );
};
