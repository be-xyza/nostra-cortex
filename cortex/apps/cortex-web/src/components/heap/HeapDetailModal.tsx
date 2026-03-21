import React, { useEffect, useMemo, useState } from "react";
import type { HeapBlockListItem, ActionSelectionContext } from "../../contracts";
import { workbenchApi } from "../../api";
import { PayloadRenderer, PayloadContent } from './PayloadRenderer';
import { NdlMetadataBlock } from '../ndl/NdlMetadataBlock';
import { useHeapActionPlan } from "./useHeapActionPlan";
import { executeHeapAction, type ActionHandlers } from "./actionExecutor";
import { HeapActionBar } from "./HeapActionBar";
import { ActionZoneRenderer } from "../commons/ActionZoneRenderer";
import { buildHeapRelationIndex, resolveHeapRelationBlock } from "./heapRelations";
import {
    buildHeapRelationUpsertRequest,
    buildMinimalHeapBlockRequest,
    createInitialHeapRelationDraft,
    type HeapRelationDraft,
} from "./heapRelationEditor";
import { useUiStore } from "../../store/uiStore";
import { useAvailableSpaces, useActiveSpaceContext } from "../../store/spacesRegistry";
import { useLayoutPreferences, applyOrder } from "../../store/layoutPreferences";
import { GripVertical } from "lucide-react";
import { AmbientGraphBackground } from "./AmbientGraphBackground";
// Removed local WorkbenchNamingModal import, using global one in ShellLayout

type TabType = 'preview' | 'attributes' | 'relations' | 'code';

interface HeapDetailModalProps {
    block: HeapBlockListItem;
    allBlocks: HeapBlockListItem[];
    onClose: () => void;
    onViewDiscussion: (artifactId: string) => void;
    onNavigateToBlock: (artifactId: string) => void;
    onRelationSaved: (artifactId: string) => void;
    ambientGraphVariant?: string;
}

export function HeapDetailModal({
    block,
    allBlocks,
    onClose,
    onViewDiscussion,
    onNavigateToBlock,
    onRelationSaved,
    ambientGraphVariant,
}: HeapDetailModalProps) {
    const { projection, surfaceJson } = block;
    const surface = (surfaceJson as Record<string, unknown>) || {};
    const sessionUser = useUiStore((state) => state.sessionUser);
    const availableSpaces = useAvailableSpaces();
    const activeSpaceId = useActiveSpaceContext();
    const [relationLoading, setRelationLoading] = useState(false);
    const layoutPrefs = useLayoutPreferences((s) => s.cache[activeSpaceId] ?? {});
    const stageChange = useLayoutPreferences((s) => s.stageChange);
    const [previewTabOrder, setPreviewTabOrder] = useState<string[] | null>(null);
    const [draggedTab, setDraggedTab] = useState<string | null>(null);

    const tabOrder = useMemo(() => {
        const baseOrder = ['preview', 'relations', 'attributes', 'code'];
        const savedOrder = layoutPrefs.modalTabs?.itemOrder || [];
        if (!savedOrder.length) return baseOrder;
        
        return applyOrder(baseOrder, layoutPrefs.modalTabs);
    }, [layoutPrefs.modalTabs]);

    const currentTabOrder = previewTabOrder || tabOrder;

    const [activeTab, setActiveTab] = useState<TabType>('preview');
    const [relationDraft, setRelationDraft] = useState<HeapRelationDraft>(() => createInitialHeapRelationDraft(block));
    const [relationBaseline, setRelationBaseline] = useState<HeapRelationDraft>(() => createInitialHeapRelationDraft(block));
    const [targetContext, setTargetContext] = useState<string>(activeSpaceId === "meta" ? "" : activeSpaceId);
    const [relationEditorOpen, setRelationEditorOpen] = useState(false);
    const [relationSaving, setRelationSaving] = useState(false);
    const [relationStatus, setRelationStatus] = useState<string | null>(null);
    const [relationError, setRelationError] = useState<string | null>(null);
    const [resolvingTargets, setResolvingTargets] = useState<Set<string>>(new Set());
    const [tagInput, setTagInput] = useState("");
    const [mentionInput, setMentionInput] = useState("");
    const [pageLinkInput, setPageLinkInput] = useState("");
    const wasWorkbenchNamedManually = useUiStore((s) => s.wasWorkbenchNamedManually);
    const activeSpaceIds = useUiStore((s) => s.activeSpaceIds);
    const setNamingModalOpen = useUiStore((s) => s.setNamingModalOpen);
    const setPendingWorkbenchAction = useUiStore((s) => s.setPendingWorkbenchAction);

    const selectionContext: ActionSelectionContext = {
        selectedArtifactIds: [block.projection.artifactId],
        activeArtifactId: block.projection.artifactId,
        selectedCount: 1,
        selectedBlockTypes: [block.projection.blockType].filter(Boolean) as string[]
    };
    const attentionEntries = [];
    const totalBadgeCount = 0;

    const { actionPlan, loading: planLoading, error, source } = useHeapActionPlan({
        pageType: "heap_detail",
        zones: ["heap_detail_header", "heap_detail_footer"],
        selection: selectionContext
    });

    useEffect(() => {
        const nextDraft = createInitialHeapRelationDraft(block);
        setRelationDraft(nextDraft);
        setRelationBaseline(nextDraft);
        setRelationEditorOpen(false);
        setRelationSaving(false);
        setRelationStatus(null);
        setRelationError(null);
        setResolvingTargets(new Set());
        setTagInput("");
        setMentionInput("");
        setPageLinkInput("");
    }, [block]);

    const actionHandlers: ActionHandlers = {
        onRegenerate: () => console.log("Regenerate inside modal for", block.projection.artifactId),
        onDeselect: () => onClose(),
        onOpenDiscussion: () => onViewDiscussion(projection.artifactId),
        onEdit: () => {
            setRelationEditorOpen((open) => !open);
            setActiveTab("relations");
        },
    };

    const headerZonePlan = actionPlan?.zones.find(z => z.zone === "heap_detail_header");
    const footerZonePlan = actionPlan?.zones.find(z => z.zone === "heap_detail_footer");

    const surfaceToPayloadContent = (surface: unknown): PayloadContent => {
        const s = surface as Record<string, unknown>;
        return {
            payload_type: s.payload_type as string || block.projection.blockType,
            structured_data: s.structured_data as Record<string, unknown> | undefined,
            data: s.data as Record<string, unknown> | undefined,
            tree: s.tree as PayloadContent["tree"] | undefined,
            text: s.text as string,
            plain_text: s.plain_text as string,
            media: s.media as PayloadContent["media"],
            a2ui: s.a2ui as PayloadContent["a2ui"],
            meta: s.meta as Record<string, unknown> | undefined,
            pointer: s.pointer as string | undefined,
            warnings: block.warnings,
        };
    };

    const payloadContent = surfaceToPayloadContent(block.surfaceJson);
    const relationIndex = useMemo(
        () => buildHeapRelationIndex(block, allBlocks),
        [allBlocks, block],
    );
    const outboundLinks = relationIndex.outboundLinks;
    const outboundMentions = relationIndex.outboundMentions;
    const backlinks = relationIndex.backlinks;
    const tagNeighbors = relationIndex.tagNeighbors;
    const plainText = extractPlainText(payloadContent);
    const wordCount = plainText.trim().length ? plainText.trim().split(/\s+/).length : 0;
    const characterCount = plainText.length;
    const relationDraftDirty = useMemo(
        () => serializeRelationDraft(relationDraft) !== serializeRelationDraft(relationBaseline),
        [relationBaseline, relationDraft],
    );
    const relationCandidates = useMemo(
        () => allBlocks.filter((candidate) => candidate.projection.artifactId !== projection.artifactId),
        [allBlocks, projection.artifactId],
    );
    const knownTagIds = useMemo(() => {
        const tagSet = new Set<string>();
        for (const candidate of relationCandidates) {
            for (const tag of candidate.projection.tags ?? []) {
                if (tag.trim()) {
                    tagSet.add(tag.trim());
                }
            }
        }
        return Array.from(tagSet).sort((left, right) => left.localeCompare(right));
    }, [relationCandidates]);
    const linkSuggestions = useMemo(
        () => backlinks.filter((item) => !relationDraft.pageLinks.includes(item.id)).slice(0, 6),
        [backlinks, relationDraft.pageLinks],
    );
    const mentionSuggestions = useMemo(() => {
        const seen = new Set(relationDraft.mentions.map((item) => item.artifactId));
        return [...backlinks, ...outboundLinks, ...tagNeighbors].filter((item) => {
            if (!item.isNavigable) return false;
            if (seen.has(item.id)) return false;
            seen.add(item.id);
            return true;
        }).slice(0, 6);
    }, [backlinks, outboundLinks, relationDraft.mentions, tagNeighbors]);
    const tagSuggestions = useMemo(
        () => knownTagIds.filter((tag) => !relationDraft.tags.includes(tag)).slice(0, 6),
        [knownTagIds, relationDraft.tags],
    );

    const formatJsonWithHighlighting = (json: any) => {
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
    };

    const navigateRelation = (artifactId: string) => {
        if (!resolveHeapRelationBlock(artifactId, allBlocks)) {
            return;
        }
        onNavigateToBlock(artifactId);
        setActiveTab("relations");
    };

    const addRelationTag = (value: string) => {
        const normalized = value.trim();
        if (!normalized) return;
        setRelationDraft((current) => (
            current.tags.includes(normalized)
                ? current
                : { ...current, tags: [...current.tags, normalized] }
        ));
        setTagInput("");
        setRelationStatus(null);
        setRelationError(null);
    };

    const addRelationMention = (artifactId: string) => {
        const normalized = artifactId.trim();
        if (!normalized) return;
        setRelationDraft((current) => (
            current.mentions.some((item) => item.artifactId === normalized)
                ? current
                : {
                    ...current,
                    mentions: [
                        ...current.mentions,
                        {
                            artifactId: normalized,
                            label: relationCandidates.find((candidate) => candidate.projection.artifactId === normalized)?.projection.title ?? normalized,
                        },
                    ],
                }
        ));
        setMentionInput("");
        setRelationStatus(null);
        setRelationError(null);
    };

    const addRelationPageLink = (artifactId: string) => {
        const normalized = artifactId.trim();
        if (!normalized) return;
        setRelationDraft((current) => (
            current.pageLinks.includes(normalized)
                ? current
                : { ...current, pageLinks: [...current.pageLinks, normalized] }
        ));
        setPageLinkInput("");
        setRelationStatus(null);
        setRelationError(null);
    };

    const removeRelationTag = (tag: string) => {
        setRelationDraft((current) => ({
            ...current,
            tags: current.tags.filter((item) => item !== tag),
        }));
        setRelationStatus(null);
        setRelationError(null);
    };

    const removeRelationMention = (artifactId: string) => {
        setRelationDraft((current) => ({
            ...current,
            mentions: current.mentions.filter((item) => item.artifactId !== artifactId),
        }));
        setRelationStatus(null);
        setRelationError(null);
    };

    const removeRelationPageLink = (artifactId: string) => {
        setRelationDraft((current) => ({
            ...current,
            pageLinks: current.pageLinks.filter((item) => item !== artifactId),
        }));
        setRelationStatus(null);
        setRelationError(null);
    };

    const resetRelationDraft = () => {
        setRelationDraft(relationBaseline);
        setTagInput("");
        setMentionInput("");
        setPageLinkInput("");
        setRelationStatus(null);
        setRelationError(null);
    };

    const saveRelationDraft = async () => {
        if (activeSpaceIds.length > 1 && !wasWorkbenchNamedManually) {
            setPendingWorkbenchAction(() => saveRelationDraft());
            setNamingModalOpen(true);
            return;
        }

        try {
            setRelationSaving(true);
            setRelationError(null);
            setRelationStatus("Writing relation map into the heap...");
            const request = buildHeapRelationUpsertRequest({
                block,
                relationDraft,
                agentId: "cortex-web.relations",
            });
            await workbenchApi.emitHeapBlock(request);
            setRelationBaseline(relationDraft);
            setRelationEditorOpen(false);
            setRelationStatus("Relation map updated.");
            onRelationSaved(projection.artifactId);
        } catch (saveError) {
            setRelationError(saveError instanceof Error ? saveError.message : String(saveError));
            setRelationStatus(null);
        } finally {
            setRelationSaving(false);
            setPendingWorkbenchAction(null);
        }
    };

    const resolveMissingTarget = async (artifactId: string) => {
        try {
            setResolvingTargets((prev) => new Set(prev).add(artifactId));
            setRelationError(null);
            setRelationStatus(`Creating placeholder for ${artifactId}...`);
            const spaceId =
                targetContext ||
                (block.projection.spaceId ??
                (typeof surface.space_id === "string" ? surface.space_id : ""));
            const request = buildMinimalHeapBlockRequest(
                artifactId,
                spaceId,
                new Date().toISOString(),
                "cortex-web.relations",
            );
            await workbenchApi.emitHeapBlock(request);
            setRelationStatus(`Created placeholder block for ${artifactId}.`);
            onRelationSaved(projection.artifactId);
        } catch (resolveError) {
            setRelationError(resolveError instanceof Error ? resolveError.message : String(resolveError));
            setRelationStatus(null);
        } finally {
            setResolvingTargets((prev) => {
                const next = new Set(prev);
                next.delete(artifactId);
                return next;
            });
        }
    };

    const handleTabDragStart = (tab: string, e: React.DragEvent) => {
        setDraggedTab(tab);
        e.dataTransfer.effectAllowed = "move";
    };

    const handleTabDragOver = (targetTab: string, e: React.DragEvent) => {
        e.preventDefault();
        if (!draggedTab || draggedTab === targetTab) return;

        const newOrder = [...currentTabOrder];
        const fromIndex = newOrder.indexOf(draggedTab);
        const toIndex = newOrder.indexOf(targetTab);

        if (fromIndex === -1 || toIndex === -1) return;

        newOrder.splice(fromIndex, 1);
        newOrder.splice(toIndex, 0, draggedTab);
        setPreviewTabOrder(newOrder);
    };

    const handleTabDrop = (e: React.DragEvent) => {
        e.preventDefault();
        if (!previewTabOrder) return;

        stageChange(activeSpaceId, (prev) => ({
            ...prev,
            modalTabs: { 
                itemOrder: previewTabOrder, 
                hidden: prev.modalTabs?.hidden || [] 
            }
        }));
        setPreviewTabOrder(null);
        setDraggedTab(null);
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-fade-in" onClick={onClose}>
            <div className="heap-modal-content relative w-full max-w-6xl max-h-[92vh] flex flex-col bg-slate-900 border border-white/5 rounded-2xl shadow-[0_0_50px_rgba(0,0,0,0.5)] overflow-hidden animate-slide-up" onClick={(e) => e.stopPropagation()}>
                {/* Soft Background Layer */}
                <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden opacity-40">
                    <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,rgba(59,130,246,0.12),transparent_70%)]" />
                    {ambientGraphVariant && ambientGraphVariant !== "off" && (
                        <AmbientGraphBackground 
                            visible={true} 
                            variant={ambientGraphVariant as any} 
                            spaceId={block.projection.spaceId || "01ARZ3NDEKTSV4RRFFQ69G5FAV"}
                        />
                    )}
                </div>

                {/* Header */}
                <div className="relative z-10 flex items-start justify-between p-6 border-b border-white/5 bg-slate-900/50 backdrop-blur-md">
                    <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-3 mb-2">
                            <NdlMetadataBlock
                                typeIndicator={projection.blockType?.toUpperCase() || "NOTE"}
                                versionChain={String(surface.version || "0.8")}
                                phase={String(surface.phase || "Alpha")}
                                confidence={typeof surface.confidence === "number" ? surface.confidence : 85}
                                authorityScope={String(surface.authority_scope || "Local")}
                                compact
                            />
                            <span className="text-slate-700 text-[10px]">•</span>
                            <p className="text-[10px] font-mono text-slate-500">
                                {projection.artifactId}
                            </p>
                        </div>
                        <h2 className="text-2xl font-bold text-slate-100 mb-1 tracking-tight">{projection.title || "Untitled Block"}</h2>
                        <p className="text-[10px] font-semibold text-slate-500 uppercase tracking-widest">
                            Emitted: {new Date(projection.emittedAt || projection.updatedAt).toLocaleString()}
                        </p>
                    </div>
                    <div className="ml-4 flex items-center gap-3">
                        {headerZonePlan && (
                            <ActionZoneRenderer
                                actions={headerZonePlan.actions}
                                layoutHint={headerZonePlan.layoutHint}
                                onActionClick={(action) => executeHeapAction(action, selectionContext, actionHandlers)}
                            />
                        )}
                        <button className="p-2 rounded-full text-slate-400 hover:text-white hover:bg-slate-800 transition-all hover:rotate-90" onClick={onClose}>✕</button>
                    </div>
                </div>

                {/* Body Content */}
                <div className="relative z-10 flex-1 overflow-y-auto bg-slate-900/20 custom-scrollbar">
                    <div className="grid gap-0 xl:grid-cols-[minmax(0,1.45fr)_minmax(320px,0.95fr)]">
                        <div className="pt-6 px-8 pb-8">
                            {(block.warnings?.length ?? 0) > 0 && (activeTab === 'preview' || activeTab === 'code') && (
                                <div className="mb-6 p-4 rounded-lg bg-amber-500/10 border border-amber-500/30 flex gap-3 items-start animate-in fade-in slide-in-from-top-2">
                                    <span className="text-xl">⚠️</span>
                                    <div>
                                        <h4 className="text-sm font-bold text-amber-500 mb-1">Block Validation Warnings</h4>
                                        <ul className="text-xs text-amber-200/80 space-y-1 list-disc list-inside">
                                            {block.warnings?.map((w, i) => (
                                                <li key={i}>{typeof w === 'string' ? w : JSON.stringify(w)}</li>
                                            ))}
                                        </ul>
                                    </div>
                                </div>
                            )}

                            {/* Tabs Navigation */}
                            <div 
                                className="flex gap-2 mb-6 bg-white/5 p-1.5 rounded-xl border border-white/5 shadow-inner w-fit"
                                onDragOver={(e) => e.preventDefault()}
                                onDrop={handleTabDrop}
                            >
                                {currentTabOrder.map((tabId: string) => {
                                    const tab = tabId as TabType;
                                    const isActive = activeTab === tab;
                                    return (
                                        <button
                                            key={tab}
                                            draggable
                                            onDragStart={(e) => handleTabDragStart(tab, e)}
                                            onDragOver={(e) => handleTabDragOver(tab, e)}
                                            onDragEnd={() => {
                                                setDraggedTab(null);
                                                setPreviewTabOrder(null);
                                            }}
                                            onClick={() => setActiveTab(tab)}
                                            className={`group flex items-center gap-2 px-5 py-2.5 text-[10px] font-black uppercase tracking-[0.15em] transition-all duration-300 rounded-lg select-none ${isActive
                                                ? 'bg-blue-500/10 text-blue-400 shadow-[0_0_20px_rgba(59,130,246,0.1)] border border-blue-500/20 ring-1 ring-blue-500/10'
                                                : 'text-slate-500 hover:text-slate-300 hover:bg-white/5'
                                                } ${draggedTab === tab ? 'opacity-40 scale-95' : ''}`}
                                        >
                                            <GripVertical className={`w-3 h-3 transition-opacity duration-200 ${isActive ? 'opacity-40' : 'opacity-0 group-hover:opacity-40'} cursor-grab active:cursor-grabbing shrink-0`} />
                                            <span className={isActive ? 'drop-shadow-[0_0_5px_rgba(96,165,250,0.5)]' : ''}>{tab}</span>
                                        </button>
                                    );
                                })}
                            </div>

                            {/* Tab Content Panels */}
                            <div className="min-h-[300px]">
                                {activeTab === 'relations' && (
                                    <div className="animate-in fade-in duration-300 space-y-8">
                                <section className="rounded-2xl border border-white/5 bg-slate-950/40 p-6">
                                    <h4 className="text-[10px] uppercase font-black text-slate-500 tracking-widest mb-4">Networked Context</h4>
                                    <p className="text-sm leading-7 text-slate-300">
                                        This block sits inside a graph of mentions, page links, and tag neighbors. Treat this surface as the place to sort it into the heap, not just inspect its payload.
                                    </p>
                                    <div className="mt-4 flex flex-wrap items-center gap-3 text-[11px] text-slate-400">
                                        <span className="rounded-full border border-white/5 bg-slate-900/60 px-3 py-1">
                                            {relationDraftDirty ? "Draft differs from saved graph" : "Draft matches saved graph"}
                                        </span>
                                        <button
                                            type="button"
                                            className="rounded-full border border-cyan-500/30 bg-cyan-500/10 px-3 py-1 font-semibold text-cyan-200 transition hover:bg-cyan-500/20"
                                            onClick={() => setRelationEditorOpen((open) => !open)}
                                        >
                                            {relationEditorOpen ? "Hide Relation Composer" : "Edit Relation Map"}
                                        </button>
                                    </div>
                                </section>
                                        <RelationSection
                                            title={`Backlinks (${backlinks.length})`}
                                            accent="cyan"
                                            items={backlinks}
                                            emptyLabel="No inbound references yet."
                                            onSelect={navigateRelation}
                                        />
                                        <RelationSection
                                            title={`Outbound Links (${outboundLinks.length})`}
                                            accent="blue"
                                            items={outboundLinks}
                                            emptyLabel="No page links yet."
                                            onSelect={navigateRelation}
                                        />
                                        <RelationSection
                                            title={`Mentions (${outboundMentions.length})`}
                                            accent="indigo"
                                            items={outboundMentions}
                                            emptyLabel="No mentions yet."
                                            onSelect={navigateRelation}
                                        />
                                    </div>
                                )}

                                {activeTab === 'preview' && (
                                    <div className="animate-in fade-in duration-300 space-y-8">
                                        <div className="bg-slate-950/40 rounded-2xl border border-white/5 p-6 shadow-inner ring-1 ring-white/5">
                                            <PayloadRenderer content={payloadContent} expanded artifactId={projection.artifactId} />
                                        </div>
                                    </div>
                                )}

                                {activeTab === 'attributes' && (
                                    <div className="animate-in fade-in duration-300 space-y-6">
                                        <h4 className="text-[10px] uppercase font-black text-slate-500 tracking-widest mb-4">Structural Attributes</h4>
                                        <div className="grid grid-cols-1 gap-2">
                                            {Object.entries(projection).map(([k, v]) => {
                                                if (typeof v !== 'object' && v !== undefined && v !== null) {
                                                    return (
                                                        <div key={k} className="flex items-center justify-between bg-slate-950/40 border border-white/5 rounded-xl px-4 py-2.5 hover:bg-slate-950/60 transition-colors">
                                                            <span className="text-xs text-slate-500 font-medium">{k}</span>
                                                            <span className="text-xs font-mono text-slate-300 bg-slate-900 px-2 py-0.5 rounded border border-white/5">{String(v)}</span>
                                                        </div>
                                                    );
                                                }
                                                return null;
                                            })}
                                        </div>
                                    </div>
                                )}

                                {activeTab === 'code' && (
                                    <div className="animate-in fade-in duration-300 space-y-6">
                                        <div className="flex items-center justify-between mb-2">
                                            <p className="text-xs text-slate-400/80 italic">
                                                Raw <code className="bg-slate-800 text-slate-300 px-1 rounded">EmitHeapBlock</code> data projection.
                                            </p>
                                        </div>

                                        <div className="space-y-6">
                                            <div>
                                                <h4 className="text-[10px] uppercase font-black text-slate-500 tracking-widest mb-3 flex items-center gap-2">
                                                    <span className="w-1.5 h-1.5 rounded-full bg-blue-500"></span> Projection
                                                </h4>
                                                <div className="bg-slate-950/80 rounded-xl border border-white/5 p-4 overflow-x-auto shadow-inner custom-scrollbar text-[10px] leading-relaxed font-mono">
                                                    {formatJsonWithHighlighting(block.projection)}
                                                </div>
                                            </div>

                                            <div>
                                                <h4 className="text-[10px] uppercase font-black text-slate-500 tracking-widest mb-3 flex items-center gap-2">
                                                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> Surface JSON
                                                </h4>
                                                <div className="bg-slate-950/80 rounded-xl border border-white/5 p-4 overflow-x-auto shadow-inner custom-scrollbar text-[10px] leading-relaxed font-mono">
                                                    {formatJsonWithHighlighting(block.surfaceJson)}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                )}
                            </div>
                        </div>

                        <aside className="border-t border-white/5 bg-slate-950/55 p-6 xl:border-l xl:border-t-0">
                            <div className="sticky top-0 space-y-6">
                                <section className="rounded-2xl border border-white/5 bg-slate-900/70 p-5">
                                    <div className="text-[10px] font-black uppercase tracking-[0.28em] text-slate-500">Current Block</div>
                                    <div className="mt-3 text-lg font-bold text-slate-100">{projection.title || "Untitled Block"}</div>
                                    <div className="mt-1 text-xs leading-6 text-slate-400">
                                        {plainText.slice(0, 180) || "No plain-text preview available."}
                                    </div>
                                </section>

                                <RelationComposerCard
                                    open={relationEditorOpen}
                                    saving={relationSaving}
                                    dirty={relationDraftDirty}
                                    relationDraft={relationDraft}
                                    relationCandidates={relationCandidates}
                                    tagSuggestions={tagSuggestions}
                                    mentionSuggestions={mentionSuggestions}
                                    linkSuggestions={linkSuggestions}
                                    knownTagIds={knownTagIds}
                                    tagInput={tagInput}
                                    mentionInput={mentionInput}
                                    pageLinkInput={pageLinkInput}
                                    relationStatus={relationStatus}
                                    relationError={relationError}
                                    onToggleOpen={() => setRelationEditorOpen((open) => !open)}
                                    onTagInputChange={setTagInput}
                                    onMentionInputChange={setMentionInput}
                                    onPageLinkInputChange={setPageLinkInput}
                                    onAddTag={addRelationTag}
                                    onAddMention={addRelationMention}
                                    onAddPageLink={addRelationPageLink}
                                    onRemoveTag={removeRelationTag}
                                    onRemoveMention={removeRelationMention}
                                    onRemovePageLink={removeRelationPageLink}
                                    onReset={resetRelationDraft}
                                    onSave={saveRelationDraft}
                                    onResolveTarget={resolveMissingTarget}
                                    resolvingTargets={resolvingTargets}
                                    targetContext={targetContext}
                                    onTargetContextChange={setTargetContext}
                                />

                                <RelationSection
                                    title={`Tag Neighbors (${tagNeighbors.length})`}
                                    accent="violet"
                                    items={tagNeighbors}
                                    emptyLabel="No neighboring tagged blocks."
                                    onSelect={navigateRelation}
                                />

                                <section className="rounded-2xl border border-white/5 bg-slate-900/70 p-5">
                                    <div className="text-[10px] font-black uppercase tracking-[0.28em] text-slate-500">Block Stats</div>
                                    <div className="mt-4 grid grid-cols-2 gap-3">
                                        <StatCard label="Words" value={wordCount} />
                                        <StatCard label="Characters" value={characterCount} />
                                        <StatCard label="Tags" value={block.projection.tags?.length ?? 0} />
                                        <StatCard label="Links" value={outboundLinks.length + outboundMentions.length + backlinks.length} />
                                    </div>
                                    <div className="mt-4 text-[11px] text-slate-500">
                                        Updated {new Date(projection.updatedAt).toLocaleString()}
                                    </div>
                                </section>
                                <section className="rounded-2xl border border-white/5 bg-slate-900/70 p-5">
                                    <div className="text-[10px] font-black uppercase tracking-[0.28em] text-slate-500">IDs & Export Surface</div>
                                    <div className="mt-3 space-y-3 text-[11px]">
                                        <div className="rounded-xl border border-white/5 bg-slate-950/70 p-3">
                                            <div className="text-slate-500">Block ID</div>
                                            <div className="mt-1 font-mono text-slate-200 break-all">{projection.artifactId}</div>
                                        </div>
                                        <div className="rounded-xl border border-white/5 bg-slate-950/70 p-3">
                                            <div className="text-slate-500">Space</div>
                                            <div className="mt-1 font-mono text-slate-200 break-all">{String(surface.space_id || "unknown")}</div>
                                        </div>
                                        <div className="rounded-xl border border-white/5 bg-slate-950/70 p-3 text-slate-400">
                                            Export and compaction surfaces stay accessible through compiled actions and the raw block inspector below.
                                        </div>
                                    </div>
                                </section>
                            </div>
                        </aside>
                    </div>
                </div>

                {/* Slim status footer */}
                <div className="px-6 py-3 border-t border-white/5 bg-slate-900/80 text-[11px] text-slate-500">
                    {planLoading
                        ? "Syncing contextual actions..."
                        : source === "fallback"
                            ? "Using fallback heap actions."
                            : "Contextual graph actions live."}
                    {error && (
                        <span className="ml-2 text-red-400 truncate" title={error}>
                            {error}
                        </span>
                    )}
                </div>

                {/* Floating selection action bar inside modal */}
                {footerZonePlan && (
                    <HeapActionBar
                        selectionZonePlan={footerZonePlan}
                        selection={selectionContext}
                        handlers={actionHandlers}
                        onCreate={() => {}}
                        status={{ loading: planLoading, source, error }}
                    />
                )}
            </div>
        </div>
    );
}

function extractPlainText(content: PayloadContent): string {
    if (content.plain_text) return content.plain_text;
    if (content.text) return content.text;
    if (content.pointer) return content.pointer;
    if (content.structured_data) return JSON.stringify(content.structured_data);
    if (content.data) return JSON.stringify(content.data);
    return "";
}

function RelationSection({
    title,
    items,
    emptyLabel,
    accent,
    onSelect,
}: {
    title: string;
    items: Array<{ id: string; title: string; subtitle?: string; isNavigable?: boolean }>;
    emptyLabel: string;
    accent: "blue" | "cyan" | "indigo" | "violet";
    onSelect?: (artifactId: string) => void;
}) {
    const accentClass = accent === "cyan"
        ? "bg-cyan-500"
        : accent === "indigo"
            ? "bg-indigo-500"
            : accent === "violet"
                ? "bg-violet-500"
                : "bg-blue-500";

    return (
        <section className="rounded-2xl border border-white/5 bg-slate-950/35 p-5">
            <h4 className="text-[10px] uppercase font-black text-slate-500 tracking-widest mb-4 flex items-center gap-2">
                <span className={`h-3 w-1 rounded-full ${accentClass}`}></span>
                {title}
            </h4>
            {items.length === 0 ? (
                <p className="text-xs text-slate-600 italic">{emptyLabel}</p>
            ) : (
                <div className="space-y-2">
                    {items.map((item) => (
                        <button
                            key={`${title}-${item.id}`}
                            type="button"
                            onClick={() => item.isNavigable && onSelect?.(item.id)}
                            disabled={!item.isNavigable}
                            className={`w-full rounded-xl border border-white/5 bg-slate-900/70 p-3 text-left ${
                                item.isNavigable ? "transition-colors hover:border-slate-600 hover:bg-slate-900" : "cursor-default"
                            }`}
                        >
                            <div className="text-sm font-semibold text-slate-100">{item.title || item.id}</div>
                            <div className="mt-1 text-[11px] uppercase tracking-[0.2em] text-slate-500">
                                {item.subtitle}
                            </div>
                            <div className="mt-2 font-mono text-[10px] text-slate-500 break-all">{item.id}</div>
                        </button>
                    ))}
                </div>
            )}
        </section>
    );
}

function StatCard({ label, value }: { label: string; value: number }) {
    return (
        <div className="rounded-xl border border-white/5 bg-slate-950/70 p-3">
            <div className="text-[10px] uppercase tracking-[0.22em] text-slate-500">{label}</div>
            <div className="mt-2 text-xl font-black text-slate-100">{value}</div>
        </div>
    );
}

function RelationComposerCard({
    open,
    saving,
    dirty,
    relationDraft,
    relationCandidates,
    tagSuggestions,
    mentionSuggestions,
    linkSuggestions,
    knownTagIds,
    tagInput,
    mentionInput,
    pageLinkInput,
    relationStatus,
    relationError,
    onToggleOpen,
    onTagInputChange,
    onMentionInputChange,
    onPageLinkInputChange,
    onAddTag,
    onAddMention,
    onAddPageLink,
    onRemoveTag,
    onRemoveMention,
    onRemovePageLink,
    onReset,
    onSave,
    onResolveTarget,
    resolvingTargets,
    targetContext,
    onTargetContextChange,
}: {
    open: boolean;
    saving: boolean;
    dirty: boolean;
    relationDraft: HeapRelationDraft;
    relationCandidates: HeapBlockListItem[];
    tagSuggestions: string[];
    mentionSuggestions: Array<{ id: string; title: string }>;
    linkSuggestions: Array<{ id: string; title: string }>;
    knownTagIds: string[];
    tagInput: string;
    mentionInput: string;
    pageLinkInput: string;
    relationStatus: string | null;
    relationError: string | null;
    onToggleOpen: () => void;
    onTagInputChange: (value: string) => void;
    onMentionInputChange: (value: string) => void;
    onPageLinkInputChange: (value: string) => void;
    onAddTag: (value: string) => void;
    onAddMention: (artifactId: string) => void;
    onAddPageLink: (artifactId: string) => void;
    onRemoveTag: (tag: string) => void;
    onRemoveMention: (artifactId: string) => void;
    onRemovePageLink: (artifactId: string) => void;
    onReset: () => void;
    onSave: () => void;
    onResolveTarget: (artifactId: string) => void;
    resolvingTargets: Set<string>;
    targetContext: string;
    onTargetContextChange: (value: string) => void;
}) {
    const availableSpaces = useAvailableSpaces();
    const candidateLookup = useMemo(
        () => new Map(relationCandidates.map((candidate) => [candidate.projection.artifactId, candidate.projection.title])),
        [relationCandidates],
    );

    return (
        <section className="rounded-2xl border border-white/5 bg-slate-900/70 p-5">
            <div className="flex items-center justify-between gap-3">
                <div>
                    <div className="text-[10px] font-black uppercase tracking-[0.28em] text-slate-500">Relation Composer</div>
                    <div className="mt-2 text-sm font-semibold text-slate-100">
                        {dirty ? "Drafting heap edges" : "Saved graph state"}
                    </div>
                </div>
                <button
                    type="button"
                    onClick={onToggleOpen}
                    className="rounded-full border border-white/5 bg-slate-950/70 px-3 py-1 text-[11px] font-semibold text-slate-200 transition hover:border-slate-500"
                >
                    {open ? "Collapse" : "Expand"}
                </button>
            </div>

            {(relationStatus || relationError) && (
                <div className={`mt-4 rounded-xl border px-3 py-2 text-xs ${
                    relationError
                        ? "border-rose-500/30 bg-rose-500/10 text-rose-200"
                        : "border-emerald-500/30 bg-emerald-500/10 text-emerald-200"
                }`}>
                    {relationError || relationStatus}
                </div>
            )}

            {open && (
                <div className="mt-5 space-y-5">
                    <div className="flex flex-col gap-2 rounded-xl border border-white/5 bg-slate-950/40 p-4">
                        <label htmlFor="target-context" className="text-[10px] font-black uppercase tracking-widest text-slate-500">
                            Target Space (For New Blocks)
                        </label>
                        <select
                            id="target-context"
                            className="w-full bg-slate-950/70 border border-white/10 rounded-xl px-3 py-2 text-xs text-slate-300 focus:outline-none focus:ring-1 focus:ring-blue-500/50"
                            value={targetContext}
                            onChange={(e) => onTargetContextChange(e.target.value)}
                        >
                            <optgroup label="Workbenches" className="bg-slate-900">
                                <option value="">Meta Workbench (contextual)</option>
                            </optgroup>
                            <optgroup label="Sovereign Spaces" className="bg-slate-900">
                                {availableSpaces.filter((s: { type: string }) => s.type !== 'global').map((space: { id: string, name: string }) => (
                                    <option key={space.id} value={space.id}>{space.name}</option>
                                ))}
                            </optgroup>
                        </select>
                    </div>

                    <EditableRelationSection
                        title="Tags"
                        items={relationDraft.tags.map((tag) => ({
                            id: tag,
                            label: candidateLookup.get(tag) ?? tag,
                            meta: tag,
                            isNew: !candidateLookup.has(tag) && !knownTagIds.includes(tag),
                        }))}
                        inputValue={tagInput}
                        onInputChange={onTagInputChange}
                        onAdd={onAddTag}
                        onRemove={onRemoveTag}
                        onResolve={onResolveTarget}
                        resolvingTargets={resolvingTargets}
                        suggestions={tagSuggestions.map((tag) => ({
                            id: tag,
                            label: candidateLookup.get(tag) ?? tag,
                        }))}
                        datalistId="heap-relation-tag-options"
                        placeholder="Add tag block id"
                    />

                    <EditableRelationSection
                        title="Mentions"
                        items={relationDraft.mentions.map((mention) => ({
                            id: mention.artifactId,
                            label: mention.label || candidateLookup.get(mention.artifactId) || mention.artifactId,
                            meta: mention.artifactId,
                            isNew: !candidateLookup.has(mention.artifactId),
                        }))}
                        inputValue={mentionInput}
                        onInputChange={onMentionInputChange}
                        onAdd={onAddMention}
                        onRemove={onRemoveMention}
                        onResolve={onResolveTarget}
                        resolvingTargets={resolvingTargets}
                        suggestions={mentionSuggestions.map((item) => ({
                            id: item.id,
                            label: item.title,
                        }))}
                        datalistId="heap-relation-mention-options"
                        placeholder="Add mention artifact id"
                    />

                    <EditableRelationSection
                        title="Page Links"
                        items={relationDraft.pageLinks.map((artifactId) => ({
                            id: artifactId,
                            label: candidateLookup.get(artifactId) ?? artifactId,
                            meta: artifactId,
                            isNew: !candidateLookup.has(artifactId),
                        }))}
                        inputValue={pageLinkInput}
                        onInputChange={onPageLinkInputChange}
                        onAdd={onAddPageLink}
                        onRemove={onRemovePageLink}
                        onResolve={onResolveTarget}
                        resolvingTargets={resolvingTargets}
                        suggestions={linkSuggestions.map((item) => ({
                            id: item.id,
                            label: item.title,
                        }))}
                        datalistId="heap-relation-link-options"
                        placeholder="Add linked block artifact id"
                    />

                    <datalist id="heap-relation-tag-options">
                        {relationCandidates.map((candidate) => (
                            <option
                                key={`tag-option-${candidate.projection.artifactId}`}
                                value={candidate.projection.artifactId}
                                label={candidate.projection.title}
                            />
                        ))}
                        {tagSuggestions.map((tag) => (
                            <option key={`tag-known-${tag}`} value={tag} />
                        ))}
                    </datalist>
                    <datalist id="heap-relation-mention-options">
                        {relationCandidates.map((candidate) => (
                            <option
                                key={`mention-option-${candidate.projection.artifactId}`}
                                value={candidate.projection.artifactId}
                                label={candidate.projection.title}
                            />
                        ))}
                    </datalist>
                    <datalist id="heap-relation-link-options">
                        {relationCandidates.map((candidate) => (
                            <option
                                key={`link-option-${candidate.projection.artifactId}`}
                                value={candidate.projection.artifactId}
                                label={candidate.projection.title}
                            />
                        ))}
                    </datalist>

                    <div className="flex flex-wrap items-center gap-3 border-t border-white/5 pt-4">
                        <button
                            type="button"
                            onClick={onSave}
                            disabled={!dirty || saving}
                            className="rounded-full border border-cyan-400/40 bg-cyan-400/15 px-4 py-2 text-xs font-black uppercase tracking-[0.2em] text-cyan-100 transition hover:bg-cyan-400/25 disabled:cursor-not-allowed disabled:opacity-40"
                        >
                            {saving ? "Saving..." : "Save Graph Edges"}
                        </button>
                        <button
                            type="button"
                            onClick={onReset}
                            disabled={!dirty || saving}
                            className="rounded-full border border-white/5 bg-slate-950/70 px-4 py-2 text-xs font-semibold text-slate-300 transition hover:border-slate-500 disabled:cursor-not-allowed disabled:opacity-40"
                        >
                            Reset Draft
                        </button>
                    </div>
                </div>
            )}
        </section>
    );
}

function EditableRelationSection({
    title,
    items,
    inputValue,
    onInputChange,
    onAdd,
    onRemove,
    onResolve,
    resolvingTargets = new Set(),
    suggestions,
    datalistId,
    placeholder,
}: {
    title: string;
    items: Array<{ id: string; label: string; meta?: string; isNew?: boolean }>;
    inputValue: string;
    onInputChange: (value: string) => void;
    onAdd: (value: string) => void;
    onRemove: (value: string) => void;
    onResolve?: (id: string) => void;
    resolvingTargets?: Set<string>;
    suggestions: Array<{ id: string; label: string }>;
    datalistId: string;
    placeholder: string;
}) {
    return (
        <section className="rounded-2xl border border-white/5 bg-slate-950/45 p-4">
            <div className="flex items-center justify-between gap-3">
                <div className="text-[10px] font-black uppercase tracking-[0.24em] text-slate-500">{title}</div>
                <div className="text-[11px] text-slate-500">{items.length} attached</div>
            </div>
            <div className="mt-3 flex gap-2">
                <input
                    value={inputValue}
                    onChange={(event) => onInputChange(event.target.value)}
                    onKeyDown={(event) => {
                        if (event.key === "Enter") {
                            event.preventDefault();
                            onAdd(inputValue);
                        }
                    }}
                    list={datalistId}
                    placeholder={placeholder}
                    className="min-w-0 flex-1 rounded-xl border border-white/5 bg-slate-950/80 px-3 py-2 text-sm text-slate-100 outline-none transition placeholder:text-slate-600 focus:border-cyan-400/60"
                />
                <button
                    type="button"
                    onClick={() => onAdd(inputValue)}
                    className="rounded-xl border border-white/5 bg-slate-900/80 px-3 py-2 text-xs font-semibold text-slate-200 transition hover:border-slate-500"
                >
                    Add
                </button>
            </div>
            {suggestions.length > 0 && (
                <div className="mt-3 flex flex-wrap gap-2">
                    {suggestions.map((suggestion) => (
                        <button
                            key={`${title}-suggestion-${suggestion.id}`}
                            type="button"
                            onClick={() => onAdd(suggestion.id)}
                            className="rounded-full border border-white/5 bg-slate-900/70 px-3 py-1.5 text-[11px] text-slate-300 transition hover:border-cyan-400/50 hover:text-cyan-100"
                        >
                            {suggestion.label}
                        </button>
                    ))}
                </div>
            )}
            <div className="mt-4 flex flex-wrap gap-2">
                {items.length === 0 ? (
                    <div className="text-xs italic text-slate-600">No {title.toLowerCase()} attached yet.</div>
                ) : (
                    items.map((item) => (
                        <div key={`${title}-item-${item.id}`} className="flex items-center gap-1 rounded-full border border-white/5 bg-slate-900/80 pr-1 pl-3 py-1 transition hover:border-slate-500">
                            <div className="flex-1 flex flex-col justify-center text-left max-w-[200px] truncate pr-2">
                                <span className={`block text-xs font-semibold ${item.isNew ? "text-amber-300" : "text-slate-100"}`}>{item.label}</span>
                                {item.meta && (
                                    <span className="mt-0.5 block font-mono text-[10px] text-slate-500">{item.meta}</span>
                                )}
                            </div>
                            {item.isNew && onResolve && (
                                <button
                                    type="button"
                                    onClick={() => onResolve(item.id)}
                                    disabled={resolvingTargets.has(item.id)}
                                    className="px-2 py-0.5 text-[9px] font-bold uppercase tracking-wider rounded-full bg-amber-500/10 text-amber-500 border border-amber-500/30 hover:bg-amber-500/20 disabled:opacity-50 transition-colors"
                                >
                                    {resolvingTargets.has(item.id) ? "Resolving..." : "Resolve"}
                                </button>
                            )}
                            <button
                                type="button"
                                onClick={() => onRemove(item.id)}
                                className="w-5 h-5 flex items-center justify-center rounded-full text-slate-400 hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                            >
                                ✕
                            </button>
                        </div>
                    ))
                )}
            </div>
        </section>
    );
}

function serializeRelationDraft(relationDraft: HeapRelationDraft): string {
    return JSON.stringify({
        tags: [...relationDraft.tags].sort(),
        mentions: relationDraft.mentions
            .map((item) => ({
                artifactId: item.artifactId,
                label: item.label ?? item.artifactId,
            }))
            .sort((left, right) => left.artifactId.localeCompare(right.artifactId)),
        pageLinks: [...relationDraft.pageLinks].sort(),
    });
}
