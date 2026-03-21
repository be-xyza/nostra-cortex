import { useState, useEffect, useMemo } from "react";
import { workbenchApi } from "../../api";
import type {
    CompiledActionPlan,
    CompiledActionPlanRequest,
    ActionSelectionContext,
    ToolbarActionDescriptor,
    SurfaceZone,
    ActionZonePlan,
} from "../../contracts";
import { useUiStore } from "../../store/uiStore";
import { useActiveSpaceContext } from "../../store/spacesRegistry";
import { buildHeapActionPlan } from "./heapActionPlan";

type FallbackHeapAction =
    ReturnType<typeof buildHeapActionPlan>["page"][number]
    | ReturnType<typeof buildHeapActionPlan>["selection"][number]
    | ReturnType<typeof buildHeapActionPlan>["detail"][number]
    | ReturnType<typeof buildHeapActionPlan>["detailHeader"][number]
    | ReturnType<typeof buildHeapActionPlan>["cardMenu"][number];

function featureFlagEnabled(value: string | undefined, defaultValue: boolean): boolean {
    if (value === undefined) return defaultValue;
    return value.toLowerCase() !== "false";
}

function fallbackAction(
    zone: SurfaceZone,
    action: FallbackHeapAction,
): ToolbarActionDescriptor {
    const actionMap: Record<string, { capabilityId: string; icon?: string; group: ToolbarActionDescriptor["group"]; kind: ToolbarActionDescriptor["kind"]; action: string; }> = {
        create: { capabilityId: "cap.heap.create", icon: "plus", group: "primary", kind: "panel_toggle", action: "create_block" },
        regenerate: { capabilityId: "cap.heap.regenerate", icon: "refresh-cw", group: "secondary", kind: "mutation", action: "regenerate" },
        refine_selection: { capabilityId: "cap.heap.refine", icon: "wand-2", group: "secondary", kind: "command", action: "refine_selection" },
        export: { capabilityId: "cap.heap.export", icon: "download", group: "secondary", kind: "download", action: "export" },
        history: { capabilityId: "cap.heap.history", icon: "history", group: "secondary", kind: "command", action: "history" },
        publish: { capabilityId: "cap.heap.publish", icon: "upload", group: "primary", kind: "mutation", action: "publish" },
        synthesize: { capabilityId: "cap.heap.synthesize", icon: "sparkles", group: "primary", kind: "command", action: "synthesize" },
        pin: { capabilityId: "cap.heap.pin", icon: "pin", group: "secondary", kind: "mutation", action: "pin" },
        delete: { capabilityId: "cap.heap.delete", icon: "trash-2", group: "danger", kind: "destructive", action: "delete" },
        discussion: { capabilityId: "cap.heap.discussion", icon: "message-square", group: "secondary", kind: "navigation", action: "view_discussion" },
        edit: { capabilityId: "cap.heap.edit", icon: "file-text", group: "secondary", kind: "command", action: "edit" },
    };
    const mapped = actionMap[action.id];

    return {
        id: `fallback.${zone}.${action.id}`,
        capabilityId: mapped.capabilityId,
        zone,
        label: action.label,
        shortLabel: action.label,
        icon: mapped.icon,
        kind: mapped.kind,
        action: mapped.action,
        priority: 0,
        group: mapped.group,
        emphasis: action.emphasis === "danger" ? "danger" : action.emphasis === "primary" ? "primary" : "default",
        visible: true,
        enabled: action.enabled,
        disabledReason: action.disabledReason,
        selectionConstraints: undefined,
        confirmation: undefined,
        stewardGate: action.id === "publish" ? { required: true } : undefined,
    };
}

function defaultZonesForPageType(pageType: "heap_board" | "heap_detail"): SurfaceZone[] {
    return pageType === "heap_detail"
        ? ["heap_detail_header", "heap_detail_footer"]
        : ["heap_page_bar", "heap_selection_bar", "heap_card_menu"];
}

function buildFallbackCompiledActionPlan({
    spaceId,
    actorRole,
    routeId,
    pageType,
    zones,
    selection,
    heapCreateFlowEnabled,
    heapParityEnabled,
}: {
    spaceId: string;
    actorRole: string;
    routeId: string;
    pageType: "heap_board" | "heap_detail";
    zones: SurfaceZone[];
    selection: ActionSelectionContext;
    heapCreateFlowEnabled: boolean;
    heapParityEnabled: boolean;
}): CompiledActionPlan {
    const fallback = buildHeapActionPlan({
        selectionCount: selection.selectedCount,
        heapCreateFlowEnabled,
        heapParityEnabled,
    });
    const zonePlans = [
        {
            zone: "heap_page_bar",
            layoutHint: "row",
            actions: fallback.page.map((action) => fallbackAction("heap_page_bar", action)),
        },
        {
            zone: "heap_selection_bar",
            layoutHint: "pillbar",
            actions: fallback.selection.map((action) => fallbackAction("heap_selection_bar", action)),
        },
        {
            zone: "heap_detail_footer",
            layoutHint: "row",
            actions: fallback.detail.map((action) => fallbackAction("heap_detail_footer", action)),
        },
        {
            zone: "heap_detail_header",
            layoutHint: "pillbar",
            actions: fallback.detailHeader.map((action) => fallbackAction("heap_detail_header", action)),
        },
        {
            zone: "heap_card_menu",
            layoutHint: "row",
            actions: fallback.cardMenu.map((action) => fallbackAction("heap_card_menu", action)),
        },
    ] satisfies ActionZonePlan[];
    const filteredZonePlans = zonePlans.filter((zonePlan) => zones.includes(zonePlan.zone));

    return {
        schemaVersion: "1.0.0",
        generatedAt: new Date().toISOString(),
        planHash: `fallback:${spaceId}:${pageType}:${selection.selectedCount}`,
        spaceId,
        routeId,
        pageType,
        actorRole,
        zones: filteredZonePlans,
    };
}

export function useHeapActionPlan({
    routeId = "/heap",
    pageType = "heap_board",
    zones,
    selection,
    activeFilters,
}: {
    routeId?: string;
    pageType?: "heap_board" | "heap_detail";
    zones?: SurfaceZone[];
    selection?: ActionSelectionContext;
    activeFilters?: CompiledActionPlanRequest["activeFilters"];
} = {}) {
    const sessionUser = useUiStore((state) => state.sessionUser);
    const activeSpaceId = useActiveSpaceContext();
    const [actionPlan, setActionPlan] = useState<CompiledActionPlan | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [source, setSource] = useState<"remote" | "fallback" | "idle">("idle");
    const heapCreateFlowEnabled = useMemo(
        () => featureFlagEnabled(import.meta.env?.VITE_HEAP_CREATE_FLOW_ENABLED, true),
        [],
    );
    const heapParityEnabled = useMemo(
        () => featureFlagEnabled(import.meta.env?.VITE_HEAP_PARITY_ENABLED, true),
        [],
    );
    const resolvedZones = useMemo(
        () => zones ?? defaultZonesForPageType(pageType),
        [pageType, zones],
    );

    useEffect(() => {
        if (!activeSpaceId || !sessionUser) return;

        const fetchPlan = async () => {
            setLoading(true);
            setError(null);
            try {
                const payload: CompiledActionPlanRequest = {
                    schemaVersion: "1.0.0",
                    spaceId: activeSpaceId,
                    actorRole: sessionUser.role,
                    routeId,
                    pageType,
                    intent: "manage_heap",
                    density: "comfortable",
                    zones: resolvedZones,
                    selection: selection || {
                        selectedArtifactIds: [],
                        selectedCount: 0,
                        selectedBlockTypes: []
                    },
                    activeFilters,
                    featureFlags: {
                        heapCreateFlowEnabled,
                        heapParityEnabled,
                    }
                };
                const plan = await workbenchApi.getSpaceActionPlan(
                    activeSpaceId,
                    payload,
                    sessionUser.role,
                    sessionUser.actorId,
                );
                setActionPlan(plan);
                setSource("remote");
            } catch (err) {
                setError(err instanceof Error ? err.message : "Failed to load action plan");
                setActionPlan(
                    buildFallbackCompiledActionPlan({
                        spaceId: activeSpaceId,
                        actorRole: sessionUser.role,
                        routeId,
                        pageType,
                        zones: resolvedZones,
                        selection: selection || {
                            selectedArtifactIds: [],
                            selectedCount: 0,
                            selectedBlockTypes: [],
                        },
                        heapCreateFlowEnabled,
                        heapParityEnabled,
                    }),
                );
                setSource("fallback");
            } finally {
                setLoading(false);
            }
        };

        void fetchPlan();
    }, [
        activeSpaceId,
        sessionUser?.role,
        routeId,
        pageType,
        sessionUser?.actorId,
        selection?.selectedCount,
        selection?.selectedArtifactIds.join(","),
        selection?.activeArtifactId,
        selection?.selectedBlockTypes?.join(","),
        activeFilters?.viewMode,
        activeFilters?.filterMode,
        activeFilters?.selectedTags?.join(","),
        activeFilters?.selectedPageLinks?.join(","),
        heapCreateFlowEnabled,
        heapParityEnabled,
        resolvedZones.join(","),
    ]);

    return { actionPlan, loading, error, source };
}
