import React, { useEffect, useRef, useState } from "react";
import { MoreHorizontal } from "lucide-react";
import type { ActionSelectionContext, ToolbarActionDescriptor } from "../../contracts";
import type { ActionHandlers } from "./actionExecutor";
import { executeHeapAction } from "./actionExecutor";

interface HeapCardActionMenuProps {
    actions: ToolbarActionDescriptor[];
    selection: ActionSelectionContext;
    handlers: ActionHandlers;
}

export function HeapCardActionMenu({ actions, selection, handlers }: HeapCardActionMenuProps) {
    const [open, setOpen] = useState(false);
    const menuRef = useRef<HTMLDivElement | null>(null);
    const visibleActions = actions.filter((action) => action.visible !== false);

    useEffect(() => {
        if (!open) return;
        const handlePointerDown = (event: MouseEvent) => {
            if (!menuRef.current?.contains(event.target as Node)) {
                setOpen(false);
            }
        };
        window.addEventListener("mousedown", handlePointerDown);
        return () => window.removeEventListener("mousedown", handlePointerDown);
    }, [open]);

    if (!visibleActions.length) {
        return null;
    }

    return (
        <div ref={menuRef} className="relative">
            <button
                onClick={(event) => {
                    event.stopPropagation();
                    setOpen((value) => !value);
                }}
                className="inline-flex h-8 w-8 items-center justify-center rounded-full border border-slate-700/60 bg-slate-950/70 text-slate-400 opacity-0 transition-all hover:border-slate-500 hover:text-slate-100 group-hover/card:opacity-100"
                aria-label="Open block actions"
                title="Block actions"
            >
                <MoreHorizontal className="h-4 w-4" />
            </button>
            {open && (
                <div
                    className="absolute right-0 top-10 z-20 flex min-w-[180px] flex-col rounded-2xl border border-slate-700/70 bg-slate-950/95 p-2 shadow-[0_24px_60px_rgba(2,6,23,0.7)] backdrop-blur-xl"
                    onClick={(event) => event.stopPropagation()}
                >
                    {visibleActions.map((action) => (
                        <button
                            key={action.id}
                            onClick={async () => {
                                if (!action.enabled) return;
                                await executeHeapAction(action, selection, handlers);
                                setOpen(false);
                            }}
                            disabled={!action.enabled}
                            className={`flex items-center justify-between rounded-xl px-3 py-2 text-left text-sm transition-colors ${
                                action.enabled
                                    ? action.emphasis === "danger"
                                        ? "text-red-300 hover:bg-red-500/10"
                                        : "text-slate-200 hover:bg-slate-800/80"
                                    : "cursor-not-allowed text-slate-600"
                            }`}
                            title={action.disabledReason || action.label}
                        >
                            <span className="font-medium">{action.label}</span>
                            {action.stewardGate?.required && (
                                <span className="rounded-full border border-blue-500/20 bg-blue-500/10 px-1.5 py-0.5 text-[9px] font-black uppercase tracking-wide text-blue-200">
                                    Gate
                                </span>
                            )}
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}
