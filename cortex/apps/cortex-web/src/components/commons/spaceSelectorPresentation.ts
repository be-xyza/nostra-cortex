export interface SpaceSelectorTriggerState {
    title: string;
    subtitle: string;
}

interface ResolveSpaceSelectorTriggerStateOptions {
    isMeta: boolean;
    isMulti: boolean;
    activeSpaceCount: number;
    activeSpaceName?: string;
    registryResolved: boolean;
}

export function resolveSpaceSelectorTriggerState(
    options: ResolveSpaceSelectorTriggerStateOptions,
): SpaceSelectorTriggerState {
    const { isMeta, isMulti, activeSpaceCount, activeSpaceName, registryResolved } = options;
    const hasActiveSpace = Boolean(activeSpaceName?.trim());

    if (!registryResolved && !isMeta && !hasActiveSpace) {
        return {
            title: "Loading Spaces...",
            subtitle: "Syncing live registry",
        };
    }

    if (isMeta) {
        return {
            title: "Meta Workbench",
            subtitle: "Aggregated Session",
        };
    }

    if (isMulti) {
        return {
            title: `${activeSpaceCount} Spaces Active`,
            subtitle: "Aggregated Session",
        };
    }

    return {
        title: activeSpaceName?.trim() || "Select Space",
        subtitle: "Active Space",
    };
}
