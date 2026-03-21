import React from "react";
import { Plus } from "lucide-react";
import type { ActionZonePlan, ActionSelectionContext } from "../../contracts";
import { ActionZoneRenderer } from "../commons/ActionZoneRenderer";
import { executeHeapAction, type ActionHandlers } from "./actionExecutor";

interface HeapActionBarProps {
    selectionZonePlan?: ActionZonePlan | null;
    selection: ActionSelectionContext;
    handlers: ActionHandlers;
    onCreate: () => void;
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
    status,
}: HeapActionBarProps) {
    const selectedCount = selection.selectedCount;

    // Hidden by default — only render when blocks are selected
    if (selectedCount === 0) return null;

    const hasActions = Boolean(
        selectionZonePlan?.actions.some((a) => a.visible !== false),
    );

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
        <div className="heap-action-bar fixed bottom-8 left-1/2 -translate-x-1/2 z-50 flex items-center gap-3 px-4 py-2 bg-cortex-950/90 rounded-full backdrop-blur-xl shadow-2xl transition-all animate-in slide-in-from-bottom duration-300 pointer-events-auto" data-selection-count={selectedCount}>
            {/* Status ring - subtle glow instead of ring */}
            <div
                className={`absolute inset-0 rounded-full pointer-events-none transition-all duration-700 ${ringColor.replace(/border-5|border/g, "ring-0").replace(/shadow-\[.*\]/g, "")}`}
            />

            {/* Selection count badge */}
            <div className="flex items-center justify-center h-8 min-w-[32px] px-2 rounded-full bg-emerald-500/15 text-emerald-300 text-[11px] font-black gap-1 tabular-nums">
                <span>{selectedCount}</span>
                <span className="opacity-60 text-[9px] uppercase tracking-tighter">Blocks</span>
            </div>

            {/* Contextual Create (+) */}
            <button
                onClick={onCreate}
                className="flex items-center justify-center w-10 h-10 rounded-full bg-blue-500/20 text-blue-300 hover:bg-blue-500/30 transition-all active:scale-90 shadow-lg shadow-blue-500/10"
                title="Contextual Create"
            >
                <Plus className="w-5 h-5" />
            </button>

            {/* Divider - removed for borderless look */}
            <div className="w-px h-5 bg-transparent" />

            {/* Selection actions from the graph */}
            {hasActions && selectionZonePlan ? (
                <div className="heap-action-bar__actions contents">
                    <ActionZoneRenderer
                        actions={selectionZonePlan.actions}
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
