import React, { useState, useEffect, useMemo, useCallback, useRef } from "react";
import { ChevronDown, Plus, Menu, MessagesSquare, Sliders, Filter, X } from "lucide-react";
import { useSearchParams, useLocation } from "react-router-dom";
import { gatewayBaseUrl, resolveWorkbenchSpaceId, workbenchApi } from "../../api.ts";
import type {
    ArtifactGovernanceEnvelope,
    EmitHeapBlockRequest,
    HeapBlockHistoryResponse,
    HeapBlockListItem,
    HeapUploadParserProfileRecord,
    HeapStewardGateValidateResponse,
    HeapUploadArtifactResponse,
    HeapUploadExtractionStatusResponse,
} from "../../contracts.ts";
import { HeapBlockCard } from "./HeapBlockCard";
import { displayBlockType } from "../a2ui/ArtifactAssetViewer";
import { HeapActionBar } from "./HeapActionBar";
import { HeapDetailModal } from "./HeapDetailModal";
import { StewardGateModal } from "./StewardGateModal";
import { useHeapActionPlan } from "./useHeapActionPlan";
import { type ActionHandlers } from "./actionExecutor";
import type { ActionSelectionContext } from "../../contracts.ts";
import { HeapFilterSidebar, HeapFilterMode } from "./HeapFilterSidebar";
import { AgentActivityPanel } from "./AgentActivityPanel";
import { CommentSidebar } from "./CommentSidebar";
import { useUiStore } from "../../store/uiStore";
import { useActiveSpaceContext, useActiveSpaceRecord } from "../../store/spacesRegistry";
import {
    buildHeapViewCounts,
    filterHeapBlocksByReviewLane,
    filterHeapBlocksByView,
    heapPrimaryViewModeParam,
    normalizeHeapPrimaryViewMode,
    type HeapPrimaryViewMode,
    type HeapReviewLane,
    readHeapBlockReviewLane,
} from "./heapViewModel";
import { buildHeapLanes, resolveHeapLaneCount } from "./heapLaneLayout";
import { resolveExploreSurfacePolicy } from "./exploreSurfacePolicy";
import { AmbientGraphBackground } from "./AmbientGraphBackground";
import { readHeapArtifactIdFromSearchParams } from "./heapArtifactRouting";
import { ChatPanel } from "./ChatPanel";
import { HeapAggregationDetailModal } from "./HeapAggregationDetailModal";
import { buildHeapArtifactHref } from "./heapArtifactRouting";
import {
    buildHeapAggregationGroups,
    buildHeapDerivedViews,
    collectHeapAggregationArtifactIds,
    type HeapAggregationGroup,
    type HeapAggregationGroupId,
    type HeapDerivedViewId,
} from "./heapAggregation.ts";
import { buildHeapViewContext } from "./heapViewRegistry.ts";
import { buildHeapRelationIndex, describeHeapRelation } from "./heapRelations";
import { createConversationThreadId, useConversationStore } from "../../store/conversationStore.ts";
import { useUserPreferences } from "../../store/userPreferences.ts";
import { useCustomViewsStore } from "../../store/customViewsStore.ts";
import type { ConversationAnchor } from "../conversations/conversationRegistry.ts";
import { ExploreSavedViewModal } from "./ExploreSavedViewModal";
import { isPublicObserverGatewayBoundary } from "../commons/shellBootstrapFallback.ts";
import {
    INITIATIVE_KICKOFF_TEMPLATES,
    buildInitiativeKickoffEmitRequest,
    canLaunchInitiativeKickoff,
    resolveInitiativeKickoffTemplate,
    type TaskRoutingContext,
} from "./initiativeKickoffTemplates.ts";
import type {
    TaskRouteDecision,
    TaskRouteExecutionResult,
    TaskRouteId,
    TaskRouteWorkflowArtifactRef,
} from "./taskRouting.ts";
import {
    buildTaskRouteExecutionResult,
    buildTaskRouteLineageSnapshot,
    buildTaskRouteSummaryEmitRequest,
    buildTaskRouteSourceStampEmitRequest,
    inferWorkflowGenerationMode,
    inferWorkflowMotifKind,
} from "./taskRouting.ts";
import { TaskRouteLineageCard } from "./TaskRouteLineageCard";
import {
    buildExploreSpaceDefaults,
    EXPLORE_AGGREGATION_MODES,
    EXPLORE_CARD_DEPTHS,
    EXPLORE_LAYOUT_MODES,
    EXPLORE_VISUALIZATION_MODES,
    type ExploreAggregationDensity,
    type ExploreAggregationMode,
    type ExploreCardDepth,
    type ExploreLayoutMode,
    type ExploreSettingVisualKind,
    type ExploreViewSettings,
    type ExploreViewOverrides,
    type ExploreVisualizationMode,
    useExploreViewSettings,
} from "./exploreViewSettings.ts";
import {
    EXPLORE_VIEW_SETTING_SECTIONS,
    resolveExploreSettingVisualKind,
    type ExploreViewSettingDescriptor,
} from "./exploreViewSettingsRegistry.ts";
import type { ExploreProjectionIntent } from "./exploreSurfacePolicy.ts";

interface HeapBlockGridProps {
    /** Optional pre-filters to scope this grid (e.g. { blockType: "scorecard" } for /system) */
    filterDefaults?: {
        spaceId?: string;
        blockType?: string;
        tag?: string;
    };
    /** Whether to show the filter sidebar (true for /heap, false for embedded use) */
    showFilterSidebar?: boolean;
}

const SEARCH_INPUT_ID = "heap-command-search";
const HEAP_DELTA_POLLING_ENABLED_KEY = "cortex.heap.deltaPolling";
const HEAP_DELTA_POLLING_INTERVAL_MS_KEY = "cortex.heap.deltaPollingIntervalMs";
const HEAP_THREAD_QUERY_KEY = "thread";
const HEAP_DERIVED_VIEW_QUERY_KEY = "derived_view";
const HEAP_VIEW_PROJECTION_QUERY_KEY = "heap_projection";
const HEAP_VIEW_LAYOUT_QUERY_KEY = "heap_layout";
const HEAP_VIEW_CARD_DEPTH_QUERY_KEY = "heap_card_depth";
const HEAP_VIEW_AGGREGATION_QUERY_KEY = "heap_aggregation";
const HEAP_VIEW_GROUP_DESCRIPTIONS_QUERY_KEY = "heap_group_descriptions";
const HEAP_VIEW_VISUALIZATION_QUERY_KEY = "heap_visualization";

type CreateMode = "create" | "generate" | "upload" | "chat" | "plan";
type ExploreDerivedAggregationMode = ExploreAggregationMode;

interface HeapDetailTrailEntry {
    artifactId: string;
    title: string;
    relation?: string;
}

type UploadLifecycleState = "selected" | "uploading" | "uploaded" | "extracting" | "indexed" | "needs_review" | "failed";

const UPLOAD_LIFECYCLE_LABELS: Record<UploadLifecycleState, string> = {
    selected: "Selected",
    uploading: "Uploading",
    uploaded: "Uploaded",
    extracting: "Extracting",
    indexed: "Indexed",
    needs_review: "Needs Review",
    failed: "Failed",
};

const UPLOAD_LIFECYCLE_CLASSNAMES: Record<UploadLifecycleState, string> = {
    selected: "text-cyan-300 bg-cyan-500/10 border-cyan-500/25",
    uploading: "text-amber-300 bg-amber-500/10 border-amber-500/25",
    uploaded: "text-blue-300 bg-blue-500/10 border-blue-500/25",
    extracting: "text-indigo-300 bg-indigo-500/10 border-indigo-500/25",
    indexed: "text-emerald-300 bg-emerald-500/10 border-emerald-500/25",
    needs_review: "text-yellow-300 bg-yellow-500/10 border-yellow-500/25",
    failed: "text-rose-300 bg-rose-500/10 border-rose-500/25",
};

function resolveUploadLifecycleClassName(state: UploadLifecycleState | null): string {
    if (!state) {
        return "text-slate-400 bg-slate-500/10 border-slate-500/20";
    }
    return UPLOAD_LIFECYCLE_CLASSNAMES[state];
}

function resolveUploadLifecycleLabel(state: UploadLifecycleState | null): string {
    if (!state) {
        return "Idle";
    }
    return UPLOAD_LIFECYCLE_LABELS[state];
}

function resolveUploadLifecycleState(status: HeapUploadExtractionStatusResponse["status"]): UploadLifecycleState {
    switch (status) {
        case "completed":
            return "indexed";
        case "needs_review":
            return "needs_review";
        case "failed":
            return "failed";
        case "submitted":
        case "running":
        default:
            return "extracting";
    }
}

function isTerminalUploadExtractionStatus(status: HeapUploadExtractionStatusResponse["status"]): boolean {
    return status === "completed" || status === "needs_review" || status === "failed";
}

function parserProfileLabel(profile: string): string {
    switch (profile) {
        case "docling":
            return "Docling";
        case "liteparse":
            return "LiteParse";
        case "markitdown":
            return "MarkItDown";
        case "auto":
        default:
            return "Auto";
    }
}

function resolveDefaultUploadParserProfileForMime(mimeType: string | null | undefined): string {
    switch (mimeType) {
        case "application/pdf":
            return "docling";
        case "text/plain":
        case "text/markdown":
        case "application/json":
        case "text/csv":
        case "text/html":
        case "application/xml":
            return "markitdown";
        default:
            return "liteparse";
    }
}

function inferUploadMimeType(file: File | null): string | null {
    if (!file) {
        return null;
    }
    if (file.type?.trim()) {
        return file.type.trim();
    }
    const lowerName = file.name.toLowerCase();
    if (lowerName.endsWith(".pdf")) return "application/pdf";
    if (lowerName.endsWith(".md")) return "text/markdown";
    if (lowerName.endsWith(".txt")) return "text/plain";
    if (lowerName.endsWith(".json")) return "application/json";
    if (lowerName.endsWith(".csv")) return "text/csv";
    if (lowerName.endsWith(".html") || lowerName.endsWith(".htm")) return "text/html";
    if (lowerName.endsWith(".xml")) return "application/xml";
    if (lowerName.endsWith(".png")) return "image/png";
    if (lowerName.endsWith(".jpg") || lowerName.endsWith(".jpeg")) return "image/jpeg";
    if (lowerName.endsWith(".webp")) return "image/webp";
    return null;
}

export function HeapBlockGrid({ filterDefaults, showFilterSidebar = false }: HeapBlockGridProps) {
    const activeSpaceId = useActiveSpaceContext();
    const activeSpace = useActiveSpaceRecord();
    const [searchParams, setSearchParams] = useSearchParams();

    const env = (import.meta as unknown as { env?: Record<string, string | boolean | undefined> }).env;
    const isDevMode = env?.DEV === true || String(env?.DEV).toLowerCase() === "true";
    const heapParityEnabled =
        ((env?.VITE_HEAP_PARITY_ENABLED as string | undefined) ?? "true").toLowerCase() !== "false";
    const heapCreateFlowEnabled =
        ((env?.VITE_HEAP_CREATE_FLOW_ENABLED as string | undefined) ?? "true").toLowerCase() !== "false";
    const heapChangedBlocksPollingEnabledFromEnv = ((env?.VITE_HEAP_CHANGED_BLOCKS_POLLING_ENABLED as string | undefined) ?? "false").toLowerCase() === "true";
    const heapChangedBlocksPollingEnabledDefault = useMemo(() => {
        const envValue = (env?.VITE_HEAP_CHANGED_BLOCKS_POLLING_ENABLED as string | undefined) ?? "false";
        return resolveHeapDeltaPollingEnabled(envValue);
    }, []);
    const heapChangedBlocksPollingIntervalDefaultMs = useMemo(() => {
        const envValue = env?.VITE_HEAP_CHANGED_BLOCKS_POLLING_INTERVAL_MS as string | undefined;
        return resolveHeapDeltaPollingIntervalMs(envValue);
    }, []);
    const [blocks, setBlocks] = useState<HeapBlockListItem[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [selectedBlockIds, setSelectedBlockIds] = useState<string[]>([]);
    const [expandedBlockId, setExpandedBlockId] = useState<string | null>(null);
    const location = useLocation();
    const routeThreadId = useMemo(
        () => searchParams.get(HEAP_THREAD_QUERY_KEY)?.trim() || null,
        [searchParams],
    );
    const routeDerivedViewId = useMemo(
        () => searchParams.get(HEAP_DERIVED_VIEW_QUERY_KEY)?.trim() || null,
        [searchParams],
    );
    const viewMode = useMemo<HeapPrimaryViewMode>(() => {
        if (location.pathname === "/inbox") return "Inbox";
        return normalizeHeapPrimaryViewMode(searchParams.get("heap_view"));
    }, [location.pathname, searchParams]);
    const [filterMode, setFilterMode] = useState<HeapFilterMode>("AND");
    const [filterTerm, setFilterTerm] = useState("");
    const [excludeTerm, setExcludeTerm] = useState("");
    const [selectedTags, setSelectedTags] = useState<string[]>([]);
    const [selectedPageLinks, setSelectedPageLinks] = useState<string[]>([]);
    const [pageLinkTerm, setPageLinkTerm] = useState("");
    const [selectedReviewLane, setSelectedReviewLane] = useState<HeapReviewLane | null>(null);
    const [multiSelectEnabled, setMultiSelectEnabled] = useState(false);
    const [regeneratingId, setRegeneratingId] = useState<string | null>(null);
    const [statusMessage, setStatusMessage] = useState<string | null>(null);
    const [createPanelOpen, setCreatePanelOpen] = useState(false);
    const [createMode, setCreateMode] = useState<CreateMode>("create");
    const [newBlockTitle, setNewBlockTitle] = useState("");
    const [newBlockType, setNewBlockType] = useState("note");
    const [newBlockText, setNewBlockText] = useState("");
    const [agentPrompt, setAgentPrompt] = useState("");
    const [uploadFile, setUploadFile] = useState<File | null>(null);
    const [uploadParserProfiles, setUploadParserProfiles] = useState<HeapUploadParserProfileRecord[]>([]);
    const [uploadParserProfilesError, setUploadParserProfilesError] = useState<string | null>(null);
    const [uploadParserProfile, setUploadParserProfile] = useState<string>("auto");
    const [uploadLifecycleState, setUploadLifecycleState] = useState<UploadLifecycleState | null>(null);
    const [uploadArtifact, setUploadArtifact] = useState<HeapUploadArtifactResponse | null>(null);
    const [uploadExtractionStatus, setUploadExtractionStatus] = useState<HeapUploadExtractionStatusResponse | null>(null);
    const [solicitRole, setSolicitRole] = useState("steward.code");
    const [solicitBudget, setSolicitBudget] = useState("50000");
    const [solicitCapabilities, setSolicitCapabilities] = useState("read,write");
    const [solicitMessage, setSolicitMessage] = useState("");
    const [isEmitting, setIsEmitting] = useState(false);
    const [chatPanelOpen, setChatPanelOpen] = useState(false);
    const [stewardGateArtifactId, setStewardGateArtifactId] = useState<string | null>(null);
    const [stewardGateValidation, setStewardGateValidation] = useState<HeapStewardGateValidateResponse | null>(null);
    const [stewardApplyingId, setStewardApplyingId] = useState<string | null>(null);
    const [stewardPublishing, setStewardPublishing] = useState(false);
    const [commentSidebarBlockId, setCommentSidebarBlockId] = useState<string | null>(null);
    const [hoveredBlockId, setHoveredBlockId] = useState<string | null>(null);
    const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
    const [settingsOpen, setSettingsOpen] = useState(false);
    const [saveViewModalOpen, setSaveViewModalOpen] = useState(false);
    const [isMobile, setIsMobile] = useState(false);
    const [chatThreadId, setChatThreadId] = useState<string | null>(null);
    const [activeDerivedViewId, setActiveDerivedViewId] = useState<HeapDerivedViewId>("board");
    const [expandedAggregationGroupId, setExpandedAggregationGroupId] = useState<HeapAggregationGroupId | null>(null);
    const [historyRecord, setHistoryRecord] = useState<HeapBlockHistoryResponse | null>(null);
    const [detailNavigationTrail, setDetailNavigationTrail] = useState<HeapDetailTrailEntry[]>([]);
    const ensureConversation = useConversationStore((state) => state.ensureConversation);
    const activeConversationThreadId = useConversationStore((state) => state.activeThreadId);
    const getConversation = useConversationStore((state) => state.getConversation);
    const sessionUser = useUiStore((state) => state.sessionUser);
    const reduceMotion = useUserPreferences((state) => state.reduceMotion);
    const saveCustomView = useCustomViewsStore((state) => state.saveView);
    const configuredGatewayTarget = gatewayBaseUrl().trim() || "same-origin /api proxy";
    const publicHost = typeof window !== "undefined"
        && window.location.hostname !== "localhost"
        && window.location.hostname !== "127.0.0.1";

    useEffect(() => {
        const checkMobile = () => setIsMobile(window.innerWidth < 1024);
        checkMobile();
        window.addEventListener("resize", checkMobile);
        return () => window.removeEventListener("resize", checkMobile);
    }, []);

    useEffect(() => {
        if (isMobile) setIsSidebarCollapsed(true);
    }, [isMobile]);

    const [heapChangedBlocksPollingEnabled, setHeapChangedBlocksPollingEnabled] = useState(heapChangedBlocksPollingEnabledDefault);
    const [heapChangedBlocksPollingIntervalMs, setHeapChangedBlocksPollingIntervalMs] = useState(heapChangedBlocksPollingIntervalDefaultMs);
    const [heapChangedBlocksPollingIntervalInput, setHeapChangedBlocksPollingIntervalInput] = useState(
        String(heapChangedBlocksPollingIntervalDefaultMs)
    );
    const effectiveHeapChangedBlocksPollingEnabled =
        heapChangedBlocksPollingEnabledFromEnv || heapChangedBlocksPollingEnabled;
    const heapDeltaPollingControlsLocked = heapChangedBlocksPollingEnabledFromEnv;
    const activePageLinkFilter = useMemo(() => {
        if (selectedPageLinks.length > 0) {
            return selectedPageLinks[0];
        }
        const trimmed = pageLinkTerm.trim();
        return trimmed.length > 0 ? trimmed : undefined;
    }, [selectedPageLinks, pageLinkTerm]);

    const fetchBlocks = useCallback(async () => {
        setLoading(true);
        try {
            const res = await workbenchApi.getHeapBlocks({
                spaceId: activeSpaceId,
                blockType: filterDefaults?.blockType,
                tag: filterDefaults?.tag,
                pageLink: activePageLinkFilter,
                limit: 100,
            });
            const nextItems = res.items || [];
            setBlocks(sortHeapBlocks(nextItems));
            if (nextItems.length > 0) {
                const latestUpdatedAt = nextItems[0]?.projection.updatedAt;
                if (latestUpdatedAt) {
                    setLastDeltaSince(latestUpdatedAt);
                }
            }
            setError(null);
        } catch (err) {
            setError(err instanceof Error ? err.message : String(err));
        } finally {
            setLoading(false);
        }
    }, [activeSpaceId, filterDefaults?.blockType, filterDefaults?.tag, activePageLinkFilter]);

    useEffect(() => { fetchBlocks(); }, [fetchBlocks]);


    useEffect(() => {
        const onShortcut = (event: KeyboardEvent) => {
            if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
                event.preventDefault();
                const input = document.getElementById(SEARCH_INPUT_ID) as HTMLInputElement | null;
                input?.focus();
                input?.select();
            }
            if (event.key === "Escape") {
                setExpandedBlockId(null);
                setCommentSidebarBlockId(null);
            }
        };
        window.addEventListener("keydown", onShortcut);
        return () => window.removeEventListener("keydown", onShortcut);
    }, []);

    // Derive view counts
    const blockCounts = useMemo(() => buildHeapViewCounts(blocks), [blocks]);

    // Derive all tags
    const allTags = useMemo(() => {
        const tagSet = new Set<string>();
        for (const b of blocks) {
            for (const t of b.projection.tags || []) tagSet.add(t);
        }
        return Array.from(tagSet).sort((a, b) => a.localeCompare(b));
    }, [blocks]);
    const allPageLinks = useMemo(() => {
        const pageLinkSet = new Set<string>();
        for (const block of blocks) {
            for (const pageLink of block.projection.pageLinks || []) {
                pageLinkSet.add(pageLink);
            }
        }
        return Array.from(pageLinkSet).sort((a, b) => a.localeCompare(b));
    }, [blocks]);

    const [lastDeltaSince, setLastDeltaSince] = useState<string | null>(null);
    const laneBoardHostRef = useRef<HTMLDivElement | null>(null);
    const [laneBoardWidth, setLaneBoardWidth] = useState(0);

    const includeTerms = useMemo(() => tokenizeQuery(filterTerm), [filterTerm]);
    const excludeTerms = useMemo(() => tokenizeQuery(excludeTerm), [excludeTerm]);
    const blocksInActiveView = useMemo(() => filterHeapBlocksByView(blocks, viewMode), [blocks, viewMode]);
    const reviewLaneCounts = useMemo<Record<HeapReviewLane, number>>(
        () => ({
            private_review: blocksInActiveView.filter((block) => readHeapBlockReviewLane(block) === "private_review").length,
            public_review: blocksInActiveView.filter((block) => readHeapBlockReviewLane(block) === "public_review").length,
        }),
        [blocksInActiveView]
    );
    const availableReviewLanes = useMemo<HeapReviewLane[]>(
        () => (["private_review", "public_review"] as const).filter((lane) => reviewLaneCounts[lane] > 0),
        [reviewLaneCounts]
    );

    const exploreSpaceDefaults = useMemo(
        () =>
            buildExploreSpaceDefaults({
                spaceArchetype: activeSpace?.archetype,
                ambientGraphVariant: normalizeVisualizationVariant(
                    activeSpace?.metadata?.theme?.ambientGraphVariant as ExploreVisualizationMode | undefined,
                ),
            }),
        [activeSpace?.archetype, activeSpace?.metadata?.theme?.ambientGraphVariant],
    );

    const exploreSessionOverrides = useMemo<ExploreViewOverrides>(() => {
        const projectionIntent = readExploreQueryParam(searchParams, HEAP_VIEW_PROJECTION_QUERY_KEY, ["overview", "story", "density", "lineage"]);
        const layoutMode = readExploreQueryParam(searchParams, HEAP_VIEW_LAYOUT_QUERY_KEY, EXPLORE_LAYOUT_MODES);
        const cardDepth = readExploreQueryParam(searchParams, HEAP_VIEW_CARD_DEPTH_QUERY_KEY, EXPLORE_CARD_DEPTHS);
        const aggregationMode = readExploreQueryParam(searchParams, HEAP_VIEW_AGGREGATION_QUERY_KEY, EXPLORE_AGGREGATION_MODES);
        const showGroupDescriptions = readExploreBooleanQueryParam(searchParams, HEAP_VIEW_GROUP_DESCRIPTIONS_QUERY_KEY);
        const visualizationMode = normalizeVisualizationVariant(
            readExploreQueryParam(searchParams, HEAP_VIEW_VISUALIZATION_QUERY_KEY, EXPLORE_VISUALIZATION_MODES) ?? undefined,
        );

        return {
            ...(projectionIntent ? { projectionIntent } : {}),
            ...(layoutMode ? { layoutMode } : {}),
            ...(cardDepth ? { cardDepth } : {}),
            ...(aggregationMode ? { aggregationMode } : {}),
            ...(showGroupDescriptions !== undefined ? { showGroupDescriptions } : {}),
            ...(visualizationMode ? { visualizationMode } : {}),
        };
    }, [searchParams]);

    const exploreView = useExploreViewSettings({
        spaceId: resolveSpaceId(activeSpaceId),
        defaults: exploreSpaceDefaults,
        sessionOverrides: exploreSessionOverrides,
        reduceMotion,
        isMobile,
    });
    const exploreSettings = exploreView.effective;
    const exploreDerivedSettings = exploreView.derived;
    const initiativeKickoffLaunchAllowed = useMemo(
        () => canLaunchInitiativeKickoff(sessionUser?.role),
        [sessionUser?.role],
    );
    const initiativeKickoffDisabledReason = initiativeKickoffLaunchAllowed
        ? null
        : "Switch to an operator, steward, or admin session to request setup review.";
    const setExploreSetting = <K extends keyof ExploreViewSettings>(
        key: K,
        nextValue: ExploreViewSettings[K],
    ) => {
        exploreView.setUserOverride({ [key]: nextValue } as Partial<ExploreViewSettings>);
    };
    const clearExploreSessionOverrides = useCallback(() => {
        const next = new URLSearchParams(searchParams);
        next.delete(HEAP_VIEW_PROJECTION_QUERY_KEY);
        next.delete(HEAP_VIEW_LAYOUT_QUERY_KEY);
        next.delete(HEAP_VIEW_CARD_DEPTH_QUERY_KEY);
        next.delete(HEAP_VIEW_AGGREGATION_QUERY_KEY);
        next.delete(HEAP_VIEW_GROUP_DESCRIPTIONS_QUERY_KEY);
        next.delete(HEAP_VIEW_VISUALIZATION_QUERY_KEY);
        setSearchParams(next, { replace: true });
    }, [searchParams, setSearchParams]);

    const buildSavedViewHref = useCallback(() => {
        if (viewMode !== "Explore") {
            return `${location.pathname}${location.search}`;
        }

        const next = new URLSearchParams();
        next.set(HEAP_VIEW_PROJECTION_QUERY_KEY, exploreSettings.projectionIntent);
        next.set(HEAP_VIEW_LAYOUT_QUERY_KEY, exploreSettings.layoutMode);
        next.set(HEAP_VIEW_CARD_DEPTH_QUERY_KEY, exploreSettings.cardDepth);
        next.set(HEAP_VIEW_AGGREGATION_QUERY_KEY, exploreSettings.aggregationMode);
        next.set(
            HEAP_VIEW_GROUP_DESCRIPTIONS_QUERY_KEY,
            String(exploreSettings.showGroupDescriptions),
        );
        next.set(HEAP_VIEW_VISUALIZATION_QUERY_KEY, exploreSettings.visualizationMode);
        if (activeDerivedViewId !== "board") {
            next.set(HEAP_DERIVED_VIEW_QUERY_KEY, activeDerivedViewId);
        }
        const query = next.toString();
        return query ? `/explore?${query}` : "/explore";
    }, [
        activeDerivedViewId,
        exploreSettings.aggregationMode,
        exploreSettings.cardDepth,
        exploreSettings.layoutMode,
        exploreSettings.projectionIntent,
        exploreSettings.showGroupDescriptions,
        exploreSettings.visualizationMode,
        location.pathname,
        location.search,
        viewMode,
    ]);

    const saveCurrentView = useCallback((label: string) => {
        const href = buildSavedViewHref();
        const saved = saveCustomView(resolveSpaceId(activeSpaceId), {
            label,
            href,
            description: `Projection ${exploreSettings.projectionIntent}, ${exploreSettings.layoutMode} layout, ${exploreSettings.cardDepth} cards.`,
        });
        setSaveViewModalOpen(false);
        setSettingsOpen(false);
        setStatusMessage(`Saved view "${saved.label}".`);
    }, [
        activeSpaceId,
        buildSavedViewHref,
        exploreSettings.cardDepth,
        exploreSettings.layoutMode,
        exploreSettings.projectionIntent,
        saveCustomView,
    ]);

    useEffect(() => {
        if (selectedReviewLane && reviewLaneCounts[selectedReviewLane] === 0) {
            setSelectedReviewLane(null);
        }
    }, [reviewLaneCounts, selectedReviewLane]);

    // Apply filters
    const filteredBlocks = useMemo(() => {
        return filterHeapBlocksByReviewLane(blocksInActiveView, selectedReviewLane).filter(b => {
            const searchable = blockSearchCorpus(b);

            const includeMatches = includeTerms.length === 0
                ? true
                : filterMode === "AND"
                    ? includeTerms.every(term => searchable.includes(term))
                    : includeTerms.some(term => searchable.includes(term));
            if (!includeMatches) return false;

            const tagMatches = selectedTags.length === 0
                ? true
                : filterMode === "AND"
                    ? selectedTags.every(tag => (b.projection.tags || []).includes(tag))
                    : selectedTags.some(tag => (b.projection.tags || []).includes(tag));
            if (!tagMatches) return false;

            const pageLinkMatches = selectedPageLinks.length === 0
                ? (
                    pageLinkTerm.trim().length === 0
                        ? true
                        : (b.projection.pageLinks || []).some((pageLink) =>
                            pageLink.toLowerCase().includes(pageLinkTerm.trim().toLowerCase())
                        )
                )
                : filterMode === "AND"
                    ? selectedPageLinks.every((pageLink) => (b.projection.pageLinks || []).includes(pageLink))
                    : selectedPageLinks.some((pageLink) => (b.projection.pageLinks || []).includes(pageLink));
            if (!pageLinkMatches) return false;

            const excludeMatches = excludeTerms.every(term => !searchable.includes(term));
            return excludeMatches;
        });
    }, [blocksInActiveView, selectedReviewLane, includeTerms, excludeTerms, selectedTags, selectedPageLinks, pageLinkTerm, filterMode]);
    const exploreSurfacePolicy = useMemo(
        () => resolveExploreSurfacePolicy({
            spaceId: resolveSpaceId(activeSpaceId),
            surfaceId: "explore.list",
            spaceArchetype: activeSpace?.archetype,
            projectionIntent: exploreView.effective.projectionIntent,
        }),
        [activeSpace?.archetype, activeSpaceId, exploreView.effective.projectionIntent]
    );
    const laneCount = useMemo(
        () => Math.min(resolveHeapLaneCount(laneBoardWidth, exploreSurfacePolicy), exploreView.derived.laneCap),
        [exploreSurfacePolicy, exploreView.derived.laneCap, laneBoardWidth]
    );
    const aggregationGroups = useMemo(
        () => (viewMode === "Explore" ? buildHeapAggregationGroups(filteredBlocks) : []),
        [filteredBlocks, viewMode]
    );
    const visibleAggregationGroupSet = useMemo(
        () => filterHeapAggregationGroupsByMode(aggregationGroups, exploreView.effective.aggregationMode),
        [aggregationGroups, exploreView.effective.aggregationMode]
    );
    const derivedViews = useMemo(
        () => (viewMode === "Explore" ? buildHeapDerivedViews(filteredBlocks, visibleAggregationGroupSet) : []),
        [filteredBlocks, viewMode, visibleAggregationGroupSet]
    );
    const aggregationArtifactIds = useMemo(
        () => collectHeapAggregationArtifactIds(visibleAggregationGroupSet),
        [visibleAggregationGroupSet]
    );
    const activeAggregationGroup = useMemo(
        () => visibleAggregationGroupSet.find((group) => `aggregate:${group.groupId}` === activeDerivedViewId) ?? null,
        [activeDerivedViewId, visibleAggregationGroupSet]
    );
    const activeDerivedView = useMemo(
        () => derivedViews.find((view) => view.id === activeDerivedViewId) ?? null,
        [activeDerivedViewId, derivedViews]
    );
    const activeHeapViewContext = useMemo(
        () => (viewMode === "Explore" ? buildHeapViewContext(activeDerivedView, visibleAggregationGroupSet) : null),
        [activeDerivedView, viewMode, visibleAggregationGroupSet]
    );
    const visibleAggregationGroups = useMemo(() => {
        if (viewMode !== "Explore") return [];
        if (activeAggregationGroup) {
            return [activeAggregationGroup];
        }
        return visibleAggregationGroupSet;
    }, [activeAggregationGroup, viewMode, visibleAggregationGroupSet]);
    const visibleBlocks = useMemo(() => {
        if (viewMode !== "Explore") {
            return filteredBlocks;
        }
        if (activeDerivedViewId === "all-blocks") {
            return filteredBlocks;
        }
        if (activeDerivedViewId !== "board") {
            return [];
        }
        return filteredBlocks.filter((block) => !aggregationArtifactIds.has(block.projection.artifactId));
    }, [activeDerivedViewId, aggregationArtifactIds, filteredBlocks, viewMode]);
    const blockLanes = useMemo(
        () => buildHeapLanes(visibleBlocks, laneCount, exploreSurfacePolicy),
        [visibleBlocks, laneCount, exploreSurfacePolicy]
    );
    const isDerivedSurfaceEmpty = visibleAggregationGroups.length === 0 && visibleBlocks.length === 0;

    useEffect(() => {
        const el = laneBoardHostRef.current;
        if (!el) return;

        const updateWidth = (width: number) => {
            const rounded = Math.round(width);
            setLaneBoardWidth((current) => (current === rounded ? current : rounded));
        };

        updateWidth(el.getBoundingClientRect().width);

        const observer = new ResizeObserver((entries) => {
            for (const entry of entries) {
                updateWidth(entry.contentRect.width);
            }
        });
        observer.observe(el);

        return () => observer.disconnect();
    }, [loading, visibleBlocks.length]);

    useEffect(() => {
        if (viewMode !== "Explore") {
            if (activeDerivedViewId !== "board") {
                setActiveDerivedViewId("board");
            }
            if (expandedAggregationGroupId) {
                setExpandedAggregationGroupId(null);
            }
            return;
        }

        const validViewIds = new Set<HeapDerivedViewId>([
            "board",
            "all-blocks",
            ...visibleAggregationGroupSet.map((group) => `aggregate:${group.groupId}` as HeapDerivedViewId),
        ]);
        if (!validViewIds.has(activeDerivedViewId)) {
            setActiveDerivedViewId("board");
        }
        if (expandedAggregationGroupId && !visibleAggregationGroupSet.some((group) => group.groupId === expandedAggregationGroupId)) {
            setExpandedAggregationGroupId(null);
        }
    }, [activeDerivedViewId, expandedAggregationGroupId, viewMode, visibleAggregationGroupSet]);

    useEffect(() => {
        if (viewMode !== "Explore" || !routeDerivedViewId) {
            return;
        }
        const validViewIds = new Set<HeapDerivedViewId>([
            "board",
            "all-blocks",
            ...visibleAggregationGroupSet.map((group) => `aggregate:${group.groupId}` as HeapDerivedViewId),
        ]);
        if (!validViewIds.has(routeDerivedViewId as HeapDerivedViewId)) {
            return;
        }
        setActiveDerivedViewId(routeDerivedViewId as HeapDerivedViewId);
    }, [routeDerivedViewId, viewMode, visibleAggregationGroupSet]);

    useEffect(() => {
        if (selectedBlockIds.length === 0) return;
        const visible = new Set(blocks.map((b) => b.projection.artifactId));
        setSelectedBlockIds((current) => current.filter((id) => visible.has(id)));
    }, [blocks, selectedBlockIds.length]);

    useEffect(() => {
        const deepLinkedArtifactId = readHeapArtifactIdFromSearchParams(searchParams);
        if (!deepLinkedArtifactId) return;
        if (!blocks.some((block) => block.projection.artifactId === deepLinkedArtifactId)) return;
        setSelectedBlockIds((current) =>
            current.length === 1 && current[0] === deepLinkedArtifactId ? current : [deepLinkedArtifactId]
        );
        setDetailNavigationTrail([]);
        setExpandedBlockId(deepLinkedArtifactId);
    }, [blocks, searchParams]);

    const selectedPrimaryId = selectedBlockIds[0] ?? null;
    const expandedBlock = useMemo(() => blocks.find(b => b.projection.artifactId === expandedBlockId), [blocks, expandedBlockId]);
    const selectedPrimaryBlock = useMemo(
        () => (selectedPrimaryId ? blocks.find((b) => b.projection.artifactId === selectedPrimaryId) ?? null : null),
        [blocks, selectedPrimaryId]
    );
    const chatConversationAnchor = useMemo<ConversationAnchor>(() => {
        if (viewMode === "Explore") {
            if (selectedBlockIds.length === 1 && selectedPrimaryBlock) {
                return {
                    kind: "block",
                    label: selectedPrimaryBlock.projection.title || selectedPrimaryBlock.projection.artifactId,
                    href: buildHeapArtifactHref(selectedPrimaryBlock.projection.artifactId, activeSpaceId),
                    routeId: "/explore",
                    artifactId: selectedPrimaryBlock.projection.artifactId,
                    blockId: selectedPrimaryBlock.projection.artifactId,
                };
            }
            if (activeDerivedViewId !== "board" && activeHeapViewContext) {
                return {
                    kind: "view",
                    label: activeHeapViewContext.viewLabel,
                    href: `/explore?${HEAP_DERIVED_VIEW_QUERY_KEY}=${encodeURIComponent(activeDerivedViewId)}`,
                    routeId: "/explore",
                    viewId: activeHeapViewContext.viewId,
                };
            }
            return {
                kind: "page",
                label: "Explore",
                href: "/explore",
                routeId: "/explore",
            };
        }

        return {
            kind: "page",
            label: location.pathname.replace(/^\//, "") || "Workbench",
            href: `${location.pathname}${location.search}`,
            routeId: location.pathname,
        };
    }, [activeDerivedViewId, activeHeapViewContext, location.pathname, location.search, selectedBlockIds.length, selectedPrimaryBlock, viewMode]);
    const chatConversationTitle = useMemo(() => {
        if (selectedBlockIds.length === 1 && selectedPrimaryBlock) {
            return selectedPrimaryBlock.projection.title || selectedPrimaryBlock.projection.artifactId;
        }
        if (viewMode === "Explore" && activeDerivedViewId !== "board" && activeHeapViewContext) {
            return activeHeapViewContext.viewLabel;
        }
        return viewMode === "Explore" ? "Explore conversation" : "Conversation";
    }, [activeHeapViewContext, activeDerivedViewId, selectedBlockIds.length, selectedPrimaryBlock, viewMode]);
    const openDetailBlock = useCallback((artifactId: string) => {
        setDetailNavigationTrail([]);
        setExpandedBlockId(artifactId);
    }, []);
    const navigateDetailBlock = useCallback((artifactId: string, context?: { relation?: string; title?: string }) => {
        if (expandedBlock && expandedBlock.projection.artifactId !== artifactId) {
            setDetailNavigationTrail((trail) => [
                ...trail,
                {
                    artifactId: expandedBlock.projection.artifactId,
                    title: expandedBlock.projection.title,
                    relation: context?.relation,
                },
            ]);
        }
        setExpandedBlockId(artifactId);
    }, [expandedBlock]);
    const goBackInDetailHistory = useCallback(() => {
        setDetailNavigationTrail((trail) => {
            if (trail.length === 0) {
                return trail;
            }
            const nextTrail = [...trail];
            const previous = nextTrail.pop();
            if (previous) {
                setExpandedBlockId(previous.artifactId);
            }
            return nextTrail;
        });
    }, []);
    const previousRouteThreadIdRef = useRef<string | null>(null);
    useEffect(() => {
        if (!routeThreadId) {
            previousRouteThreadIdRef.current = null;
            return;
        }
        if (previousRouteThreadIdRef.current === routeThreadId) {
            return;
        }
        previousRouteThreadIdRef.current = routeThreadId;
        const record = ensureConversation({
            threadId: routeThreadId,
            title: chatConversationTitle,
            anchor: chatConversationAnchor,
        });
        setChatThreadId(record.threadId);
        setChatPanelOpen(true);
    }, [chatConversationAnchor, chatConversationTitle, ensureConversation, routeThreadId]);
    const openChatConversation = useCallback(() => {
        const currentThreadRecord = chatThreadId ? getConversation(chatThreadId) : null;
        const activeThreadRecord = activeConversationThreadId ? getConversation(activeConversationThreadId) : null;
        const currentThreadMatchesAnchor =
            currentThreadRecord ? !currentThreadRecord.anchor || currentThreadRecord.anchor.href === chatConversationAnchor.href : false;
        const activeThreadMatchesAnchor =
            activeThreadRecord ? !activeThreadRecord.anchor || activeThreadRecord.anchor.href === chatConversationAnchor.href : false;
        const record = ensureConversation({
            threadId:
                routeThreadId ??
                (chatThreadId && currentThreadMatchesAnchor ? chatThreadId : null) ??
                (activeConversationThreadId && activeThreadMatchesAnchor ? activeConversationThreadId : null) ??
                createConversationThreadId(),
            title: chatConversationTitle,
            anchor: chatConversationAnchor,
        });
        setChatThreadId(record.threadId);
        const next = new URLSearchParams(searchParams);
        next.set(HEAP_THREAD_QUERY_KEY, record.threadId);
        if (chatConversationAnchor.kind === "view") {
            next.set(HEAP_DERIVED_VIEW_QUERY_KEY, activeDerivedViewId);
        }
        setSearchParams(next);
        setChatPanelOpen(true);
    }, [activeConversationThreadId, activeDerivedViewId, chatConversationAnchor, chatConversationTitle, chatThreadId, ensureConversation, getConversation, routeThreadId, searchParams, setSearchParams]);

    const selectionContext = useMemo<ActionSelectionContext>(() => {
        return {
            selectedArtifactIds: selectedBlockIds,
            activeArtifactId: expandedBlockId || undefined,
            selectedCount: selectedBlockIds.length,
            selectedBlockTypes: Array.from(new Set(
                selectedBlockIds.map(id => blocks.find(b => b.projection.artifactId === id)?.projection.blockType).filter(Boolean) as string[]
            ))
        };
    }, [selectedBlockIds, blocks, expandedBlockId]);
 
    // Use a ref to always have the latest selection context in handlers, bypassing stale closures
    const selectionRef = useRef(selectionContext);
    useEffect(() => {
        selectionRef.current = selectionContext;
    }, [selectionContext]);

    const activeFilters = useMemo(() => ({
        viewMode,
        filterMode,
        selectedReviewLane,
        selectedTags,
        selectedPageLinks,
    }), [filterMode, selectedPageLinks, selectedReviewLane, selectedTags, viewMode]);
    const { actionPlan, loading: actionPlanLoading, error: actionPlanError, source: actionPlanSource } = useHeapActionPlan({
        selection: selectionContext,
        zones: ["heap_page_bar", "heap_selection_bar"],
        activeFilters,
    });
    const pageZonePlan = useMemo(
        () => actionPlan?.zones.find((zone) => zone.zone === "heap_page_bar") ?? null,
        [actionPlan]
    );
    const selectionZonePlan = useMemo(
        () => actionPlan?.zones.find((zone) => zone.zone === "heap_selection_bar") ?? null,
        [actionPlan]
    );
    const cardMenuContext = useMemo<ActionSelectionContext>(() => ({
        selectedArtifactIds: ["heap-card-context"],
        activeArtifactId: "heap-card-context",
        selectedCount: 1,
        selectedBlockTypes: ["note"],
    }), []);
    const { actionPlan: cardActionPlan } = useHeapActionPlan({
        selection: cardMenuContext,
        zones: ["heap_card_menu"],
        activeFilters,
    });
    const cardMenuZonePlan = useMemo(
        () => cardActionPlan?.zones.find((zone) => zone.zone === "heap_card_menu") ?? null,
        [cardActionPlan]
    );

    const [selectionMessage, setSelectionMessage] = useState<string | null>(null);

    const handleSelection = (blockId: string, event: React.MouseEvent<HTMLDivElement>) => {
        const toggleSelection = heapParityEnabled && (multiSelectEnabled || event.metaKey || event.ctrlKey);
        if (!toggleSelection) {
            setSelectedBlockIds([blockId]);
            setSelectionMessage("Block Selected");
            return;
        }
        setSelectedBlockIds((current) => {
            if (current.includes(blockId)) {
                return current.filter((item) => item !== blockId);
            }
            setSelectionMessage("Block Selected");
            return [...current, blockId];
        });
    };

    useEffect(() => {
        if (selectionMessage) {
            const timer = setTimeout(() => setSelectionMessage(null), 3000);
            return () => clearTimeout(timer);
        }
    }, [selectionMessage]);

    const resolveActionSelectionIds = useCallback(
        (actionSelection?: ActionSelectionContext) => actionSelection?.selectedArtifactIds ?? selectedBlockIds,
        [selectedBlockIds],
    );

    const handlePinToggled = () => {
        fetchBlocks();
        setSelectedBlockIds([]);
        setStatusMessage("Pin state updated.");
    };

    const handleDeleted = () => {
        fetchBlocks();
        setSelectedBlockIds([]);
        setStatusMessage("Selected blocks deleted.");
    };

    const handleRegenerate = (actionSelection?: ActionSelectionContext) => {
        const artifactId = resolveActionSelectionIds(actionSelection)[0];
        if (!artifactId) return;
        setRegeneratingId(artifactId);
        setTimeout(() => setRegeneratingId(null), 1500);
        setStatusMessage("Regeneration requested (UI simulation).");
    };

    const handleContextBundle = async (actionSelection?: ActionSelectionContext) => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled by VITE_HEAP_PARITY_ENABLED.");
            return;
        }
        const actionIds = resolveActionSelectionIds(actionSelection);
        if (actionIds.length === 0) return;
        try {
            const bundle = await workbenchApi.createHeapContextBundle(actionIds);
            setStatusMessage(`Context bundle prepared: ${bundle.context_bundle.block_count} blocks.`);
            console.info("Heap context bundle", bundle);
        } catch (err) {
            setStatusMessage(`Context bundle failed: ${err instanceof Error ? err.message : String(err)}`);
        }
    };

    const handleExport = async (actionSelection?: ActionSelectionContext) => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled by VITE_HEAP_PARITY_ENABLED.");
            return;
        }
        const actionIds = resolveActionSelectionIds(actionSelection);
        if (actionIds.length === 0) return;
        try {
            if (actionIds.length === 1) {
                const exportPayload = await workbenchApi.getHeapBlockExport(actionIds[0], "json");
                downloadJson(`heap-block-${actionIds[0]}.json`, exportPayload);
                setStatusMessage("Single block export downloaded.");
                return;
            }
            const bundle = await workbenchApi.createHeapContextBundle(actionIds);
            downloadJson(`heap-context-${Date.now()}.json`, bundle);
            setStatusMessage(`Bundle export downloaded (${actionIds.length} blocks).`);
        } catch (err) {
            setStatusMessage(`Export failed: ${err instanceof Error ? err.message : String(err)}`);
        }
    };

    const handleHistory = async (actionSelection?: ActionSelectionContext) => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled by VITE_HEAP_PARITY_ENABLED.");
            return;
        }
        const actionIds = resolveActionSelectionIds(actionSelection);
        if (actionIds.length !== 1) {
            setStatusMessage("History requires exactly one selected block.");
            return;
        }
        try {
            const history = await workbenchApi.getHeapBlockHistory(actionIds[0]);
            setHistoryRecord(history);
            setStatusMessage(`History loaded: ${history.versions.length} audit events.`);
        } catch (err) {
            setStatusMessage(`History failed: ${err instanceof Error ? err.message : String(err)}`);
        }
    };

    const handleSynthesize = async (actionSelection?: ActionSelectionContext) => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled.");
            return;
        }
        // Prefer state-derived Ref for absolute latest data
        const actionIds = selectionRef.current.selectedArtifactIds;
        
        if (actionIds.length < 3) {
            setStatusMessage(`Synthesize requires at least 3 blocks (selected: ${actionIds.length})`);
            return;
        }

        try {
            setStatusMessage("Agent 'steward.synth' is processing selection...");

            // Artificial delay for "Agent Processing" feel
            await new Promise(resolve => setTimeout(resolve, 1500));

            const spaceId = resolveSpaceId(activeSpaceId);
            const emitRequest: EmitHeapBlockRequest = {
                schema_version: "1.0.0",
                mode: "heap",
                space_id: spaceId,
                source: {
                    agent_id: "steward.synth",
                    emitted_at: new Date().toISOString(),
                },
                block: {
                    type: "note",
                    title: `Synthesis: ${actionIds.length} Blocks`,
                    attributes: {
                        origin: "bulk_synthesis",
                        synth_model: "nostra-large-v1"
                    }
                },
                content: {
                    payload_type: "rich_text",
                    rich_text: {
                        plain_text: `### Executive Summary\n\nThis synthesis contains a multi-agent distillation of the following blocks: ${actionIds.join(", ")}.\n\n- [x] Context reconciled\n- [x] Conflict resolution applied\n- [ ] Pending validation by human reviewer\n\n**Generated insights:** The current selection indicates a 15% deviation in expected behavior markers. Recommended action: Deep audit of relational links.`
                    }
                }
            };

            await workbenchApi.emitHeapBlock(emitRequest);
            setStatusMessage("Synthesis record added to this space.");
            fetchBlocks();
            setSelectedBlockIds([]);
        } catch (err) {
            setStatusMessage(`Synthesis failed: ${err instanceof Error ? err.message : String(err)}`);
        }
    };

    const buildPublishGovernanceEnvelope = (artifactId: string): ArtifactGovernanceEnvelope => {
        const nowIso = new Date().toISOString();
        const nonce = `nonce-${Date.now()}`;
        return {
            approvedBy: "Systems Steward",
            rationale: "Heap publish via steward gate workflow.",
            approvedAt: nowIso,
            actorId: "cortex-web",
            decisionProof: {
                decisionId: `decision-${artifactId}-${Date.now()}`,
                signature: "cortex-web-signature",
                signer: "cortex-web",
                algorithm: "ed25519",
                nonce,
                expiresAt: new Date(Date.now() + 10 * 60 * 1000).toISOString(),
            },
            nonce,
            expiresAt: new Date(Date.now() + 10 * 60 * 1000).toISOString(),
        };
    };

    const publishWithStewardGate = async (artifactId: string, stewardGateToken?: string) => {
        await workbenchApi.publishArtifact(artifactId, {
            notes: "Published from Heap via Steward Gate.",
            governance: buildPublishGovernanceEnvelope(artifactId),
            stewardGateToken,
        });
    };

    const handlePublish = async (actionSelection?: ActionSelectionContext) => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled by VITE_HEAP_PARITY_ENABLED.");
            return;
        }
        const actionIds = resolveActionSelectionIds(actionSelection);
        if (actionIds.length !== 1) {
            setStatusMessage("Publish requires exactly one selected block.");
            return;
        }
        const artifactId = actionIds[0];
        try {
            const validation = await workbenchApi.validateHeapStewardGate(artifactId);
            if (validation.status === "pass") {
                await publishWithStewardGate(artifactId, validation.stewardGateToken);
                setStatusMessage("Block published.");
                fetchBlocks();
                setSelectedBlockIds([]);
                return;
            }
            setStewardGateArtifactId(artifactId);
            setStewardGateValidation(validation);
        } catch (err) {
            setStatusMessage(`Publish failed: ${err instanceof Error ? err.message : String(err)}`);
        }
    };

    const handleStewardGateRevalidate = async () => {
        if (!stewardGateArtifactId) return;
        try {
            const validation = await workbenchApi.validateHeapStewardGate(stewardGateArtifactId);
            setStewardGateValidation(validation);
        } catch (err) {
            setStatusMessage(`Steward gate revalidate failed: ${err instanceof Error ? err.message : String(err)}`);
        }
    };

    const handleStewardGateApply = async (enrichmentId: string) => {
        if (!stewardGateArtifactId) return;
        try {
            setStewardApplyingId(enrichmentId);
            const response = await workbenchApi.applyHeapStewardEnrichment(stewardGateArtifactId, enrichmentId);
            setStewardGateValidation(response.validation);
            fetchBlocks();
            setStatusMessage("Steward enrichment applied.");
        } catch (err) {
            setStatusMessage(`Apply enrichment failed: ${err instanceof Error ? err.message : String(err)}`);
        } finally {
            setStewardApplyingId(null);
        }
    };

    const actionHandlers = useMemo<ActionHandlers>(() => ({
        onDeselect: () => setSelectedBlockIds([]),
        onPinToggled: handlePinToggled,
        onDeleted: handleDeleted,
        onRegenerate: () => handleRegenerate(selectionRef.current),
        onContextBundle: () => handleContextBundle(selectionRef.current),
        onExport: () => handleExport(selectionRef.current),
        onHistory: () => handleHistory(selectionRef.current),
        onPublish: () => handlePublish(selectionRef.current),
        onSynthesize: () => handleSynthesize(selectionRef.current),
        onCreateBlock: () => setCreatePanelOpen(open => !open),
        onOpenDiscussion: (actionSelection) => {
            const artifactId = (actionSelection || selectionRef.current).selectedArtifactIds[0];
            if (!artifactId) return;
            setCommentSidebarBlockId(artifactId);
        },
    }), [
        handlePinToggled, handleDeleted, handleRegenerate,
        handleContextBundle, handleExport, handleHistory,
        handlePublish, handleSynthesize
    ]);

    const handleStewardGatePublish = async () => {
        if (!stewardGateArtifactId || !stewardGateValidation) return;
        try {
            setStewardPublishing(true);
            let token = stewardGateValidation.stewardGateToken;
            if (!token) {
                const validation = await workbenchApi.validateHeapStewardGate(stewardGateArtifactId);
                setStewardGateValidation(validation);
                token = validation.stewardGateToken;
            }
            await publishWithStewardGate(stewardGateArtifactId, token);
            setStatusMessage("Block published.");
            setStewardGateValidation(null);
            setStewardGateArtifactId(null);
            setSelectedBlockIds([]);
            fetchBlocks();
        } catch (err) {
            setStatusMessage(`Publish failed: ${err instanceof Error ? err.message : String(err)}`);
        } finally {
            setStewardPublishing(false);
        }
    };

    const clearCreateForm = useCallback(() => {
        setNewBlockTitle("");
        setNewBlockType("note");
        setNewBlockText("");
        setAgentPrompt("");
        setUploadFile(null);
        setUploadParserProfile("auto");
        setUploadLifecycleState(null);
        setUploadArtifact(null);
        setUploadExtractionStatus(null);
        setUploadParserProfilesError(null);
        setSolicitRole("steward.code");
        setSolicitBudget("50000");
        setSolicitCapabilities("read,write");
        setSolicitMessage("");
    }, []);

    const handleUploadFileChange = useCallback((file: File | null) => {
        setUploadFile(file);
        setUploadArtifact(null);
        setUploadExtractionStatus(null);
        setUploadLifecycleState(file ? "selected" : null);
        setStatusMessage(file ? `Selected ${file.name}.` : "Upload file cleared.");
    }, []);

    const selectedUploadMimeType = useMemo(() => inferUploadMimeType(uploadFile), [uploadFile]);
    const defaultUploadParserProfile = useMemo(
        () => resolveDefaultUploadParserProfileForMime(selectedUploadMimeType),
        [selectedUploadMimeType],
    );
    const availableUploadParserProfiles = useMemo(
        () =>
            uploadParserProfiles.filter(
                (profile) => profile.parser_profile === "auto" || profile.configured,
            ),
        [uploadParserProfiles],
    );

    useEffect(() => {
        if (!createPanelOpen || createMode !== "upload") {
            return;
        }
        const actorRole = sessionUser?.role || "operator";
        const actorId = sessionUser?.actorId || "cortex-web";
        let cancelled = false;
        void workbenchApi
            .getHeapUploadParserProfiles(actorRole, actorId)
            .then((response) => {
                if (cancelled) {
                    return;
                }
                setUploadParserProfiles(response.items || []);
                setUploadParserProfilesError(null);
            })
            .catch((err) => {
                if (cancelled) {
                    return;
                }
                setUploadParserProfiles([]);
                setUploadParserProfilesError(err instanceof Error ? err.message : String(err));
            });
        return () => {
            cancelled = true;
        };
    }, [createMode, createPanelOpen, sessionUser?.actorId, sessionUser?.role]);

    const applyTaskRouteDecision = useCallback(async (
        routeId: TaskRouteId,
        context: TaskRoutingContext,
        decision: TaskRouteDecision,
    ): Promise<TaskRouteExecutionResult> => {
        const sourceArtifactId = expandedBlock?.projection.artifactId;
        if (!sourceArtifactId) {
            setStatusMessage("Open a task block before applying a route.");
            return {
                route_id: routeId,
                route_label: decision.label,
                confidence_hint: decision.confidence_hint,
                source_task_artifact_id: "",
                routed_at: new Date().toISOString(),
                success: false,
                failure_reason: "Open a task block before applying a route.",
            };
        }

        const spaceId = resolveSpaceId(activeSpaceId);
        const routedAt = new Date().toISOString();
        let workflowArtifact: TaskRouteWorkflowArtifactRef | undefined;

        try {
            setIsEmitting(true);
            setStatusMessage(`Applying ${decision.label.toLowerCase()} to ${context.title}.`);

            if (routeId === "proposal_generation") {
                const motifKind = inferWorkflowMotifKind(context);
                const generationMode = inferWorkflowGenerationMode(context);
                const scope = { spaceId, routeId: "workflow_draft", role: context.agent_role };
                const intentResponse = await workbenchApi.postWorkflowIntent({
                    intent: context.objective,
                    motifKind,
                    scope,
                    authorityCeiling: "l2",
                    createdBy: "cortex-web",
                    sourceMode: "hybrid",
                });
                const candidateResponse = await workbenchApi.postWorkflowCandidates({
                    intent: context.objective,
                    motifKind,
                    scope,
                    generationMode,
                    count: 3,
                    createdBy: "cortex-web",
                    sourceMode: "hybrid",
                });
                const candidate = candidateResponse.candidates.find((entry) => entry.validation.valid && entry.compileResult?.valid)
                    ?? candidateResponse.candidates[0];
                if (!candidate) {
                    throw new Error("No workflow candidates were generated.");
                }
                const staged = await workbenchApi.stageWorkflowCandidate(candidateResponse.candidateSetId, {
                    candidateId: candidate.candidateId,
                    stagedBy: "cortex-web",
                    rationale: decision.rationale,
                    expectedInputHash: candidate.inputHash,
                });
                const proposal = await workbenchApi.proposeWorkflowDraft(staged.workflowDraftId, {
                    proposedBy: "cortex-web",
                    rationale: decision.rationale,
                });
                const intentRecord = intentResponse.workflowIntent as Record<string, unknown>;
                const proposalRecord = proposal.proposal as Record<string, unknown>;
                workflowArtifact = {
                    workflow_intent_id: typeof intentRecord.workflowIntentId === "string" ? intentRecord.workflowIntentId : undefined,
                    candidate_set_id: candidateResponse.candidateSetId,
                    workflow_draft_id: staged.workflowDraftId,
                    proposal_id: typeof proposalRecord.proposalId === "string" ? proposalRecord.proposalId : undefined,
                    definition_id: typeof proposalRecord.definitionId === "string" ? proposalRecord.definitionId : undefined,
                    scope_key: typeof proposalRecord.scopeKey === "string" ? proposalRecord.scopeKey : `${spaceId}:workflow`,
                    motif_kind: motifKind,
                    generation_mode: generationMode,
                    proposal_digest_path: typeof proposalRecord.proposalId === "string"
                        ? `/api/cortex/workflow-drafts/proposals/${encodeURIComponent(proposalRecord.proposalId)}/digest`
                        : undefined,
                };
            }

            const emitted = await workbenchApi.emitHeapBlock(
                buildTaskRouteSummaryEmitRequest(context, {
                    source_task_artifact_id: sourceArtifactId,
                    space_id: spaceId,
                    route_id: routeId,
                    decision,
                    routed_at: routedAt,
                    workflow: workflowArtifact,
                }),
            );
            await workbenchApi.emitHeapBlock(
                buildTaskRouteSourceStampEmitRequest(context, {
                    source_block: expandedBlock,
                    route_id: routeId,
                    decision,
                    summary_artifact_id: emitted.artifactId,
                    routed_at: routedAt,
                    workflow: workflowArtifact,
                }),
            );
            await fetchBlocks();
            setDetailNavigationTrail([
                {
                    artifactId: sourceArtifactId,
                    title: context.title,
                    relation: routeId,
                },
            ]);
            setExpandedBlockId(emitted.artifactId);
            setStatusMessage(
                routeId === "proposal_generation"
                    ? `Generated a governed workflow proposal for ${context.title}.`
                    : `Routed ${context.title} via ${decision.label.toLowerCase()}.`,
            );
            return buildTaskRouteExecutionResult({
                route_id: routeId,
                route_label: decision.label,
                confidence_hint: decision.confidence_hint,
                source_task_artifact_id: sourceArtifactId,
                summary_artifact_id: emitted.artifactId,
                stamped_source_artifact_id: sourceArtifactId,
                routed_at: routedAt,
                workflow: workflowArtifact,
                success: true,
            });
        } catch (err) {
            const failureReason = err instanceof Error ? err.message : String(err);
            setStatusMessage(`Route application failed: ${failureReason}`);
            return buildTaskRouteExecutionResult({
                route_id: routeId,
                route_label: decision.label,
                confidence_hint: decision.confidence_hint,
                source_task_artifact_id: sourceArtifactId,
                routed_at: routedAt,
                workflow: workflowArtifact,
                success: false,
                failure_reason: failureReason,
            });
        } finally {
            setIsEmitting(false);
        }
    }, [activeSpaceId, expandedBlock, fetchBlocks]);

    const loadInitiativeKickoffTemplate = useCallback(async (templateId: string) => {
        if (isEmitting) {
            return;
        }
        if (!initiativeKickoffLaunchAllowed) {
            setStatusMessage(initiativeKickoffDisabledReason);
            return;
        }
        const template = resolveInitiativeKickoffTemplate(templateId);
        if (!template) {
            setStatusMessage(`Unknown initiative kickoff template: ${templateId}`);
            return;
        }
        try {
            setIsEmitting(true);
            setStatusMessage(`Requesting steward review for ${template.title}.`);
            const emitted = await workbenchApi.emitHeapBlock(
                buildInitiativeKickoffEmitRequest(template, resolveSpaceId(activeSpaceId)),
                sessionUser?.role || "operator",
                sessionUser?.actorId || "cortex-web",
            );
            await fetchBlocks();
            const nextParams = new URLSearchParams(searchParams);
            nextParams.set("heap_view", heapPrimaryViewModeParam("Proposals"));
            setSearchParams(nextParams, { replace: true });
            clearCreateForm();
            setCreatePanelOpen(false);
            setDetailNavigationTrail([]);
            setExpandedBlockId(emitted.artifactId);
            setStatusMessage(`${template.title} plan-backed review request added and opened in Proposals.`);
        } catch (err) {
            setStatusMessage(`Plan-backed review request failed: ${err instanceof Error ? err.message : String(err)}`);
        } finally {
            setIsEmitting(false);
        }
    }, [
        activeSpaceId,
        clearCreateForm,
        fetchBlocks,
        initiativeKickoffDisabledReason,
        initiativeKickoffLaunchAllowed,
        isEmitting,
        searchParams,
        sessionUser?.actorId,
        sessionUser?.role,
        setSearchParams,
    ]);

    const buildUploadEmitPayload = (
        artifact: HeapUploadArtifactResponse,
        resolvedTitle: string,
        spaceId: string,
        emittedAt: string,
    ): EmitHeapBlockRequest => ({
        schema_version: "1.0.0",
        mode: "heap",
        space_id: spaceId,
        source: {
            agent_id: "cortex-web",
            emitted_at: emittedAt,
        },
        block: {
            type: "upload",
            title: resolvedTitle,
            attributes: {
                file_name: artifact.name,
                mime_type: artifact.mime_type,
                file_size: String(artifact.file_size),
            },
        },
        content: {
            payload_type: "pointer",
            pointer: artifact.resource_ref,
        },
        files: [
            {
                hash: artifact.hash,
                file_size: artifact.file_size,
                name: artifact.name,
                mime_type: artifact.mime_type,
                path: artifact.resource_ref,
                is_uploaded: true,
                thumbnails: artifact.thumbnails,
            },
        ],
    });

    const buildEmitPayload = (): EmitHeapBlockRequest => {
        const spaceId = resolveSpaceId(activeSpaceId);
        const emittedAt = new Date().toISOString();
        const titleFallback = createMode === "generate"
            ? "Generated Heap Block"
            : createMode === "upload"
                ? (uploadFile?.name || "Uploaded File Block")
                : "Untitled Block";
        const resolvedTitle = (newBlockTitle || titleFallback).trim();

        if (createMode === "upload") {
            if (!uploadArtifact || !uploadFile) {
                throw new Error("Upload mode requires a completed upload artifact.");
            }
            return buildUploadEmitPayload(uploadArtifact, resolvedTitle, spaceId, emittedAt);
        }

        if (createMode === "generate") {
            return {
                schema_version: "1.0.0",
                mode: "heap",
                space_id: spaceId,
                source: {
                    agent_id: "cortex-web",
                    emitted_at: emittedAt,
                },
                block: {
                    type: "generated",
                    title: resolvedTitle,
                    attributes: {
                        origin: "playground_prompt",
                    },
                },
                content: {
                    payload_type: "structured_data",
                    structured_data: {
                        intent: "generate_with_agent",
                        prompt: agentPrompt.trim(),
                        model: "local",
                        status: "simulated",
                    },
                },
            };
        }

        if (createMode === "chat") {
            return {
                schema_version: "1.0.0",
                mode: "heap",
                space_id: spaceId,
                source: {
                    agent_id: "cortex-web",
                    emitted_at: emittedAt,
                },
                block: {
                    type: "agent_solicitation",
                    title: resolvedTitle || "Review Request",
                },
                content: {
                    payload_type: "structured_data",
                    structured_data: {
                        space_id: spaceId,
                        type: "agent_solicitation",
                        role: solicitRole.trim(),
                        message: solicitMessage.trim() || agentPrompt.trim() || `Route the ${resolvedTitle} task.`,
                        required_capabilities: solicitCapabilities.split(",").map((s) => s.trim()).filter(Boolean),
                        budget: { max: parseInt(solicitBudget, 10) || 50000 },
                        authority_scope: "L1",
                    },
                },
            };
        }

        return {
            schema_version: "1.0.0",
            mode: "heap",
            space_id: spaceId,
            source: {
                agent_id: "cortex-web",
                emitted_at: emittedAt,
            },
            block: {
                type: newBlockType.trim() || "note",
                title: resolvedTitle,
            },
            content: {
                payload_type: "rich_text",
                rich_text: {
                    plain_text: newBlockText.trim() || "New heap block",
                },
            },
        };
    };

    const emitCreatedBlock = async () => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled by VITE_HEAP_PARITY_ENABLED.");
            return;
        }
        if (createMode === "generate" && !agentPrompt.trim()) {
            setStatusMessage("Generate with Agent requires a prompt.");
            return;
        }
        if (createMode === "upload" && !uploadFile) {
            setStatusMessage("Upload mode requires a file.");
            return;
        }
        const spaceId = resolveSpaceId(activeSpaceId);
        const emittedAt = new Date().toISOString();
        const titleFallback = createMode === "generate"
            ? "Generated Heap Block"
            : createMode === "upload"
                ? (uploadFile?.name || "Uploaded File Block")
                : "Untitled Block";
        const resolvedTitle = (newBlockTitle || titleFallback).trim();
        const actorRole = sessionUser?.role || "operator";
        const actorId = sessionUser?.actorId || "cortex-web";
        const sleep = (ms: number) => new Promise((resolve) => window.setTimeout(resolve, ms));
        try {
            setIsEmitting(true);
            if (createMode === "upload") {
                setUploadLifecycleState("uploading");
                setStatusMessage(`Uploading ${uploadFile?.name || "file"}...`);
                const uploaded = await workbenchApi.uploadHeapFile(
                    {
                        file: uploadFile!,
                        spaceId,
                        title: resolvedTitle,
                        sourceAgentId: actorId,
                    },
                    actorRole,
                    actorId,
                );
                setUploadArtifact(uploaded);
                setUploadLifecycleState("uploaded");
                setStatusMessage(`File uploaded. Emitting heap block for ${uploaded.name}...`);
                await workbenchApi.emitHeapBlock(
                    buildUploadEmitPayload(uploaded, resolvedTitle, spaceId, emittedAt),
                    actorRole,
                    actorId,
                );
                await fetchBlocks();
                if (uploaded.extraction_supported) {
                    const requestedParserProfile = uploadParserProfile.trim() || "auto";
                    setUploadLifecycleState("extracting");
                    setStatusMessage(
                        `Heap block created. Extraction queued for ${uploaded.name} using ${parserProfileLabel(requestedParserProfile)}...`,
                    );
                    const queued = await workbenchApi.triggerHeapUploadExtraction(
                        uploaded.upload_id,
                        requestedParserProfile,
                        actorRole,
                        actorId,
                    );
                    setUploadExtractionStatus(queued);
                    let currentStatus = await workbenchApi.getHeapUploadExtractionStatus(uploaded.upload_id, actorRole, actorId);
                    let attempts = 0;
                    while (!isTerminalUploadExtractionStatus(currentStatus.status) && attempts < 6) {
                        await sleep(750);
                        currentStatus = await workbenchApi.getHeapUploadExtractionStatus(uploaded.upload_id, actorRole, actorId);
                        attempts += 1;
                    }
                    setUploadExtractionStatus(currentStatus);
                    setUploadLifecycleState(resolveUploadLifecycleState(currentStatus.status));
                    if (currentStatus.status === "completed") {
                        setStatusMessage(`Upload indexed: ${uploaded.name}.`);
                    } else if (currentStatus.status === "needs_review") {
                        setStatusMessage(`Upload needs review: ${uploaded.name}.`);
                    } else if (currentStatus.status === "failed") {
                        setStatusMessage(`Upload extraction failed: ${uploaded.name}.`);
                    } else {
                        setStatusMessage(`Extraction submitted for ${uploaded.name}.`);
                    }
                    await fetchBlocks();
                } else {
                    setStatusMessage(`File uploaded and added: ${uploaded.name}. Extraction is not supported.`);
                }
                setCreatePanelOpen(false);
                clearCreateForm();
                return;
            }
            await workbenchApi.emitHeapBlock(buildEmitPayload(), actorRole, actorId);
            await fetchBlocks();
            setCreatePanelOpen(false);
            clearCreateForm();
            setStatusMessage("Heap block created.");
        } catch (err) {
            setStatusMessage(`Create block failed: ${err instanceof Error ? err.message : String(err)}`);
        } finally {
            setIsEmitting(false);
        }
    };

    const toggleTag = (tag: string) => {
        setSelectedTags((current) => (current.includes(tag) ? current.filter((item) => item !== tag) : [...current, tag]));
    };
    const togglePageLink = (pageLink: string) => {
        setSelectedPageLinks((current) =>
            current.includes(pageLink) ? current.filter((item) => item !== pageLink) : [...current, pageLink]
        );
    };

    useEffect(() => {
        if (!effectiveHeapChangedBlocksPollingEnabled) {
            return;
        }
        const intervalMs = Number.isFinite(heapChangedBlocksPollingIntervalMs) && heapChangedBlocksPollingIntervalMs > 0
            ? Math.max(500, Math.trunc(heapChangedBlocksPollingIntervalMs))
            : 15000;
        const timer = window.setInterval(async () => {
            try {
                const response = await workbenchApi.getHeapChangedBlocks({
                    spaceId: activeSpaceId,
                    blockType: filterDefaults?.blockType,
                    pageLink: activePageLinkFilter,
                    changedSince: lastDeltaSince || undefined,
                    includeDeleted: true,
                    limit: 100,
                });
                if ((response.changed.length === 0 && response.deleted.length === 0)) {
                    return;
                }
                setBlocks((current) => reconcileHeapDelta(current, response.changed, response.deleted));
                const newestTimestamp = pickNewestTimestamp(response.changed, response.deleted, lastDeltaSince);
                if (newestTimestamp) {
                    setLastDeltaSince(newestTimestamp);
                }
            } catch (err) {
                console.warn("Heap changed_blocks polling failed", err);
            }
        }, intervalMs);
        return () => window.clearInterval(timer);
    }, [
        effectiveHeapChangedBlocksPollingEnabled,
        heapChangedBlocksPollingIntervalMs,
        activeSpaceId,
        filterDefaults?.blockType,
        activePageLinkFilter,
        lastDeltaSince,
    ]);

    const persistHeapDeltaPollingEnabled = (enabled: boolean) => {
        if (heapDeltaPollingControlsLocked) {
            return;
        }
        setHeapChangedBlocksPollingEnabled(enabled);
        if (typeof window === "undefined") {
            return;
        }
        try {
            if (enabled) {
                window.localStorage.setItem(HEAP_DELTA_POLLING_ENABLED_KEY, "1");
            } else {
                window.localStorage.removeItem(HEAP_DELTA_POLLING_ENABLED_KEY);
            }
        } catch {
            // Ignore localStorage access failures in restricted environments.
        }
    };

    const persistHeapDeltaPollingInterval = (intervalMs: number) => {
        if (heapDeltaPollingControlsLocked) {
            return;
        }
        const clamped = clampHeapDeltaPollingIntervalMs(intervalMs);
        setHeapChangedBlocksPollingIntervalMs(clamped);
        setHeapChangedBlocksPollingIntervalInput(String(clamped));
        if (typeof window === "undefined") {
            return;
        }
        try {
            window.localStorage.setItem(HEAP_DELTA_POLLING_INTERVAL_MS_KEY, String(clamped));
        } catch {
            // Ignore localStorage access failures in restricted environments.
        }
    };

    const commitHeapDeltaPollingIntervalInput = () => {
        if (heapDeltaPollingControlsLocked) {
            return;
        }
        const parsed = Number(heapChangedBlocksPollingIntervalInput);
        if (!Number.isFinite(parsed) || parsed <= 0) {
            setHeapChangedBlocksPollingIntervalInput(String(heapChangedBlocksPollingIntervalMs));
            return;
        }
        persistHeapDeltaPollingInterval(parsed);
    };

    if (error) {
        if (isPublicObserverGatewayBoundary(error, configuredGatewayTarget, publicHost)) {
            return (
                <div className="m-4 max-w-3xl rounded-2xl border border-sky-300/15 bg-sky-300/7 px-5 py-4 text-sm text-sky-50 shadow-[0_18px_54px_-38px_rgba(56,189,248,0.45)]">
                    <div className="flex flex-wrap items-center gap-2 font-semibold">
                        <span className="h-2 w-2 rounded-full bg-sky-300 shadow-[0_0_14px_rgba(125,211,252,0.9)]" />
                        <span>Read-only observer mode</span>
                        <span className="rounded-full border border-white/10 bg-white/[0.04] px-2 py-0.5 text-[10px] uppercase tracking-[0.16em] text-sky-100/65">
                            Operator actions gated
                        </span>
                    </div>
                    <p className="mt-2 leading-6 text-sky-50/72">
                        This public browser session cannot reach the private gateway target, so live heap data is limited until a trusted local or verified operator session is used.
                    </p>
                    <p className="mt-2 break-all text-[11px] leading-5 text-sky-50/45">
                        Gateway target: {configuredGatewayTarget}
                    </p>
                </div>
            );
        }
        return <div style={{ padding: "1rem", background: "rgba(127,29,29,0.5)", border: "1px solid #7f1d1d", color: "#fca5a5", borderRadius: "0.5rem" }}>Failed to load heap: {error}</div>;
    }

    return (
        <div className="heap-surface flex h-full w-full overflow-hidden select-none relative">
            {/* Sidebar Backdrop (Mobile Only) */}
            {isMobile && !isSidebarCollapsed && (
                <div 
                    className="fixed inset-0 bg-slate-950/60 backdrop-blur-sm z-90 animate-in fade-in duration-300"
                    onClick={() => setIsSidebarCollapsed(true)}
                />
            )}

            {/* Sidebar with collapse transition */}
            {showFilterSidebar && (
                <div className={`
                    ${isMobile ? "fixed left-0 top-0 bottom-0 z-100 shadow-2xl" : "relative flex shrink-0"}
                    transition-all duration-300 ease-in-out
                    ${isSidebarCollapsed ? (isMobile ? "-translate-x-full" : "w-0 overflow-hidden") : "w-64 translate-x-0"}
                `}>
                    <HeapFilterSidebar
                        filterTerm={filterTerm}
                        onFilterTermChange={setFilterTerm}
                        excludeTerm={excludeTerm}
                        onExcludeTermChange={setExcludeTerm}
                        filterMode={filterMode}
                        onFilterModeChange={setFilterMode}
                        allTags={allTags}
                        selectedTags={selectedTags}
                        onToggleTag={toggleTag}
                        allPageLinks={allPageLinks}
                        selectedPageLinks={selectedPageLinks}
                        onTogglePageLink={togglePageLink}
                        pageLinkTerm={pageLinkTerm}
                        onPageLinkTermChange={setPageLinkTerm}
                        availableReviewLanes={availableReviewLanes}
                        reviewLaneCounts={reviewLaneCounts}
                        selectedReviewLane={selectedReviewLane}
                        onReviewLaneChange={setSelectedReviewLane}
                        viewCounts={blockCounts}
                        viewMode={viewMode}
                        onViewModeChange={(nextMode) => {
                            const nextParams = new URLSearchParams(searchParams);
                            nextParams.set("heap_view", heapPrimaryViewModeParam(nextMode));
                            setSearchParams(nextParams, { replace: true });
                        }}
                        multiSelectEnabled={multiSelectEnabled}
                        onToggleMultiSelect={() => setMultiSelectEnabled((value) => !value)}
                        searchInputId={SEARCH_INPUT_ID}
                        heapParityEnabled={heapParityEnabled}
                        isCollapsed={isSidebarCollapsed}
                        onToggleCollapse={() => setIsSidebarCollapsed(!isSidebarCollapsed)}
                    />
                </div>
            )}

            {/* Main Content */}
            <div className="heap-block-grid flex h-full w-full bg-cortex-surface-base overflow-hidden relative">
                {/* Ambient Background Graph */}
                {!isMobile && exploreSettings.visualizationMode !== "off" && (
                    <AmbientGraphBackground
                        visible={true}
                        variant={exploreSettings.visualizationMode as "2d" | "3d"}
                        spaceId={resolveSpaceId(activeSpaceId)}
                    />
                )}

                {heapParityEnabled && <AgentActivityPanel spaceId={resolveSpaceId(activeSpaceId)} />}

                {/* Scrollable Area */}
                <div className="flex-1 flex flex-col overflow-y-auto custom-scrollbar relative z-10 bg-transparent">
                    {/* Header - now sticky within the scrollable div */}
                    <header id="heap-grid-header" className="min-h-[56px] flex items-center justify-between px-3 py-3 sm:px-4 sticky top-0 z-30 flex-wrap gap-3 glass-panel backdrop-blur-xl rounded-none shadow-sm border-b border-white/5">
                        <div className="flex items-center gap-3 flex-wrap min-w-0">
                            {!isMobile && isSidebarCollapsed && (
                                <button
                                    onClick={() => setIsSidebarCollapsed(false)}
                                    className="p-1.5 rounded-lg hover:bg-white/5 text-cortex-500 hover:text-white transition-colors mr-1"
                                    title="Show Sidebar"
                                >
                                    <Menu className="w-5 h-5" />
                                </button>
                            )}
                            <div className="min-w-0">
                                <h2 className="text-lg font-bold text-cortex-50 tracking-tight">
                                    {viewMode}
                                    <span className="ml-2 text-cortex-500 font-medium text-sm uppercase tracking-widest hidden sm:inline">Heap Blocks</span>
                                </h2>
                                <div className="mt-1 hidden flex-wrap gap-2 text-[10px] uppercase tracking-[0.24em] text-cortex-400 md:flex">
                                    <span className="rounded-full border border-white/8 bg-white/3 px-2 py-1 text-cortex-300/80">
                                        {exploreSettings.projectionIntent}
                                    </span>
                                    <span className="rounded-full border border-white/8 bg-white/3 px-2 py-1 text-cortex-300/80">
                                        {exploreSettings.layoutMode}
                                    </span>
                                    <span className="rounded-full border border-white/8 bg-white/3 px-2 py-1 text-cortex-300/80">
                                        {exploreSettings.cardDepth}
                                    </span>
                                    <span className="rounded-full border border-white/8 bg-white/3 px-2 py-1 text-cortex-300/80">
                                        {exploreSettings.aggregationMode}
                                    </span>
                                    <span className="rounded-full border border-white/8 bg-white/3 px-2 py-1 text-cortex-300/80">
                                        {exploreSettings.visualizationMode}
                                    </span>
                                </div>
                            </div>
                            {(includeTerms.length > 0 || selectedTags.length > 0) && (
                                <span className="text-[9px] uppercase font-black px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20 shadow-sm">{filterMode} MATCH</span>
                            )}
                            {selectedPageLinks.length > 0 && (
                                <span className="text-[9px] uppercase font-black px-2 py-0.5 rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 shadow-sm">{selectedPageLinks.length} LINKS</span>
                            )}
                            {selectedReviewLane && (
                                <span className="text-[9px] uppercase font-black px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-300 border border-emerald-500/20 shadow-sm">
                                    {selectedReviewLane === "private_review" ? "PRIVATE" : "PUBLIC"}
                                </span>
                            )}
                            {excludeTerms.length > 0 && (
                                <span className="text-[9px] uppercase font-black px-2 py-0.5 rounded-full bg-red-500/10 text-red-300 border border-red-500/20 shadow-sm">NOT {excludeTerms.length}</span>
                            )}
                        </div>
                        <div className="flex max-w-full flex-wrap items-center gap-1.5 rounded-2xl border border-white/8 bg-white/3 p-1.5 shadow-sm backdrop-blur-sm sm:rounded-full">
                            <button
                                onClick={() => setIsSidebarCollapsed(prev => !prev)}
                                className={`flex h-8 items-center gap-1.5 rounded-full px-2.5 transition-all duration-200 ${
                                    !isSidebarCollapsed
                                        ? "bg-blue-600/80 text-white shadow-sm"
                                        : "bg-cortex-800/60 text-cortex-500 hover:text-cortex-300"
                                }`}
                                title="Toggle filters"
                                aria-label="Toggle filters"
                            >
                                <Filter className="w-3.5 h-3.5" />
                                <span className="hidden sm:inline text-[10px] font-semibold uppercase tracking-[0.18em]">Filter</span>
                            </button>

                            <details
                                className="relative"
                                open={settingsOpen}
                                onToggle={(event) => {
                                    setSettingsOpen((event.currentTarget as HTMLDetailsElement).open);
                                }}
                            >
                                <summary
                                    className={`flex list-none items-center gap-1.5 rounded-full transition-all duration-200 cursor-pointer px-2.5 py-2 text-[10px] font-semibold uppercase tracking-[0.18em] ${
                                        settingsOpen
                                            ? "bg-slate-700 text-white shadow-sm"
                                            : "bg-cortex-800/60 text-cortex-500 hover:text-cortex-300"
                                    }`}
                                    title="Details"
                                    aria-label="Details"
                                >
                                    <span>Details</span>
                                    <ChevronDown className="w-3 h-3 opacity-80" />
                                </summary>

                                {settingsOpen && (
                                    <>
                                        <div className="fixed inset-0 z-40" onClick={() => setSettingsOpen(false)} />
                                        <div className="fixed right-2 top-16 z-50 max-h-[calc(100vh-5rem)] w-[min(22rem,calc(100vw-1rem))] overflow-y-auto rounded-3xl border border-white/10 bg-[linear-gradient(180deg,rgba(15,23,42,0.98),rgba(2,6,23,0.96))] shadow-2xl animate-in fade-in zoom-in-95 duration-200">
                                            <div className="absolute inset-0 pointer-events-none bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.08),transparent_42%),radial-gradient(circle_at_bottom_left,rgba(59,130,246,0.08),transparent_35%)]" />
                                            <div className="relative p-3.5">
                                                <div className="flex items-start justify-between gap-3">
                                                    <div>
                                                        <h4 className="text-[10px] font-black uppercase tracking-[0.24em] text-cortex-500">Details</h4>
                                                        <p className="mt-1.5 text-[11px] leading-6 text-cortex-300/70">
                                                            Space defaults first, then personal overrides, then session links.
                                                        </p>
                                                    </div>
                                                    <button
                                                        type="button"
                                                        onClick={() => setSettingsOpen(false)}
                                                        className="rounded-full border border-white/10 bg-white/4 px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-cortex-200"
                                                    >
                                                        Close
                                                    </button>
                                                </div>

                                                {Object.keys(exploreSessionOverrides).length > 0 && (
                                                    <div className="mt-3 rounded-2xl border border-cyan-500/15 bg-cyan-500/8 px-3 py-2 text-[11px] text-cyan-100/80 flex items-center justify-between gap-2">
                                                        <span>URL overrides active.</span>
                                                        <button
                                                            type="button"
                                                            onClick={() => {
                                                                clearExploreSessionOverrides();
                                                            }}
                                                            className="rounded-full border border-cyan-400/20 bg-cyan-500/10 px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.18em]"
                                                        >
                                                            Clear URL
                                                        </button>
                                                    </div>
                                                )}

                                                <div className="mt-3 space-y-2.5">
                                                    {EXPLORE_VIEW_SETTING_SECTIONS.map((section) => (
                                                        <div key={section.id} className="rounded-2xl border border-white/10 bg-white/3 p-2.5">
                                                            <div className="flex items-start justify-between gap-3">
                                                                <div>
                                                                    <div className="text-[10px] font-black uppercase tracking-[0.22em] text-cortex-500">{section.title}</div>
                                                                    <p className="mt-1.5 text-[11px] leading-6 text-cortex-300/60">{section.description}</p>
                                                                </div>
                                                                {section.icon === "sliders" && <Sliders className="w-3.5 h-3.5 text-cortex-500/70" />}
                                                            </div>
                                                            <div className="mt-2.5 space-y-2">
                                                                {section.controls.map((control) => (
                                                                    <ExploreSettingsGroup
                                                                        key={control.key}
                                                                        descriptor={control}
                                                                        source={describeExploreSettingOrigin(exploreView.provenance[control.key])}
                                                                        value={exploreSettings[control.key]}
                                                                        controlKind={resolveExploreSettingVisualKind(control, {
                                                                            isCompactPanel: exploreSettings.layoutMode === "compact",
                                                                        })}
                                                                        onChange={(nextValue) => {
                                                                            setExploreSetting(
                                                                                control.key,
                                                                                nextValue as ExploreViewSettings[typeof control.key],
                                                                            );
                                                                        }}
                                                                    />
                                                                ))}
                                                            </div>
                                                        </div>
                                                    ))}
                                                </div>

                                                <div className="mt-3 flex items-center justify-between gap-3">
                                                    <button
                                                        type="button"
                                                        onClick={() => setSaveViewModalOpen(true)}
                                                        className="rounded-full border border-cyan-400/15 bg-cyan-500/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.18em] text-cyan-100"
                                                    >
                                                        Save View
                                                    </button>
                                                    <button
                                                        type="button"
                                                        onClick={() => {
                                                            exploreView.resetUserOverrides();
                                                        }}
                                                        className="rounded-full border border-white/10 bg-white/3 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.18em] text-cortex-200"
                                                    >
                                                        Reset overrides
                                                    </button>
                                                    <div className="text-[10px] uppercase tracking-[0.24em] text-cortex-500">
                                                        {exploreDerivedSettings.laneCap} lanes · {exploreDerivedSettings.groupPreviewCount} preview
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </>
                                )}
                            </details>

                            <button
                                onClick={() => {
                                    if (chatPanelOpen) {
                                        setChatPanelOpen(false);
                                        return;
                                    }
                                    openChatConversation();
                                }}
                                className={`flex h-8 items-center gap-1.5 rounded-full px-2.5 transition-all duration-200 ${
                                    chatPanelOpen
                                        ? "bg-indigo-600/80 text-white shadow-sm shadow-indigo-500/20"
                                        : "bg-cortex-800/60 text-cortex-500 hover:text-cortex-300"
                                }`}
                                title="Chat with Eudaemon"
                                aria-label="Chat with Eudaemon"
                            >
                                <MessagesSquare className="w-3.5 h-3.5" />
                                <span className="hidden sm:inline text-[10px] font-semibold uppercase tracking-[0.18em]">Chat</span>
                            </button>
                        </div>
                    </header>

                    {statusMessage && (
                        <div className="mx-4 mt-3 rounded-2xl border border-cyan-400/10 bg-cortex-surface-panel/70 px-4 py-3 text-sm text-cortex-200 shadow-lg backdrop-blur-md">
                            {statusMessage}
                        </div>
                    )}

                    {/* Floating Action Bar (Selection-driven) */}
                    <HeapActionBar
                        selection={selectionContext}
                        selectionZonePlan={selectionZonePlan}
                        handlers={actionHandlers}
                        onCreate={() => {
                            setCreateMode("create");
                            setCreatePanelOpen(true);
                        }}
                        onChat={openChatConversation}
                        canPublish={["operator", "steward", "admin"].includes(sessionUser?.role ?? "")}
                        status={
                            actionPlanLoading
                                ? { loading: true, source: "idle", error: null }
                                : actionPlanError
                                ? { loading: false, source: "idle", error: actionPlanError }
                                : { loading: false, source: actionPlanSource, error: null }
                        }
                    />

                    {/* Create Tool Panel Overlay */}
                    {createPanelOpen && (
                        <div className="fixed inset-0 z-100 flex items-center justify-center bg-slate-950/40 backdrop-blur-sm animate-in fade-in duration-300">
                             <div 
                                className="absolute inset-0" 
                                onClick={() => setCreatePanelOpen(false)} 
                            />
                            <div className="relative w-full max-w-2xl bg-cortex-900 border border-cortex-700/50 rounded-2xl shadow-3xl overflow-hidden animate-in slide-in-from-bottom-8 zoom-in-95 duration-300">
                                <div className="p-6 border-b border-cortex-800 flex items-center justify-between bg-cortex-800/20">
                                    <div className="flex items-center gap-2">
                                        <div className="w-8 h-8 rounded-full bg-blue-500/20 flex items-center justify-center">
                                            <Plus className="w-5 h-5 text-blue-400" />
                                        </div>
                                        <h3 className="text-lg font-bold text-slate-200 tracking-tight">Create New Block</h3>
                                    </div>
                                    <button 
                                        onClick={() => setCreatePanelOpen(false)}
                                        className="w-10 h-10 rounded-full flex items-center justify-center hover:bg-cortex-700/50 text-cortex-400 hover:text-white transition-colors"
                                    >
                                        <Plus className="w-6 h-6 rotate-45" />
                                    </button>
                                </div>
                                <div className="p-6 max-h-[80vh] overflow-y-auto custom-scrollbar">
                                    <div className="flex flex-wrap gap-2 mb-8">
                                        {(["create", "generate", "upload", "chat", "plan"] as CreateMode[]).map((mode) => (
                                            <button
                                                key={mode}
                                                onClick={() => setCreateMode(mode)}
                                                className={`px-5 py-2 rounded-full text-xs font-black tracking-widest uppercase transition-all duration-300 border ${
                                                    createMode === mode
                                                        ? "bg-blue-600 border-blue-400 text-white shadow-lg shadow-blue-600/20 scale-105"
                                                        : "bg-cortex-800/50 border-cortex-700/50 text-cortex-400 hover:text-white hover:bg-cortex-700/60"
                                                }`}
                                            >
                                                {mode}
                                            </button>
                                        ))}
                                    </div>

                                    {createMode === "create" && (
                                        <div className="space-y-6 animate-in fade-in slide-in-from-top-2 duration-300">
                                            <div>
                                                <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">Title</label>
                                                <input
                                                    className="w-full bg-cortex-950 border border-cortex-800/80 rounded-xl px-4 py-3 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500/40 focus:border-blue-500/50 transition-all placeholder:text-cortex-700"
                                                    placeholder="A meaningful title..."
                                                    value={newBlockTitle}
                                                    onChange={(e) => setNewBlockTitle(e.target.value)}
                                                />
                                            </div>
                                            <div>
                                                <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">Block Type</label>
                                                <select
                                                    className="w-full bg-cortex-950 border border-cortex-800/80 rounded-xl px-4 py-3 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500/40 transition-all cursor-pointer appearance-none"
                                                    value={newBlockType}
                                                    onChange={(e) => setNewBlockType(e.target.value)}
                                                >
                                                    <option value="note">Note</option>
                                                    <option value="task">Task</option>
                                                    <option value="system">System</option>
                                                    <option value="report">Report</option>
                                                </select>
                                            </div>
                                            <div>
                                                <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">Content</label>
                                                <textarea
                                                    className="w-full bg-cortex-950 border border-cortex-800/80 rounded-xl px-4 py-3 text-sm text-slate-200 h-32 resize-none focus:outline-none focus:ring-2 focus:ring-blue-500/40 transition-all placeholder:text-cortex-700"
                                                    placeholder="Rich text content..."
                                                    value={newBlockText}
                                                    onChange={(e) => setNewBlockText(e.target.value)}
                                                />
                                            </div>
                                        </div>
                                    )}

                                    {createMode === "generate" && (
                                        <div className="space-y-6 animate-in fade-in slide-in-from-top-2 duration-300">
                                            <div>
                                                <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">Agent Instruction</label>
                                                <textarea
                                                    className="w-full bg-cortex-950 border border-cortex-800/80 rounded-xl px-4 py-3 text-sm text-slate-200 h-32 resize-none focus:outline-none focus:ring-2 focus:ring-blue-500/40 transition-all placeholder:text-cortex-700"
                                                    placeholder="e.g. 'Synthesize the last 5 security logs into a summary report'"
                                                    value={agentPrompt}
                                                    onChange={(e) => setAgentPrompt(e.target.value)}
                                                />
                                            </div>
                                        </div>
                                    )}

                                    {createMode === "upload" && (
                                        <div className="space-y-6 animate-in fade-in slide-in-from-top-2 duration-300">
                                            <div className="relative border-2 border-dashed border-cortex-800 rounded-2xl p-8 flex flex-col items-center justify-center hover:border-blue-500/40 transition-colors cursor-pointer group bg-cortex-950/50">
                                                <Plus className="w-10 h-10 text-cortex-600 mb-4 group-hover:text-blue-400 transition-colors" />
                                                <p className="text-sm text-cortex-400 group-hover:text-cortex-200 transition-colors">Select a file to import into the heap</p>
                                                <input
                                                    type="file"
                                                    className="opacity-0 absolute inset-0 w-full h-full cursor-pointer"
                                                    onChange={(e) => handleUploadFileChange(e.target.files?.[0] || null)}
                                                />
                                                {uploadFile && (
                                                    <div className="mt-4 flex flex-wrap items-center justify-center gap-2">
                                                        <p className="text-emerald-400 text-xs font-bold bg-emerald-500/10 px-3 py-1 rounded-full uppercase tracking-tighter shadow-sm border border-emerald-500/30">
                                                            Selected: {uploadFile.name}
                                                        </p>
                                                        <span className={`rounded-full border px-3 py-1 text-[10px] font-black uppercase tracking-[0.24em] ${resolveUploadLifecycleClassName(uploadLifecycleState)}`}>
                                                            {resolveUploadLifecycleLabel(uploadLifecycleState)}
                                                        </span>
                                                    </div>
                                                )}
                                            </div>
                                            <div className="rounded-2xl border border-white/10 bg-cortex-950/70 p-5">
                                                <div className="flex flex-wrap items-center justify-between gap-3">
                                                    <div>
                                                        <p className="text-[10px] font-black uppercase tracking-[0.24em] text-cortex-500">Extraction lane</p>
                                                        <p className="mt-2 text-sm font-semibold text-slate-100">
                                                            Default lane: {parserProfileLabel(defaultUploadParserProfile)}
                                                        </p>
                                                        <p className="mt-1 text-xs leading-5 text-slate-400">
                                                            Providers are runtime inventory. Parser backends like LiteParse are selected and reviewed on artifacts.
                                                        </p>
                                                    </div>
                                                    {selectedUploadMimeType && (
                                                        <span className="rounded-full border border-white/10 bg-white/4 px-3 py-1 text-[10px] font-black uppercase tracking-[0.24em] text-slate-300">
                                                            {selectedUploadMimeType}
                                                        </span>
                                                    )}
                                                </div>
                                                <div className="mt-4">
                                                    <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">
                                                        Requested parser
                                                    </label>
                                                    <select
                                                        className="w-full bg-cortex-950 border border-cortex-800/80 rounded-xl px-4 py-3 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500/40 transition-all cursor-pointer appearance-none"
                                                        value={uploadParserProfile}
                                                        onChange={(e) => setUploadParserProfile(e.target.value)}
                                                    >
                                                        {(availableUploadParserProfiles.length > 0
                                                            ? availableUploadParserProfiles
                                                            : [{ parser_profile: "auto", parser_hint: "auto", role: "primary", configured: true, supports_mime: [] }]
                                                        ).map((profile) => (
                                                            <option key={profile.parser_profile} value={profile.parser_profile}>
                                                                {parserProfileLabel(profile.parser_profile)} · {profile.role}
                                                            </option>
                                                        ))}
                                                    </select>
                                                    <p className="mt-2 text-xs leading-5 text-slate-400">
                                                        Auto keeps the runtime default. Choose LiteParse here or in the detail modal to compare parser output on the same uploaded artifact.
                                                    </p>
                                                    {uploadParserProfilesError && (
                                                        <p className="mt-2 text-xs text-rose-300">
                                                            Parser profile inventory unavailable: {uploadParserProfilesError}
                                                        </p>
                                                    )}
                                                    {uploadExtractionStatus?.requested_parser_profile && (
                                                        <p className="mt-2 text-xs text-slate-300">
                                                            Requested: {parserProfileLabel(uploadExtractionStatus.requested_parser_profile)}
                                                            {uploadExtractionStatus.parser_backend ? ` · Resolved: ${parserProfileLabel(uploadExtractionStatus.parser_backend)}` : ""}
                                                        </p>
                                                    )}
                                                </div>
                                            </div>
                                        </div>
                                    )}

                                    {createMode === "chat" && (
                                        <div className="space-y-6 animate-in fade-in slide-in-from-top-2 duration-300">
                                            <div>
                                                <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">Target Role</label>
                                                <input
                                                    className="w-full bg-cortex-950 border border-cortex-800/80 rounded-xl px-4 py-3 text-sm text-slate-200 focus:outline-none focus:border-blue-500"
                                                    value={solicitRole}
                                                    onChange={(e) => setSolicitRole(e.target.value)}
                                                />
                                            </div>
                                            <div>
                                                <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">Cycle Budget</label>
                                                <input
                                                    className="w-full bg-cortex-950 border border-cortex-800/80 rounded-xl px-4 py-3 text-sm text-slate-200 focus:outline-none focus:border-blue-500"
                                                    value={solicitBudget}
                                                    onChange={(e) => setSolicitBudget(e.target.value)}
                                                />
                                            </div>
                                            <div>
                                                <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">Required Capabilities</label>
                                                <input
                                                    className="w-full bg-cortex-950 border border-cortex-800/80 rounded-xl px-4 py-3 text-sm text-slate-200 focus:outline-none focus:border-blue-500"
                                                    value={solicitCapabilities}
                                                    onChange={(e) => setSolicitCapabilities(e.target.value)}
                                                />
                                            </div>
                                            <div>
                                                <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">Solicitation Summary</label>
                                                <textarea
                                                    className="w-full bg-cortex-950 border border-cortex-800/80 rounded-xl px-4 py-3 text-sm text-slate-200 h-28 resize-none focus:outline-none focus:ring-2 focus:ring-blue-500/40 transition-all placeholder:text-cortex-700"
                                                    placeholder="Summarize the route choice, bottlenecks, and the next action."
                                                    value={solicitMessage}
                                                    onChange={(e) => setSolicitMessage(e.target.value)}
                                                />
                                            </div>
                                        </div>
                                    )}

                                    {createMode === "plan" && (
                                        <div className="space-y-6 animate-in fade-in slide-in-from-top-2 duration-300">
                                            <div className="rounded-2xl border border-cyan-400/10 bg-cyan-500/5 p-4">
                                                <div className="text-[10px] font-black uppercase tracking-[0.28em] text-cyan-300/80">
                                                    Plan-backed requests
                                                </div>
                                                <p className="mt-2 text-sm leading-6 text-cortex-300/75">
                                                    Emit an ordinary heap review request from active initiative planning metadata. This is a create flow, not a separate heap feature.
                                                </p>
                                            </div>

                                            {!initiativeKickoffLaunchAllowed && initiativeKickoffDisabledReason && (
                                                <div className="rounded-2xl border border-amber-400/10 bg-amber-500/10 px-4 py-3 text-sm leading-6 text-amber-100/85">
                                                    {initiativeKickoffDisabledReason}
                                                </div>
                                            )}

                                            <div className="space-y-3">
                                                {INITIATIVE_KICKOFF_TEMPLATES.map((entry) => (
                                                    <div
                                                        key={entry.template.id}
                                                        className="rounded-2xl border border-white/8 bg-cortex-950/60 p-4"
                                                    >
                                                        <div className="flex items-start justify-between gap-4">
                                                            <div className="min-w-0">
                                                                <div className="text-sm font-semibold tracking-tight text-cortex-100">
                                                                    {entry.template.title}
                                                                </div>
                                                                <div className="mt-1 text-xs uppercase tracking-[0.24em] text-cortex-500">
                                                                    Plan-backed review request
                                                                </div>
                                                                <p className="mt-3 text-sm leading-6 text-cortex-300/75">
                                                                    {entry.description}
                                                                </p>
                                                                <div className="mt-3 flex flex-wrap gap-2">
                                                                    <span className="rounded-full border border-white/8 bg-white/4 px-2.5 py-1 text-[10px] uppercase tracking-[0.22em] text-cortex-300/80">
                                                                        {entry.template.agentRole}
                                                                    </span>
                                                                    <span className="rounded-full border border-white/8 bg-white/4 px-2.5 py-1 text-[10px] uppercase tracking-[0.22em] text-cortex-300/80">
                                                                        {entry.template.requiredCapabilities.length} capabilities
                                                                    </span>
                                                                    <span className="rounded-full border border-white/8 bg-white/4 px-2.5 py-1 text-[10px] uppercase tracking-[0.22em] text-cortex-300/80">
                                                                        {entry.template.referencePaths.length} refs
                                                                    </span>
                                                                </div>
                                                            </div>
                                                            <button
                                                                type="button"
                                                                onClick={() => loadInitiativeKickoffTemplate(entry.template.id)}
                                                                disabled={isEmitting || !initiativeKickoffLaunchAllowed}
                                                                className={`shrink-0 rounded-full px-4 py-2 text-[10px] font-black uppercase tracking-[0.2em] transition-all ${
                                                                    isEmitting || !initiativeKickoffLaunchAllowed
                                                                        ? "cursor-not-allowed border border-white/8 bg-white/5 text-cortex-500"
                                                                        : "border border-cyan-400/20 bg-cyan-500/10 text-cyan-100 hover:bg-cyan-500/18 hover:border-cyan-400/30"
                                                                }`}
                                                            >
                                                                Emit request
                                                            </button>
                                                        </div>
                                                    </div>
                                                ))}
                                            </div>
                                        </div>
                                    )}

                                    <div className="mt-10 flex justify-end gap-3 pt-6 border-t border-cortex-800">
                                        <button
                                            onClick={() => setCreatePanelOpen(false)}
                                            className="px-6 py-2.5 rounded-full text-xs font-bold text-cortex-400 hover:text-white hover:bg-cortex-800 transition-all border border-transparent hover:border-cortex-700"
                                        >
                                            {createMode === "plan" ? "Close" : "Cancel"}
                                        </button>
                                        {createMode !== "plan" && (
                                            <button
                                                disabled={isEmitting}
                                                onClick={emitCreatedBlock}
                                                className="px-8 py-2.5 bg-blue-600 text-white rounded-full text-xs font-black uppercase tracking-widest hover:bg-blue-500 active:scale-95 disabled:opacity-50 disabled:active:scale-100 transition-all shadow-xl shadow-blue-600/20"
                                            >
                                                {isEmitting ? "Emitting..." : "Create Block"}
                                            </button>
                                        )}
                                    </div>
                                </div>
                            </div>
                        </div>
                    )}

                    <div className="flex-1 min-h-0 flex relative">
                        <div className="flex-1 overflow-y-auto relative">
                            {loading ? (
                                <div style={{ padding: "2rem", color: "#64748b", textAlign: "center" }}>Loading blocks...</div>
                            ) : isDerivedSurfaceEmpty ? (
                                <div className="heap-empty-state flex flex-col items-center justify-center h-full w-full px-6 opacity-75 hover:opacity-100 transition-opacity duration-500">
                                    <div className="mb-5 flex h-16 w-16 items-center justify-center rounded-2xl border border-slate-700/50 bg-slate-800/50 shadow-xl">
                                        <Filter className="h-7 w-7 text-slate-500/70" />
                                    </div>
                                    <h3 className="text-lg font-bold tracking-tight text-slate-300 mb-2">No blocks found</h3>
                                    <p className="text-sm text-slate-500 max-w-sm text-center">There are no blocks matching the current view constraints. Try adjusting your filters or generating new content.</p>
                                </div>
                            ) : (
                                <div className={`relative w-full ${exploreSettings.layoutMode === "compact" ? "pt-5 pb-12 px-4" : exploreSettings.layoutMode === "open" ? "pt-8 pb-16 px-6" : "pt-6 pb-14 px-5"}`}>
                                    {derivedViews.length > 0 && (
                                        <section className="mb-4 rounded-2xl border border-white/6 bg-white/3 px-4 py-4 shadow-[0_18px_50px_rgba(0,0,0,0.18)] backdrop-blur-sm">
                                            <div className="flex items-start justify-between gap-4 flex-wrap">
                                                <div className="min-w-0">
                                                    <div className="text-[10px] font-black uppercase tracking-[0.32em] text-cortex-500">Views</div>
                                                    <p className="mt-1 text-sm text-cortex-300/75">
                                                        Showing recent updates, proposals, evidence, and agent activity for this Space.
                                                    </p>
                                                </div>
                                                <div className="flex flex-wrap gap-2">
                                                    {derivedViews.map((derivedView) => {
                                                        const isActive = activeDerivedViewId === derivedView.id;
                                                        return (
                                                            <button
                                                                key={derivedView.id}
                                                                type="button"
                                                                onClick={() => setActiveDerivedViewId(derivedView.id)}
                                                                className={`rounded-full border px-3 py-2 text-left transition-colors ${
                                                                    isActive
                                                                        ? "border-cyan-400/30 bg-cyan-500/10 text-cyan-100"
                                                                        : "border-white/8 bg-white/3 text-cortex-300 hover:border-white/14 hover:bg-white/5"
                                                                }`}
                                                            >
                                                                <div className="text-[11px] font-semibold tracking-tight">{derivedView.label}</div>
                                                                <div className="mt-0.5 text-[10px] uppercase tracking-[0.24em] opacity-70">
                                                                    {derivedView.count} records
                                                                </div>
                                                            </button>
                                                        );
                                                    })}
                                                </div>
                                            </div>
                                            {activeHeapViewContext && (
                                                <div className="mt-3 flex flex-wrap items-center gap-2 text-[11px] text-cortex-400">
                                                    <span className="rounded-full border border-white/8 bg-white/3 px-2 py-1 uppercase tracking-[0.24em] text-cortex-500">
                                                        Context
                                                    </span>
                                                    {activeHeapViewContext.recentTitles.slice(0, 2).map((title) => (
                                                        <span
                                                            key={title}
                                                            className="rounded-full border border-white/8 bg-white/3 px-2 py-1 text-cortex-300/80"
                                                        >
                                                            {title}
                                                        </span>
                                                    ))}
                                                    {activeHeapViewContext.signals.slice(0, 2).map((signal) => (
                                                        <span
                                                            key={signal.label}
                                                            className="rounded-full border border-cyan-500/15 bg-cyan-500/8 px-2 py-1 text-cyan-100/80"
                                                        >
                                                            {signal.label}
                                                        </span>
                                                    ))}
                                                </div>
                                            )}
                                        </section>
                                    )}

                                    {visibleAggregationGroups.length > 0 && (
                                        <div className="mb-6 space-y-3">
                                            {visibleAggregationGroups.map((group) => (
                                                <section
                                                    key={group.groupId}
                                                    className="overflow-hidden rounded-2xl border border-white/6 bg-slate-950/55 shadow-[0_18px_40px_rgba(0,0,0,0.18)]"
                                                >
                                                    <div className="flex items-start justify-between gap-4 px-4 py-3">
                                                        <div className="min-w-0">
                                                            <div className="flex items-center gap-2 flex-wrap">
                                                                <h3 className="text-sm font-semibold tracking-tight text-cortex-50">{group.label}</h3>
                                                                <span className="rounded-full border border-cyan-500/20 bg-cyan-500/10 px-2 py-0.5 text-[10px] uppercase tracking-[0.24em] text-cyan-200">
                                                                    {group.count} records
                                                                </span>
                                                            </div>
                                                            {exploreSettings.showGroupDescriptions && (
                                                                <p className="mt-1 text-xs leading-5 text-cortex-300/70">{group.description}</p>
                                                            )}
                                                        </div>
                                                        <button
                                                            type="button"
                                                            onClick={() => setExpandedAggregationGroupId(group.groupId)}
                                                            className="shrink-0 rounded-full border border-white/10 bg-white/3 px-3 py-2 text-[11px] font-semibold text-cortex-200 transition-colors hover:bg-white/6"
                                                        >
                                                            Open
                                                        </button>
                                                    </div>
                                                    <div className="border-t border-white/6">
                                                        <div className="divide-y divide-white/6">
                                                            {group.items.slice(0, exploreDerivedSettings.groupPreviewCount).map((item) => (
                                                                <div
                                                                    key={item.artifactId}
                                                                    className="flex items-start justify-between gap-4 px-4 py-2.5"
                                                                >
                                                                    <div className="min-w-0">
                                                                        <div className="flex items-center gap-2 flex-wrap">
                                                                            <span className="truncate text-sm font-medium text-cortex-100">{item.title}</span>
                                                                            <span className={`rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-[0.22em] ${
                                                                                item.tone === "approved"
                                                                                    ? "border-emerald-500/20 bg-emerald-500/10 text-emerald-300"
                                                                                    : item.tone === "rejected"
                                                                                        ? "border-rose-500/20 bg-rose-500/10 text-rose-300"
                                                                                        : "border-white/10 bg-white/4 text-cortex-300"
                                                                            }`}>
                                                                                {item.badge ?? displayBlockType(item.blockType)}
                                                                            </span>
                                                                        </div>
                                                                        <p className="mt-1 line-clamp-2 text-xs leading-5 text-cortex-300/75">{item.summary}</p>
                                                                        <div className="mt-1 text-[11px] text-cortex-400">
                                                                            {group.columns
                                                                                .slice(0, exploreSettings.layoutMode === "compact" ? 1 : 2)
                                                                                .map((column) => `${column.label}: ${item.fields[column.key] ?? "n/a"}`)
                                                                                .join("  •  ")}
                                                                        </div>
                                                                    </div>
                                                                    <div className="shrink-0 text-[11px] text-cortex-500">
                                                                        {new Date(item.updatedAt).toLocaleString(undefined, { month: "short", day: "numeric", hour: "numeric", minute: "2-digit" })}
                                                                    </div>
                                                                </div>
                                                            ))}
                                                        </div>
                                                        {exploreSettings.showGroupDescriptions && group.count > exploreDerivedSettings.groupPreviewCount && (
                                                            <div className="border-t border-white/6 px-4 py-3 text-[11px] text-cortex-400">
                                                                + {group.count - exploreDerivedSettings.groupPreviewCount} more records hidden from this preview.
                                                            </div>
                                                        )}
                                                    </div>
                                                </section>
                                            ))}
                                        </div>
                                    )}
                                    <div
                                        ref={laneBoardHostRef}
                                        className={`heap-lane-board grid items-start isolate ${exploreSettings.layoutMode === "compact" ? "gap-4" : "gap-5"}`}
                                        style={{ gridTemplateColumns: `repeat(${isMobile ? 1 : laneCount}, minmax(0, 1fr))` }}
                                    >
                                        {blockLanes.map((lane, laneIndex) => (
                                            <div key={`lane-${laneIndex}`} className="heap-lane-board__lane flex min-w-0 flex-col gap-4">
                                                {lane.map((b) => (
                                                    <div
                                                        id={`wrapper-${b.projection.artifactId}`}
                                                        key={b.projection.artifactId}
                                                        className="relative min-w-0 group hover:z-10 heap-lane-board__item"
                                                        onMouseEnter={() => setHoveredBlockId(b.projection.artifactId)}
                                                        onMouseLeave={() => setHoveredBlockId(null)}
                                                    >
                                                        <HeapBlockCard
                                                            block={b}
                                                            isSelected={selectedBlockIds.includes(b.projection.artifactId)}
                                                            isRegenerating={regeneratingId === b.projection.artifactId}
                                                            onClick={(event) => handleSelection(b.projection.artifactId, event)}
                                                            onDoubleClick={() => openDetailBlock(b.projection.artifactId)}
                                                            cardActions={cardMenuZonePlan?.actions ?? []}
                                                            cardActionSelection={{
                                                                selectedArtifactIds: [b.projection.artifactId],
                                                                activeArtifactId: b.projection.artifactId,
                                                                selectedCount: 1,
                                                                selectedBlockTypes: [b.projection.blockType],
                                                            }}
                                                            actionHandlers={actionHandlers}
                                                            presentationDepth={exploreSettings.cardDepth}
                                                            onOpenComments={() => {
                                                                setCommentSidebarBlockId(b.projection.artifactId);
                                                            }}
                                                        />
                                                    </div>
                                                ))}
                                            </div>
                                        ))}
                                    </div>
                                    {hoveredBlockId && (
                                        <RelationalOverlay
                                            hoveredBlockId={hoveredBlockId}
                                            blocks={visibleBlocks}
                                        />
                                    )}
                                </div>
                            )}
                        </div>
                    </div>

                    {commentSidebarBlockId && (
                        <CommentSidebar
                            blockId={commentSidebarBlockId}
                            onClose={() => setCommentSidebarBlockId(null)}
                        />
                    )}
                </div>

                {/* Detail Modal */}
                {expandedBlock && (
                    <HeapDetailModal
                        block={expandedBlock}
                        allBlocks={blocks}
                        ambientGraphVariant={exploreSettings.visualizationMode}
                        onClose={() => {
                            setExpandedBlockId(null);
                            setDetailNavigationTrail([]);
                        }}
                        onNavigateToBlock={navigateDetailBlock}
                        onRelationSaved={(artifactId) => {
                            fetchBlocks();
                            setStatusMessage(`Relation map updated for ${artifactId}.`);
                        }}
                        onUploadExtractionUpdated={(message) => {
                            fetchBlocks();
                            setStatusMessage(message);
                        }}
                        onRegenerate={handleRegenerate}
                        onTaskRouteSelected={(routeId, context, decision) => {
                            applyTaskRouteDecision(routeId, context, decision);
                        }}
                        onViewDiscussion={(id) => {
                            setExpandedBlockId(null);
                            setCommentSidebarBlockId(id);
                        }}
                        navigationTrail={detailNavigationTrail}
                        onNavigateBack={goBackInDetailHistory}
                    />
                )}

                {expandedAggregationGroupId && (
                    <HeapAggregationDetailModal
                        group={visibleAggregationGroups.find((group) => group.groupId === expandedAggregationGroupId)!}
                        onClose={() => setExpandedAggregationGroupId(null)}
                        onOpenBlock={(artifactId) => {
                            setSelectedBlockIds([artifactId]);
                            setExpandedBlockId(artifactId);
                        }}
                    />
                )}

                {stewardGateArtifactId && stewardGateValidation && (
                    <StewardGateModal
                        artifactId={stewardGateArtifactId}
                        gate={stewardGateValidation}
                        applyingId={stewardApplyingId}
                        publishing={stewardPublishing}
                        onClose={() => {
                            setStewardGateArtifactId(null);
                            setStewardGateValidation(null);
                        }}
                        onApply={handleStewardGateApply}
                        onPublish={handleStewardGatePublish}
                        onRevalidate={handleStewardGateRevalidate}
                    />
                )}

                <ExploreSavedViewModal
                    isOpen={saveViewModalOpen}
                    onClose={() => setSaveViewModalOpen(false)}
                    onConfirm={saveCurrentView}
                    initialLabel={`${exploreSettings.projectionIntent} ${exploreSettings.layoutMode}`}
                    description="Save the current Explore projection, layout, and aggregation controls as a sidebar shortcut."
                />

                {historyRecord && (
                    <HeapHistoryModal
                        history={historyRecord}
                        artifactTitle={expandedBlock?.projection.title ?? selectedPrimaryBlock?.projection.title}
                        currentBlock={blocks.find((candidate) => candidate.projection.artifactId === historyRecord.artifact_id) ?? expandedBlock ?? selectedPrimaryBlock ?? null}
                        allBlocks={blocks}
                        onClose={() => setHistoryRecord(null)}
                        onOpenArtifact={(artifactId) => {
                            const sourceTitle = expandedBlock?.projection.title ?? selectedPrimaryBlock?.projection.title ?? historyRecord.artifact_id;
                            setHistoryRecord(null);
                            setDetailNavigationTrail([
                                {
                                    artifactId: historyRecord.artifact_id,
                                    title: sourceTitle,
                                    relation: "history",
                                },
                            ]);
                            setExpandedBlockId(artifactId);
                        }}
                    />
                )}

                {/* Create FAB — hidden if blocks are selected */}
                {heapCreateFlowEnabled && selectionContext.selectedCount === 0 && (
                    <button
                        onClick={() => setCreatePanelOpen((open) => !open)}
                        className={`fixed bottom-8 right-8 z-50 flex items-center justify-center w-14 h-14 rounded-full shadow-2xl transition-all active:scale-90 ${
                            createPanelOpen
                                ? "bg-red-500/80 border border-red-400/50 text-white hover:bg-red-500 rotate-45"
                                : "bg-blue-600 border border-blue-500/50 text-white hover:bg-blue-500 hover:scale-110 shadow-blue-500/30"
                        }`}
                        title={createPanelOpen ? "Close Create Panel" : "Create New Block"}
                    >
                        <Plus className="w-7 h-7 stroke-3" />
                    </button>
                )}

                {selectionMessage && (
                    <div className="fixed top-8 left-1/2 -translate-x-1/2 z-100 bg-slate-800/80 backdrop-blur-xl border border-emerald-500/30 text-emerald-400 px-6 py-2 rounded-full shadow-2xl animate-in fade-in slide-in-from-top-4 duration-300 font-medium">
                        {selectionMessage}
                    </div>
                )}
                {/* Chat Panel */}
                <ChatPanel
                    key={activeSpaceId}
                    isOpen={chatPanelOpen}
                    onClose={() => setChatPanelOpen(false)}
                    contextBlockIds={selectionContext.selectedArtifactIds}
                    viewMode={viewMode}
                    heapViewContext={activeHeapViewContext}
                    threadId={chatThreadId ?? routeThreadId ?? undefined}
                    gatewayUrl={gatewayBaseUrl()}
                />
            </div>
        </div>
    );
}

function ExploreSettingsGroup({
    descriptor,
    source,
    value,
    onChange,
    controlKind,
}: {
    descriptor: ExploreViewSettingDescriptor;
    source: string;
    value: string | boolean;
    onChange: (value: string | boolean) => void;
    controlKind: ExploreSettingVisualKind;
}) {
    const title = descriptor.title;
    const options = descriptor.options;
    const description = descriptor.description;
    const isToggleOn = value === true || value === "true";
    const selectedIndex = Math.max(
        0,
        options.findIndex((option) => option.value === String(value)),
    );
    const selectedLabel = options[selectedIndex]?.label ?? options[0]?.label ?? String(value);

    return (
        <div className="rounded-2xl border border-white/8 bg-white/3 p-2.25">
            <div className="flex items-center justify-between gap-3">
                <div>
                    <div className="text-[9px] font-black uppercase tracking-[0.3em] text-cortex-500">{title}</div>
                    <div className="mt-1 text-[10px] leading-4 text-cortex-400/80">{description}</div>
                    <div className="mt-1 text-[9px] uppercase tracking-[0.22em] text-cortex-400">{source}</div>
                </div>
                {controlKind === "slider" && (
                    <div className="text-[10px] font-medium uppercase tracking-[0.18em] text-cortex-300">
                        {selectedLabel}
                    </div>
                )}
            </div>
            {controlKind === "slider" ? (
                <div className="mt-2.5 rounded-2xl border border-white/8 bg-black/15 px-3 py-2.5">
                    <input
                        type="range"
                        min={0}
                        max={Math.max(0, options.length - 1)}
                        step={1}
                        value={selectedIndex}
                        onChange={(event) => {
                            const nextIndex = Number(event.currentTarget.value);
                            const nextValue = options[nextIndex]?.value;
                            if (nextValue) {
                                onChange(nextValue);
                            }
                        }}
                        className="w-full accent-cyan-400"
                        aria-label={title}
                    />
                    <div className="mt-2 flex items-center justify-between text-[9px] uppercase tracking-[0.18em] text-cortex-400">
                        <span>{options[0]?.label ?? "Low"}</span>
                        <span>{options[options.length - 1]?.label ?? "High"}</span>
                    </div>
                </div>
            ) : controlKind === "toggle" ? (
                <button
                    type="button"
                    aria-pressed={isToggleOn}
                    onClick={() => onChange(!isToggleOn)}
                    className={`mt-2.5 flex w-full items-center justify-between rounded-2xl border px-3 py-2.5 text-left transition-colors ${
                        isToggleOn
                            ? "border-cyan-400/30 bg-cyan-500/10 text-cyan-100"
                            : "border-white/8 bg-white/3 text-cortex-300 hover:border-white/14 hover:bg-white/5"
                    }`}
                >
                    <span className="text-[10px] font-semibold uppercase tracking-[0.18em]">
                        {isToggleOn ? "On" : "Off"}
                    </span>
                    <span className={`relative h-5 w-9 rounded-full border transition-colors ${
                        isToggleOn
                            ? "border-cyan-400/40 bg-cyan-500/25"
                            : "border-white/10 bg-white/8"
                    }`}>
                        <span className={`absolute top-0.5 h-4 w-4 rounded-full bg-white shadow-sm transition-transform ${
                            isToggleOn ? "translate-x-4" : "translate-x-0.5"
                        }`} />
                    </span>
                </button>
            ) : (
                <div className={`mt-2.5 grid gap-1.5 ${options.length > 3 ? "grid-cols-2" : "grid-cols-3"}`}>
                    {options.map((option) => {
                        const isActive = value === option.value;
                        return (
                            <button
                                key={option.value}
                                type="button"
                                aria-pressed={isActive}
                                onClick={() => onChange(option.value)}
                                className={`min-h-8 rounded-xl border px-2.5 py-1.5 text-left transition-colors ${
                                    controlKind === "chips"
                                        ? `rounded-full px-2 py-1 ${
                                            isActive
                                                ? "border-cyan-400/30 bg-cyan-500/10 text-cyan-100"
                                                : "border-white/8 bg-white/3 text-cortex-300 hover:border-white/14 hover:bg-white/5"
                                        }`
                                        : isActive
                                            ? "border-cyan-400/30 bg-cyan-500/10 text-cyan-100"
                                            : "border-white/8 bg-white/3 text-cortex-300 hover:border-white/14 hover:bg-white/5"
                                }`}
                            >
                                <div className="text-[10px] font-semibold tracking-tight">{option.label}</div>
                            </button>
                        );
                    })}
                </div>
            )}
        </div>
    );
}

function describeExploreSettingOrigin(origin: "space" | "user" | "session"): string {
    switch (origin) {
        case "session":
            return "Session override";
        case "user":
            return "Personal override";
        default:
            return "Space default";
    }
}

function readExploreQueryParam<T extends string>(
    searchParams: URLSearchParams,
    key: string,
    allowedValues: readonly T[],
): T | undefined {
    const value = searchParams.get(key)?.trim();
    if (!value) {
        return undefined;
    }
    return (allowedValues as readonly string[]).includes(value) ? (value as T) : undefined;
}

function readExploreBooleanQueryParam(
    searchParams: URLSearchParams,
    key: string,
): boolean | undefined {
    const value = searchParams.get(key)?.trim().toLowerCase();
    if (!value) {
        return undefined;
    }
    if (value === "true" || value === "1" || value === "yes" || value === "on") {
        return true;
    }
    if (value === "false" || value === "0" || value === "no" || value === "off") {
        return false;
    }
    return undefined;
}

function normalizeVisualizationVariant(
    value?: ExploreVisualizationMode | string,
): ExploreVisualizationMode | undefined {
    if (value === "off" || value === "2d" || value === "3d") {
        return value;
    }
    return undefined;
}

function filterHeapAggregationGroupsByMode(
    groups: HeapAggregationGroup[],
    mode: ExploreAggregationMode,
): HeapAggregationGroup[] {
    switch (mode) {
        case "prompt_like":
            return groups.filter((group) => group.groupId === "prompt-like");
        case "steward_feedback":
            return groups.filter((group) => group.groupId === "steward-feedback");
        case "none":
            return [];
        default:
            return groups;
    }
}

function HeapHistoryModal({
    history,
    artifactTitle,
    currentBlock,
    allBlocks,
    onClose,
    onOpenArtifact,
}: {
    history: HeapBlockHistoryResponse;
    artifactTitle?: string;
    currentBlock: HeapBlockListItem | null;
    allBlocks: HeapBlockListItem[];
    onClose: () => void;
    onOpenArtifact: (artifactId: string) => void;
}) {
    const relationIndex = useMemo(
        () => (currentBlock ? buildHeapRelationIndex(currentBlock, allBlocks) : null),
        [allBlocks, currentBlock],
    );
    const routeLineage = useMemo(
        () => (currentBlock && relationIndex ? buildTaskRouteLineageSnapshot(currentBlock, relationIndex) : null),
        [currentBlock, relationIndex],
    );

    return (
        <div className="fixed inset-0 z-120 flex items-center justify-center bg-slate-950/75 px-4 py-8 backdrop-blur-md">
            <div className="max-h-[90vh] w-full max-w-4xl overflow-hidden rounded-[28px] border border-white/10 bg-[linear-gradient(180deg,rgba(8,20,40,0.96),rgba(4,10,24,0.96))] shadow-[0_32px_120px_-48px_rgba(0,0,0,0.8)]">
                <div className="flex items-center justify-between border-b border-white/8 px-6 py-5">
                    <div>
                        <div className="text-[11px] uppercase tracking-[0.28em] text-white/40">History</div>
                        <div className="mt-2 text-xl font-semibold text-white">{artifactTitle || history.artifact_id}</div>
                        <div className="mt-1 text-xs text-white/45">Artifact ID: {history.artifact_id}</div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="rounded-full border border-white/10 bg-white/4 p-2 text-white/70 transition hover:border-white/20 hover:bg-white/8 hover:text-white"
                    >
                        <X className="h-4 w-4" />
                    </button>
                </div>
                <div className="grid max-h-[calc(90vh-88px)] gap-6 overflow-y-auto p-6 lg:grid-cols-3">
                    <section className="rounded-2xl border border-white/8 bg-white/3 p-5 lg:col-span-1">
                        <div className="text-[10px] uppercase tracking-[0.24em] text-white/45">Audit Events</div>
                        <div className="mt-4 space-y-3">
                            {history.versions.length === 0 ? (
                                <p className="text-sm text-white/45">No audit events recorded yet.</p>
                            ) : history.versions.map((entry) => (
                                <div key={`${entry.version}-${entry.timestamp}`} className="rounded-xl border border-white/6 bg-slate-950/45 p-3">
                                    <div className="text-xs font-medium text-white/85">{entry.mutation_type}</div>
                                    <div className="mt-1 text-[11px] text-white/55">{entry.timestamp}</div>
                                    <div className="mt-2 text-[11px] text-white/45">Actor: {entry.actor}</div>
                                </div>
                            ))}
                        </div>
                    </section>
                    <section className="rounded-2xl border border-white/8 bg-white/3 p-5 lg:col-span-1">
                        <div className="text-[10px] uppercase tracking-[0.24em] text-white/45">Revisions</div>
                        <div className="mt-4 space-y-3">
                            {history.revisions?.length ? history.revisions.map((revision) => (
                                <div key={revision.revisionId} className="rounded-xl border border-white/6 bg-slate-950/45 p-3">
                                    <div className="text-xs font-medium text-white/85">Revision {revision.revisionNumber}</div>
                                    <div className="mt-1 text-[11px] text-white/55">{revision.createdAt}</div>
                                    <div className="mt-2 text-[11px] text-white/45">By: {revision.createdBy}</div>
                                    {revision.parentRevisionId && (
                                        <div className="mt-1 text-[11px] text-white/35">Parent: {revision.parentRevisionId}</div>
                                    )}
                                </div>
                            )) : (
                                <p className="text-sm text-white/45">No revision history recorded yet.</p>
                            )}
                        </div>
                    </section>
                    <section className="rounded-2xl border border-white/8 bg-white/3 p-5 lg:col-span-1">
                        <div className="text-[10px] uppercase tracking-[0.24em] text-white/45">Parser Runs</div>
                        <p className="mt-2 text-xs leading-5 text-white/45">
                            These are sibling extraction views on the same uploaded artifact, not artifact revisions.
                        </p>
                        <div className="mt-4 space-y-3">
                            {history.uploadExtractionRuns?.length ? history.uploadExtractionRuns.map((run) => (
                                <div key={run.job_id} className="rounded-xl border border-white/6 bg-slate-950/45 p-3">
                                    <div className="flex flex-wrap items-center gap-2">
                                        <span className="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-2 py-0.5 text-[10px] uppercase tracking-[0.16em] text-cyan-100/85">
                                            {parserProfileLabel(run.parser_backend || run.requested_parser_profile || "auto")}
                                        </span>
                                        <span className="rounded-full border border-white/8 bg-white/4 px-2 py-0.5 text-[10px] uppercase tracking-[0.16em] text-white/55">
                                            {run.status.replace(/_/g, " ")}
                                        </span>
                                    </div>
                                    <div className="mt-2 text-[11px] text-white/55">
                                        Requested {parserProfileLabel(run.requested_parser_profile || "auto")}
                                    </div>
                                    <div className="mt-1 text-[11px] text-white/45">
                                        {run.created_at || run.last_updated_at || "unknown time"}
                                    </div>
                                    <div className="mt-2 flex flex-wrap gap-3 text-[11px] text-white/55">
                                        {run.confidence !== undefined && <span>confidence {run.confidence.toFixed(2)}</span>}
                                        {run.page_count !== undefined && <span>pages {run.page_count}</span>}
                                        {run.block_count !== undefined && <span>blocks {run.block_count}</span>}
                                    </div>
                                    {run.flags?.length ? (
                                        <div className="mt-2 flex flex-wrap gap-2">
                                            {run.flags.map((flag) => (
                                                <span key={flag} className="rounded-full border border-white/8 bg-white/4 px-2 py-0.5 text-[10px] font-mono text-white/60">
                                                    {flag}
                                                </span>
                                            ))}
                                        </div>
                                    ) : null}
                                </div>
                            )) : (
                                <p className="text-sm text-white/45">No parser runs recorded yet.</p>
                            )}
                        </div>
                    </section>
                    <section className="rounded-2xl border border-white/8 bg-white/3 p-5 lg:col-span-1">
                        <div className="text-[10px] uppercase tracking-[0.24em] text-white/45">Route Lineage</div>
                        <p className="mt-2 text-xs leading-5 text-white/45">
                            This view reuses the stamped task snapshot and relation graph so the routed decision stays visible in history.
                        </p>
                        <div className="mt-4">
                            <TaskRouteLineageCard lineage={routeLineage} onOpenArtifact={onOpenArtifact} />
                        </div>
                    </section>
                    <section className="rounded-2xl border border-white/8 bg-white/3 p-5 lg:col-span-1">
                        <div className="text-[10px] uppercase tracking-[0.24em] text-white/45">Historical Links</div>
                        <p className="mt-2 text-xs leading-5 text-white/45">
                            These links show what this record points to, what points back to it, and how the records relate over time.
                        </p>
                        <div className="mt-4 space-y-3">
                            {history.lineage?.length ? history.lineage.map((relation) => (
                                <button
                                    key={`${relation.relation}-${relation.artifactId}`}
                                    type="button"
                                    onClick={() => onOpenArtifact(relation.artifactId)}
                                    className="w-full rounded-xl border border-white/6 bg-slate-950/45 p-3 text-left transition hover:border-cyan-300/25 hover:bg-cyan-300/8"
                                >
                                    <div className="flex flex-wrap gap-1.5">
                                        <span className="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-2 py-0.5 text-[10px] uppercase tracking-[0.16em] text-cyan-100/80">
                                            {describeHeapRelation(relation.relation)}
                                        </span>
                                        {relation.blockType && (
                                            <span className="rounded-full border border-white/8 bg-white/4 px-2 py-0.5 text-[10px] uppercase tracking-[0.16em] text-white/55">
                                                {relation.blockType}
                                            </span>
                                        )}
                                    </div>
                                    <div className="mt-1 text-sm font-medium text-white/90">{relation.title || relation.artifactId}</div>
                                    <div className="mt-1 text-[11px] text-white/45">{relation.blockType || relation.subtitle || relation.artifactId}</div>
                                </button>
                            )) : (
                                <p className="text-sm text-white/45">No historical links recorded yet.</p>
                            )}
                            {history.legacyDuplicates?.length ? (
                                <div className="rounded-xl border border-amber-300/15 bg-amber-300/8 p-3">
                                    <div className="text-[11px] uppercase tracking-[0.18em] text-amber-100/65">Legacy Prompt Groups</div>
                                    <div className="mt-2 space-y-2">
                                        {history.legacyDuplicates.map((group) => (
                                            <div key={`${group.title}-${group.contentHash || "none"}`} className="text-[11px] text-white/70">
                                                <div className="font-medium text-white/85">{group.title}</div>
                                                <div className="mt-1 text-white/45">{group.artifactIds.join(", ")}</div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            ) : null}
                        </div>
                    </section>
                </div>
            </div>
        </div>
    );
}

function RelationalOverlay({ hoveredBlockId, blocks }: { hoveredBlockId: string; blocks: HeapBlockListItem[] }) {
    const [lines, setLines] = useState<Array<{ x1: number; y1: number; x2: number; y2: number }>>([]);
    const hoveredBlock = blocks.find(b => b.projection.artifactId === hoveredBlockId);

    useEffect(() => {
        if (!hoveredBlock) return;

        const mentions = [
            ...(hoveredBlock.projection.mentionsInline || []),
            ...(hoveredBlock.projection.pageLinks || [])
        ];

        const sourceEl = document.getElementById(`card-${hoveredBlockId}`);
        if (!sourceEl) return;

        const containerEl = sourceEl.closest(".heap-lane-board"); // Get specific grid container
        if (!containerEl) return;

        const containerRect = containerEl.getBoundingClientRect();
        const sourceRect = sourceEl.getBoundingClientRect();

        const sourceX = sourceRect.left + sourceRect.width / 2 - containerRect.left;
        const sourceY = sourceRect.top + sourceRect.height / 2 - containerRect.top;

        const nextLines = mentions.map(mId => {
            const targetEl = document.getElementById(`card-${mId}`);
            if (!targetEl) return null;

            const targetRect = targetEl.getBoundingClientRect();
            return {
                x1: sourceX,
                y1: sourceY,
                x2: targetRect.left + targetRect.width / 2 - containerRect.left,
                y2: targetRect.top + targetRect.height / 2 - containerRect.top
            };
        }).filter(Boolean) as typeof lines;

        setLines(nextLines);
    }, [hoveredBlockId, blocks, hoveredBlock]);

    if (lines.length === 0) return null;

    return (
        <svg
            className="absolute inset-0 pointer-events-none z-0 overflow-visible"
            style={{ width: '100%', height: '100%' }}
        >
            <defs>
                <linearGradient id="line-gradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="#3b82f6" stopOpacity="0.6" />
                    <stop offset="100%" stopColor="#8b5cf6" stopOpacity="0.3" />
                </linearGradient>
            </defs>
            {lines.map((l, i) => {
                const dx = l.x2 - l.x1;
                const dy = l.y2 - l.y1;
                const path = `M ${l.x1} ${l.y1} C ${l.x1 + dx / 2} ${l.y1}, ${l.x1 + dx / 2} ${l.y2}, ${l.x2} ${l.y2}`;
                return (
                    <path
                        key={i}
                        d={path}
                        stroke="url(#line-gradient)"
                        strokeWidth="2"
                        fill="none"
                        strokeDasharray="4 4"
                        className="animate-pulse"
                    />
                );
            })}
        </svg>
    );
}

function resolveHeapDeltaPollingEnabled(envValue: string | undefined): boolean {
    if (!envValue) return false;
    if (envValue.toLowerCase() === "true") {
        return true;
    }
    if (typeof window === "undefined") {
        return false;
    }
    try {
        return window.localStorage.getItem(HEAP_DELTA_POLLING_ENABLED_KEY) === "1";
    } catch {
        return false;
    }
}

function clampHeapDeltaPollingIntervalMs(intervalMs: number): number {
    return Math.min(120000, Math.max(500, Math.trunc(intervalMs)));
}

function resolveHeapDeltaPollingIntervalMs(envValue?: string): number {
    const envNumber = envValue ? Number(envValue) : Number.NaN;
    if (Number.isFinite(envNumber) && envNumber > 0) {
        return clampHeapDeltaPollingIntervalMs(envNumber);
    }
    if (typeof window === "undefined") {
        return 15000;
    }
    try {
        const raw = window.localStorage.getItem(HEAP_DELTA_POLLING_INTERVAL_MS_KEY);
        const parsed = raw ? Number(raw) : Number.NaN;
        if (Number.isFinite(parsed) && parsed > 0) {
            return clampHeapDeltaPollingIntervalMs(parsed);
        }
    } catch {
        // Ignore localStorage access failures and use default.
    }
    return 15000;
}

function extractBehaviors(block: HeapBlockListItem): string[] {
    const surface = block.surfaceJson as Record<string, unknown> | undefined;
    return (surface?.behaviors as string[]) || [];
}

function tokenizeQuery(input: string): string[] {
    return input
        .split(/[\s,]+/)
        .map((token) => token.trim().toLowerCase())
        .filter(Boolean);
}

function blockSearchCorpus(block: HeapBlockListItem): string {
    const surface = block.surfaceJson as Record<string, unknown>;
    const behaviors = extractBehaviors(block);
    const parts = [
        block.projection.title,
        block.projection.blockType,
        (block.projection.tags || []).join(" "),
        (block.projection.mentionsInline || []).join(" "),
        (block.projection.pageLinks || []).join(" "),
        behaviors.join(" "),
        JSON.stringify(surface)
    ];
    return parts.join(" ").toLowerCase();
}

function sortHeapBlocks(blocks: HeapBlockListItem[]): HeapBlockListItem[] {
    return [...blocks].sort((left, right) => {
        const leftUpdatedAt = left.projection.updatedAt;
        const rightUpdatedAt = right.projection.updatedAt;
        return rightUpdatedAt.localeCompare(leftUpdatedAt) || right.projection.artifactId.localeCompare(left.projection.artifactId);
    });
}

function reconcileHeapDelta(
    current: HeapBlockListItem[],
    changed: HeapBlockListItem[],
    deleted: Array<{ artifactId: string; deletedAt: string }>
): HeapBlockListItem[] {
    const byArtifact = new Map<string, HeapBlockListItem>();
    for (const block of current) {
        byArtifact.set(block.projection.artifactId, block);
    }
    for (const block of changed) {
        byArtifact.set(block.projection.artifactId, block);
    }
    for (const tombstone of deleted) {
        byArtifact.delete(tombstone.artifactId);
    }
    return sortHeapBlocks(Array.from(byArtifact.values()));
}

function pickNewestTimestamp(
    changed: HeapBlockListItem[],
    deleted: Array<{ artifactId: string; deletedAt: string }>,
    fallback: string | null
): string | null {
    let newest = fallback;
    for (const block of changed) {
        if (!newest || block.projection.updatedAt > newest) {
            newest = block.projection.updatedAt;
        }
    }
    for (const tombstone of deleted) {
        if (!newest || tombstone.deletedAt > newest) {
            newest = tombstone.deletedAt;
        }
    }
    return newest;
}

function downloadJson(filename: string, payload: unknown): void {
    const text = typeof payload === "string" ? payload : JSON.stringify(payload, null, 2);
    const blob = new Blob([text], { type: "application/json;charset=utf-8" });
    const href = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = href;
    anchor.download = filename;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    URL.revokeObjectURL(href);
}

function resolveSpaceId(candidate?: string): string {
    const normalized = candidate?.trim();
    if (normalized) return normalized;
    return resolveWorkbenchSpaceId();
}

function formatJsonWithHighlighting(json: any): React.ReactNode {
    const str = JSON.stringify(json, null, 2);
    if (!str) return null;

    return str.split("\n").map((line, i) => {
        const parts = line.split(/(".*?"|:|\d+)/);
        return (
            <div key={i} className="whitespace-pre">
                {parts.map((part, j) => {
                    if (part.startsWith('"') && part.endsWith('"')) {
                        const isKey = line.includes(`${part}:`);
                        return <span key={j} className={isKey ? "text-purple-400 font-semibold" : "text-emerald-400"}>{part}</span>;
                    }
                    if (part === ":") return <span key={j} className="text-slate-500 mr-1">:</span>;
                    if (/^\d+$/.test(part)) return <span key={j} className="text-orange-400">{part}</span>;
                    return <span key={j} className="text-slate-400">{part}</span>;
                })}
            </div>
        );
    });
}
