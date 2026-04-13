import { useEffect, useMemo, useState } from "react";

import { resolveExploreSurfacePolicy, type ExploreProjectionIntent } from "./exploreSurfacePolicy.ts";

export type ExploreLayoutMode = "compact" | "balanced" | "open";
export type ExploreCardDepth = "title" | "summary" | "full";
export type ExploreAggregationMode = "all" | "prompt_like" | "steward_feedback" | "none";
export type ExploreAggregationDensity = "tight" | "balanced" | "expanded";
export type ExploreVisualizationMode = "off" | "2d" | "3d";
export type ExploreSettingOrigin = "space" | "user" | "session";
export type ExploreSettingPrimitive = "enum" | "range" | "boolean";
export type ExploreSettingVisualKind = "chips" | "segmented" | "slider" | "toggle";

export interface ExploreViewSettings {
    projectionIntent: ExploreProjectionIntent;
    layoutMode: ExploreLayoutMode;
    cardDepth: ExploreCardDepth;
    aggregationMode: ExploreAggregationMode;
    aggregationDensity: ExploreAggregationDensity;
    showGroupDescriptions: boolean;
    visualizationMode: ExploreVisualizationMode;
}

export interface ExploreViewSettingsDerived {
    laneCap: number;
    groupPreviewCount: number;
    showCardMetadata: boolean;
    showCardRelations: boolean;
}

export interface ExploreViewDefaults extends ExploreViewSettings {}
export interface ExploreViewOverrides extends Partial<ExploreViewSettings> {}

export interface ExploreViewResolvedSettings {
    defaults: ExploreViewDefaults;
    userOverrides: ExploreViewOverrides;
    sessionOverrides: ExploreViewOverrides;
    effective: ExploreViewSettings;
    derived: ExploreViewSettingsDerived;
    provenance: Record<keyof ExploreViewSettings, ExploreSettingOrigin>;
}

export interface BuildExploreSpaceDefaultsInput {
    spaceArchetype?: string;
    ambientGraphVariant?: ExploreVisualizationMode;
}

export interface ResolveExploreViewSettingsInput {
    defaults: ExploreViewDefaults;
    userOverrides?: ExploreViewOverrides;
    sessionOverrides?: ExploreViewOverrides;
    reduceMotion?: boolean;
    isMobile?: boolean;
}

export const EXPLORE_LAYOUT_MODES: ExploreLayoutMode[] = ["compact", "balanced", "open"];
export const EXPLORE_CARD_DEPTHS: ExploreCardDepth[] = ["title", "summary", "full"];
export const EXPLORE_AGGREGATION_MODES: ExploreAggregationMode[] = [
    "all",
    "prompt_like",
    "steward_feedback",
    "none",
];
export const EXPLORE_AGGREGATION_DENSITIES: ExploreAggregationDensity[] = [
    "tight",
    "balanced",
    "expanded",
];
export const EXPLORE_VISUALIZATION_MODES: ExploreVisualizationMode[] = ["off", "2d", "3d"];

const STORAGE_PREFIX = "cortex.explore.view.";

const LAYOUT_MODE_CAPS: Record<ExploreLayoutMode, ExploreViewSettingsDerived> = {
    compact: {
        laneCap: 4,
        groupPreviewCount: 1,
        showCardMetadata: false,
        showCardRelations: false,
    },
    balanced: {
        laneCap: 3,
        groupPreviewCount: 2,
        showCardMetadata: false,
        showCardRelations: false,
    },
    open: {
        laneCap: 2,
        groupPreviewCount: 3,
        showCardMetadata: true,
        showCardRelations: true,
    },
};

const AGGREGATION_DENSITY_PREVIEW_COUNTS: Record<ExploreAggregationDensity, number> = {
    tight: 1,
    balanced: 2,
    expanded: 4,
};

export function buildExploreSpaceDefaults({
    spaceArchetype,
    ambientGraphVariant,
}: BuildExploreSpaceDefaultsInput): ExploreViewDefaults {
    const surfacePolicy = resolveExploreSurfacePolicy({ spaceArchetype });
    const projectionIntent = surfacePolicy.projectionIntent;

    return {
        projectionIntent,
        layoutMode: projectionIntent === "density" ? "compact" : "balanced",
        cardDepth: "summary",
        aggregationMode:
            projectionIntent === "lineage" ? "steward_feedback" : "all",
        aggregationDensity: projectionIntent === "density" ? "tight" : "balanced",
        showGroupDescriptions: projectionIntent !== "density",
        visualizationMode: ambientGraphVariant ?? "off",
    };
}

export function normalizeExploreViewSettings(
    value: Partial<ExploreViewSettings> | null | undefined,
    fallback: ExploreViewSettings,
): ExploreViewSettings {
    if (!value) {
        return fallback;
    }

    return {
        projectionIntent: isExploreProjectionIntent(value.projectionIntent)
            ? value.projectionIntent
            : fallback.projectionIntent,
        layoutMode: isExploreLayoutMode(value.layoutMode)
            ? value.layoutMode
            : fallback.layoutMode,
        cardDepth: isExploreCardDepth(value.cardDepth)
            ? value.cardDepth
            : fallback.cardDepth,
        aggregationMode: isExploreAggregationMode(value.aggregationMode)
            ? value.aggregationMode
            : fallback.aggregationMode,
        aggregationDensity: isExploreAggregationDensity(value.aggregationDensity)
            ? value.aggregationDensity
            : fallback.aggregationDensity,
        showGroupDescriptions: typeof value.showGroupDescriptions === "boolean"
            ? value.showGroupDescriptions
            : fallback.showGroupDescriptions,
        visualizationMode: isExploreVisualizationMode(value.visualizationMode)
            ? value.visualizationMode
            : fallback.visualizationMode,
    };
}

export function resolveExploreViewSettings(
    input: ResolveExploreViewSettingsInput,
): ExploreViewResolvedSettings {
    const userOverrides = normalizeExploreOverrides(input.userOverrides);
    const sessionOverrides = normalizeExploreOverrides(input.sessionOverrides);

    const baseEffective = normalizeExploreViewSettings(input.defaults, input.defaults);
    const userEffective = normalizeExploreViewSettings(
        { ...baseEffective, ...userOverrides },
        baseEffective,
    );
    const sessionEffective = normalizeExploreViewSettings(
        { ...userEffective, ...sessionOverrides },
        userEffective,
    );
    const effective = applyRuntimeSafeguards(sessionEffective, {
        reduceMotion: input.reduceMotion ?? false,
        isMobile: input.isMobile ?? false,
    });

    return {
        defaults: input.defaults,
        userOverrides,
        sessionOverrides,
        effective,
        derived: resolveExploreDerivedSettings(effective, {
            reduceMotion: input.reduceMotion ?? false,
            isMobile: input.isMobile ?? false,
        }),
        provenance: resolveExploreSettingProvenance(
            input.defaults,
            userOverrides,
            sessionOverrides,
            sessionEffective,
        ),
    };
}

export function readExploreUserOverrides(spaceId: string): ExploreViewOverrides {
    if (typeof window === "undefined") {
        return {};
    }

    try {
        const raw = window.localStorage.getItem(storageKey(spaceId));
        if (!raw) {
            return {};
        }
        return normalizeExploreOverrides(JSON.parse(raw));
    } catch {
        return {};
    }
}

export function writeExploreUserOverrides(spaceId: string, overrides: ExploreViewOverrides): void {
    if (typeof window === "undefined") {
        return;
    }

    try {
        const normalized = normalizeExploreOverrides(overrides);
        if (Object.keys(normalized).length === 0) {
            window.localStorage.removeItem(storageKey(spaceId));
            return;
        }
        window.localStorage.setItem(storageKey(spaceId), JSON.stringify(normalized));
    } catch {
        // Best-effort persistence only.
    }
}

export function clearExploreUserOverrides(spaceId: string): void {
    writeExploreUserOverrides(spaceId, {});
}

export function useExploreViewSettings(input: {
    spaceId: string;
    defaults: ExploreViewDefaults;
    sessionOverrides?: ExploreViewOverrides;
    reduceMotion?: boolean;
    isMobile?: boolean;
}) {
    const [userOverrides, setUserOverrides] = useState<ExploreViewOverrides>(() =>
        readExploreUserOverrides(input.spaceId),
    );

    useEffect(() => {
        setUserOverrides(readExploreUserOverrides(input.spaceId));
    }, [input.spaceId]);

    useEffect(() => {
        writeExploreUserOverrides(input.spaceId, userOverrides);
    }, [input.spaceId, userOverrides]);

    const resolved = useMemo(
        () =>
            resolveExploreViewSettings({
                defaults: input.defaults,
                userOverrides,
                sessionOverrides: input.sessionOverrides,
                reduceMotion: input.reduceMotion,
                isMobile: input.isMobile,
            }),
        [input.defaults, input.isMobile, input.reduceMotion, input.sessionOverrides, userOverrides],
    );

    const setUserOverride = (patch: Partial<ExploreViewSettings>) => {
        setUserOverrides((current) => normalizeExploreOverrides({ ...current, ...patch }));
    };

    const resetUserOverrides = () => {
        setUserOverrides({});
    };

    return {
        ...resolved,
        setUserOverride,
        resetUserOverrides,
        userOverrides,
    };
}

function storageKey(spaceId: string): string {
    return `${STORAGE_PREFIX}${spaceId}`;
}

function normalizeExploreOverrides(value: unknown): ExploreViewOverrides {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
        return {};
    }

    const record = value as Record<string, unknown>;
    const overrides: ExploreViewOverrides = {};

    if (isExploreProjectionIntent(record.projectionIntent)) {
        overrides.projectionIntent = record.projectionIntent;
    }
    if (isExploreLayoutMode(record.layoutMode)) {
        overrides.layoutMode = record.layoutMode;
    }
    if (isExploreCardDepth(record.cardDepth)) {
        overrides.cardDepth = record.cardDepth;
    }
    if (isExploreAggregationMode(record.aggregationMode)) {
        overrides.aggregationMode = record.aggregationMode;
    }
    if (isExploreVisualizationMode(record.visualizationMode)) {
        overrides.visualizationMode = record.visualizationMode;
    }
    if (typeof record.showGroupDescriptions === "boolean") {
        overrides.showGroupDescriptions = record.showGroupDescriptions;
    }

    return overrides;
}

function resolveExploreDerivedSettings(
    settings: ExploreViewSettings,
    runtime: { reduceMotion: boolean; isMobile: boolean },
): ExploreViewSettingsDerived {
    const base = LAYOUT_MODE_CAPS[settings.layoutMode];
    const previewCount = AGGREGATION_DENSITY_PREVIEW_COUNTS[settings.aggregationDensity];
    const safePreviewCount = runtime.isMobile ? Math.min(previewCount, 2) : previewCount;
    return {
        laneCap: base.laneCap,
        groupPreviewCount: safePreviewCount,
        showCardMetadata: base.showCardMetadata || settings.cardDepth === "full",
        showCardRelations: base.showCardRelations || settings.cardDepth === "full",
    };
}

function applyRuntimeSafeguards(
    settings: ExploreViewSettings,
    runtime: { reduceMotion: boolean; isMobile: boolean },
): ExploreViewSettings {
    const next = { ...settings };

    if ((runtime.reduceMotion || runtime.isMobile) && next.visualizationMode === "3d") {
        next.visualizationMode = "2d";
    }

    if (runtime.isMobile && next.layoutMode === "open") {
        next.layoutMode = "balanced";
    }

    if (runtime.isMobile) {
        next.showGroupDescriptions = false;
    }

    return next;
}

function resolveExploreSettingProvenance(
    defaults: ExploreViewDefaults,
    userOverrides: ExploreViewOverrides,
    sessionOverrides: ExploreViewOverrides,
    effective: ExploreViewSettings,
): Record<keyof ExploreViewSettings, ExploreSettingOrigin> {
    return {
        projectionIntent: resolveOrigin(defaults.projectionIntent, userOverrides.projectionIntent, sessionOverrides.projectionIntent, effective.projectionIntent),
        layoutMode: resolveOrigin(defaults.layoutMode, userOverrides.layoutMode, sessionOverrides.layoutMode, effective.layoutMode),
        cardDepth: resolveOrigin(defaults.cardDepth, userOverrides.cardDepth, sessionOverrides.cardDepth, effective.cardDepth),
        aggregationMode: resolveOrigin(defaults.aggregationMode, userOverrides.aggregationMode, sessionOverrides.aggregationMode, effective.aggregationMode),
        aggregationDensity: resolveOrigin(defaults.aggregationDensity, userOverrides.aggregationDensity, sessionOverrides.aggregationDensity, effective.aggregationDensity),
        showGroupDescriptions: resolveOrigin(defaults.showGroupDescriptions, userOverrides.showGroupDescriptions, sessionOverrides.showGroupDescriptions, effective.showGroupDescriptions),
        visualizationMode: resolveOrigin(defaults.visualizationMode, userOverrides.visualizationMode, sessionOverrides.visualizationMode, effective.visualizationMode),
    };
}

function resolveOrigin<T>(
    defaultValue: T,
    userValue: T | undefined,
    sessionValue: T | undefined,
    effectiveValue: T,
): ExploreSettingOrigin {
    if (sessionValue !== undefined && sessionValue === effectiveValue) {
        return "session";
    }
    if (userValue !== undefined && userValue === effectiveValue && userValue !== defaultValue) {
        return "user";
    }
    return "space";
}

function isExploreProjectionIntent(value: unknown): value is ExploreProjectionIntent {
    return value === "overview" || value === "story" || value === "density" || value === "lineage";
}

function isExploreLayoutMode(value: unknown): value is ExploreLayoutMode {
    return value === "compact" || value === "balanced" || value === "open";
}

function isExploreCardDepth(value: unknown): value is ExploreCardDepth {
    return value === "title" || value === "summary" || value === "full";
}

function isExploreAggregationMode(value: unknown): value is ExploreAggregationMode {
    return value === "all" || value === "prompt_like" || value === "steward_feedback" || value === "none";
}

function isExploreAggregationDensity(value: unknown): value is ExploreAggregationDensity {
    return value === "tight" || value === "balanced" || value === "expanded";
}

function isExploreVisualizationMode(value: unknown): value is ExploreVisualizationMode {
    return value === "off" || value === "2d" || value === "3d";
}
