import {
    EXPLORE_AGGREGATION_DENSITIES,
    EXPLORE_AGGREGATION_MODES,
    EXPLORE_CARD_DEPTHS,
    EXPLORE_LAYOUT_MODES,
    EXPLORE_VISUALIZATION_MODES,
    type ExploreAggregationDensity,
    type ExploreAggregationMode,
    type ExploreCardDepth,
    type ExploreLayoutMode,
    type ExploreSettingPrimitive,
    type ExploreSettingVisualKind,
    type ExploreVisualizationMode,
} from "./exploreViewSettings.ts";

export type ExploreViewSettingKey =
    | "projectionIntent"
    | "layoutMode"
    | "cardDepth"
    | "aggregationMode"
    | "aggregationDensity"
    | "showGroupDescriptions"
    | "visualizationMode";

export interface ExploreViewSettingOption {
    value: string;
    label: string;
}

export interface ExploreViewSettingDescriptor {
    key: ExploreViewSettingKey;
    title: string;
    description: string;
    primitive: ExploreSettingPrimitive;
    visualKind?: ExploreSettingVisualKind;
    options: ExploreViewSettingOption[];
}

export interface ExploreViewSettingSection {
    id: "surface" | "aggregate";
    title: string;
    description: string;
    icon?: "sliders";
    controls: ExploreViewSettingDescriptor[];
}

export const EXPLORE_VIEW_SETTING_SECTIONS: ExploreViewSettingSection[] = [
    {
        id: "surface",
        title: "Surface",
        description: "Primary exploration defaults for reading the board.",
        controls: [
            {
                key: "projectionIntent",
                title: "Projection",
                description: "Choose the interpretive lens for the space.",
                primitive: "enum",
                visualKind: "segmented",
                options: [
                    { value: "overview", label: "Overview" },
                    { value: "story", label: "Story" },
                    { value: "density", label: "Density" },
                    { value: "lineage", label: "Lineage" },
                ],
            },
            {
                key: "layoutMode",
                title: "Layout",
                description: "Set how tightly the lane board compacts content.",
                primitive: "enum",
                options: EXPLORE_LAYOUT_MODES.map((value) => ({
                    value,
                    label: labelizeLayoutMode(value),
                })),
            },
            {
                key: "cardDepth",
                title: "Card Depth",
                description: "Control how much detail each card reveals inline.",
                primitive: "enum",
                options: EXPLORE_CARD_DEPTHS.map((value) => ({
                    value,
                    label: labelizeCardDepth(value),
                })),
            },
            {
                key: "visualizationMode",
                title: "Visualization",
                description: "Toggle background graph intensity.",
                primitive: "enum",
                options: EXPLORE_VISUALIZATION_MODES.map((value) => ({
                    value,
                    label: labelizeVisualizationMode(value),
                })),
            },
        ],
    },
    {
        id: "aggregate",
        title: "Aggregate Views",
        description: "Shape grouped prompt and steward views without expanding the board.",
        icon: "sliders",
        controls: [
            {
                key: "aggregationMode",
                title: "Aggregation",
                description: "Choose which grouped views remain visible.",
                primitive: "enum",
                options: EXPLORE_AGGREGATION_MODES.map((value) => ({
                    value,
                    label: labelizeAggregationMode(value),
                })),
            },
            {
                key: "aggregationDensity",
                title: "Preview Depth",
                description: "Set how much of each grouped view is shown inline.",
                primitive: "range",
                visualKind: "slider",
                options: EXPLORE_AGGREGATION_DENSITIES.map((value) => ({
                    value,
                    label: labelizeAggregationDensity(value),
                })),
            },
            {
                key: "showGroupDescriptions",
                title: "Group Descriptions",
                description: "Show or hide compact summaries beneath grouped views.",
                primitive: "boolean",
                visualKind: "toggle",
                options: [
                    { value: "true", label: "On" },
                    { value: "false", label: "Off" },
                ],
            },
        ],
    },
];

export function resolveExploreSettingVisualKind(
    descriptor: ExploreViewSettingDescriptor,
    input: { isCompactPanel?: boolean } = {},
): ExploreSettingVisualKind {
    if (descriptor.visualKind) {
        return descriptor.visualKind;
    }

    switch (descriptor.primitive) {
        case "range":
            return "slider";
        case "boolean":
            return "toggle";
        default:
            return input.isCompactPanel || descriptor.options.length > 3 ? "chips" : "segmented";
    }
}

function labelizeLayoutMode(value: ExploreLayoutMode): string {
    switch (value) {
        case "compact":
            return "Compact";
        case "open":
            return "Open";
        default:
            return "Balanced";
    }
}

function labelizeCardDepth(value: ExploreCardDepth): string {
    switch (value) {
        case "title":
            return "Title";
        case "full":
            return "Full";
        default:
            return "Summary";
    }
}

function labelizeVisualizationMode(value: ExploreVisualizationMode): string {
    switch (value) {
        case "2d":
            return "2D";
        case "3d":
            return "3D";
        default:
            return "Off";
    }
}

function labelizeAggregationMode(value: ExploreAggregationMode): string {
    switch (value) {
        case "prompt_like":
            return "Prompts";
        case "steward_feedback":
            return "Logs";
        case "none":
            return "None";
        default:
            return "All";
    }
}

function labelizeAggregationDensity(value: ExploreAggregationDensity): string {
    switch (value) {
        case "tight":
            return "Tight";
        case "expanded":
            return "Expanded";
        default:
            return "Balanced";
    }
}
