export interface SpaceSelectorTriggerState {
    title: string;
    subtitle: string;
}

interface ResolveSpaceSelectorTriggerStateOptions {
    isMeta: boolean;
    isMulti: boolean;
    activeSpaceCount: number;
    activeSpaceName?: string;
    activeSpaceSourceMode?: string;
    activeSpaceReadiness?: string;
    registryResolved: boolean;
}

export function resolveSpaceSelectorTriggerState(
    options: ResolveSpaceSelectorTriggerStateOptions,
): SpaceSelectorTriggerState {
    const {
        isMeta,
        isMulti,
        activeSpaceCount,
        activeSpaceName,
        activeSpaceSourceMode,
        activeSpaceReadiness,
        registryResolved,
    } = options;
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

    const sourceModeLabel = (() => {
        switch (activeSpaceSourceMode) {
            case "observed":
                return "Observed Live Space";
            case "preview":
                return "Preview Space";
            case "draft":
                return "Draft Space";
            default:
                return "Registered Space";
        }
    })();
    const readinessLabel = (() => {
        switch (activeSpaceReadiness) {
            case "pass":
                return "pass";
            case "fail":
                return "fail";
            case "in_progress":
                return "in progress";
            default:
                return "";
        }
    })();

    return {
        title: activeSpaceName?.trim() || "Select Space",
        subtitle: readinessLabel ? `${sourceModeLabel} · ${readinessLabel}` : sourceModeLabel,
    };
}
