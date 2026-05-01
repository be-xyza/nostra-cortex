import React from "react";
import { ToolbarActionDescriptor } from "../../contracts";
import {
    Plus,
    Trash,
    Trash2,
    Download,
    History,
    FileText,
    CheckCircle,
    RefreshCcw,
    RefreshCw,
    LayoutGrid,
    Upload,
    Sparkles,
    Pin,
    Wand2,
    MessageSquare,
    GitBranch,
    CircleHelp,
} from "lucide-react";

const ICON_MAP: Record<string, React.ReactNode> = {
    "plus": <Plus className="w-4 h-4 stroke-3" />,
    "trash": <Trash className="w-4 h-4" />,
    "trash2": <Trash2 className="w-4 h-4" />,
    "download": <Download className="w-4 h-4" />,
    "history": <History className="w-4 h-4" />,
    "filetext": <FileText className="w-4 h-4" />,
    "checkcircle": <CheckCircle className="w-4 h-4" />,
    "refreshccw": <RefreshCcw className="w-4 h-4" />,
    "refreshcw": <RefreshCw className="w-4 h-4" />,
    "layoutgrid": <LayoutGrid className="w-4 h-4" />,
    "upload": <Upload className="w-4 h-4" />,
    "sparkles": <Sparkles className="w-4 h-4" />,
    "pin": <Pin className="w-4 h-4" />,
    "wand2": <Wand2 className="w-4 h-4" />,
    "messagesquare": <MessageSquare className="w-4 h-4" />,
    "gitbranch": <GitBranch className="w-4 h-4" />,
};

function normalizeIconKey(icon: string): string {
    return icon.replace(/[^a-z0-9]/gi, "").toLowerCase();
}

interface ActionZoneRendererProps {
    actions: ToolbarActionDescriptor[];
    onActionClick: (action: ToolbarActionDescriptor) => void;
    layoutHint?: "row" | "pillbar";
    iconOnly?: boolean;
}

export function ActionZoneRenderer({ actions, onActionClick, layoutHint = "pillbar", iconOnly = false }: ActionZoneRendererProps) {
    const visibleActions = actions.filter((action) => action.visible !== false);
    if (!visibleActions.length) return null;

    return (
        <div className={`flex max-w-full flex-wrap items-center gap-1 ${layoutHint === "pillbar" ? "px-0 sm:px-2" : ""}`}>
            {visibleActions.map((action, i) => {
                const iconNode = action.icon
                    ? (ICON_MAP[normalizeIconKey(action.icon)] || <CircleHelp className="w-4 h-4 opacity-70" />)
                    : null;
                const isPrimary = action.emphasis === "primary" || action.emphasis === "accent";
                const isDanger = action.emphasis === "danger";

                let btnClass = "flex min-h-8 shrink-0 items-center justify-center whitespace-nowrap transition-colors ";
                let spanClass = "tracking-wide ";

                if (layoutHint === "pillbar") {
                    btnClass += "p-2 rounded-full text-xs font-medium gap-1.5 ";
                    if (action.enabled) {
                        if (isPrimary) {
                            btnClass += "px-3 py-1.5 border border-emerald-500/30 bg-emerald-500/10 text-emerald-300 hover:bg-emerald-500/20 hover:text-emerald-200 shadow-[0_0_15px_rgba(16,185,129,0.1)] active:scale-95";
                        } else if (isDanger) {
                            btnClass += "text-red-500 hover:bg-red-500/10 hover:text-red-400";
                        } else if (action.id.includes("publish")) {
                            btnClass += "px-3 py-1.5 border border-blue-500/30 bg-blue-500/10 text-blue-300 hover:bg-blue-500/20 hover:text-blue-200 active:scale-95";
                        } else {
                            btnClass += "text-slate-400 hover:bg-white/5 hover:text-slate-200";
                        }
                    } else {
                        btnClass += "text-slate-600 cursor-not-allowed opacity-50";
                        if (isPrimary) btnClass += " border border-slate-700/30";
                    }
                } else {
                    btnClass += "px-4 py-1.5 text-xs rounded-full font-bold tracking-tight border gap-2 ";
                    if (action.enabled) {
                        if (isPrimary) {
                            btnClass += "bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-500/20 border-blue-400/30 active:scale-95";
                        } else if (isDanger) {
                            btnClass += "bg-red-500/10 text-red-500 hover:bg-red-500/20 border-red-500/30 active:scale-95";
                        } else {
                            btnClass += "bg-cortex-800/80 hover:bg-cortex-700 text-cortex-200 border-cortex-700 shadow-md active:scale-95";
                        }
                    } else {
                        btnClass += "bg-cortex-900/40 text-cortex-600 border-cortex-800 cursor-not-allowed opacity-50";
                    }
                }

                return (
                    <React.Fragment key={action.id}>
                        <button
                            className={btnClass}
                            onClick={() => onActionClick(action)}
                            disabled={!action.enabled}
                            title={action.disabledReason || action.label}
                        >
                            {iconNode}
                            {!iconOnly && (action.shortLabel || action.label) && (
                                <span className={spanClass}>{action.shortLabel || action.label}</span>
                            )}
                        </button>
                        {i < visibleActions.length - 1 && action.group !== visibleActions[i + 1].group && layoutHint === "pillbar" && (
                            <div className="w-px h-6 bg-white/5 my-auto mx-1" />
                        )}
                    </React.Fragment>
                );
            })}
        </div>
    );
}
