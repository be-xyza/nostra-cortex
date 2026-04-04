import type { HeapBlockListItem } from "../../contracts";

type HeapSummaryBlock = Pick<HeapBlockListItem, "projection" | "surfaceJson">;

export function summarizeHeapBlockText(block: HeapSummaryBlock): string {
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

function getStructuredData(block: HeapSummaryBlock): Record<string, unknown> | null {
    const surface = asRecord(block.surfaceJson);
    if (!surface) {
        return null;
    }
    const typedData = asRecord(surface.structured_data);
    return typedData ?? surface;
}

function readCandidateText(value: unknown): string | null {
    if (typeof value !== "string") {
        return null;
    }
    const normalized = sanitizeReadableText(value);
    return normalized && normalized !== "No text summary available." ? normalized : null;
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

function asRecord(value: unknown): Record<string, unknown> | null {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
        return null;
    }
    return value as Record<string, unknown>;
}
