import React from 'react';
import type { A2uiSurfaceState } from './useA2uiProcessor';
import { A2uiNode } from './components';

interface A2uiRootProps {
    surface: A2uiSurfaceState;
    onAction?: (actionName: string, context?: any) => void;
}

export const A2uiRoot: React.FC<A2uiRootProps> = ({ surface, onAction }) => {
    if (!surface.rootComponentId) {
        return (
            <div className="flex items-center justify-center p-12 text-gray-500 animate-pulse">
                Awaiting UI Stream...
            </div>
        );
    }

    // Inject structural CSS specifically for the surface wrapper, although
    // we are primarily relying on Tailwind via the components mapping now.
    return (
        <div className="w-full h-full relative" data-a2ui-surface={surface.surfaceId}>
            {/* Dynamic styles injected from the A2UI spec (if provided) */}
            {surface.styles && Object.keys(surface.styles).length > 0 && (
                <style dangerouslySetInnerHTML={{
                    __html: `:root { ${Object.entries(surface.styles).map(([k, v]) => `${k}: ${v};`).join('\n')} }`
                }} />
            )}

            <A2uiNode
                id={surface.rootComponentId}
                surface={surface}
                onAction={onAction}
            />
        </div>
    );
};
