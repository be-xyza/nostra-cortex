import React, { useEffect, useMemo, useState } from "react";
import type {
    HeapBlockListItem,
    ActionSelectionContext,
    HeapUploadExtractionRunDetail,
    HeapUploadExtractionRunRecord,
    HeapUploadExtractionStatusResponse,
    HeapUploadParserProfileRecord,
} from "../../contracts";
import { workbenchApi } from "../../api";
import { PayloadRenderer, PayloadContent } from './PayloadRenderer';
import { displayBlockType } from "../a2ui/ArtifactAssetViewer";
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
import { useLayoutPreferences, applyOrder, EMPTY_PREFS } from "../../store/layoutPreferences";
import { Check, ChevronLeft, Copy, GripVertical } from "lucide-react";
import { AmbientGraphBackground } from "./AmbientGraphBackground";
import { createHeapDetailActionHandlers } from "./heapDetailActions";
import { describeHeapRelation, type HeapRelationItem } from "./heapRelations";
import { TaskRoutingDecisionCard } from "./TaskRoutingDecisionCard";
import type { TaskRouteDecision, TaskRouteId } from "./taskRouting.ts";
import type { TaskRoutingContext } from "./initiativeKickoffTemplates.ts";
import { TaskRouteLineageCard } from "./TaskRouteLineageCard";
import { buildTaskRouteLineageSnapshot } from "./taskRouting.ts";
import { buildSolicitationRenderModel } from "./solicitationRenderModel";
// Removed local WorkbenchNamingModal import, using global one in ShellLayout

type TabType = 'preview' | 'attributes' | 'relations' | 'code';
type ParserComparisonDecision = "structure" | "layout" | "needs_rerun";

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

function parserProfileTone(profile: string): string {
    switch (profile) {
        case "docling":
            return "border-cyan-300/20 bg-cyan-300/10 text-cyan-100";
        case "liteparse":
            return "border-violet-300/20 bg-violet-300/10 text-violet-100";
        case "markitdown":
            return "border-emerald-300/20 bg-emerald-300/10 text-emerald-100";
        case "auto":
        default:
            return "border-white/10 bg-white/[0.04] text-white/75";
    }
}

function statusTone(status: string): string {
    switch (status) {
        case "completed":
        case "indexed":
            return "border-emerald-300/20 bg-emerald-300/10 text-emerald-100";
        case "needs_review":
            return "border-amber-300/20 bg-amber-300/10 text-amber-100";
        case "failed":
            return "border-rose-300/20 bg-rose-300/10 text-rose-100";
        case "running":
        case "extracting":
            return "border-indigo-300/20 bg-indigo-300/10 text-indigo-100";
        case "uploaded":
        case "submitted":
        default:
            return "border-blue-300/20 bg-blue-300/10 text-blue-100";
    }
}

function resolveMetricCardClassName(tone: "cyan" | "violet" | "emerald" | "amber" | "rose" | "slate" = "slate"): string {
    switch (tone) {
        case "cyan":
            return "border-cyan-400/18 bg-cyan-500/10 text-cyan-100";
        case "violet":
            return "border-violet-400/18 bg-violet-500/10 text-violet-100";
        case "emerald":
            return "border-emerald-400/18 bg-emerald-500/10 text-emerald-100";
        case "amber":
            return "border-amber-400/18 bg-amber-500/10 text-amber-100";
        case "rose":
            return "border-rose-400/18 bg-rose-500/10 text-rose-100";
        case "slate":
        default:
            return "border-white/8 bg-white/[0.04] text-white/85";
    }
}

function prettifyUploadState(value: string): string {
    return value.replace(/_/g, " ");
}

function isRecord(value: unknown): value is Record<string, unknown> {
    return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function asReadableToken(value: unknown, fallback = "Unavailable"): string {
    if (typeof value !== "string" || !value.trim()) {
        return fallback;
    }
    return value
        .trim()
        .split(/[_\-.]+/g)
        .filter(Boolean)
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(" ");
}

function buildReadableHeapSummary(content: PayloadContent, fallbackText: string): string {
    const structuredData = isRecord(content.structured_data)
        ? content.structured_data
        : isRecord(content.data)
            ? content.data
            : null;
    const solicitationModel = structuredData ? buildSolicitationRenderModel(structuredData) : null;
    const candidate =
        solicitationModel?.summary
        ?? (typeof content.plain_text === "string" ? content.plain_text : null)
        ?? (typeof content.text === "string" ? content.text : null)
        ?? fallbackText;
    const normalized = candidate.replace(/\s+/g, " ").trim();
    if (!normalized || /^[\[{]/.test(normalized)) {
        return "Structured heap payload. Use Preview for the readable review surface and Code for raw data.";
    }
    return normalized;
}

function shortenId(value: string, length = 12): string {
    return value.length <= length ? value : `${value.slice(0, length)}...`;
}

interface HeapDetailModalProps {
    block: HeapBlockListItem;
    allBlocks: HeapBlockListItem[];
    onClose: () => void;
    onUploadExtractionUpdated?: (message: string) => void;
    onViewDiscussion: (artifactId: string) => void;
    onNavigateToBlock: (artifactId: string, context?: { relation?: string; title?: string }) => void;
    onRelationSaved: (artifactId: string) => void;
    onRegenerate?: (selection: ActionSelectionContext) => void;
    onTaskRouteSelected?: (
        routeId: TaskRouteId,
        context: TaskRoutingContext,
        decision: TaskRouteDecision,
    ) => void;
    ambientGraphVariant?: string;
    navigationTrail?: Array<{ artifactId: string; title: string; relation?: string }>;
    onNavigateBack?: () => void;
}

export function HeapDetailModal({
    block,
    allBlocks,
    onClose,
    onUploadExtractionUpdated,
    onViewDiscussion,
    onNavigateToBlock,
    onRelationSaved,
    onRegenerate,
    onTaskRouteSelected,
    ambientGraphVariant,
    navigationTrail = [],
    onNavigateBack,
}: HeapDetailModalProps) {
    const { projection, surfaceJson } = block;
    const surface = (surfaceJson as Record<string, unknown>) || {};
    const sessionUser = useUiStore((state) => state.sessionUser);
    const availableSpaces = useAvailableSpaces();
    const activeSpaceId = useActiveSpaceContext();
    const [relationLoading, setRelationLoading] = useState(false);
    const layoutPrefs = useLayoutPreferences((s) => s.cache[activeSpaceId] ?? EMPTY_PREFS);
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
    const [uploadParserProfiles, setUploadParserProfiles] = useState<HeapUploadParserProfileRecord[]>([]);
    const [uploadExtractionStatus, setUploadExtractionStatus] = useState<HeapUploadExtractionStatusResponse | null>(null);
    const [uploadExtractionRuns, setUploadExtractionRuns] = useState<HeapUploadExtractionRunRecord[]>([]);
    const [selectedRunJobId, setSelectedRunJobId] = useState<string | null>(null);
    const [compareRunJobId, setCompareRunJobId] = useState<string | null>(null);
    const [selectedRunDetail, setSelectedRunDetail] = useState<HeapUploadExtractionRunDetail | null>(null);
    const [compareRunDetail, setCompareRunDetail] = useState<HeapUploadExtractionRunDetail | null>(null);
    const [uploadExtractionError, setUploadExtractionError] = useState<string | null>(null);
    const [selectedParserProfile, setSelectedParserProfile] = useState<string>("auto");
    const [rerunSubmitting, setRerunSubmitting] = useState(false);
    const [comparisonDecision, setComparisonDecision] = useState<ParserComparisonDecision>("structure");
    const [comparisonNotes, setComparisonNotes] = useState("");
    const [comparisonSaving, setComparisonSaving] = useState(false);
    const [comparisonMessage, setComparisonMessage] = useState<string | null>(null);
    const [comparisonError, setComparisonError] = useState<string | null>(null);
    const [copiedField, setCopiedField] = useState<string | null>(null);
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
    const uploadId = typeof projection.attributes?.upload_id === "string" ? projection.attributes.upload_id : null;
    const requestedParserProfileAttribute =
        typeof projection.attributes?.requested_parser_profile === "string"
            ? projection.attributes.requested_parser_profile
            : null;
    const resolvedParserBackendAttribute =
        typeof projection.attributes?.parser_backend === "string"
            ? projection.attributes.parser_backend
            : null;

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

    useEffect(() => {
        if (!uploadId) {
            setUploadParserProfiles([]);
            setUploadExtractionStatus(null);
            setUploadExtractionRuns([]);
            setSelectedRunJobId(null);
            setCompareRunJobId(null);
            setSelectedRunDetail(null);
            setCompareRunDetail(null);
            setUploadExtractionError(null);
            setSelectedParserProfile("auto");
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
            })
            .catch(() => {
                if (!cancelled) {
                    setUploadParserProfiles([]);
                }
            });
        void workbenchApi
            .getHeapUploadExtractionStatus(uploadId, actorRole, actorId)
            .then((response) => {
                if (cancelled) {
                    return;
                }
                setUploadExtractionStatus(response);
                setUploadExtractionError(null);
                setSelectedParserProfile(response.requested_parser_profile || "auto");
            })
            .catch((err) => {
                if (cancelled) {
                    return;
                }
                setUploadExtractionStatus(null);
                setUploadExtractionError(err instanceof Error ? err.message : String(err));
                setSelectedParserProfile(requestedParserProfileAttribute || "auto");
            });
        void workbenchApi
            .getHeapUploadExtractionRuns(uploadId, actorRole, actorId)
            .then((response) => {
                if (cancelled) {
                    return;
                }
                const items = response.items || [];
                setUploadExtractionRuns(items);
                setSelectedRunJobId((current) => current || items[0]?.job_id || null);
                setCompareRunJobId((current) => {
                    if (current) {
                        return current;
                    }
                    if (items.length > 1) {
                        return items[1].job_id;
                    }
                    return null;
                });
            })
            .catch(() => {
                if (!cancelled) {
                    setUploadExtractionRuns([]);
                }
            });
        return () => {
            cancelled = true;
        };
    }, [requestedParserProfileAttribute, sessionUser?.actorId, sessionUser?.role, uploadId]);

    useEffect(() => {
        if (!uploadId || !selectedRunJobId) {
            setSelectedRunDetail(null);
            return;
        }
        const actorRole = sessionUser?.role || "operator";
        const actorId = sessionUser?.actorId || "cortex-web";
        let cancelled = false;
        void workbenchApi
            .getHeapUploadExtractionRun(uploadId, selectedRunJobId, actorRole, actorId)
            .then((response) => {
                if (!cancelled) {
                    setSelectedRunDetail(response);
                }
            })
            .catch(() => {
                if (!cancelled) {
                    setSelectedRunDetail(null);
                }
            });
        return () => {
            cancelled = true;
        };
    }, [selectedRunJobId, sessionUser?.actorId, sessionUser?.role, uploadId]);

    useEffect(() => {
        if (!uploadId || !compareRunJobId) {
            setCompareRunDetail(null);
            return;
        }
        const actorRole = sessionUser?.role || "operator";
        const actorId = sessionUser?.actorId || "cortex-web";
        let cancelled = false;
        void workbenchApi
            .getHeapUploadExtractionRun(uploadId, compareRunJobId, actorRole, actorId)
            .then((response) => {
                if (!cancelled) {
                    setCompareRunDetail(response);
                }
            })
            .catch(() => {
                if (!cancelled) {
                    setCompareRunDetail(null);
                }
            });
        return () => {
            cancelled = true;
        };
    }, [compareRunJobId, sessionUser?.actorId, sessionUser?.role, uploadId]);

    const actionHandlers: ActionHandlers = createHeapDetailActionHandlers({
        artifactId: projection.artifactId,
        blockType: projection.blockType,
        onClose,
        onViewDiscussion,
        onRegenerate,
        onToggleRelations: () => {
            setRelationEditorOpen((open) => !open);
            setActiveTab("relations");
        },
    });

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
    const routeLineage = useMemo(
        () => buildTaskRouteLineageSnapshot(block, relationIndex),
        [block, relationIndex],
    );
    const outboundLinks = relationIndex.outboundLinks;
    const outboundMentions = relationIndex.outboundMentions;
    const backlinks = relationIndex.backlinks;
    const tagNeighbors = relationIndex.tagNeighbors;
    const semanticLineage = relationIndex.semanticLineage;
    const plainText = extractPlainText(payloadContent);
    const readableSummary = buildReadableHeapSummary(payloadContent, plainText);
    const summaryWordCount = readableSummary.trim().length ? readableSummary.trim().split(/\s+/).length : 0;
    const wordCount = plainText.trim().length ? plainText.trim().split(/\s+/).length : summaryWordCount;
    const characterCount = plainText.length || readableSummary.length;
    const resolvedSpaceId = useMemo(() => {
        const surfaceData = surface as Record<string, unknown>;
        const structured = (
            (surfaceData.structured_data as Record<string, unknown> | undefined)
            ?? (surfaceData.data as Record<string, unknown> | undefined)
            ?? surfaceData
        );
        const spaceValue = projection.spaceId ?? structured.space_id ?? structured.spaceId ?? surfaceData.space_id;
        return typeof spaceValue === "string" && spaceValue.trim().length > 0 ? spaceValue : "unknown";
    }, [projection.spaceId, surface]);
    const resolvedSpaceRecord = availableSpaces.find((space) => space.id === resolvedSpaceId);
    const activeSpaceRecord = availableSpaces.find((space) => space.id === activeSpaceId);
    const displaySpaceLabel = resolvedSpaceId !== "unknown"
        ? (resolvedSpaceRecord?.name || resolvedSpaceId)
        : (activeSpaceId !== "meta" ? `Viewing in ${activeSpaceRecord?.name || activeSpaceId}` : null);
    const structuredPayload = isRecord(payloadContent.structured_data)
        ? payloadContent.structured_data
        : isRecord(payloadContent.data)
            ? payloadContent.data
            : null;
    const solicitationModel = structuredPayload ? buildSolicitationRenderModel(structuredPayload) : null;
    const capabilityLabels = solicitationModel?.capabilityLabels ?? [];
    const authorityLabel = solicitationModel?.authorityScopeLabel
        ? asReadableToken(solicitationModel.authorityScopeLabel)
        : asReadableToken(surface.authority_scope, "Local Review");
    const requestedRoleLabel = solicitationModel?.requestedRoleLabel
        ? asReadableToken(solicitationModel.requestedRoleLabel)
        : "Not requested";
    const productionReadinessLabel = String(projection.attributes?.bootstrap ?? "") === "localhost_dev"
        ? "Local dev fixture"
        : source === "fallback"
            ? "Fallback actions"
            : "Live action plan";
    const artifactIdDisplay = shortenId(projection.artifactId);
    const copyStateForArtifactId = copiedField === "artifact-id"
        ? "copied"
        : copiedField === "artifact-id-failed"
            ? "failed"
            : "idle";
    const hasCapabilityLabels = capabilityLabels.length > 0;
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
    const currentTrailEntry = navigationTrail[navigationTrail.length - 1] ?? null;
    const effectiveUploadExtractionStatus = uploadExtractionStatus ?? (uploadId ? {
        job_id: "",
        upload_id: uploadId,
        status:
            typeof projection.attributes?.extraction_status === "string"
                ? (projection.attributes.extraction_status as HeapUploadExtractionStatusResponse["status"])
                : "submitted",
        requested_parser_profile: requestedParserProfileAttribute ?? undefined,
        parser_backend: resolvedParserBackendAttribute ?? undefined,
        confidence:
            typeof projection.attributes?.extraction_confidence === "string"
                ? Number(projection.attributes.extraction_confidence)
                : undefined,
        flags:
            typeof projection.attributes?.extraction_flags === "string"
                ? projection.attributes.extraction_flags.split(",").map((flag) => flag.trim()).filter(Boolean)
                : undefined,
        result_ref:
            typeof projection.attributes?.extraction_result_ref === "string"
                ? projection.attributes.extraction_result_ref
                : undefined,
        summary:
            typeof projection.attributes?.extraction_summary === "string"
                ? projection.attributes.extraction_summary
                : undefined,
        page_count:
            typeof projection.attributes?.extraction_page_count === "string"
                ? Number(projection.attributes.extraction_page_count)
                : undefined,
        block_count:
            typeof projection.attributes?.extraction_block_count === "string"
                ? Number(projection.attributes.extraction_block_count)
                : undefined,
        last_updated_at: undefined,
    } : null);
    const selectableCompareRuns = useMemo(
        () => uploadExtractionRuns.filter((run) => run.job_id !== selectedRunJobId),
        [selectedRunJobId, uploadExtractionRuns],
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

    const navigateRelation = (item: HeapRelationItem) => {
        const relation = item.relations?.[0];
        if (!resolveHeapRelationBlock(item.id, allBlocks)) {
            return;
        }
        onNavigateToBlock(item.id, { relation, title: item.title });
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

    const rerunUploadExtraction = async () => {
        if (!uploadId) {
            return;
        }
        const actorRole = sessionUser?.role || "operator";
        const actorId = sessionUser?.actorId || "cortex-web";
        const requestedProfile = selectedParserProfile.trim() || "auto";
        const sleep = (ms: number) => new Promise((resolve) => window.setTimeout(resolve, ms));
        try {
            setRerunSubmitting(true);
            setUploadExtractionError(null);
            const queued = await workbenchApi.triggerHeapUploadExtraction(
                uploadId,
                requestedProfile,
                actorRole,
                actorId,
            );
            setUploadExtractionStatus({
                job_id: queued.job_id,
                upload_id: queued.upload_id,
                status: queued.status,
                requested_parser_profile: queued.requested_parser_profile,
            });
            let currentStatus = await workbenchApi.getHeapUploadExtractionStatus(uploadId, actorRole, actorId);
            let attempts = 0;
            while (!["completed", "needs_review", "failed"].includes(currentStatus.status) && attempts < 10) {
                await sleep(750);
                currentStatus = await workbenchApi.getHeapUploadExtractionStatus(uploadId, actorRole, actorId);
                attempts += 1;
            }
            setUploadExtractionStatus(currentStatus);
            const runsResponse = await workbenchApi.getHeapUploadExtractionRuns(uploadId, actorRole, actorId);
            setUploadExtractionRuns(runsResponse.items || []);
            setSelectedRunJobId(currentStatus.job_id || runsResponse.items?.[0]?.job_id || null);
            setCompareRunJobId((current) => {
                if (current && runsResponse.items.some((item) => item.job_id === current)) {
                    return current;
                }
                return runsResponse.items.find((item) => item.job_id !== (currentStatus.job_id || runsResponse.items?.[0]?.job_id))?.job_id || null;
            });
            const message = `Extraction rerun finished with ${prettifyUploadState(currentStatus.status)} using ${parserProfileLabel(currentStatus.parser_backend || requestedProfile)}.`;
            onUploadExtractionUpdated?.(message);
        } catch (err) {
            setUploadExtractionError(err instanceof Error ? err.message : String(err));
        } finally {
            setRerunSubmitting(false);
        }
    };

    const saveComparisonFinding = async () => {
        if (!uploadId || !selectedRunDetail || !compareRunDetail) {
            return;
        }
        const actorRole = sessionUser?.role || "operator";
        const actorId = sessionUser?.actorId || "cortex-web";
        const preferredLabel =
            comparisonDecision === "structure"
                ? "better for structure"
                : comparisonDecision === "layout"
                    ? "better for layout"
                    : "needs rerun";
        const primaryParser = parserProfileLabel(
            selectedRunDetail.parser_backend || selectedRunDetail.requested_parser_profile || "unknown",
        );
        const compareParser = parserProfileLabel(
            compareRunDetail.parser_backend || compareRunDetail.requested_parser_profile || "unknown",
        );
        const lines = [
            `Parser comparison finding for ${projection.title || projection.artifactId}`,
            "",
            `Upload: ${uploadId}`,
            `Primary run: ${selectedRunDetail.job_id} (${primaryParser})`,
            `Compare run: ${compareRunDetail.job_id} (${compareParser})`,
            `Decision: ${preferredLabel}`,
            "",
            `${primaryParser}: ${summarizeRunDetail(selectedRunDetail)}`,
            `${compareParser}: ${summarizeRunDetail(compareRunDetail)}`,
        ];
        if (comparisonNotes.trim()) {
            lines.push("", "Reviewer notes:", comparisonNotes.trim());
        }

        try {
            setComparisonSaving(true);
            setComparisonError(null);
            setComparisonMessage(null);
            await workbenchApi.emitHeapBlock(
                {
                    schema_version: "1.0.0",
                    mode: "heap",
                    space_id: resolvedSpaceId,
                    source: {
                        agent_id: "cortex-web.parser-compare",
                        emitted_at: new Date().toISOString(),
                    },
                    block: {
                        type: "note",
                        title: `Parser comparison: ${projection.title || projection.artifactId}`,
                        attributes: {
                            origin: "parser_comparison_finding",
                            upload_id: uploadId,
                            primary_job_id: selectedRunDetail.job_id,
                            compare_job_id: compareRunDetail.job_id,
                            preferred_for: comparisonDecision,
                            primary_parser: selectedRunDetail.parser_backend || selectedRunDetail.requested_parser_profile || "unknown",
                            compare_parser: compareRunDetail.parser_backend || compareRunDetail.requested_parser_profile || "unknown",
                        },
                    },
                    content: {
                        payload_type: "rich_text",
                        rich_text: {
                            plain_text: lines.join("\n"),
                        },
                    },
                    relations: {
                        page_links: [{ to_block_id: projection.artifactId }],
                    },
                },
                actorRole,
                actorId,
            );
            setComparisonMessage("Comparison finding saved into the heap.");
            setComparisonNotes("");
            onRelationSaved(projection.artifactId);
        } catch (err) {
            setComparisonError(err instanceof Error ? err.message : String(err));
        } finally {
            setComparisonSaving(false);
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

    const handleCopyArtifactId = async () => {
        try {
            await navigator.clipboard.writeText(projection.artifactId);
            setCopiedField("artifact-id");
            window.setTimeout(() => {
                setCopiedField((current) => (current === "artifact-id" ? null : current));
            }, 1500);
        } catch {
            setCopiedField("artifact-id-failed");
            window.setTimeout(() => {
                setCopiedField((current) => (current === "artifact-id-failed" ? null : current));
            }, 1500);
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-slate-950/88 p-2 backdrop-blur-md animate-fade-in sm:items-center sm:p-4" onClick={onClose}>
            <div
                className="heap-modal-content relative flex max-h-[96dvh] w-full max-w-[min(96vw,96rem)] flex-col overflow-hidden rounded-2xl border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.98),rgba(2,6,23,0.95))] shadow-[0_0_0_1px_rgba(255,255,255,0.02),0_32px_100px_-42px_rgba(0,0,0,0.95)] animate-slide-up sm:max-h-[92vh] sm:rounded-[1.9rem]"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Soft Background Layer */}
                <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden opacity-55">
                    <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_0%,rgba(34,211,238,0.14),transparent_40%),radial-gradient(circle_at_95%_8%,rgba(168,85,247,0.10),transparent_26%),radial-gradient(circle_at_0%_20%,rgba(59,130,246,0.08),transparent_25%)]" />
                    {ambientGraphVariant && ambientGraphVariant !== "off" && (
                        <AmbientGraphBackground 
                            visible={true} 
                            variant={ambientGraphVariant as any} 
                            spaceId={block.projection.spaceId || "01ARZ3NDEKTSV4RRFFQ69G5FAV"}
                        />
                    )}
                </div>

                {/* Header */}
                <div className="relative z-10 border-b border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.82),rgba(15,23,42,0.54))] backdrop-blur-xl">
                    <div className="flex flex-col items-stretch justify-between gap-3 px-4 py-3 sm:flex-row sm:items-start sm:px-6 sm:py-4">
                        <div className="min-w-0 flex-1">
                            <div className="flex flex-wrap items-center gap-1.5 sm:gap-2">
                                <span className="rounded-full border border-cyan-400/20 bg-cyan-500/10 px-2.5 py-0.5 text-[10px] font-semibold uppercase tracking-[0.18em] text-cyan-100 sm:px-3 sm:py-1">
                                    Artifact detail
                                </span>
                                <NdlMetadataBlock
                                    typeIndicator={displayBlockType(projection.blockType || "note")}
                                    versionChain={typeof surface.version === "string" || typeof surface.version === "number" ? String(surface.version) : undefined}
                                    phase={typeof surface.phase === "string" ? surface.phase : undefined}
                                    confidence={typeof surface.confidence === "number" ? surface.confidence : undefined}
                                    authorityScope={typeof surface.authority_scope === "string" ? surface.authority_scope : undefined}
                                    compact
                                />
                                <button
                                    type="button"
                                    onClick={handleCopyArtifactId}
                                    title={projection.artifactId}
                                    className="inline-flex items-center gap-1 rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-0.5 font-mono text-[10px] tracking-[0.14em] text-white/60 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white sm:px-3 sm:py-1"
                                >
                                    <span>{artifactIdDisplay}</span>
                                    {copyStateForArtifactId === "copied" ? <Check className="h-3 w-3 text-emerald-300" /> : <Copy className="h-3 w-3" />}
                                </button>
                                {uploadId && (
                                    <span className="rounded-full border border-violet-400/20 bg-violet-500/10 px-2.5 py-0.5 text-[10px] font-semibold uppercase tracking-[0.18em] text-violet-100 sm:px-3 sm:py-1">
                                        Upload-backed
                                    </span>
                                )}
                            </div>
                            <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] leading-5 text-slate-300/70 sm:mt-2.5">
                                {displaySpaceLabel ? (
                                    <>
                                        <span className="font-medium text-white/68">
                                            {displaySpaceLabel}
                                        </span>
                                        <span className="hidden h-1 w-1 rounded-full bg-white/20 sm:inline-block" aria-hidden="true" />
                                    </>
                                ) : null}
                                <span className="font-medium text-white/68">
                                    Added {new Date(projection.emittedAt || projection.updatedAt).toLocaleString()}
                                </span>
                                {currentTrailEntry && (
                                    <>
                                        {onNavigateBack && (
                                            <button
                                                type="button"
                                                onClick={onNavigateBack}
                                                className="inline-flex items-center gap-1 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1.5 font-semibold text-white/80 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                                            >
                                                <ChevronLeft className="h-3.5 w-3.5" />
                                                Back
                                            </button>
                                        )}
                                        <span className="rounded-full border border-white/10 bg-white/[0.03] px-3 py-1.5 uppercase tracking-[0.18em] text-white/45">
                                            Opened from
                                        </span>
                                        <span className="rounded-full border border-white/10 bg-slate-950/50 px-3 py-1.5 font-medium text-white/82">
                                            {currentTrailEntry.title}
                                        </span>
                                        {currentTrailEntry.relation && (
                                            <span className="rounded-full border border-violet-400/20 bg-violet-400/10 px-3 py-1.5 font-medium text-violet-100">
                                                via {describeHeapRelation(currentTrailEntry.relation)}
                                            </span>
                                        )}
                                    </>
                                )}
                            </div>
                        </div>
                        <div className="flex w-full items-start justify-between gap-2 sm:ml-4 sm:w-auto sm:justify-end">
                            <div className="flex min-w-0 flex-1 flex-wrap items-start gap-2 sm:flex-none sm:justify-end">
                                {headerZonePlan && (
                                    <ActionZoneRenderer
                                        actions={headerZonePlan.actions}
                                        layoutHint={headerZonePlan.layoutHint}
                                        onActionClick={(action) => executeHeapAction(action, selectionContext, actionHandlers)}
                                    />
                                )}
                            </div>
                            <button
                                className="rounded-full border border-white/10 bg-white/[0.04] p-2 text-slate-400 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white hover:rotate-90"
                                onClick={onClose}
                                aria-label="Close artifact detail"
                            >
                                ✕
                            </button>
                        </div>
                    </div>
                </div>

                {/* Body Content */}
                <div className="relative z-10 flex-1 overflow-y-auto bg-slate-900/20 custom-scrollbar">
                    <div className="grid min-w-0 gap-0 xl:grid-cols-[minmax(0,1.45fr)_minmax(320px,0.95fr)]">
                        <div className="min-w-0 px-4 pb-24 pt-4 sm:px-8 sm:pb-8 sm:pt-5">
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

                            <section className="mb-5 overflow-hidden rounded-2xl border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.88),rgba(2,6,23,0.82))] p-4 shadow-[0_24px_80px_-46px_rgba(0,0,0,0.88)] sm:p-5">
                                <div className="flex flex-wrap items-start justify-between gap-3">
                                    <div className="min-w-0 flex-1">
                                        <div className="text-[10px] font-black uppercase tracking-[0.26em] text-cortex-500">
                                            Overview
                                        </div>
                                        <h3 className="mt-1.5 max-w-full break-words text-lg font-semibold tracking-tight text-white sm:text-xl">
                                            {projection.title || "Untitled Block"}
                                        </h3>
                                        <p className="mt-1.5 max-w-3xl overflow-hidden text-sm leading-6 text-white/62 [display:-webkit-box] [-webkit-box-orient:vertical] [-webkit-line-clamp:4] xl:[-webkit-line-clamp:3]">
                                            {readableSummary || "No preview text is available yet. Use the detail tabs to inspect relations, provenance, or raw surface data."}
                                        </p>
                                    </div>
                                    <button
                                        type="button"
                                        onClick={handleCopyArtifactId}
                                        title={projection.artifactId}
                                        className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1.5 text-[10px] font-semibold uppercase tracking-[0.18em] text-white/72 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                                    >
                                        {copyStateForArtifactId === "copied" ? <Check className="h-3.5 w-3.5 text-emerald-300" /> : <Copy className="h-3.5 w-3.5" />}
                                        {copyStateForArtifactId === "copied" ? "Copied ID" : copyStateForArtifactId === "failed" ? "Copy failed" : "Copy ID"}
                                    </button>
                                </div>

                                <div className="mt-4 grid grid-cols-2 gap-2 xl:grid-cols-4">
                                    <DetailMetricCard
                                        label="Artifact ID"
                                        value={artifactIdDisplay}
                                        valueTitle={projection.artifactId}
                                        subtitle="Canonical heap identity"
                                        tone="cyan"
                                    />
                                    <DetailMetricCard
                                        label="Payload"
                                        value={payloadContent.payload_type || block.projection.blockType || "n/a"}
                                        subtitle="Surface payload type"
                                        tone="slate"
                                    />
                                    <DetailMetricCard
                                        label="Links"
                                        value={String(outboundLinks.length + outboundMentions.length + backlinks.length)}
                                        subtitle="Graph context and lineage"
                                        tone="violet"
                                    />
                                    <DetailMetricCard
                                        label="Updated"
                                        value={new Date(projection.updatedAt).toLocaleDateString()}
                                        subtitle="Last synchronized surface"
                                        tone="emerald"
                                    />
                                </div>
                                <div className="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-3">
                                    <DetailMetricCard
                                        label="Authority"
                                        value={authorityLabel}
                                        subtitle="Review boundary"
                                        tone="amber"
                                    />
                                    <DetailMetricCard
                                        label="Requested"
                                        value={requestedRoleLabel}
                                        subtitle="Human/agent handoff"
                                        tone="cyan"
                                    />
                                    <DetailMetricCard
                                        label="Runtime"
                                        value={productionReadinessLabel}
                                        subtitle="Capability backing"
                                        tone={source === "fallback" ? "amber" : "slate"}
                                    />
                                </div>
                                {hasCapabilityLabels ? (
                                    <div className="mt-4 rounded-2xl border border-white/8 bg-white/[0.03] p-3">
                                        <div className="text-[10px] font-black uppercase tracking-[0.28em] text-slate-500">
                                            Needed capabilities
                                        </div>
                                        <div className="mt-2 flex flex-wrap gap-2">
                                            {capabilityLabels.map((capability) => (
                                                <span
                                                    key={capability}
                                                    className="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-2.5 py-1 text-[10px] font-semibold text-cyan-100"
                                                >
                                                    {asReadableToken(capability)}
                                                </span>
                                            ))}
                                        </div>
                                    </div>
                                ) : null}
                            </section>

                            {/* Tabs Navigation */}
                            <div 
                                className="mb-6 flex w-full max-w-full gap-2 overflow-x-auto rounded-2xl border border-white/8 bg-white/[0.05] p-1.5 shadow-inner sm:w-fit"
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
                                            className={`group flex shrink-0 items-center gap-2 rounded-xl px-4 py-2.5 text-[10px] font-black uppercase tracking-[0.16em] transition-all duration-300 select-none sm:px-5 ${isActive
                                                ? 'border border-cyan-400/20 bg-cyan-500/12 text-cyan-100 shadow-[0_0_20px_rgba(34,211,238,0.08)]'
                                                : 'text-slate-500 hover:bg-white/[0.05] hover:text-slate-200'
                                                } ${draggedTab === tab ? 'opacity-40 scale-95' : ''}`}
                                        >
                                            <GripVertical className={`w-3 h-3 transition-opacity duration-200 ${isActive ? 'opacity-45' : 'opacity-0 group-hover:opacity-40'} cursor-grab active:cursor-grabbing shrink-0`} />
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
                                    <h4 className="text-[10px] uppercase font-black text-slate-500 tracking-widest mb-4">Historical Context</h4>
                                    <p className="text-sm leading-7 text-slate-300">
                                        Follow the record across time: what it references, what points back to it, and which templates or runs shaped the current state.
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
                                        <RelationSection
                                            title={`History (${semanticLineage.length})`}
                                            accent="violet"
                                            items={semanticLineage}
                                            emptyLabel="No historical links yet."
                                            onSelect={navigateRelation}
                                        />
                                    </div>
                                )}

                                {activeTab === 'preview' && (
                                    <div className="animate-in fade-in duration-300 space-y-8">
                                        {routeLineage && (
                                            <TaskRouteLineageCard
                                                lineage={routeLineage}
                                                onOpenArtifact={onNavigateToBlock}
                                            />
                                        )}
                                        {projection.blockType === "task" && (
                                            <TaskRoutingDecisionCard
                                                attributes={projection.attributes}
                                                onRouteSelected={onTaskRouteSelected}
                                            />
                                        )}
                                        {uploadId && (
                                            <section className="overflow-hidden rounded-[1.6rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.9),rgba(2,6,23,0.84))] p-5 shadow-[0_24px_70px_-42px_rgba(0,0,0,0.9)]">
                                                <div className="flex flex-wrap items-start justify-between gap-4">
                                                    <div className="min-w-0">
                                                        <h4 className="text-[10px] uppercase font-black tracking-[0.32em] text-cortex-500">
                                                            Extraction dossier
                                                        </h4>
                                                        <p className="mt-2 text-sm leading-6 text-slate-200">
                                                            Requested {parserProfileLabel(effectiveUploadExtractionStatus?.requested_parser_profile || selectedParserProfile)}
                                                            {" · "}
                                                            Resolved {parserProfileLabel(effectiveUploadExtractionStatus?.parser_backend || resolvedParserBackendAttribute || "unknown")}
                                                        </p>
                                                        <p className="mt-1 text-xs text-slate-400">
                                                            Status {prettifyUploadState(effectiveUploadExtractionStatus?.status || "uploaded")}
                                                            {uploadExtractionRuns.length > 0 ? ` · ${uploadExtractionRuns.length} parser view${uploadExtractionRuns.length === 1 ? "" : "s"}` : ""}
                                                        </p>
                                                        <div className="mt-4 flex flex-wrap gap-2">
                                                            <span className={`rounded-full border px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] ${statusTone(effectiveUploadExtractionStatus?.status || "uploaded")}`}>
                                                                {prettifyUploadState(effectiveUploadExtractionStatus?.status || "uploaded")}
                                                            </span>
                                                            <span className={`rounded-full border px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] ${parserProfileTone(effectiveUploadExtractionStatus?.requested_parser_profile || selectedParserProfile)}`}>
                                                                requested {parserProfileLabel(effectiveUploadExtractionStatus?.requested_parser_profile || selectedParserProfile)}
                                                            </span>
                                                            <span className={`rounded-full border px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] ${parserProfileTone(effectiveUploadExtractionStatus?.parser_backend || resolvedParserBackendAttribute || "auto")}`}>
                                                                resolved {parserProfileLabel(effectiveUploadExtractionStatus?.parser_backend || resolvedParserBackendAttribute || "unknown")}
                                                            </span>
                                                        </div>
                                                    </div>
                                                    <div className="min-w-[240px] rounded-[1.35rem] border border-white/8 bg-white/[0.03] p-4">
                                                        <label className="block text-[10px] font-black uppercase tracking-[0.3em] text-white/45 mb-2">
                                                            Rerun with
                                                        </label>
                                                        <select
                                                            className="w-full rounded-2xl border border-white/10 bg-slate-950/80 px-4 py-3 text-sm text-slate-200 outline-none transition focus:border-cyan-300/40 focus:ring-2 focus:ring-cyan-500/30"
                                                            value={selectedParserProfile}
                                                            onChange={(event) => setSelectedParserProfile(event.target.value)}
                                                            disabled={rerunSubmitting}
                                                        >
                                                            {(uploadParserProfiles.length > 0
                                                                ? uploadParserProfiles.filter((profile) => profile.parser_profile === "auto" || profile.configured)
                                                                : [{ parser_profile: "auto", configured: true, supports_mime: [], role: "primary", parser_hint: "auto" }]
                                                            ).map((profile) => (
                                                                <option key={profile.parser_profile} value={profile.parser_profile}>
                                                                    {parserProfileLabel(profile.parser_profile)} · {profile.role}
                                                                </option>
                                                            ))}
                                                        </select>
                                                        <button
                                                            type="button"
                                                            onClick={() => void rerunUploadExtraction()}
                                                            disabled={rerunSubmitting}
                                                            className="mt-3 inline-flex items-center rounded-full border border-cyan-300/25 bg-cyan-300/12 px-4 py-2 text-[11px] font-black uppercase tracking-[0.2em] text-cyan-100 transition hover:bg-cyan-300/20 disabled:cursor-not-allowed disabled:opacity-50"
                                                        >
                                                            {rerunSubmitting ? "Running..." : "Rerun extraction"}
                                                        </button>
                                                    </div>
                                                </div>

                                                <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                                                    <DetailMetricCard
                                                        label="Confidence"
                                                        value={effectiveUploadExtractionStatus?.confidence !== undefined ? effectiveUploadExtractionStatus.confidence.toFixed(2) : "n/a"}
                                                        tone="cyan"
                                                    />
                                                    <DetailMetricCard
                                                        label="Pages"
                                                        value={effectiveUploadExtractionStatus?.page_count !== undefined ? String(effectiveUploadExtractionStatus.page_count) : "n/a"}
                                                        tone="violet"
                                                    />
                                                    <DetailMetricCard
                                                        label="Blocks"
                                                        value={effectiveUploadExtractionStatus?.block_count !== undefined ? String(effectiveUploadExtractionStatus.block_count) : "n/a"}
                                                        tone="emerald"
                                                    />
                                                    <DetailMetricCard
                                                        label="Preview state"
                                                        value={prettifyUploadState(effectiveUploadExtractionStatus?.status || "uploaded")}
                                                        tone="amber"
                                                    />
                                                </div>

                                                {effectiveUploadExtractionStatus?.flags?.length ? (
                                                    <div className="mt-4 flex flex-wrap gap-2">
                                                        {effectiveUploadExtractionStatus.flags.map((flag) => (
                                                            <span key={flag} className="rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 text-[10px] font-mono text-slate-300">
                                                                {flag}
                                                            </span>
                                                        ))}
                                                    </div>
                                                ) : null}
                                                {effectiveUploadExtractionStatus?.summary && (
                                                    <p className="mt-4 max-w-4xl text-sm leading-6 text-slate-300">{effectiveUploadExtractionStatus.summary}</p>
                                                )}
                                                {uploadExtractionRuns.length > 0 && (
                                                    <div className="mt-6 space-y-5">
                                                        <div>
                                                            <div className="text-[10px] uppercase font-black tracking-[0.32em] text-slate-500">Parser views</div>
                                                            <div className="mt-3 grid gap-3 lg:grid-cols-2 xl:grid-cols-3">
                                                                {uploadExtractionRuns.map((run) => {
                                                                    const isSelected = run.job_id === selectedRunJobId;
                                                                    return (
                                                                        <button
                                                                            key={run.job_id}
                                                                            type="button"
                                                                            onClick={() => setSelectedRunJobId(run.job_id)}
                                                                            className={`group rounded-[1.35rem] border p-4 text-left transition-all duration-200 ${
                                                                                isSelected
                                                                                    ? "border-cyan-300/35 bg-cyan-300/10 shadow-[0_18px_45px_-35px_rgba(34,211,238,0.55)]"
                                                                                    : "border-white/8 bg-white/[0.03] hover:border-white/16 hover:bg-white/[0.06]"
                                                                            }`}
                                                                        >
                                                                            <div className="flex items-start justify-between gap-3">
                                                                                <div className="min-w-0">
                                                                                    <div className="text-[10px] uppercase tracking-[0.26em] text-slate-500">
                                                                                        {run.status}
                                                                                    </div>
                                                                                    <div className="mt-2 text-sm font-semibold text-slate-100">
                                                                                        {parserProfileLabel(run.parser_backend || run.requested_parser_profile || "auto")}
                                                                                    </div>
                                                                                    <div className="mt-1 text-xs text-slate-400">
                                                                                        Requested {parserProfileLabel(run.requested_parser_profile || "auto")}
                                                                                    </div>
                                                                                </div>
                                                                                <span className="rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-1 text-[10px] font-mono text-white/65">
                                                                                    {run.job_id.slice(0, 10)}
                                                                                </span>
                                                                            </div>
                                                                            <div className="mt-4 flex flex-wrap gap-2 text-[10px] font-mono text-slate-400">
                                                                                {run.confidence !== undefined && <span>conf {run.confidence.toFixed(2)}</span>}
                                                                                {run.page_count !== undefined && <span>pages {run.page_count}</span>}
                                                                                {run.block_count !== undefined && <span>blocks {run.block_count}</span>}
                                                                            </div>
                                                                        </button>
                                                                    );
                                                                })}
                                                            </div>
                                                        </div>
                                                        {selectedRunDetail && (
                                                            <div className="grid gap-4 xl:grid-cols-[minmax(0,1.05fr)_minmax(320px,0.95fr)]">
                                                                <ParserRunDetailCard title="Selected parser view" detail={selectedRunDetail} />
                                                                <section className="overflow-hidden rounded-[1.35rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.9),rgba(2,6,23,0.84))] p-4">
                                                                    <div className="text-[10px] uppercase font-black tracking-[0.32em] text-slate-500">Compare against</div>
                                                                    <select
                                                                        className="mt-3 w-full rounded-2xl border border-white/10 bg-slate-950/80 px-4 py-3 text-sm text-slate-200 outline-none transition focus:border-cyan-300/40 focus:ring-2 focus:ring-cyan-500/30"
                                                                        value={compareRunJobId || ""}
                                                                        onChange={(event) => setCompareRunJobId(event.target.value || null)}
                                                                    >
                                                                        <option value="">Select another run</option>
                                                                        {selectableCompareRuns.map((run) => (
                                                                            <option key={run.job_id} value={run.job_id}>
                                                                                {parserProfileLabel(run.parser_backend || run.requested_parser_profile || "auto")} · {prettifyUploadState(run.status)}
                                                                            </option>
                                                                        ))}
                                                                    </select>
                                                                    {compareRunDetail ? (
                                                                        <div className="mt-4 space-y-4">
                                                                            <ParserRunDetailCard title="Comparison view" detail={compareRunDetail} compact />
                                                                            <div className="rounded-[1.35rem] border border-white/8 bg-white/[0.03] p-4">
                                                                                <div className="text-[10px] uppercase font-black tracking-[0.32em] text-slate-500">Human review</div>
                                                                                <div className="mt-3 flex flex-wrap gap-2">
                                                                                    {([
                                                                                        { value: "structure", label: "Better for structure" },
                                                                                        { value: "layout", label: "Better for layout" },
                                                                                        { value: "needs_rerun", label: "Needs rerun" },
                                                                                    ] as const).map((option) => (
                                                                                        <button
                                                                                            key={option.value}
                                                                                            type="button"
                                                                                            onClick={() => setComparisonDecision(option.value)}
                                                                                            className={`rounded-full border px-3 py-1.5 text-[11px] font-semibold transition ${
                                                                                                comparisonDecision === option.value
                                                                                                    ? "border-violet-300/35 bg-violet-300/12 text-violet-100"
                                                                                                    : "border-white/10 bg-white/[0.04] text-white/70 hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                                                                                            }`}
                                                                                        >
                                                                                            {option.label}
                                                                                        </button>
                                                                                    ))}
                                                                                </div>
                                                                                <textarea
                                                                                    className="mt-3 min-h-[108px] w-full rounded-2xl border border-white/10 bg-slate-950/90 px-4 py-3 text-sm text-slate-200 placeholder:text-slate-500 outline-none focus:border-violet-300/40 focus:ring-2 focus:ring-violet-500/30"
                                                                                    value={comparisonNotes}
                                                                                    onChange={(event) => setComparisonNotes(event.target.value)}
                                                                                    placeholder="Capture layout fidelity, structure quality, OCR concerns, or why this run should be trusted."
                                                                                />
                                                                                <button
                                                                                    type="button"
                                                                                    onClick={() => void saveComparisonFinding()}
                                                                                    disabled={comparisonSaving}
                                                                                    className="mt-3 inline-flex items-center rounded-full border border-violet-300/25 bg-violet-300/12 px-4 py-2 text-[11px] font-black uppercase tracking-[0.2em] text-violet-100 transition hover:bg-violet-300/20 disabled:cursor-not-allowed disabled:opacity-50"
                                                                                >
                                                                                    {comparisonSaving ? "Saving..." : "Save comparison finding"}
                                                                                </button>
                                                                                {comparisonMessage && <p className="mt-3 text-xs text-emerald-300">{comparisonMessage}</p>}
                                                                                {comparisonError && <p className="mt-3 text-xs text-rose-300">{comparisonError}</p>}
                                                                            </div>
                                                                        </div>
                                                                    ) : (
                                                                        <p className="mt-4 text-xs text-slate-500">
                                                                            Pick another parser run to compare structure, layout, and confidence side by side.
                                                                        </p>
                                                                    )}
                                                                </section>
                                                            </div>
                                                        )}
                                                    </div>
                                                )}
                                                {uploadExtractionError && (
                                                    <p className="mt-3 text-xs text-rose-300">{uploadExtractionError}</p>
                                                )}
                                            </section>
                                        )}
                                        <div className="overflow-hidden rounded-[1.6rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.86),rgba(2,6,23,0.8))] p-6 shadow-[0_24px_80px_-46px_rgba(0,0,0,0.9)] ring-1 ring-white/5">
                                            <PayloadRenderer content={payloadContent} expanded artifactId={projection.artifactId} />
                                        </div>
                                    </div>
                                )}

                                {activeTab === 'attributes' && (
                                    <div className="animate-in fade-in duration-300 space-y-6">
                                        <h4 className="mb-4 text-[10px] font-black uppercase tracking-[0.32em] text-slate-500">Structural Attributes</h4>
                                        {uploadId && effectiveUploadExtractionStatus && (
                                            <div className="rounded-[1.35rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.88),rgba(2,6,23,0.82))] p-4 shadow-[0_20px_60px_-45px_rgba(0,0,0,0.9)]">
                                                <div className="text-[10px] uppercase font-black tracking-[0.32em] text-cortex-500">Extraction status</div>
                                                <div className="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
                                                    <AttributeRow label="requested_parser_profile" value={effectiveUploadExtractionStatus.requested_parser_profile || "auto"} />
                                                    <AttributeRow label="parser_backend" value={effectiveUploadExtractionStatus.parser_backend || "unknown"} />
                                                    <AttributeRow label="status" value={effectiveUploadExtractionStatus.status} />
                                                    <AttributeRow label="confidence" value={effectiveUploadExtractionStatus.confidence?.toFixed(2) || "n/a"} />
                                                    <AttributeRow label="page_count" value={String(effectiveUploadExtractionStatus.page_count ?? "n/a")} />
                                                    <AttributeRow label="block_count" value={String(effectiveUploadExtractionStatus.block_count ?? "n/a")} />
                                                </div>
                                            </div>
                                        )}
                                        <div className="grid grid-cols-1 gap-2">
                                            {Object.entries(projection).map(([k, v]) => {
                                                if (typeof v !== 'object' && v !== undefined && v !== null) {
                                                    return (
                                                        <div key={k} className="flex items-center justify-between rounded-2xl border border-white/8 bg-white/[0.03] px-4 py-2.5 transition-colors hover:border-white/16 hover:bg-white/[0.05]">
                                                            <span className="text-xs font-medium text-white/50">{k}</span>
                                                            <span className="rounded-full border border-white/10 bg-slate-950/70 px-2 py-0.5 text-xs font-mono text-slate-200">{String(v)}</span>
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
                                                <h4 className="mb-3 flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.32em] text-slate-500">
                                                    <span className="w-1.5 h-1.5 rounded-full bg-blue-500"></span> Projection
                                                </h4>
                                                <div className="overflow-x-auto rounded-[1.35rem] border border-white/8 bg-slate-950/80 p-4 shadow-inner custom-scrollbar text-[10px] leading-relaxed font-mono">
                                                    {formatJsonWithHighlighting(block.projection)}
                                                </div>
                                            </div>

                                            <div>
                                                <h4 className="mb-3 flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.32em] text-slate-500">
                                                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> Surface JSON
                                                </h4>
                                                <div className="overflow-x-auto rounded-[1.35rem] border border-white/8 bg-slate-950/80 p-4 shadow-inner custom-scrollbar text-[10px] leading-relaxed font-mono">
                                                    {formatJsonWithHighlighting(block.surfaceJson)}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                )}
                            </div>
                        </div>

                        <aside className="min-w-0 border-t border-white/8 bg-[linear-gradient(180deg,rgba(2,6,23,0.9),rgba(15,23,42,0.7))] p-4 sm:p-6 xl:border-l xl:border-t-0 xl:border-white/8">
                            <div className="sticky top-0 space-y-6">
                                <section className="rounded-[1.35rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.9),rgba(2,6,23,0.82))] p-5 shadow-[0_20px_60px_-45px_rgba(0,0,0,0.9)]">
                                    <div className="text-[10px] font-black uppercase tracking-[0.32em] text-cortex-500">Artifact synopsis</div>
                                    <div className="mt-3 text-lg font-semibold tracking-tight text-white">{projection.title || "Untitled Block"}</div>
                                    <div className="mt-1 text-xs leading-6 text-slate-400">
                                        {readableSummary || "No plain-text preview available."}
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
                                <RelationSection
                                    title={`History (${semanticLineage.length})`}
                                    accent="cyan"
                                    items={semanticLineage}
                                    emptyLabel="No historical links yet."
                                    onSelect={navigateRelation}
                                />

                                <section className="rounded-[1.35rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.88),rgba(2,6,23,0.82))] p-5 shadow-[0_20px_60px_-45px_rgba(0,0,0,0.9)]">
                                    <div className="text-[10px] font-black uppercase tracking-[0.32em] text-slate-500">Block Stats</div>
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
                                <section className="rounded-[1.35rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.88),rgba(2,6,23,0.82))] p-5 shadow-[0_20px_60px_-45px_rgba(0,0,0,0.9)]">
                                    <div className="text-[10px] font-black uppercase tracking-[0.32em] text-slate-500">IDs & Export Surface</div>
                                    <div className="mt-3 space-y-3 text-[11px]">
                                        <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-3">
                                            <div className="text-slate-500">Block ID</div>
                                            <div className="mt-1 font-mono text-slate-100 break-all">{projection.artifactId}</div>
                                        </div>
                                        <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-3">
                                            <div className="text-slate-500">Space</div>
                                            <div className="mt-1 font-mono text-slate-100 break-all">{resolvedSpaceId}</div>
                                        </div>
                                        <div className="rounded-2xl border border-white/8 bg-white/[0.03] p-3 text-slate-400">
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

function AttributeRow({ label, value }: { label: string; value: string }) {
    return (
        <div className="flex items-center justify-between rounded-2xl border border-white/8 bg-white/[0.03] px-4 py-2.75 transition-colors hover:border-white/16 hover:bg-white/[0.05]">
            <span className="text-xs font-medium text-white/50">{label}</span>
            <span className="rounded-full border border-white/10 bg-slate-950/70 px-2.5 py-1 text-xs font-mono text-slate-200">{value}</span>
        </div>
    );
}

function summarizeRunDetail(detail: HeapUploadExtractionRunDetail): string {
    const parts: string[] = [
        `${prettifyUploadState(detail.status)}`,
        `${detail.page_count ?? 0} pages`,
        `${detail.block_count ?? 0} blocks`,
    ];
    if (detail.confidence !== undefined) {
        parts.push(`confidence ${detail.confidence.toFixed(2)}`);
    }
    return parts.join(" · ");
}

function ParserRunDetailCard({
    title,
    detail,
    compact = false,
}: {
    title: string;
    detail: HeapUploadExtractionRunDetail;
    compact?: boolean;
}) {
    return (
        <section className="overflow-hidden rounded-[1.35rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.92),rgba(2,6,23,0.86))] p-4 shadow-[0_20px_60px_-45px_rgba(0,0,0,0.9)]">
            <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                    <div className="text-[10px] uppercase font-black tracking-[0.32em] text-cortex-500">{title}</div>
                    <div className="mt-2 text-sm font-semibold text-white">
                        {parserProfileLabel(detail.parser_backend || detail.requested_parser_profile || "unknown")}
                    </div>
                    <div className="mt-1 text-xs text-slate-400">
                        Requested {parserProfileLabel(detail.requested_parser_profile || "auto")} · Status {prettifyUploadState(detail.status)}
                    </div>
                </div>
                <div className="rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 text-[10px] font-mono text-white/65">
                    {detail.job_id}
                </div>
            </div>
            <div className="mt-4 grid grid-cols-2 gap-2">
                <AttributeRow label="created_at" value={detail.created_at || "n/a"} />
                <AttributeRow label="last_updated_at" value={detail.last_updated_at || "n/a"} />
                <AttributeRow label="confidence" value={detail.confidence?.toFixed(2) || "n/a"} />
                <AttributeRow label="page_count" value={String(detail.page_count ?? "n/a")} />
                <AttributeRow label="block_count" value={String(detail.block_count ?? "n/a")} />
                <AttributeRow label="model_id" value={detail.model_id || "n/a"} />
            </div>
            {detail.attempted_backends?.length ? (
                <div className="mt-3 flex flex-wrap gap-2">
                    {detail.attempted_backends.map((backend) => (
                        <span key={backend} className="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-3 py-1 text-[10px] font-mono text-cyan-100/85">
                            {backend}
                        </span>
                    ))}
                </div>
            ) : null}
            {detail.flags?.length ? (
                <div className="mt-3 flex flex-wrap gap-2">
                    {detail.flags.map((flag) => (
                        <span key={flag} className="rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 text-[10px] font-mono text-slate-300">
                            {flag}
                        </span>
                    ))}
                </div>
            ) : null}
            {detail.summary ? (
                <p className="mt-3 text-sm leading-6 text-slate-300">{detail.summary}</p>
            ) : null}
            {detail.first_page_preview?.length ? (
                <div className="mt-4 rounded-[1.25rem] border border-white/8 bg-white/[0.03] p-4">
                    <div className="text-[10px] uppercase font-black tracking-[0.32em] text-slate-500">Preview</div>
                    <div className="mt-2 text-sm leading-6 text-slate-300">
                        {detail.first_page_preview.join(" ")}
                    </div>
                    {detail.first_page_block_count !== undefined && !compact ? (
                        <div className="mt-2 text-[11px] text-slate-500">
                            First page blocks: {detail.first_page_block_count}
                        </div>
                    ) : null}
                </div>
            ) : null}
        </section>
    );
}

function RelationSection({
    title,
    items,
    emptyLabel,
    accent,
    onSelect,
}: {
    title: string;
    items: HeapRelationItem[];
    emptyLabel: string;
    accent: "blue" | "cyan" | "indigo" | "violet";
    onSelect?: (item: HeapRelationItem) => void;
}) {
    const accentClass = accent === "cyan"
        ? "bg-cyan-500"
        : accent === "indigo"
            ? "bg-indigo-500"
            : accent === "violet"
                ? "bg-violet-500"
                : "bg-blue-500";

    return (
        <section className="rounded-[1.35rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.84),rgba(2,6,23,0.76))] p-5 shadow-[0_20px_60px_-45px_rgba(0,0,0,0.9)]">
            <h4 className="mb-4 flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.32em] text-slate-500">
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
                            onClick={() => item.isNavigable && onSelect?.(item)}
                            disabled={!item.isNavigable}
                            className={`w-full rounded-2xl border border-white/8 bg-white/[0.03] p-3 text-left ${
                                item.isNavigable ? "transition-colors hover:border-white/16 hover:bg-white/[0.06]" : "cursor-default"
                            }`}
                        >
                            {item.relations?.length ? (
                                <div className="mb-2 flex flex-wrap gap-1.5">
                                    {item.relations.map((relation) => (
                                        <span
                                            key={relation}
                                            className="rounded-full border border-white/8 bg-white/[0.04] px-2 py-0.5 text-[10px] uppercase tracking-[0.16em] text-white/55"
                                        >
                                            {describeHeapRelation(relation)}
                                        </span>
                                    ))}
                                </div>
                            ) : null}
                            <div className="text-sm font-semibold text-white">{item.title || item.id}</div>
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

function DetailMetricCard({
    label,
    value,
    tone = "slate",
    subtitle,
    valueTitle,
}: {
    label: string;
    value: string | number;
    tone?: "cyan" | "violet" | "emerald" | "amber" | "rose" | "slate";
    subtitle?: string;
    valueTitle?: string;
}) {
    return (
        <div className={`min-w-0 rounded-xl border px-2.5 py-1.5 shadow-[0_12px_32px_-28px_rgba(0,0,0,0.9)] sm:px-3 sm:py-2 ${resolveMetricCardClassName(tone)}`}>
            <div className="flex min-w-0 items-baseline justify-between gap-2 sm:gap-3">
                <div className="shrink-0 text-[9px] uppercase tracking-[0.18em] text-white/45">{label}</div>
                <div className="min-w-0 truncate text-right text-sm font-bold tracking-tight text-white sm:text-base" title={valueTitle}>{value}</div>
            </div>
            {subtitle ? <div className="mt-0.5 hidden truncate text-[10px] leading-4 text-white/50 sm:block">{subtitle}</div> : null}
        </div>
    );
}

function StatCard({ label, value }: { label: string; value: number }) {
    return (
        <DetailMetricCard label={label} value={value} />
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
