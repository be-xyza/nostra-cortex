import type { HeapBlockListItem } from "../../contracts.ts";
import { buildSolicitationRenderModel, buildStewardFeedbackRenderModel } from "./solicitationRenderModel.ts";

export type HeapAggregationGroupId = "prompt-like" | "steward-feedback";
export type HeapDerivedViewId = "board" | `aggregate:${HeapAggregationGroupId}`;

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
            label: "All Blocks",
            description: "The primary board view with grouped meta-blocks and remaining individual blocks.",
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
    return {
        groupId: "prompt-like",
        label: "Prompts",
        description: "Recent prompts, requests, and solicitations grouped into a compact list.",
        count: blocks.length,
        columns: [
            { key: "role", label: "Requested Role" },
            { key: "authority", label: "Authority" },
            { key: "budget", label: "Budget" },
        ],
        memberArtifactIds: blocks.map((block) => block.projection.artifactId),
        items: blocks.map((block) => {
            const typedData = getStructuredData(block);
            const solicitation = typedData ? buildSolicitationRenderModel(typedData) : null;
            const summary = solicitation?.summary ?? readShortText(block);
            const blockKind = describePromptLikeBlock(block, typedData);
            const fields = {
                role: solicitation?.roleLabel ?? blockKind,
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
    return {
        groupId: "steward-feedback",
        label: "Steward Logs",
        description: "Recorded approvals, rejections, and steward notes grouped into a compact list.",
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
            const summary = feedback?.summary ?? readShortText(block);
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

function readShortText(block: HeapBlockListItem): string {
    const typedData = getStructuredData(block);
    const candidate =
        readCandidateText(typedData?.plain_text) ??
        readCandidateText(typedData?.text) ??
        readCandidateText(typedData?.summary) ??
        readCandidateText(typedData?.message) ??
        readCandidateText(typedData?.feedback) ??
        readCandidateText(typedData?.notes) ??
        readCandidateText(typedData?.prompt) ??
        readCandidateText(typedData?.description) ??
        extractReadableText(typedData) ??
        extractReadableText(asRecord(block.surfaceJson));
    if (candidate) {
        return collapseWhitespace(candidate);
    }

    return block.projection.title?.trim() || "No text summary available.";
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

function extractReadableText(value: unknown, depth = 0): string | null {
    if (depth > 3 || value == null) {
        return null;
    }
    if (typeof value === "string") {
        const normalized = sanitizeReadableText(value);
        if (!normalized) {
            return null;
        }
        if (looksLikeSerializedStructure(value.trim()) || looksLikeCodeSnippet(normalized)) {
            return null;
        }
        return normalized;
    }
    if (Array.isArray(value)) {
        for (const entry of value) {
            const text = extractReadableText(entry, depth + 1);
            if (text) {
                return text;
            }
        }
        return null;
    }
    const record = asRecord(value);
    if (!record) {
        return null;
    }

    const priorityKeys = [
        "plain_text",
        "text",
        "summary",
        "message",
        "feedback",
        "notes",
        "description",
        "prompt",
        "content",
        "body",
        "label",
        "title",
    ] as const;
    for (const key of priorityKeys) {
        const text = extractReadableText(record[key], depth + 1);
        if (text) {
            return text;
        }
    }
    return null;
}

function looksLikeSerializedStructure(value: string): boolean {
    const trimmed = value.trim();
    return trimmed.startsWith("{") || trimmed.startsWith("[");
}

function collapseWhitespace(value: string): string {
    const normalized = value.replace(/\s+/g, " ").trim();
    if (normalized.length <= 180) {
        return normalized;
    }
    return `${normalized.slice(0, 177)}...`;
}

function sanitizeReadableText(value: string): string | null {
    const withoutCodeBlocks = value.replace(/```[\s\S]*?```/g, " ");
    const withoutInlineCode = withoutCodeBlocks.replace(/`([^`]*)`/g, "$1");
    const withoutLinks = withoutInlineCode.replace(/\[([^\]]+)\]\([^)]+\)/g, "$1");
    const withoutMarkdown = withoutLinks.replace(/(^|\n)\s{0,3}[>#*-]+\s*/g, " ");
    const normalized = collapseWhitespace(
        withoutMarkdown
            .replace(/[_*~]+/g, " ")
            .replace(/\s+/g, " "),
    );
    return normalized || null;
}

function looksLikeCodeSnippet(value: string): boolean {
    const trimmed = value.trim();
    if (!trimmed) {
        return false;
    }
    if (trimmed.includes("```")) {
        return true;
    }
    const codeKeywordPattern = /\b(const|let|var|function|return|import|export|class|interface|type|fn|impl|struct|enum)\b|=>/;
    const htmlPattern = /<\w+[^>]*>.*<\/\w+>/s;
    return (
        (/[{};]/.test(trimmed) && codeKeywordPattern.test(trimmed)) ||
        htmlPattern.test(trimmed)
    );
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

function readCandidateText(value: unknown): string | null {
    const candidate = asString(value);
    if (!candidate) {
        return null;
    }
    const normalized = sanitizeReadableText(candidate);
    if (!normalized) {
        return null;
    }
    if (looksLikeSerializedStructure(candidate) || looksLikeCodeSnippet(normalized)) {
        return null;
    }
    return normalized;
}
