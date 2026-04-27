import React from "react";
import {
    Hash,
    ArrowRight,
    Search,
    X,
    Layers,
    Siren,
    Inbox,
    Compass,
    PanelLeftClose,
    Pin,
    Rows3,
    History,
    Edit3,
    Archive,
    ShieldCheck,
    CheckSquare2,
    ClipboardCheck,
} from "lucide-react";
import type { HeapPrimaryViewMode, HeapReviewLane, HeapViewCounts } from "./heapViewModel";
export type HeapFilterMode = "AND" | "OR";

interface HeapFilterSidebarProps {
    filterTerm: string;
    onFilterTermChange: (term: string) => void;
    excludeTerm: string;
    onExcludeTermChange: (term: string) => void;
    filterMode: HeapFilterMode;
    onFilterModeChange: (mode: HeapFilterMode) => void;
    allTags: string[];
    selectedTags: string[];
    onToggleTag: (tag: string) => void;
    allPageLinks: string[];
    selectedPageLinks: string[];
    onTogglePageLink: (pageLink: string) => void;
    pageLinkTerm: string;
    onPageLinkTermChange: (term: string) => void;
    availableReviewLanes: HeapReviewLane[];
    reviewLaneCounts: Record<HeapReviewLane, number>;
    selectedReviewLane: HeapReviewLane | null;
    onReviewLaneChange: (lane: HeapReviewLane | null) => void;
    viewCounts: HeapViewCounts;
    viewMode: HeapPrimaryViewMode;
    onViewModeChange: (mode: HeapPrimaryViewMode) => void;
    multiSelectEnabled: boolean;
    onToggleMultiSelect: () => void;
    searchInputId?: string;
    heapParityEnabled?: boolean;
    isCollapsed?: boolean;
    onToggleCollapse?: () => void;
}

export function HeapFilterSidebar({
    filterTerm,
    onFilterTermChange,
    excludeTerm,
    onExcludeTermChange,
    filterMode,
    onFilterModeChange,
    allTags,
    selectedTags,
    onToggleTag,
    allPageLinks,
    selectedPageLinks,
    onTogglePageLink,
    pageLinkTerm,
    onPageLinkTermChange,
    availableReviewLanes,
    reviewLaneCounts,
    selectedReviewLane,
    onReviewLaneChange,
    viewCounts,
    viewMode,
    onViewModeChange,
    multiSelectEnabled,
    onToggleMultiSelect,
    searchInputId,
    heapParityEnabled = true,
    isCollapsed = false,
    onToggleCollapse,
}: HeapFilterSidebarProps) {
    return (
        <aside className="w-64 max-w-[85vw] bg-cortex-surface-panel/40 backdrop-blur-xl border-r border-white/5 flex flex-col shrink-0 overflow-hidden shadow-2xl z-10 transition-all duration-500">
            {/* Branding Header */}
            <div className="p-5 flex items-center gap-3">
                <div className="flex-1 min-w-0">
                    <h1 className="font-bold text-sm tracking-wide text-cortex-50 truncate">EXPLORE</h1>
                </div>
                <button
                    onClick={onToggleCollapse}
                    className="p-1.5 rounded-lg hover:bg-white/5 text-cortex-500 hover:text-white transition-colors"
                    title="Collapse Sidebar"
                >
                    <PanelLeftClose className="w-4 h-4" />
                </button>
            </div>

            <div className="p-4 flex-1 overflow-y-auto custom-scrollbar">
                <div className="mb-8">
                    <div className="flex flex-col gap-2">
                        {[
                            { id: 'Explore' as HeapPrimaryViewMode, label: 'Explore', icon: <Compass className="w-4 h-4" /> },
                            { id: 'Inbox' as HeapPrimaryViewMode, label: 'Inbox', icon: <Inbox className="w-4 h-4" /> },
                            { id: 'Drafts' as HeapPrimaryViewMode, label: 'Drafts', icon: <Edit3 className="w-4 h-4" /> },
                            { id: 'Tasks' as HeapPrimaryViewMode, label: 'Tasks', icon: <CheckSquare2 className="w-4 h-4" /> },
                            { id: 'Proposals' as HeapPrimaryViewMode, label: 'Proposals', icon: <ClipboardCheck className="w-4 h-4" /> },
                            { id: 'Activity' as HeapPrimaryViewMode, label: 'Activity', icon: <History className="w-4 h-4" /> },
                            { id: 'Pinned' as HeapPrimaryViewMode, label: 'Pinned', icon: <Pin className="w-4 h-4" /> },
                            { id: 'Archive' as HeapPrimaryViewMode, label: 'Archive', icon: <Archive className="w-4 h-4" /> },
                        ].map((item) => {
                            const isActive = viewMode === item.id;
                            const count = viewCounts[item.id];
                            return (
                                <button
                                    key={item.id}
                                    onClick={() => onViewModeChange(item.id)}
                                    title={`${count} records in ${item.label}. Count is calculated from the current heap data.`}
                                    className={`flex items-center justify-between px-3 py-2.5 rounded-xl transition-all duration-200 group ${
                                        isActive
                                            ? "bg-blue-500/10 text-blue-400 shadow-sm border border-blue-500/10"
                                            : "text-cortex-500 hover:bg-cortex-900/50 hover:text-cortex-300 border border-transparent"
                                    }`}
                                >
                                    <div className="flex items-center gap-3">
                                        <span className={isActive ? "text-blue-400" : "text-cortex-600 group-hover:text-cortex-400"}>
                                            {item.icon}
                                        </span>
                                        <span className="text-xs font-semibold tracking-tight">{item.label}</span>
                                    </div>
                                    <span className={`text-[10px] tabular-nums px-2 py-0.5 rounded-full ${
                                        isActive ? "bg-blue-500/20 text-blue-300" : "bg-cortex-900/80 text-cortex-600"
                                    }`}>
                                        {count}
                                    </span>
                                </button>
                            );
                        })}
                    </div>

                    {viewCounts.Urgent > 0 && (
                        <div className="mt-4 p-3 rounded-xl bg-amber-500/5 border border-amber-500/10 animate-pulse flex items-center gap-3">
                            <Siren className="w-4 h-4 text-amber-500" />
                            <span className="text-[11px] font-bold text-amber-200/80 tracking-tight">
                                {viewCounts.Urgent} need attention
                            </span>
                        </div>
                    )}

                    {availableReviewLanes.length > 0 && (
                        <div className="mt-4 p-3 rounded-xl bg-cortex-900/30 border border-white/5">
                            <div className="flex items-center gap-2 px-1 mb-3">
                                <ShieldCheck className="w-3.5 h-3.5 text-cortex-500" />
                                <span className="text-[10px] font-black uppercase tracking-widest text-cortex-500">
                                    Review work
                                </span>
                            </div>
                            <div className="flex flex-wrap gap-1.5">
                                <button
                                    className={`px-2.5 py-1 rounded-md text-[10px] font-mono transition-all ${
                                        selectedReviewLane === null
                                            ? "bg-blue-500/10 text-blue-400"
                                            : "bg-cortex-900/30 text-cortex-500 hover:bg-cortex-800/50 hover:text-cortex-300"
                                    }`}
                                    onClick={() => onReviewLaneChange(null)}
                                >
                                    All
                                </button>
                                {availableReviewLanes.map((lane) => (
                                    <button
                                        key={lane}
                                        className={`px-2.5 py-1 rounded-md text-[10px] font-mono transition-all ${
                                            selectedReviewLane === lane
                                                ? "bg-blue-500/10 text-blue-400"
                                                : "bg-cortex-900/30 text-cortex-500 hover:bg-cortex-800/50 hover:text-cortex-300"
                                        }`}
                                        onClick={() => onReviewLaneChange(lane)}
                                    >
                                        {lane === "private_review" ? "Private review" : "Public review"}{" "}
                                        <span className="opacity-70">({reviewLaneCounts[lane]})</span>
                                    </button>
                                ))}
                            </div>
                        </div>
                    )}
                </div>

                {/* Search */}
                <div>
                    <div className="flex gap-1.5 mb-4">
                        <button
                            className={`flex-1 px-3 py-1.5 rounded-full text-[10px] font-bold tracking-widest transition-all ${filterMode === "AND"
                                ? "bg-blue-500/10 text-blue-400 shadow-sm shadow-blue-500/10"
                                : "bg-cortex-900/50 text-cortex-500 hover:text-cortex-300"
                                }`}
                            onClick={() => onFilterModeChange("AND")}
                        >
                            ALL
                        </button>
                        <button
                            className={`flex-1 px-3 py-1.5 rounded-full text-[10px] font-bold tracking-widest transition-all ${filterMode === "OR"
                                ? "bg-blue-500/10 text-blue-400 shadow-sm shadow-blue-500/10"
                                : "bg-cortex-900/50 text-cortex-500 hover:text-cortex-300"
                                }`}
                            onClick={() => onFilterModeChange("OR")}
                        >
                            ANY
                        </button>
                        <button
                            className={`flex-1 px-3 py-1.5 rounded-full text-[10px] font-bold tracking-widest transition-all ${multiSelectEnabled
                                ? "bg-blue-500/10 text-blue-400 shadow-sm shadow-blue-500/10"
                                : "bg-cortex-900/50 text-cortex-500 hover:text-cortex-300"
                                } ${!heapParityEnabled ? "opacity-30 cursor-not-allowed" : ""}`}
                            disabled={!heapParityEnabled}
                            onClick={onToggleMultiSelect}
                        >
                            MULTI
                        </button>
                    </div>

                    <div className="space-y-2.5">
                        <div className="relative">
                            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-cortex-600" />
                            <input
                                id={searchInputId}
                                type="text"
                                placeholder="Search"
                                className="w-full bg-cortex-surface-base/40 backdrop-blur-sm border border-white/5 rounded-xl pl-9 pr-3 py-2.5 text-xs text-cortex-50 placeholder-cortex-600 focus:outline-none focus:ring-1 focus:ring-blue-500/30 transition-all font-mono"
                                value={filterTerm}
                                onChange={(e) => onFilterTermChange(e.target.value)}
                            />
                        </div>
                        <div className="relative">
                            <X className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-cortex-600" />
                            <input
                                type="text"
                                placeholder="Exclude"
                                className="w-full bg-cortex-surface-base/40 backdrop-blur-sm border border-white/5 rounded-xl pl-9 pr-3 py-2.5 text-xs text-cortex-50 placeholder-cortex-600 focus:outline-none focus:ring-1 focus:ring-red-500/20 transition-all font-mono"
                                value={excludeTerm}
                                onChange={(e) => onExcludeTermChange(e.target.value)}
                            />
                        </div>
                    </div>

                    {/* Tag Cloud */}
                    {allTags.length > 0 && (
                        <div className="mt-10">
                            <div className="px-2 mb-3">
                                <Hash className="w-3.5 h-3.5 text-cortex-500" />
                            </div>
                            <div className="flex flex-wrap gap-1.5 px-1">
                                {allTags.map(tag => (
                                    <button
                                        key={tag}
                                        className={`px-2.5 py-1 rounded-md text-[10px] font-mono transition-all ${selectedTags.includes(tag)
                                            ? "bg-blue-500/10 text-blue-400"
                                            : "bg-cortex-900/30 text-cortex-500 hover:bg-cortex-800/50 hover:text-cortex-300"
                                            }`}
                                        onClick={() => onToggleTag(tag)}
                                    >
                                        #{tag}
                                    </button>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* Page Link Facets */}
                    <div className="mt-10 pt-6 border-t border-white/5">
                        <div className="px-2 mb-3">
                            <Layers className="w-3.5 h-3.5 text-cortex-500" />
                        </div>
                        <div className="flex flex-col gap-2.5">
                            <input
                                type="text"
                                placeholder="Filter page links"
                                className="w-full bg-cortex-900/40 rounded-lg px-3 py-2 text-[10px] text-cortex-50 placeholder-cortex-600 focus:outline-none focus:bg-cortex-900/60 transition-all font-mono border-none"
                                value={pageLinkTerm}
                                onChange={(e) => onPageLinkTermChange(e.target.value)}
                            />
                        </div>
                        {allPageLinks.length > 0 && (
                            <div className="flex flex-wrap gap-1.5 px-1">
                                {allPageLinks
                                    .filter((pageLink) =>
                                        pageLinkTerm.trim().length === 0
                                            ? true
                                            : pageLink.toLowerCase().includes(pageLinkTerm.trim().toLowerCase())
                                    )
                                    .slice(0, 16)
                                    .map((pageLink) => (
                                        <button
                                            key={pageLink}
                                            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[10px] font-mono transition-all ${selectedPageLinks.includes(pageLink)
                                                ? "bg-cyan-500/10 text-cyan-300 ring-1 ring-cyan-500/10"
                                                : "bg-cortex-900/30 text-cortex-500 hover:bg-cortex-800/50 hover:text-cortex-300"
                                                }`}
                                            onClick={() => onTogglePageLink(pageLink)}
                                            title={pageLink}
                                        >
                                            <ArrowRight className="w-2.5 h-2.5 opacity-40 group-hover:opacity-100 transition-opacity" />
                                            {pageLink.length > 12 ? `${pageLink.slice(0, 10)}…` : pageLink}
                                        </button>
                                    ))}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </aside>
    );
}
