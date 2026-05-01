import React from "react";
import { useNavigate } from "react-router-dom";
import {
    Target,
    FileText,
    Image,
    CheckSquare,
    BarChart,
    Settings,
    Shield,
    Activity,
    MapPin,
    MessageSquare,
    ArrowUpRight,
    Link2,
    AlertTriangle,
    Clock,
    Cpu,
    Network,
    ListChecks,
    ClipboardCheck,
    MessagesSquare,
} from "lucide-react";
import type { HeapBlockListItem } from "../../contracts";
import type { ActionSelectionContext, ToolbarActionDescriptor } from "../../contracts";
import { useUiStore } from "../../store/uiStore";
import { PayloadRenderer, PayloadContent } from "./PayloadRenderer";
import { A2UIInterpreter, type A2UINode } from "../a2ui/A2UIInterpreter";
import { NdlMetadataBlock } from "../ndl/NdlMetadataBlock";
import { HeapCardActionMenu } from "./HeapCardActionMenu";
import type { ActionHandlers } from "./actionExecutor";
import { summarizeHeapBlockText } from "./heapTextSummary.ts";
import type { ExploreCardDepth } from "./exploreViewSettings.ts";
import { displayBlockType } from "../a2ui/ArtifactAssetViewer";
import { formatHeapCardTimestamp } from "./heapCardTimestamp.ts";

interface HeapBlockCardProps {
    block: HeapBlockListItem;
    isSelected: boolean;
    onClick: (event: React.MouseEvent<HTMLDivElement>) => void;
    onDoubleClick: () => void;
    onDelete?: () => void;
    isRegenerating?: boolean;
    onOpenComments?: () => void;
    cardActions?: ToolbarActionDescriptor[];
    cardActionSelection?: ActionSelectionContext;
    actionHandlers?: ActionHandlers;
    presentationDepth?: ExploreCardDepth;
    children?: React.ReactNode;
}

const BEHAVIOR_BADGE_COLOR: Record<string, string> = {
    urgent: "red",
    pinned: "yellow",
    completed: "green",
    "read-only": "slate",
    collapsed: "slate",
};

const TYPE_COLOR: Record<string, string> = {
    scorecard: "red",
    note: "blue",
    media: "purple",
    task: "green",
    checklist: "green",
    agent_solicitation: "indigo",
    action_plan: "amber",
    compiled_plan: "amber",
    chat_thread: "indigo",
    chart: "cyan",
    a2ui: "cyan",
    gate_summary: "red",
    telemetry: "slate",
    capability: "cyan",
    pattern: "purple",
};

const TYPE_ICON: Record<string, React.ReactNode> = {
    scorecard: <Target className="w-3.5 h-3.5" />,
    note: <FileText className="w-3.5 h-3.5" />,
    media: <Image className="w-3.5 h-3.5" />,
    task: <CheckSquare className="w-3.5 h-3.5" />,
    checklist: <ListChecks className="w-3.5 h-3.5" />,
    agent_solicitation: <ClipboardCheck className="w-3.5 h-3.5" />,
    action_plan: <ClipboardCheck className="w-3.5 h-3.5" />,
    compiled_plan: <ClipboardCheck className="w-3.5 h-3.5" />,
    chat_thread: <MessagesSquare className="w-3.5 h-3.5" />,
    chart: <BarChart className="w-3.5 h-3.5" />,
    a2ui: <Settings className="w-3.5 h-3.5" />,
    gate_summary: <Shield className="w-3.5 h-3.5" />,
    telemetry: <Activity className="w-3.5 h-3.5" />,
    pointer: <MapPin className="w-3.5 h-3.5" />,
    capability: <Cpu className="w-3.5 h-3.5" />,
    pattern: <Network className="w-3.5 h-3.5" />,
};

const UPLOAD_STATUS_ATTRIBUTE_KEYS = new Set([
    "upload_id",
    "resource_ref",
    "upload_status",
    "extraction_status",
    "requested_parser_profile",
    "parser_backend",
    "extraction_confidence",
    "extraction_flags",
    "extraction_result_ref",
    "extraction_summary",
    "extraction_page_count",
    "extraction_block_count",
]);

const EXTRACTION_STATUS_BADGE_CLASS: Record<string, string> = {
    uploaded: "text-blue-300 bg-blue-500/10 border-blue-500/25",
    extracting: "text-indigo-300 bg-indigo-500/10 border-indigo-500/25",
    indexed: "text-emerald-300 bg-emerald-500/10 border-emerald-500/25",
    completed: "text-emerald-300 bg-emerald-500/10 border-emerald-500/25",
    needs_review: "text-yellow-300 bg-yellow-500/10 border-yellow-500/25",
    failed: "text-rose-300 bg-rose-500/10 border-rose-500/25",
};

function prettifyUploadState(value: string): string {
    return value.replace(/_/g, " ");
}

function surfaceToPayloadContent(block: HeapBlockListItem): PayloadContent {
    const surface = block.surfaceJson || {};
    const payloadType = (surface as Record<string, unknown>).payload_type as string
        || (surface as Record<string, unknown>).payloadType as string
        || block.projection.blockType
        || "structured_data";

    return {
        payload_type: payloadType,
        text: (surface as Record<string, unknown>).text as string | undefined,
        plain_text: (surface as Record<string, unknown>).plain_text as string | undefined,
        media: (surface as Record<string, unknown>).media as PayloadContent["media"],
        data: (surface as Record<string, unknown>).data as Record<string, unknown> | undefined,
        structured_data: (surface as Record<string, unknown>).structured_data as Record<string, unknown> | undefined,
        tree: (surface as Record<string, unknown>).tree as PayloadContent["tree"],
        a2ui: (surface as Record<string, unknown>).a2ui as PayloadContent["a2ui"],
        meta: (surface as Record<string, unknown>).meta as Record<string, unknown> | undefined,
        pointer: (surface as Record<string, unknown>).pointer as string | undefined,
        warnings: block.warnings,
    };
}

function surfaceToNestedNode(block: HeapBlockListItem): A2UINode | null {
    const surface = block.surfaceJson as Record<string, unknown> | undefined;
    const candidate = surface?.nestedA2uiTree;
    if (!candidate || typeof candidate !== "object" || Array.isArray(candidate)) {
        return null;
    }
    return candidate as A2UINode;
}

function isReviewRequestPayload(content: PayloadContent): boolean {
    const data = content.structured_data || content.data;
    return Boolean(data && typeof data === "object" && !Array.isArray(data) && data.type === "agent_solicitation");
}

export function HeapBlockCard({
    block,
    isSelected,
    onClick,
    onDoubleClick,
    isRegenerating,
    onOpenComments,
    cardActions = [],
    cardActionSelection,
    actionHandlers,
    presentationDepth = "full",
    children,
}: HeapBlockCardProps) {
    const navigate = useNavigate();
    const commentCount = useUiStore((state) => (state.comments[block.projection.artifactId] || []).length);
    const { projection, surfaceJson, pinnedAt } = block;
    const attributes = projection.attributes || {};
    const blockType = projection.blockType || "note";
    const uploadStatus = typeof attributes.upload_status === "string" ? attributes.upload_status : null;
    const extractionStatus = typeof attributes.extraction_status === "string" ? attributes.extraction_status : null;
    const requestedParserProfile = typeof attributes.requested_parser_profile === "string" ? attributes.requested_parser_profile : null;
    const parserBackend = typeof attributes.parser_backend === "string" ? attributes.parser_backend : null;
    const extractionConfidence = typeof attributes.extraction_confidence === "string" ? attributes.extraction_confidence : null;
    const extractionPageCount = typeof attributes.extraction_page_count === "string" ? attributes.extraction_page_count : null;
    const extractionBlockCount = typeof attributes.extraction_block_count === "string" ? attributes.extraction_block_count : null;
    const displayAttributes = Object.entries(attributes).filter(([key]) => !UPLOAD_STATUS_ATTRIBUTE_KEYS.has(key));
    const surface = (surfaceJson as Record<string, unknown>) || {};
    const behaviors = surface.behaviors as string[] || [];
    const behaviorBadges = pinnedAt && !behaviors.includes("pinned")
        ? [...behaviors, "pinned"]
        : behaviors;
    const blockIcon = TYPE_ICON[blockType] || <FileText className="w-3.5 h-3.5" />;
    const isCollapsed = behaviors.includes("collapsed");
    const emittedAt = projection.emittedAt || projection.updatedAt;
    const summaryText = summarizeHeapBlockText(block);
    const showFullSurface = presentationDepth === "full";
    const showMetadata = presentationDepth !== "title";
    const showRelations = presentationDepth === "full";
    const showPayload = presentationDepth === "full";
    const showChildren = presentationDepth === "full";

    const cardClass = [
        "heap-block-card heap-block-card--surface h-fit flex flex-col rounded-xl overflow-hidden cursor-pointer select-none relative group/card",
        isSelected ? "ring-2 ring-blue-500/50 border-blue-500/30 shadow-blue-500/10 translate-z-10" : "",
        isCollapsed ? "opacity-60" : "",
        isRegenerating ? "animate-pulse ring-2 ring-blue-500/40 shadow-[0_0_20px_rgba(59,130,246,0.3)] bg-blue-950/20" : "",
    ].filter(Boolean).join(" ");

    const payloadContent = surfaceToPayloadContent(block);
    const nestedNode = surfaceToNestedNode(block);
    const author = (attributes.author as string) || "System Intelligence";
    const showInlinePayload = showPayload && !isReviewRequestPayload(payloadContent);

    return (
        <div
            id={`card-${block.projection.artifactId}`}
            className={cardClass.replace("hover:-translate-y-1", "")}
            onClick={onClick}
            onDoubleClick={(e) => { e.stopPropagation(); onDoubleClick(); }}
        >
            {/* Glow Effect on Hover */}
            <div className={`absolute inset-0 bg-linear-to-br from-blue-500/5 to-purple-600/5 opacity-0 group-hover/card:opacity-100 transition-opacity duration-500 pointer-events-none`} />
            <div className={`heap-block-card__header flex justify-between items-start gap-3 ${showFullSurface ? "p-3" : "p-2.5"} bg-cortex-surface-panel/40 border-b border-white/5`}>
                <div className="flex min-w-0 flex-col gap-1">
                    <div className="flex items-center gap-1.5">
                        <span className={`text-[11px] leading-none font-black tracking-widest uppercase flex items-center gap-1.5 ${
                            TYPE_COLOR[blockType] === 'red' ? 'text-red-400' :
                            TYPE_COLOR[blockType] === 'blue' ? 'text-blue-400' :
                            TYPE_COLOR[blockType] === 'green' ? 'text-green-400' :
                            TYPE_COLOR[blockType] === 'cyan' ? 'text-cyan-400' :
                            TYPE_COLOR[blockType] === 'purple' ? 'text-purple-400' :
                            TYPE_COLOR[blockType] === 'yellow' ? 'text-yellow-400' :
                            TYPE_COLOR[blockType] === 'amber' ? 'text-amber-400' :
                            TYPE_COLOR[blockType] === 'indigo' ? 'text-indigo-400' :
                            'text-slate-400'
                        }`}>
                            {blockIcon} {displayBlockType(blockType)}
                        </span>
                        {(blockType === "task" || blockType === "checklist") && (() => {
                            const items = ((surface as Record<string, unknown>).checklist_items as Array<{ done?: boolean }>) || [];
                            if (items.length === 0) return null;
                            const done = items.filter(i => i.done).length;
                            const pct = Math.round((done / items.length) * 100);
                            return (
                                <span className={`ml-1.5 text-[9px] font-mono px-1.5 py-0.5 rounded-full ${
                                    pct === 100 ? "bg-green-500/20 text-green-300" : "bg-slate-800/60 text-slate-400"
                                }`}>{done}/{items.length}</span>
                            );
                        })()}
                        {(block.warnings?.length ?? 0) > 0 && (
                            <AlertTriangle className="w-3 h-3 text-amber-500 animate-pulse ml-1" />
                        )}
                    </div>
                    <div className="flex items-center gap-1.5 text-slate-500 mt-0.5">
                        <span className="text-[9px] font-bold uppercase tracking-tighter leading-none">
                            {formatHeapCardTimestamp(emittedAt)}
                        </span>
                    </div>
                </div>
                <div className="flex max-w-[45%] shrink-0 flex-wrap justify-end gap-1.5 sm:max-w-[65%]">
                    {behaviorBadges.map((behavior) => {
                        const bColor = BEHAVIOR_BADGE_COLOR[behavior] || "slate";
                        return (
                            <span
                                key={behavior}
                                className={`text-[8px] uppercase font-bold tracking-widest px-1.5 py-0.5 rounded-sm shadow-sm border ${
                                    bColor === "red" ? "text-red-400 bg-red-400/5 border-red-400/20" :
                                    bColor === "yellow" ? "text-yellow-400 bg-yellow-400/5 border-yellow-400/20" :
                                    bColor === "green" ? "text-green-400 bg-green-400/5 border-green-400/20" :
                                    "text-slate-400 bg-slate-800/50 border-slate-700/50"
                                }`}
                            >
                                {behavior}
                            </span>
                        );
                    })}
                </div>
            </div>

            <div className={`bg-transparent flex-1 flex flex-col relative z-10 ${showFullSurface ? "p-3" : "p-2.5"}`}>
                {showMetadata && (
                    <div className="flex items-center justify-between mb-2 gap-2">
                        {showFullSurface ? (
                            <NdlMetadataBlock
                                versionChain={typeof surface.version === "string" || typeof surface.version === "number" ? String(surface.version) : undefined}
                                phase={typeof surface.phase === "string" ? surface.phase : undefined}
                                confidence={typeof surface.confidence === "number" ? surface.confidence : undefined}
                                authorityScope={typeof surface.authority_scope === "string" ? surface.authority_scope : undefined}
                                compact
                            />
                        ) : (
                            <div className="text-[10px] uppercase tracking-[0.24em] text-cortex-500">
                                {displayBlockType(blockType)}
                            </div>
                        )}
                        {cardActions.length > 0 && cardActionSelection && actionHandlers && (
                            <div className="flex items-center gap-1.5 opacity-60 hover:opacity-100 transition-opacity">
                                <HeapCardActionMenu
                                    actions={cardActions}
                                    selection={cardActionSelection}
                                    handlers={actionHandlers}
                                />
                            </div>
                        )}
                    </div>
                )}

                <h3 className={`font-bold text-slate-100 px-0.5 leading-6 ${presentationDepth === "title" ? "text-sm" : "text-[15px]"} ${showFullSurface ? "mb-2" : "mb-1.5"} line-clamp-2`}>
                    {projection.title}
                </h3>

                {presentationDepth !== "title" && summaryText !== projection.title && (
                    <p className={`px-0.5 text-slate-300/80 ${presentationDepth === "full" ? "text-sm leading-6 line-clamp-3" : "text-xs leading-5 line-clamp-2"}`}>
                        {summaryText}
                    </p>
                )}

                {showFullSurface && !isCollapsed && displayAttributes.length > 0 && (
                    <div className="flex flex-wrap gap-1.5 my-2.5">
                        {displayAttributes.map(([k, v]) => (
                            <div key={k} className="inline-flex items-center text-[9px] bg-slate-950/60 rounded-md px-1.5 py-0.5" title={`${k}: ${v}`}>
                                <span className="text-slate-500 mr-1 font-bold">{k}:</span>
                                <span className="font-mono text-slate-400 truncate max-w-[100px]">{String(v)}</span>
                            </div>
                        ))}
                    </div>
                )}

                {showFullSurface && !isCollapsed && blockType === "upload" && (
                    <div className="my-3 rounded-xl border border-white/8 bg-slate-950/40 p-3">
                        <div className="text-[10px] font-black uppercase tracking-[0.22em] text-cortex-500">Extraction</div>
                        <div className="mt-2 flex flex-wrap gap-1.5">
                            {uploadStatus && (
                                <span className="text-[10px] font-mono border border-blue-500/20 bg-blue-500/10 text-blue-300 px-2 py-0.5 rounded-full uppercase tracking-[0.18em]">
                                    {prettifyUploadState(uploadStatus)}
                                </span>
                            )}
                            {extractionStatus && (
                                <span className={`text-[10px] font-mono border px-2 py-0.5 rounded-full uppercase tracking-[0.18em] ${EXTRACTION_STATUS_BADGE_CLASS[extractionStatus] ?? "text-slate-300 bg-slate-500/10 border-slate-500/20"}`}>
                                    {prettifyUploadState(extractionStatus)}
                                </span>
                            )}
                            {requestedParserProfile && (
                                <span className="text-[10px] font-mono border border-white/10 bg-white/4 text-slate-300 px-2 py-0.5 rounded-full uppercase tracking-[0.18em]">
                                    requested {requestedParserProfile}
                                </span>
                            )}
                            {parserBackend && (
                                <span className="text-[10px] font-mono border border-white/10 bg-white/4 text-slate-300 px-2 py-0.5 rounded-full uppercase tracking-[0.18em]">
                                    resolved {parserBackend}
                                </span>
                            )}
                        </div>
                        {(extractionConfidence || extractionPageCount || extractionBlockCount) && (
                            <div className="mt-3 flex flex-wrap gap-3 text-[11px] text-slate-400">
                                {extractionConfidence && <span>confidence {extractionConfidence}</span>}
                                {extractionPageCount && <span>pages {extractionPageCount}</span>}
                                {extractionBlockCount && <span>blocks {extractionBlockCount}</span>}
                            </div>
                        )}
                    </div>
                )}

                {showInlinePayload && !isCollapsed && !nestedNode && !children && <PayloadRenderer content={payloadContent} />}
                {showInlinePayload && !isCollapsed && nestedNode && (
                    <div className="mt-3 pl-3 bg-white/5 rounded-lg flex flex-col gap-3">
                        <A2UIInterpreter node={nestedNode} />
                    </div>
                )}
                {showChildren && !isCollapsed && children && (
                    <div className="mt-3 pl-3 bg-white/5 rounded-lg flex flex-col gap-3">{children}</div>
                )}
            </div>

            {/* Footer */}
            <div className={`px-3 ${showFullSurface ? "py-2" : "py-1.5"} bg-cortex-surface-base/40 border-t border-white/5 flex flex-wrap gap-1.5 items-center`}>
                <div className="flex min-w-0 items-center gap-1.5 opacity-80 hover:opacity-100 transition-opacity mr-auto">
                    <div className="w-4 h-4 rounded-full bg-slate-800 flex items-center justify-center text-[8px] font-black text-slate-300 shadow-inner shrink-0 leading-none">
                        {author.substring(0, 2).toUpperCase()}
                    </div>
                    <span className="truncate text-[9px] font-bold text-slate-500 uppercase tracking-widest leading-none">
                        {author.split(" ")[0]}
                    </span>
                </div>
                {blockType === "upload" && uploadStatus && (
                    <span className="text-[10px] font-mono border border-blue-500/20 bg-blue-500/10 text-blue-300 px-2 py-0.5 rounded-full uppercase tracking-[0.18em]">
                        {prettifyUploadState(uploadStatus)}
                    </span>
                )}
                {blockType === "upload" && extractionStatus && (
                    <span className={`text-[10px] font-mono border px-2 py-0.5 rounded-full uppercase tracking-[0.18em] ${EXTRACTION_STATUS_BADGE_CLASS[extractionStatus] ?? "text-slate-300 bg-slate-500/10 border-slate-500/20"}`}>
                        {prettifyUploadState(extractionStatus)}
                    </span>
                )}
                {blockType === "upload" && parserBackend && (
                    <span className="text-[10px] font-mono border border-white/10 bg-white/4 text-slate-300 px-2 py-0.5 rounded-full uppercase tracking-[0.18em]">
                        {parserBackend}
                    </span>
                )}
                {showFullSurface && payloadContent.payload_type !== blockType && (
                    <span className="heap-payload-label text-[10px] font-mono text-slate-600 flex items-center gap-1 ml-2">
                        <FileText className="w-2.5 h-2.5" />
                        {payloadContent.payload_type}
                    </span>
                )}
                {showFullSurface && projection.pageLinks?.map((pageLink) => (
                    <button
                        key={pageLink}
                        onClick={(e) => { e.stopPropagation(); navigate(`/?space_id=${pageLink}`); }}
                        className="heap-page-link-chip text-[10px] font-mono bg-cyan-900/10 text-cyan-400 px-2 py-0.5 rounded-full cursor-pointer hover:bg-cyan-500/10 hover:text-cyan-300 transition-all inline-flex items-center gap-1"
                    >
                        <ArrowUpRight className="w-2.5 h-2.5 opacity-60" />
                        {pageLink.substring(0, 8)}
                    </button>
                ))}
                {showRelations && projection.mentionsInline?.map((m) => (
                    <button
                        key={m}
                        onClick={(e) => { e.stopPropagation(); navigate(`/?block_id=${m}`); }}
                        className="heap-mention-chip text-[10px] font-mono bg-indigo-900/10 text-indigo-400 px-2 py-0.5 rounded-full cursor-pointer hover:bg-indigo-500/10 hover:text-indigo-300 transition-all inline-flex items-center gap-1"
                    >
                        <Link2 className="w-2.5 h-2.5 opacity-60" />
                        {m.substring(0, 8)}
                    </button>
                ))}
                {showRelations && projection.tags?.map((t) => (
                    <span key={t} className="heap-tag-chip max-w-[8rem] truncate text-[10px] bg-white/5 text-slate-500 px-2 py-0.5 rounded-full cursor-default font-medium">#{t}</span>
                ))}
                <div className="flex items-center gap-1 ml-auto text-slate-500 hover:text-slate-300 transition-colors">
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            onOpenComments?.();
                        }}
                        className="heap-card-comment-btn flex items-center gap-1.5 px-2 py-1 rounded-md hover:bg-white/5 transition-all group/comments"
                        title="View Discussions"
                    >
                        <MessageSquare className="w-3.5 h-3.5 text-slate-500 group-hover/comments:text-blue-400 transition-colors" />
                        <span className="text-[10px] font-mono text-slate-500 group-hover/comments:text-blue-400">
                            {commentCount}
                        </span>
                    </button>
                </div>
            </div>
        </div>
    );
}

export { surfaceToPayloadContent };
