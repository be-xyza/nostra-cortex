import type { HeapAggregationGroup, HeapDerivedView, HeapDerivedViewId } from "./heapAggregation.ts";

export interface HeapViewSignal {
    label: string;
    summary: string;
    prompt: string;
}

export interface HeapViewContextSnapshot {
    viewId: HeapDerivedViewId;
    viewLabel: string;
    description: string;
    itemCount: number;
    recentTitles: string[];
    signals: HeapViewSignal[];
}

type HeapGroupId = Exclude<HeapDerivedViewId, "board"> extends `aggregate:${infer GroupId}` ? GroupId : never;

export interface HeapDerivedViewRegistryEntry {
    groupId: HeapGroupId;
    label: string;
    description: string;
    buildSignals: (group: HeapAggregationGroup) => HeapViewSignal[];
}

const PROMPT_LIKE_VIEW_REGISTRY_ENTRY: HeapDerivedViewRegistryEntry = {
    groupId: "prompt-like",
    label: "Prompts",
    description: "Recent prompts, requests, and solicitations grouped into a compact list.",
    buildSignals: (group) => {
        const newest = group.items[0];
        return [
            {
                label: "Newest request",
                summary: newest ? newest.title : "No prompt blocks",
                prompt: newest
                    ? `Review the newest prompt block "${newest.title}" and summarize its request shape, including role, authority, and budget when present.`
                    : "Review the current prompt feed and summarize the latest request shape.",
            },
            {
                label: "Pattern check",
                summary: `${group.count} prompt blocks`,
                prompt: `Compare the ${group.count} prompt blocks and identify repeated roles, authority scopes, capability requests, or request shapes.`,
            },
        ];
    },
};

const STEWARD_FEEDBACK_VIEW_REGISTRY_ENTRY: HeapDerivedViewRegistryEntry = {
    groupId: "steward-feedback",
    label: "Steward Logs",
    description: "Recorded approvals, rejections, and steward notes grouped into a compact list.",
    buildSignals: (group) => {
        const newest = group.items[0];
        return [
            {
                label: "Latest decision",
                summary: newest ? newest.title : "No steward logs",
                prompt: newest
                    ? `Summarize the latest steward decision "${newest.title}" and note the decision, subject, and parent block.`
                    : "Review the steward log stream and summarize the latest decisions.",
            },
            {
                label: "Decision mix",
                summary: `${group.count} steward records`,
                prompt: `Summarize the ${group.count} steward records and separate approvals from rejections and notes.`,
            },
        ];
    },
};

export const HEAP_DERIVED_VIEW_REGISTRY: HeapDerivedViewRegistryEntry[] = [
    PROMPT_LIKE_VIEW_REGISTRY_ENTRY,
    STEWARD_FEEDBACK_VIEW_REGISTRY_ENTRY,
];

export function resolveHeapDerivedViewRegistryEntry(
    groupId: HeapDerivedViewRegistryEntry["groupId"],
): HeapDerivedViewRegistryEntry | null {
    return HEAP_DERIVED_VIEW_REGISTRY.find((entry) => entry.groupId === groupId) ?? null;
}

export function buildHeapViewContext(
    activeDerivedView: HeapDerivedView | null,
    aggregationGroups: HeapAggregationGroup[],
): HeapViewContextSnapshot | null {
    if (!activeDerivedView) {
        return null;
    }

    if (activeDerivedView.kind === "aggregate" && activeDerivedView.groupId) {
        const group = aggregationGroups.find((item) => item.groupId === activeDerivedView.groupId) ?? null;
        const registryEntry = group ? resolveHeapDerivedViewRegistryEntry(group.groupId) : null;
        if (!group || !registryEntry) {
            return null;
        }

        return {
            viewId: activeDerivedView.id,
            viewLabel: activeDerivedView.label,
            description: activeDerivedView.description,
            itemCount: group.count,
            recentTitles: group.items.slice(0, 3).map((item) => item.title),
            signals: registryEntry.buildSignals(group),
        };
    }

    const recentGroups = aggregationGroups.slice(0, 2);
    const signals = recentGroups.flatMap((group) => {
        const registryEntry = resolveHeapDerivedViewRegistryEntry(group.groupId);
        return registryEntry ? registryEntry.buildSignals(group).slice(0, 1) : [];
    });

    return {
        viewId: activeDerivedView.id,
        viewLabel: activeDerivedView.label,
        description: activeDerivedView.description,
        itemCount: activeDerivedView.count,
        recentTitles: recentGroups.flatMap((group) => group.items.slice(0, 1).map((item) => item.title)).slice(0, 3),
        signals,
    };
}
