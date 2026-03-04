import React, { useState, useEffect, useMemo, useCallback } from "react";
import { workbenchApi } from "../../api";
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
import { HeapFilterSidebar, HeapFilterMode, HeapViewMode } from "./HeapFilterSidebar";
import "./heap.css";

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
const DEFAULT_WORKSPACE_ID = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
const HEAP_DELTA_POLLING_ENABLED_KEY = "cortex.heap.deltaPolling";
const HEAP_DELTA_POLLING_INTERVAL_MS_KEY = "cortex.heap.deltaPollingIntervalMs";

type CreateMode = "create" | "generate" | "upload";

export function HeapBlockGrid({ filterDefaults, showFilterSidebar = false }: HeapBlockGridProps) {
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
    const [viewMode, setViewMode] = useState<HeapViewMode>("All");
    const [filterMode, setFilterMode] = useState<HeapFilterMode>("AND");
    const [filterTerm, setFilterTerm] = useState("");
    const [excludeTerm, setExcludeTerm] = useState("");
    const [selectedTags, setSelectedTags] = useState<string[]>([]);
    const [selectedPageLinks, setSelectedPageLinks] = useState<string[]>([]);
    const [pageLinkTerm, setPageLinkTerm] = useState("");
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
    const [isEmitting, setIsEmitting] = useState(false);
    const [stewardGateArtifactId, setStewardGateArtifactId] = useState<string | null>(null);
    const [stewardGateValidation, setStewardGateValidation] = useState<HeapStewardGateValidateResponse | null>(null);
    const [stewardApplyingId, setStewardApplyingId] = useState<string | null>(null);
    const [stewardPublishing, setStewardPublishing] = useState(false);
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
            spaceId: filterDefaults?.spaceId,
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
    }, [filterDefaults?.spaceId, filterDefaults?.blockType, filterDefaults?.tag, activePageLinkFilter]);

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
            }
        };
        window.addEventListener("keydown", onShortcut);
        return () => window.removeEventListener("keydown", onShortcut);
    }, []);

    // Derive view counts
    const blockCounts = useMemo(() => {
        const counts: Record<HeapViewMode, number> = { All: 0, Pinned: 0, Urgent: 0, Unlinked: 0 };
        for (const b of blocks) {
            counts.All++;
            const behaviors = extractBehaviors(b);
            if (behaviors.includes("pinned") || b.pinnedAt) counts.Pinned++;
            if (behaviors.includes("urgent")) counts.Urgent++;
            if (!b.projection.mentionsInline?.length) counts.Unlinked++;
        }
        return counts;
    }, [blocks]);

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

    const includeTerms = useMemo(() => tokenizeQuery(filterTerm), [filterTerm]);
    const excludeTerms = useMemo(() => tokenizeQuery(excludeTerm), [excludeTerm]);

    // Apply filters
    const filteredBlocks = useMemo(() => {
        return blocks.filter(b => {
            const behaviors = extractBehaviors(b);
            if (viewMode === "Pinned" && !(behaviors.includes("pinned") || !!b.pinnedAt)) return false;
            if (viewMode === "Urgent" && !behaviors.includes("urgent")) return false;
            if (viewMode === "Unlinked" && !!b.projection.mentionsInline?.length) return false;

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
    }, [blocks, viewMode, includeTerms, excludeTerms, selectedTags, selectedPageLinks, pageLinkTerm, filterMode]);

    useEffect(() => {
        if (selectedBlockIds.length === 0) return;
        const visible = new Set(blocks.map((b) => b.projection.artifactId));
        setSelectedBlockIds((current) => current.filter((id) => visible.has(id)));
    }, [blocks, selectedBlockIds.length]);

    const selectedPrimaryId = selectedBlockIds[0] ?? null;
    const expandedBlock = useMemo(() => blocks.find(b => b.projection.artifactId === expandedBlockId), [blocks, expandedBlockId]);
    const selectedPrimaryBlock = useMemo(
        () => (selectedPrimaryId ? blocks.find((b) => b.projection.artifactId === selectedPrimaryId) ?? null : null),
        [blocks, selectedPrimaryId]
    );

    const handleSelection = (blockId: string, event: React.MouseEvent<HTMLDivElement>) => {
        const toggleSelection = heapParityEnabled && (multiSelectEnabled || event.metaKey || event.ctrlKey);
        if (!toggleSelection) {
            setSelectedBlockIds([blockId]);
            return;
        }
        setSelectedBlockIds((current) => {
            if (current.includes(blockId)) {
                return current.filter((item) => item !== blockId);
            }
            return [...current, blockId];
        });
    };

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

    const handleRegenerate = () => {
        if (!selectedPrimaryId) return;
        setRegeneratingId(selectedPrimaryId);
        setTimeout(() => setRegeneratingId(null), 1500);
        setStatusMessage("Regeneration requested (UI simulation).");
    };

    const handleContextBundle = async () => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled by VITE_HEAP_PARITY_ENABLED.");
            return;
        }
        if (selectedBlockIds.length === 0) return;
        try {
            const bundle = await workbenchApi.createHeapContextBundle(selectedBlockIds);
            setStatusMessage(`Context bundle prepared: ${bundle.context_bundle.block_count} blocks.`);
            console.info("Heap context bundle", bundle);
        } catch (err) {
            setStatusMessage(`Context bundle failed: ${err instanceof Error ? err.message : String(err)}`);
        }
    };

    const handleExport = async () => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled by VITE_HEAP_PARITY_ENABLED.");
            return;
        }
        if (selectedBlockIds.length === 0) return;
        try {
            if (selectedBlockIds.length === 1) {
                const exportPayload = await workbenchApi.getHeapBlockExport(selectedBlockIds[0], "json");
                downloadJson(`heap-block-${selectedBlockIds[0]}.json`, exportPayload);
                setStatusMessage("Single block export downloaded.");
                return;
            }
            const bundle = await workbenchApi.createHeapContextBundle(selectedBlockIds);
            downloadJson(`heap-context-${Date.now()}.json`, bundle);
            setStatusMessage(`Bundle export downloaded (${selectedBlockIds.length} blocks).`);
        } catch (err) {
            setStatusMessage(`Export failed: ${err instanceof Error ? err.message : String(err)}`);
        }
    };

    const handleHistory = async () => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled by VITE_HEAP_PARITY_ENABLED.");
            return;
        }
        if (selectedBlockIds.length !== 1) {
            setStatusMessage("History requires exactly one selected block.");
            return;
        }
        try {
            const history = await workbenchApi.getHeapBlockHistory(selectedBlockIds[0]);
            setStatusMessage(`History loaded: ${history.versions.length} events.`);
            console.info("Heap history", history);
        } catch (err) {
            setStatusMessage(`History failed: ${err instanceof Error ? err.message : String(err)}`);
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

    const handlePublish = async () => {
        if (!heapParityEnabled) {
            setStatusMessage("Heap parity features are disabled by VITE_HEAP_PARITY_ENABLED.");
            return;
        }
        if (selectedBlockIds.length !== 1) {
            setStatusMessage("Publish requires exactly one selected block.");
            return;
        }
        const artifactId = selectedBlockIds[0];
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
    };

    const buildEmitPayload = (): EmitHeapBlockRequest => {
        const workspaceId = resolveWorkspaceId(filterDefaults?.spaceId);
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
                workspace_id: workspaceId,
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
                workspace_id: workspaceId,
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

        return {
            schema_version: "1.0.0",
            mode: "heap",
            workspace_id: workspaceId,
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
                    spaceId: filterDefaults?.spaceId,
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
        filterDefaults?.spaceId,
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
        <div className="heap-surface" style={{ display: "flex", height: "100%", backgroundColor: "#0a0f1c" }}>
            {/* Sidebar */}
            {showFilterSidebar && (
                <HeapFilterSidebar
                    viewMode={viewMode}
                    onViewModeChange={setViewMode}
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
                    blockCounts={blockCounts}
                    multiSelectEnabled={multiSelectEnabled}
                    onToggleMultiSelect={() => setMultiSelectEnabled((value) => !value)}
                    searchInputId={SEARCH_INPUT_ID}
                    heapParityEnabled={heapParityEnabled}
                />
            )}

            {/* Main Content */}
            <main style={{ flex: 1, display: "flex", flexDirection: "column", position: "relative" }}>
                {/* Header */}
                <header className="heap-surface__glass-panel" style={{ minHeight: "64px", borderBottom: "1px solid var(--cortex-800)", display: "flex", alignItems: "center", justifyContent: "space-between", padding: "0.75rem 1.5rem", position: "sticky", top: 0, zIndex: 10, flexWrap: "wrap", gap: "0.5rem" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: "0.75rem", flexWrap: "wrap" }}>
                        <h2 style={{ fontSize: "1.125rem", fontWeight: 600, color: "#f1f5f9" }}>
                            {viewMode} Blocks
                        </h2>
                        {(includeTerms.length > 0 || selectedTags.length > 0) && (
                            <span className="heap-badge heap-badge--outline heap-badge--blue">{filterMode} match</span>
                        )}
                        {selectedPageLinks.length > 0 && (
                            <span className="heap-badge heap-badge--outline heap-badge--blue">pageLink {selectedPageLinks.length}</span>
                        )}
                        {excludeTerms.length > 0 && (
                            <span className="heap-badge heap-badge--outline heap-badge--red">NOT {excludeTerms.length}</span>
                        )}
                        {filterDefaults?.blockType && (
                            <span className="heap-badge heap-badge--outline heap-badge--purple">type: {filterDefaults.blockType}</span>
                        )}
                        {statusMessage && (
                            <span className="heap-badge heap-badge--outline heap-badge--slate">{statusMessage}</span>
                        )}
                    </div>
                    <div style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
                        {isDevMode && (
                            <div
                                style={{
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "0.35rem",
                                    border: "1px solid var(--cortex-800)",
                                    borderRadius: "0.45rem",
                                    padding: "0.2rem 0.35rem",
                                    background: "rgba(15, 23, 42, 0.55)",
                                }}
                                title={
                                    heapDeltaPollingControlsLocked
                                        ? "Delta polling is locked by VITE_HEAP_CHANGED_BLOCKS_POLLING_ENABLED=true"
                                        : "Dev-only delta polling control"
                                }
                            >
                                <span style={{ fontSize: "0.72rem", color: "#94a3b8", fontWeight: 600 }}>Delta Poll</span>
                                <button
                                    className="heap-filter-sidebar__tag-btn"
                                    onClick={() => persistHeapDeltaPollingEnabled(!heapChangedBlocksPollingEnabled)}
                                    disabled={heapDeltaPollingControlsLocked}
                                    aria-label="Toggle heap delta polling"
                                    style={heapDeltaPollingControlsLocked ? { opacity: 0.55, cursor: "not-allowed" } : undefined}
                                >
                                    {effectiveHeapChangedBlocksPollingEnabled ? "On" : "Off"}
                                </button>
                                <input
                                    type="number"
                                    min={500}
                                    step={100}
                                    value={heapChangedBlocksPollingIntervalInput}
                                    onChange={(event) => setHeapChangedBlocksPollingIntervalInput(event.target.value)}
                                    onKeyDown={(event) => {
                                        if (event.key === "Enter") {
                                            commitHeapDeltaPollingIntervalInput();
                                        }
                                    }}
                                    disabled={heapDeltaPollingControlsLocked}
                                    aria-label="Heap delta polling interval in milliseconds"
                                    style={{
                                        width: "4.75rem",
                                        borderRadius: "0.3rem",
                                        border: "1px solid var(--cortex-800)",
                                        background: "rgba(15, 23, 42, 0.8)",
                                        color: "#cbd5e1",
                                        fontSize: "0.75rem",
                                        padding: "0.2rem 0.35rem",
                                        opacity: heapDeltaPollingControlsLocked ? 0.55 : 1,
                                    }}
                                />
                                <button
                                    className="heap-filter-sidebar__tag-btn"
                                    onClick={commitHeapDeltaPollingIntervalInput}
                                    disabled={heapDeltaPollingControlsLocked}
                                    aria-label="Apply heap delta polling interval"
                                    style={heapDeltaPollingControlsLocked ? { opacity: 0.55, cursor: "not-allowed" } : undefined}
                                >
                                    Apply
                                </button>
                            </div>
                        )}
                        <button
                            onClick={() => setSelectedBlockIds([])}
                            className="heap-filter-sidebar__tag-btn"
                        >
                            Clear Select
                        </button>
                        {heapCreateFlowEnabled && (
                            <button
                                onClick={() => setCreatePanelOpen((open) => !open)}
                                style={{
                                    background: "var(--cortex-accent)", color: "#fff", padding: "6px 12px", borderRadius: "0.375rem",
                                    fontSize: "0.875rem", fontWeight: 600, border: "none", cursor: "pointer", display: "flex", alignItems: "center", gap: "0.45rem",
                                    boxShadow: "0 4px 14px rgba(59,130,246,0.25)"
                                }}
                                aria-label="Open create block panel"
                            >
                                + Create
                            </button>
                        )}
                    </div>
                </header>

                {heapCreateFlowEnabled && createPanelOpen && (
                    <section style={{ borderBottom: "1px solid var(--cortex-800)", background: "rgba(7, 11, 20, 0.7)", padding: "0.85rem 1.5rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
                        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "0.5rem", flexWrap: "wrap" }}>
                            <div style={{ display: "flex", gap: "0.45rem", flexWrap: "wrap" }}>
                                <button className={`heap-filter-sidebar__tag-btn ${createMode === "create" ? "heap-filter-sidebar__tag-btn--active" : ""}`} onClick={() => setCreateMode("create")}>Create Block</button>
                                <button className={`heap-filter-sidebar__tag-btn ${createMode === "generate" ? "heap-filter-sidebar__tag-btn--active" : ""}`} onClick={() => setCreateMode("generate")}>Generate with Agent</button>
                                <button className={`heap-filter-sidebar__tag-btn ${createMode === "upload" ? "heap-filter-sidebar__tag-btn--active" : ""}`} onClick={() => setCreateMode("upload")}>Upload</button>
                            </div>
                            <button className="heap-filter-sidebar__tag-btn" onClick={() => setCreatePanelOpen(false)}>Close</button>
                        </div>
                        <div style={{ display: "grid", gridTemplateColumns: "minmax(220px, 0.9fr) minmax(0, 1.3fr)", gap: "0.65rem" }}>
                            <input
                                type="text"
                                value={newBlockTitle}
                                onChange={(event) => setNewBlockTitle(event.target.value)}
                                className="heap-filter-sidebar__search"
                                placeholder="Block title"
                            />
                            {createMode === "create" && (
                                <input
                                    type="text"
                                    value={newBlockType}
                                    onChange={(event) => setNewBlockType(event.target.value)}
                                    className="heap-filter-sidebar__search"
                                    placeholder="Block type (note/task/chart)"
                                />
                            )}
                            {createMode === "generate" && (
                                <input
                                    type="text"
                                    value={agentPrompt}
                                    onChange={(event) => setAgentPrompt(event.target.value)}
                                    className="heap-filter-sidebar__search"
                                    placeholder="Prompt for Generate with Agent"
                                />
                            )}
                            {createMode === "upload" && (
                                <input
                                    type="file"
                                    className="heap-filter-sidebar__search"
                                    onChange={(event) => setUploadFile(event.target.files?.[0] || null)}
                                />
                            )}
                        </div>
                        {createMode === "create" && (
                            <textarea
                                value={newBlockText}
                                onChange={(event) => setNewBlockText(event.target.value)}
                                className="heap-filter-sidebar__search"
                                style={{ minHeight: "84px", resize: "vertical" }}
                                placeholder="Write block content..."
                            />
                        )}
                        <div style={{ display: "flex", justifyContent: "flex-end" }}>
                            <button
                                onClick={emitCreatedBlock}
                                disabled={isEmitting}
                                style={{
                                    background: "var(--cortex-accent)",
                                    color: "#fff",
                                    padding: "6px 12px",
                                    borderRadius: "0.375rem",
                                    fontSize: "0.82rem",
                                    border: "none",
                                    cursor: isEmitting ? "not-allowed" : "pointer",
                                    opacity: isEmitting ? 0.6 : 1
                                }}
                            >
                                {isEmitting ? "Creating..." : "Create Block"}
                            </button>
                        </div>
                    </section>
                )}

                {/* Block Grid + Schema Inspector */}
                <div style={{ flex: 1, minHeight: 0, display: "flex" }}>
                    <div className="heap-scroll" style={{ flex: 1, overflowY: "auto", padding: "1.5rem", position: "relative" }}>
                        {loading ? (
                            <div style={{ padding: "2rem", color: "#64748b", textAlign: "center" }}>Loading blocks...</div>
                        ) : filteredBlocks.length === 0 ? (
                            <div className="heap-empty-state">
                                <div className="heap-empty-state__icon-ring">
                                    <span style={{ fontSize: "2rem", opacity: 0.5 }}>◻</span>
                                </div>
                                <p style={{ fontSize: "0.875rem" }}>No blocks found in this view.</p>
                            </div>
                        ) : (
                            <div className="heap-masonry-grid" style={{ paddingBottom: "6rem" }}>
                                {filteredBlocks.map(b => (
                                    <HeapBlockCard
                                        key={b.projection.artifactId}
                                        block={b}
                                        isSelected={selectedBlockIds.includes(b.projection.artifactId)}
                                        isRegenerating={regeneratingId === b.projection.artifactId}
                                        onClick={(event) => handleSelection(b.projection.artifactId, event)}
                                        onDoubleClick={() => setExpandedBlockId(b.projection.artifactId)}
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                    {selectedPrimaryBlock && (
                        <aside className="heap-inspector-panel">
                            <div className="heap-inspector-panel__header">
                                <h3 className="heap-inspector-panel__title">Schema Inspector</h3>
                                <button className="heap-modal__close-btn" onClick={() => setSelectedBlockIds([])} aria-label="Close schema inspector">✕</button>
                            </div>
                            <div className="heap-inspector-panel__body">
                                <p style={{ fontSize: "12px", color: "#94a3b8", lineHeight: 1.55, marginBottom: "0.75rem" }}>
                                    EmitHeapBlock projection and surface payload for the selected heap block.
                                </p>
                                <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
                                    <div>
                                        <h4 className="heap-modal__section-label">Projection</h4>
                                        <pre className="heap-inspector-panel__json">{JSON.stringify(selectedPrimaryBlock.projection, null, 2)}</pre>
                                    </div>
                                    <div>
                                        <h4 className="heap-modal__section-label">Surface Json</h4>
                                        <pre className="heap-inspector-panel__json">{JSON.stringify(selectedPrimaryBlock.surfaceJson, null, 2)}</pre>
                                    </div>
                                    {(selectedPrimaryBlock.warnings?.length ?? 0) > 0 && (
                                        <div>
                                            <h4 className="heap-modal__section-label">Warnings</h4>
                                            <pre className="heap-inspector-panel__json">{JSON.stringify(selectedPrimaryBlock.warnings, null, 2)}</pre>
                                        </div>
                                    )}
                                </div>
                            </div>
                        </aside>
                    )}
                </div>

                {/* Floating Action Bar */}
                {selectedBlockIds.length > 0 && (
                    <HeapActionBar
                        selectedBlockIds={selectedBlockIds}
                        onDeselect={() => setSelectedBlockIds([])}
                        onPinToggled={handlePinToggled}
                        onDeleted={handleDeleted}
                        onRegenerate={handleRegenerate}
                        onContextBundle={handleContextBundle}
                        onExport={handleExport}
                        onHistory={handleHistory}
                        onPublish={handlePublish}
                    />
                )}

                {/* Detail Modal */}
                {expandedBlock && (
                    <HeapDetailModal
                        block={expandedBlock}
                        onClose={() => setExpandedBlockId(null)}
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
            </main>
        </div>
    );
}

function resolveHeapDeltaPollingEnabled(envValue: string): boolean {
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

function resolveWorkspaceId(candidate?: string): string {
    if (candidate && isUlid(candidate)) {
        return candidate;
    }
    return DEFAULT_WORKSPACE_ID;
}

function isUlid(value: string): boolean {
    return /^[0-9A-HJKMNP-TV-Z]{26}$/.test(value);
}
