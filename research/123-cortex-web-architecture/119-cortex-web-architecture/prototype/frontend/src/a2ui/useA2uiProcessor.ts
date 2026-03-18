import { useState, useCallback } from 'react';

// --- A2UI Types ---
export type ComponentId = string;

export interface A2uiAction {
    name: string;
    context?: Array<{ key: string; value: { literalString?: string } }>;
}

export interface AnyComponent {
    id: ComponentId;
    component: {
        [componentType: string]: any; // e.g. "Text": { text: { literalString: "Hello" } }
    };
}

export interface A2uiSurfaceState {
    surfaceId: string;
    rootComponentId: string | null;
    components: Map<ComponentId, AnyComponent>;
    styles: Record<string, string>;
}

// --- Processor Hook ---
export function useA2uiProcessor() {
    const [surfaces, setSurfaces] = useState<Map<string, A2uiSurfaceState>>(new Map());

    const processMessage = useCallback((message: any) => {
        setSurfaces((prevSurfaces) => {
            // Create a shallow copy of the surface map to trigger React re-renders
            const newSurfaces = new Map(prevSurfaces);

            // Handle specific message types mapping to our A2UI proto
            if ('beginRendering' in message) {
                const payload = message.beginRendering;
                const currentSurface = newSurfaces.get(payload.surfaceId) || {
                    surfaceId: payload.surfaceId,
                    rootComponentId: null,
                    components: new Map(),
                    styles: {},
                };

                newSurfaces.set(payload.surfaceId, {
                    ...currentSurface,
                    rootComponentId: payload.root,
                    styles: payload.styles ?? currentSurface.styles,
                });

            } else if ('surfaceUpdate' in message) {
                const payload = message.surfaceUpdate;
                const currentSurface = newSurfaces.get(payload.surfaceId) || {
                    surfaceId: payload.surfaceId,
                    rootComponentId: null,
                    components: new Map(),
                    styles: {},
                };

                // We must clone the components map to ensure referential equality checks trigger updates
                const nextComponents = new Map(currentSurface.components);

                for (const comp of payload.components) {
                    nextComponents.set(comp.id, comp);
                }

                newSurfaces.set(payload.surfaceId, {
                    ...currentSurface,
                    components: nextComponents,
                });

            } else if ('deleteSurface' in message) {
                if (message.deleteSurface.delete === true) {
                    const surfaceId = message.surfaceId || message.deleteSurface.surfaceId; // fallback
                    if (surfaceId) newSurfaces.delete(surfaceId);
                }
            } else {
                console.warn("[A2uiProcessor] Unrecognized message format:", message);
            }

            return newSurfaces;
        });
    }, []);

    return {
        surfaces,
        processMessage,
    };
}
