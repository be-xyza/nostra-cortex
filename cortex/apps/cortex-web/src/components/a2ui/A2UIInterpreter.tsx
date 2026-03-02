import React from 'react';
import { WidgetRegistry } from './WidgetRegistry';

const warnedUnknownTypes = new Set<string>();

export type A2UINode = {
    id?: string;
    type?: string;
    componentProperties?: Record<string, any>;
    children?: {
        explicitList?: A2UINode[];
    };
};

export function A2UIInterpreter({ node }: { node: A2UINode }): React.ReactElement | null {
    if (!node) return null;

    // Resolve type from explicit 'type' field or infer from first key of componentProperties
    let resolvedType = node.type;

    if (!resolvedType && node.componentProperties) {
        resolvedType = Object.keys(node.componentProperties)[0];
    }

    // Support connecting custom application widgets embedded inside generic A2UI Containers
    const props = node.componentProperties || {};
    if (resolvedType === 'Container' && props.widgetType && typeof props.widgetType === 'string') {
        resolvedType = props.widgetType;
    }

    if (!resolvedType || !WidgetRegistry[resolvedType]) {
        const typeLabel = String(resolvedType || "unknown");
        if (!warnedUnknownTypes.has(typeLabel)) {
            warnedUnknownTypes.add(typeLabel);
            console.warn(`[A2UI] Unknown component type: ${typeLabel}`);
        }

        let childElements: React.ReactNode = null;
        if (node.children?.explicitList) {
            childElements = node.children.explicitList.map((child, index) => (
                <A2UIInterpreter key={child.id || index} node={child} />
            ));
        }

        return (
            <div className="a2ui-unknown-widget border border-cortex-line rounded-cortex bg-cortex-bg-panel p-3 flex flex-col gap-2">
                <div className="text-xs uppercase tracking-wide text-cortex-ink-muted">Unsupported Widget</div>
                <div className="text-sm text-cortex-bad">{typeLabel}</div>
                <div className="text-xs text-cortex-ink-faint">Rendered via safe fallback. Update WidgetRegistry to support this component type.</div>
                {childElements}
            </div>
        );
    }

    const Component = WidgetRegistry[resolvedType];

    let childElements: React.ReactNode = null;
    if (node.children?.explicitList) {
        childElements = node.children.explicitList.map((child, index) => (
            <A2UIInterpreter key={child.id || index} node={child} />
        ));
    }

    return (
        <Component id={node.id || 'unnamed-node'} componentProperties={node.componentProperties || {}}>
            {childElements}
        </Component>
    );
}
