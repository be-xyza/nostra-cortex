import React from "react";
import { Plus, MessagesSquare } from "lucide-react";
import type { ActionZonePlan, ActionSelectionContext } from "../../contracts";
import { ActionZoneRenderer } from "../commons/ActionZoneRenderer";
import { executeHeapAction, type ActionHandlers } from "./actionExecutor";

interface HeapActionBarProps {
    selectionZonePlan?: ActionZonePlan | null;
    selection: ActionSelectionContext;
    handlers: ActionHandlers;
    onCreate: () => void;
    onChat?: () => void;
    canPublish?: boolean;
    status?: {
        loading: boolean;
        source: "remote" | "fallback" | "idle";
        error: string | null;
    };
}

/**
 * Selection-driven floating action bar.
 * Hidden by default — appears only when one or more blocks are selected.
 * All shapes are circular (rounded-full).
 */
export function HeapActionBar({
    selectionZonePlan,
    selection,
    handlers,
    onCreate,
    onChat,
    canPublish = false,
    status,
}: HeapActionBarProps) {
    const selectedCount = selection.selectedCount;
    const recordLabel = selectedCount === 1 ? "record" : "records";

    // Hidden by default — only render when blocks are selected
    if (selectedCount === 0) return null;

    const hasActions = Boolean(
        selectionZonePlan?.actions.some((a) => a.visible !== false && (canPublish || !isPublishAction(a))),
    );
    const contributorActions = (selectionZonePlan?.actions ?? [])
        .filter((action) => canPublish || !isPublishAction(action))
        .map(contributorActionLabel);

    // Status ring color
    const ringColor = (() => {
        if (!status) return "border-emerald-500/20";
        if (status.error)
            return "border-red-500/50 shadow-[0_0_8px_rgba(239,68,68,0.3)]";
        if (status.source === "fallback")
            return "border-amber-500/50 shadow-[0_0_8px_rgba(245,158,11,0.3)]";
        if (status.loading) return "border-blue-500/50 animate-pulse";
        return "border-emerald-500/50 shadow-[0_0_8px_rgba(16,185,129,0.3)]";
    })();

    return (
        <div className="heap-action-bar fixed inset-x-2 bottom-4 z-50 flex max-w-[calc(100vw-1rem)] items-center gap-2 overflow-x-auto px-3 py-2 bg-cortex-950/90 rounded-2xl backdrop-blur-xl shadow-2xl transition-all animate-in slide-in-from-bottom duration-300 pointer-events-auto sm:bottom-8 sm:left-1/2 sm:right-auto sm:w-max sm:-translate-x-1/2 sm:gap-3 sm:rounded-full sm:px-4" data-selection-count={selectedCount}>
            {/* Status ring - subtle glow instead of ring */}
            <div
                className={`absolute inset-0 rounded-full pointer-events-none transition-all duration-700 ${ringColor.replace(/border-5|border/g, "ring-0").replace(/shadow-\[.*\]/g, "")}`}
            />

            {/* Selection count badge */}
            <div className="flex h-8 min-w-[32px] shrink-0 items-center justify-center gap-1 rounded-full bg-emerald-500/15 px-2 text-[11px] font-black tabular-nums text-emerald-300">
                <span>{selectedCount}</span>
                <span className="opacity-60 text-[9px] uppercase tracking-tighter">{recordLabel}</span>
            </div>

            {/* Contextual Create (+) */}
            <button
                onClick={onCreate}
                className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-blue-500/20 text-blue-300 shadow-lg shadow-blue-500/10 transition-all hover:bg-blue-500/30 active:scale-90"
                title="Contextual Create"
            >
                <Plus className="w-5 h-5" />
            </button>

            {/* Chat with context */}
            {onChat && (
                <button
                    onClick={onChat}
                    className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-indigo-500/20 text-indigo-300 shadow-lg shadow-indigo-500/10 transition-all hover:bg-indigo-500/30 active:scale-90"
                    title="Chat about selection"
                >
                    <MessagesSquare className="w-4.5 h-4.5" />
                </button>
            )}

            {/* Divider - removed for borderless look */}
            <div className="h-5 w-px shrink-0 bg-transparent" />

            {/* Selection actions from the graph */}
            {hasActions ? (
                <div className="heap-action-bar__actions contents">
                    <ActionZoneRenderer
                        actions={contributorActions}
                        onActionClick={(action) =>
                            executeHeapAction(action, selection, handlers)
                        }
                        layoutHint="pillbar"
                        iconOnly={false}
                    />
                </div>
            ) : (
                <span className="text-[10px] text-cortex-500 font-medium px-2">
                    No actions available
                </span>
            )}
        </div>
    );
}

function isPublishAction(action: { id?: string; action?: string; label?: string }): boolean {
    return action.id === "publish" || action.action === "publish" || action.label === "Publish";
}

function contributorActionLabel<T extends { id?: string; action?: string; label: string; title?: string; shortLabel?: string }>(action: T): T {
    const key = action.id || action.action;
    if (key === "regenerate" || action.label === "Regen") {
        return { ...action, label: "Refresh", shortLabel: "Refresh", title: "Refresh this record" };
    }
    if (key === "refine_selection" || action.label === "Refine Selection") {
        return { ...action, label: "Improve summary", shortLabel: "Improve summary", title: "Improve the selected summary" };
    }
    if (key === "synthesize" || action.label === "Synthesize") {
        return { ...action, label: "Summarize selected", shortLabel: "Summarize selected", title: "Summarize selected records" };
    }
    return action;
}
