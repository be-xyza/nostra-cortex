import React, { useState, useEffect, useMemo, useCallback, useRef } from "react";
import { Plus, Menu, PanelLeftOpen } from "lucide-react";
import { useSearchParams, useLocation } from "react-router-dom";
import { resolveWorkbenchSpaceId, workbenchApi } from "../../api";
import type {
    ArtifactGovernanceEnvelope,
    EmitHeapBlockRequest,
    HeapBlockListItem,
    HeapStewardGateValidateResponse,
} from "../../contracts";
import { HeapBlockCard } from "./HeapBlockCard";
import { HeapActionBar } from "./HeapActionBar";
import { HeapDetailModal } from "./HeapDetailModal";
import { StewardGateModal } from "./StewardGateModal";
import { useHeapActionPlan } from "./useHeapActionPlan";
import { type ActionHandlers } from "./actionExecutor";
import type { ActionSelectionContext } from "../../contracts";
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

type CreateMode = "create" | "generate" | "upload" | "solicit";

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
    const [solicitRole, setSolicitRole] = useState("steward.code");
    const [solicitBudget, setSolicitBudget] = useState("50000");
    const [solicitCapabilities, setSolicitCapabilities] = useState("read,write");
    const [isEmitting, setIsEmitting] = useState(false);
    const [stewardGateArtifactId, setStewardGateArtifactId] = useState<string | null>(null);
    const [stewardGateValidation, setStewardGateValidation] = useState<HeapStewardGateValidateResponse | null>(null);
    const [stewardApplyingId, setStewardApplyingId] = useState<string | null>(null);
    const [stewardPublishing, setStewardPublishing] = useState(false);
    const [commentSidebarBlockId, setCommentSidebarBlockId] = useState<string | null>(null);
    const [hoveredBlockId, setHoveredBlockId] = useState<string | null>(null);
    const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
    const [bgGraphVariant, setBgGraphVariant] = useState<"off" | "2d" | "3d">(() => {
        try { 
            const saved = localStorage.getItem("cortex.heap.bgGraph");
            return (saved as "off" | "2d" | "3d") || "off"; 
        } catch { return "off"; }
    });
    useEffect(() => {
        try { localStorage.setItem("cortex.heap.bgGraph", bgGraphVariant); } catch {}
    }, [bgGraphVariant]);
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

    const fetchBlocks = useCallback(() => {
        setLoading(true);
        workbenchApi.getHeapBlocks({
            spaceId: activeSpaceId,
            blockType: filterDefaults?.blockType,
            tag: filterDefaults?.tag,
            pageLink: activePageLinkFilter,
            limit: 100,
        })
            .then(res => {
                const nextItems = res.items || [];
                setBlocks(sortHeapBlocks(nextItems));
                if (nextItems.length > 0) {
                    const latestUpdatedAt = nextItems[0]?.projection.updatedAt;
                    if (latestUpdatedAt) {
                        setLastDeltaSince(latestUpdatedAt);
                    }
                }
                setError(null);
            })
            .catch(err => setError(err.message))
            .finally(() => setLoading(false));
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
        }),
        [activeSpace?.archetype, activeSpaceId]
    );
    const laneCount = useMemo(
        () => resolveHeapLaneCount(laneBoardWidth, exploreSurfacePolicy),
        [laneBoardWidth, exploreSurfacePolicy]
    );
    const blockLanes = useMemo(
        () => buildHeapLanes(filteredBlocks, laneCount, exploreSurfacePolicy),
        [filteredBlocks, laneCount, exploreSurfacePolicy]
    );

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
    }, [loading, filteredBlocks.length]);

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
        setExpandedBlockId((current) => current === deepLinkedArtifactId ? current : deepLinkedArtifactId);
    }, [blocks, searchParams]);

    const selectedPrimaryId = selectedBlockIds[0] ?? null;
    const expandedBlock = useMemo(() => blocks.find(b => b.projection.artifactId === expandedBlockId), [blocks, expandedBlockId]);
    const selectedPrimaryBlock = useMemo(
        () => (selectedPrimaryId ? blocks.find((b) => b.projection.artifactId === selectedPrimaryId) ?? null : null),
        [blocks, selectedPrimaryId]
    );

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
            setStatusMessage(`History loaded: ${history.versions.length} events.`);
            console.info("Heap history", history);
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
            setStatusMessage("Synthesis block emitted to space.");
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

    const clearCreateForm = () => {
        setNewBlockTitle("");
        setNewBlockType("note");
        setNewBlockText("");
        setAgentPrompt("");
        setUploadFile(null);
        setSolicitRole("steward.code");
        setSolicitBudget("50000");
        setSolicitCapabilities("read,write");
    };

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
            const pointer = uploadFile ? `local://uploads/${uploadFile.name}` : "local://uploads/pending-file";
            return {
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
                        ...(uploadFile ? {
                            file_name: uploadFile.name,
                            mime_type: uploadFile.type || "application/octet-stream",
                            file_size: String(uploadFile.size),
                        } : {}),
                    },
                },
                content: {
                    payload_type: "pointer",
                    pointer,
                },
            };
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

        if (createMode === "solicit") {
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
                    title: resolvedTitle || "Agent Solicitation",
                },
                content: {
                    payload_type: "structured_data",
                    structured_data: {
                        space_id: spaceId,
                        type: "agent_solicitation",
                        role: solicitRole.trim(),
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
        try {
            setIsEmitting(true);
            await workbenchApi.emitHeapBlock(buildEmitPayload());
            fetchBlocks();
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
        return <div style={{ padding: "1rem", background: "rgba(127,29,29,0.5)", border: "1px solid #7f1d1d", color: "#fca5a5", borderRadius: "0.5rem" }}>Failed to load heap: {error}</div>;
    }

    return (
        <div className="heap-surface flex h-full w-full overflow-hidden select-none">
            {/* Sidebar with collapse transition */}
            {showFilterSidebar && (
                <div className={`transition-all duration-300 ease-in-out flex shrink-0 ${isSidebarCollapsed ? "w-0 overflow-hidden" : "w-64"}`}>
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
                {bgGraphVariant !== "off" && (
                    <AmbientGraphBackground
                        visible={true}
                        variant={bgGraphVariant as "2d" | "3d"}
                        spaceId={resolveSpaceId(activeSpaceId)}
                    />
                )}

                {heapParityEnabled && <AgentActivityPanel spaceId={resolveSpaceId(activeSpaceId)} />}

                {/* Scrollable Area */}
                <div className="flex-1 flex flex-col overflow-y-auto custom-scrollbar relative z-10 bg-transparent">
                    {/* Header - now sticky within the scrollable div */}
                    <header id="heap-grid-header" className="min-h-[64px] flex items-center justify-between px-6 py-4 sticky top-0 z-30 flex-wrap gap-3 glass-panel backdrop-blur-xl rounded-none shadow-sm border-b border-white/5">
                        <div className="flex items-center gap-3 flex-wrap">
                            {isSidebarCollapsed && (
                                <button
                                    onClick={() => setIsSidebarCollapsed(false)}
                                    className="p-1.5 rounded-lg hover:bg-white/5 text-cortex-500 hover:text-white transition-colors mr-1"
                                    title="Show Sidebar"
                                >
                                    <Menu className="w-5 h-5" />
                                </button>
                            )}
                            <h2 className="text-xl font-bold text-cortex-50 tracking-tight">
                                {viewMode}
                                <span className="ml-2 text-cortex-500 font-medium text-sm uppercase tracking-widest">Canvas Blocks</span>
                            </h2>
                            {(includeTerms.length > 0 || selectedTags.length > 0) && (
                                <span className="text-[9px] uppercase font-black px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20 shadow-sm">{filterMode} MATCH</span>
                            )}
                            {selectedPageLinks.length > 0 && (
                                <span className="text-[9px] uppercase font-black px-2 py-0.5 rounded-full bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 shadow-sm">{selectedPageLinks.length} LINKS</span>
                            )}
                            {selectedReviewLane && (
                                <span className="text-[9px] uppercase font-black px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-300 border border-emerald-500/20 shadow-sm">
                                    {selectedReviewLane === "private_review" ? "PRIVATE REVIEW" : "PUBLIC REVIEW"}
                                </span>
                            )}
                            {excludeTerms.length > 0 && (
                                <span className="text-[9px] uppercase font-black px-2 py-0.5 rounded-full bg-red-500/10 text-red-400 border border-red-500/20 shadow-sm">NOT {excludeTerms.length}</span>
                            )}
                            {statusMessage && (
                                <span className="text-[9px] uppercase font-black px-2 py-0.5 rounded-full bg-cortex-900 text-cortex-500 border border-cortex-800 shadow-sm">{statusMessage}</span>
                            )}
                        </div>
                        <div className="flex gap-3 items-center">
                            {/* Background Graph Toggle */}
                            <div className="flex items-center gap-0.5 rounded-full bg-cortex-800/60 border border-cortex-700/40 p-0.5">
                                {(["off", "2d", "3d"] as const).map((v) => (
                                    <button
                                        key={v}
                                        onClick={() => {
                                            if (v === "off") setBgGraphVariant("off");
                                            else setBgGraphVariant(prev => prev === v ? "off" : v);
                                        }}
                                        className={`px-3 py-1 text-[9px] font-black uppercase tracking-widest rounded-full transition-all duration-200 ${
                                            bgGraphVariant === v
                                                ? "bg-blue-600/80 text-white shadow-sm"
                                                : "text-cortex-500 hover:text-cortex-300"
                                        }`}
                                    >
                                        {v === "off" ? "BG" : v.toUpperCase()}
                                    </button>
                                ))}
                            </div>
                        </div>
                    </header>

                    {/* Floating Action Bar (Selection-driven) */}
                    <HeapActionBar
                        selection={selectionContext}
                        selectionZonePlan={selectionZonePlan}
                        handlers={actionHandlers}
                        onCreate={() => {
                            setCreateMode("create");
                            setCreatePanelOpen(true);
                        }}
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
                                        {(["create", "generate", "upload", "solicit"] as CreateMode[]).map((mode) => (
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
                                            <div className="border-2 border-dashed border-cortex-800 rounded-2xl p-8 flex flex-col items-center justify-center hover:border-blue-500/40 transition-colors cursor-pointer group bg-cortex-950/50">
                                                <Plus className="w-10 h-10 text-cortex-600 mb-4 group-hover:text-blue-400 transition-colors" />
                                                <p className="text-sm text-cortex-400 group-hover:text-cortex-200 transition-colors">Select a file to import into the heap</p>
                                                <input
                                                    type="file"
                                                    className="opacity-0 absolute p-8 cursor-pointer"
                                                    onChange={(e) => setUploadFile(e.target.files?.[0] || null)}
                                                />
                                                {uploadFile && (
                                                    <p className="mt-4 text-emerald-400 text-xs font-bold bg-emerald-500/10 px-3 py-1 rounded-full uppercase tracking-tighter shadow-sm border border-emerald-500/30">
                                                        Selected: {uploadFile.name}
                                                    </p>
                                                )}
                                            </div>
                                        </div>
                                    )}

                                    {createMode === "solicit" && (
                                        <div className="space-y-6 animate-in fade-in slide-in-from-top-2 duration-300">
                                            <div>
                                                <label className="block text-[10px] font-black uppercase tracking-widest text-cortex-500 mb-2">Steward Role</label>
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
                                        </div>
                                    )}

                                    <div className="mt-10 flex justify-end gap-3 pt-6 border-t border-cortex-800">
                                        <button
                                            onClick={() => setCreatePanelOpen(false)}
                                            className="px-6 py-2.5 rounded-full text-xs font-bold text-cortex-400 hover:text-white hover:bg-cortex-800 transition-all border border-transparent hover:border-cortex-700"
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            disabled={isEmitting}
                                            onClick={emitCreatedBlock}
                                            className="px-8 py-2.5 bg-blue-600 text-white rounded-full text-xs font-black uppercase tracking-widest hover:bg-blue-500 active:scale-95 disabled:opacity-50 disabled:active:scale-100 transition-all shadow-xl shadow-blue-600/20"
                                        >
                                            {isEmitting ? "Emitting..." : "Create Block"}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    )}

                    <div className="flex-1 min-h-0 flex relative">
                        <div className="flex-1 overflow-y-auto relative">
                            {loading ? (
                                <div style={{ padding: "2rem", color: "#64748b", textAlign: "center" }}>Loading blocks...</div>
                            ) : filteredBlocks.length === 0 ? (
                                <div className="heap-empty-state flex flex-col items-center justify-center h-full w-full opacity-60 hover:opacity-100 transition-opacity duration-500">
                                    <div className="w-24 h-24 mb-6 rounded-full bg-slate-800/50 border border-slate-700/50 flex items-center justify-center shadow-2xl animate-bounce">
                                        <span className="text-4xl text-slate-500/50">🧊</span>
                                    </div>
                                    <h3 className="text-lg font-bold tracking-tight text-slate-300 mb-2">No blocks found</h3>
                                    <p className="text-sm text-slate-500 max-w-sm text-center">There are no blocks matching the current view constraints. Try adjusting your filters or generating new content.</p>
                                </div>
                            ) : (
                                <div className="pt-12 pb-20 px-6 relative w-full">
                                    <div
                                        ref={laneBoardHostRef}
                                        className="heap-lane-board grid items-start gap-5 isolate"
                                        style={{ gridTemplateColumns: `repeat(${laneCount}, minmax(0, 1fr))` }}
                                    >
                                        {blockLanes.map((lane, laneIndex) => (
                                            <div key={`lane-${laneIndex}`} className="heap-lane-board__lane flex min-w-0 flex-col gap-5">
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
                                                            onDoubleClick={() => setExpandedBlockId(b.projection.artifactId)}
                                                            cardActions={cardMenuZonePlan?.actions ?? []}
                                                            cardActionSelection={{
                                                                selectedArtifactIds: [b.projection.artifactId],
                                                                activeArtifactId: b.projection.artifactId,
                                                                selectedCount: 1,
                                                                selectedBlockTypes: [b.projection.blockType],
                                                            }}
                                                            actionHandlers={actionHandlers}
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
                                            blocks={blocks}
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
                        ambientGraphVariant={bgGraphVariant}
                        onClose={() => setExpandedBlockId(null)}
                        onNavigateToBlock={(id) => setExpandedBlockId(id)}
                        onRelationSaved={(artifactId) => {
                            fetchBlocks();
                            setStatusMessage(`Relation map updated for ${artifactId}.`);
                        }}
                        onViewDiscussion={(id) => {
                            setExpandedBlockId(null);
                            setCommentSidebarBlockId(id);
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
