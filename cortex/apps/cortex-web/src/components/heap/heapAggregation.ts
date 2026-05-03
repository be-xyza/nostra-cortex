import type { HeapBlockListItem } from "../../contracts.ts";
import { buildSolicitationRenderModel, buildStewardFeedbackRenderModel } from "./solicitationRenderModel.ts";
import { summarizeHeapBlockText } from "./heapTextSummary.ts";
import { resolveHeapDerivedViewRegistryEntry } from "./heapViewRegistry.ts";
import { buildHeapContributorCardModel } from "./heapContributorCardModel.ts";

export type HeapAggregationGroupId = "usage-report" | "agent-work" | "suggested-improvements" | "prompt-like" | "steward-feedback";
export type HeapDerivedViewId = "board" | "all-blocks" | `aggregate:${HeapAggregationGroupId}`;

export interface HeapAggregationColumn {
    key: string;
    label: string;
}

export interface HeapAggregationItem {
    artifactId: string;
    title: string;
    blockType: string;
    updatedAt: string;
    summary: string;
    fields: Record<string, string>;
    details: string[];
    tone?: "approved" | "rejected" | "neutral";
    badge?: string;
    source: HeapBlockListItem;
}

export interface HeapAggregationGroup {
    groupId: HeapAggregationGroupId;
    label: string;
    description: string;
    count: number;
    columns: HeapAggregationColumn[];
    memberArtifactIds: string[];
    items: HeapAggregationItem[];
}

export interface HeapDerivedView {
    id: HeapDerivedViewId;
    label: string;
    description: string;
    count: number;
    kind: "board" | "aggregate";
    groupId?: HeapAggregationGroupId;
}

const PROMPT_BLOCK_TYPES = new Set([
    "agent_solicitation",
    "benchmark_solicitation",
    "prompt",
    "prompt_request",
]);

const STEWARD_FEEDBACK_BLOCK_TYPES = new Set([
    "steward_feedback",
]);

export function buildHeapAggregationGroups(
    blocks: HeapBlockListItem[],
): HeapAggregationGroup[] {
    const promptLike = blocks
        .filter((block) => isPromptLikeBlock(block))
        .sort((left, right) => right.projection.updatedAt.localeCompare(left.projection.updatedAt));

    const stewardFeedback = blocks
        .filter((block) => isStewardFeedbackBlock(block))
        .sort((left, right) => right.projection.updatedAt.localeCompare(left.projection.updatedAt));

    return [
        buildContributorDigestGroup("usage-report", "Recent activity summaries", "Repeated system activity records grouped into one contributor-friendly digest.", blocks.filter((block) => block.projection.blockType === "usage_report")),
        buildContributorDigestGroup("agent-work", "Recent agent work", "Agent execution records grouped so the feed shows work patterns instead of repeated raw logs.", blocks.filter((block) => block.projection.blockType === "agent_execution_record")),
        buildContributorDigestGroup("suggested-improvements", "Suggested improvements", "Optimization proposals grouped for review without flooding the main feed.", blocks.filter((block) => block.projection.blockType === "self_optimization_proposal")),
        buildPromptGroup(promptLike),
        buildStewardFeedbackGroup(stewardFeedback),
    ].filter((group): group is HeapAggregationGroup => group.count > 0);
}

export function buildHeapDerivedViews(
    blocks: HeapBlockListItem[],
    groups: HeapAggregationGroup[],
): HeapDerivedView[] {
    return [
        {
            id: "board",
            label: "Relevant updates",
            description: "Recent updates, proposals, evidence, and agent activity for this Space.",
            count: blocks.length,
            kind: "board",
        },
        {
            id: "all-blocks",
            label: "All Blocks",
            description: "Complete record stream, including repeated system and telemetry records.",
            count: blocks.length,
            kind: "board",
        },
        ...groups.map((group) => ({
            id: `aggregate:${group.groupId}` as HeapDerivedViewId,
            label: group.label,
            description: group.description,
            count: group.count,
            kind: "aggregate" as const,
            groupId: group.groupId,
        })),
    ];
}

function buildContributorDigestGroup(
    groupId: Extract<HeapAggregationGroupId, "usage-report" | "agent-work" | "suggested-improvements">,
    label: string,
    description: string,
    blocks: HeapBlockListItem[],
): HeapAggregationGroup {
    return {
        groupId,
        label,
        description,
        count: blocks.length,
        columns: [
            { key: "status", label: "Status" },
            { key: "source", label: "Source" },
            { key: "relevance", label: "Relevance" },
        ],
        memberArtifactIds: blocks.map((block) => block.projection.artifactId),
        items: blocks
            .sort((left, right) => right.projection.updatedAt.localeCompare(left.projection.updatedAt))
            .map((block) => {
                const model = buildHeapContributorCardModel(block);
                return {
                    artifactId: block.projection.artifactId,
                    title: model.displayTitle,
                    blockType: block.projection.blockType,
                    updatedAt: block.projection.updatedAt,
                    summary: model.plainSummary,
                    fields: {
                        status: model.statusLabel,
                        source: model.sourceLabel,
                        relevance: model.relevanceLabel,
                    },
                    details: [
                        `Status: ${model.statusLabel}`,
                        `Source: ${model.sourceLabel}`,
                        `Relevance: ${model.relevanceLabel}`,
                    ],
                    badge: model.friendlyTypeLabel,
                    source: block,
                };
            }),
    };
}

export function collectHeapAggregationArtifactIds(groups: HeapAggregationGroup[]): Set<string> {
    const artifactIds = new Set<string>();
    for (const group of groups) {
        for (const artifactId of group.memberArtifactIds) {
            artifactIds.add(artifactId);
        }
    }
    return artifactIds;
}

function buildPromptGroup(blocks: HeapBlockListItem[]): HeapAggregationGroup {
    const registryEntry = resolveHeapDerivedViewRegistryEntry("prompt-like");
    return {
        groupId: "prompt-like",
        label: registryEntry?.label ?? "Prompts",
        description: registryEntry?.description ?? "Recent prompts, requests, and solicitations grouped into a compact list.",
        count: blocks.length,
        columns: [
            { key: "role", label: "Role" },
            { key: "authority", label: "Authority" },
            { key: "budget", label: "Budget" },
        ],
        memberArtifactIds: blocks.map((block) => block.projection.artifactId),
        items: blocks.map((block) => {
            const typedData = getStructuredData(block);
            const solicitation = typedData ? buildSolicitationRenderModel(typedData) : null;
            const summary = solicitation?.summary ?? summarizeHeapBlockText(block);
            const blockKind = describePromptLikeBlock(block, typedData);
            const fields = {
                role: solicitation?.roleLabel ?? "unspecified",
                authority: solicitation?.authorityScopeLabel ?? "unspecified",
                budget: solicitation?.budgetLabel ?? "n/a",
            };
            const details = [
                `Role: ${fields.role}`,
                `Authority: ${fields.authority}`,
                `Budget: ${fields.budget}`,
                solicitation?.capabilityLabels.length ? `Capabilities: ${solicitation.capabilityLabels.join(", ")}` : null,
            ].filter((value): value is string => Boolean(value));

            return {
                artifactId: block.projection.artifactId,
                title: block.projection.title,
                blockType: block.projection.blockType,
                updatedAt: block.projection.updatedAt,
                summary,
                fields,
                details,
                badge: blockKind,
                source: block,
            };
        }),
    };
}

function buildStewardFeedbackGroup(blocks: HeapBlockListItem[]): HeapAggregationGroup {
    const registryEntry = resolveHeapDerivedViewRegistryEntry("steward-feedback");
    return {
        groupId: "steward-feedback",
        label: registryEntry?.label ?? "Steward Logs",
        description: registryEntry?.description ?? "Recorded approvals, rejections, and steward notes grouped into a compact list.",
        count: blocks.length,
        columns: [
            { key: "decision", label: "Decision" },
            { key: "submittedBy", label: "By" },
            { key: "parent", label: "Parent" },
        ],
        memberArtifactIds: blocks.map((block) => block.projection.artifactId),
        items: blocks.map((block) => {
            const typedData = getStructuredData(block);
            const feedback = typedData ? buildStewardFeedbackRenderModel(typedData) : null;
            const summary = feedback?.summary ?? summarizeHeapBlockText(block);
            const fields = {
                decision: feedback?.decisionLabel ?? block.projection.blockType,
                submittedBy: feedback?.submittedBy ?? "unknown",
                parent: feedback?.parentArtifactId ?? "n/a",
            };
            const details = [
                `Decision: ${fields.decision}`,
                `By: ${fields.submittedBy}`,
                feedback?.submittedAt ? `At: ${feedback.submittedAt}` : null,
                `Parent: ${fields.parent}`,
            ].filter((value): value is string => Boolean(value));

            return {
                artifactId: block.projection.artifactId,
                title: block.projection.title,
                blockType: block.projection.blockType,
                updatedAt: block.projection.updatedAt,
                summary,
                fields,
                details,
                tone: feedback?.decisionTone,
                badge: "feedback",
                source: block,
            };
        }),
    };
}

function isPromptLikeBlock(block: HeapBlockListItem): boolean {
    if (PROMPT_BLOCK_TYPES.has(block.projection.blockType)) {
        return true;
    }

    const typedData = getStructuredData(block);
    return Boolean(typedData && PROMPT_BLOCK_TYPES.has(asString(typedData.type) ?? ""));
}

function isStewardFeedbackBlock(block: HeapBlockListItem): boolean {
    if (STEWARD_FEEDBACK_BLOCK_TYPES.has(block.projection.blockType)) {
        return true;
    }

    const typedData = getStructuredData(block);
    return Boolean(typedData && STEWARD_FEEDBACK_BLOCK_TYPES.has(asString(typedData.type) ?? ""));
}

function getStructuredData(block: HeapBlockListItem): Record<string, unknown> | null {
    const surface = asRecord(block.surfaceJson);
    if (!surface) {
        return null;
    }
    const typedData = asRecord(surface.structured_data);
    return typedData ?? surface;
}

function describePromptLikeBlock(
    block: HeapBlockListItem,
    typedData: Record<string, unknown> | null,
): string {
    const explicitType = asString(typedData?.type) ?? block.projection.blockType;
    switch (explicitType) {
        case "agent_solicitation":
        case "benchmark_solicitation":
            return "solicitation";
        case "prompt_request":
            return "request";
        case "prompt":
            return "prompt";
        default:
            return humanizeToken(explicitType);
    }
}

function humanizeToken(value: string): string {
    return value
        .replace(/[_-]+/g, " ")
        .trim()
        .toLowerCase();
}

function asRecord(value: unknown): Record<string, unknown> | null {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
        return null;
    }
    return value as Record<string, unknown>;
}

function asString(value: unknown): string | null {
    if (typeof value !== "string") {
        return null;
    }
    const normalized = value.trim();
    return normalized.length > 0 ? normalized : null;
}
