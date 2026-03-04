import React from "react";
import "./heap.css";

export type HeapViewMode = "All" | "Pinned" | "Urgent" | "Unlinked";
export type HeapFilterMode = "AND" | "OR";

interface HeapFilterSidebarProps {
    viewMode: HeapViewMode;
    onViewModeChange: (mode: HeapViewMode) => void;
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
    blockCounts: Record<HeapViewMode, number>;
    multiSelectEnabled: boolean;
    onToggleMultiSelect: () => void;
    searchInputId?: string;
    heapParityEnabled?: boolean;
}

const VIEW_MODES: HeapViewMode[] = ["All", "Pinned", "Urgent", "Unlinked"];
const VIEW_ICONS: Record<HeapViewMode, string> = {
    All: "▦",
    Pinned: "📌",
    Urgent: "⚠",
    Unlinked: "🔗",
};

export function HeapFilterSidebar({
    viewMode,
    onViewModeChange,
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
    blockCounts,
    multiSelectEnabled,
    onToggleMultiSelect,
    searchInputId,
    heapParityEnabled = true,
}: HeapFilterSidebarProps) {
    return (
        <aside className="heap-filter-sidebar heap-scroll">
            {/* Branding Header */}
            <div style={{ padding: "1.25rem", borderBottom: "1px solid var(--cortex-800)", display: "flex", alignItems: "center", gap: "0.75rem" }}>
                <div style={{ background: "rgba(59,130,246,0.2)", padding: "0.5rem", borderRadius: "0.5rem" }}>
                    <span style={{ fontSize: "1.25rem" }}>◈</span>
                </div>
                <div>
                    <h1 style={{ fontWeight: 700, fontSize: "0.875rem", letterSpacing: "0.05em", color: "#f1f5f9" }}>Cortex Heap</h1>
                    <p style={{ fontSize: "10px", color: "#64748b", fontFamily: "monospace" }}>Workspace: Research</p>
                </div>
            </div>

            <div style={{ padding: "1rem", flex: 1 }}>
                {/* View Modes */}
                <div style={{ marginBottom: "1.5rem" }}>
                    <h2 className="heap-filter-sidebar__section-label">Heap Views</h2>
                    <nav style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                        {VIEW_MODES.map(v => (
                            <button
                                key={v}
                                onClick={() => onViewModeChange(v)}
                                className={`heap-filter-sidebar__nav-btn ${viewMode === v ? "heap-filter-sidebar__nav-btn--active" : ""}`}
                            >
                                <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
                                    <span style={{ opacity: 0.7 }}>{VIEW_ICONS[v]}</span>
                                    {v}
                                </div>
                                <span className="heap-filter-sidebar__count">{blockCounts[v]}</span>
                            </button>
                        ))}
                    </nav>
                </div>

                {/* Search */}
                <div>
                    <h2 className="heap-filter-sidebar__section-label">Compound Filter</h2>
                    <div style={{ display: "flex", gap: "6px", marginBottom: "0.5rem" }}>
                        <button
                            className={`heap-filter-sidebar__tag-btn ${filterMode === "AND" ? "heap-filter-sidebar__tag-btn--active" : ""}`}
                            onClick={() => onFilterModeChange("AND")}
                        >
                            AND
                        </button>
                        <button
                            className={`heap-filter-sidebar__tag-btn ${filterMode === "OR" ? "heap-filter-sidebar__tag-btn--active" : ""}`}
                            onClick={() => onFilterModeChange("OR")}
                        >
                            OR
                        </button>
                        <button
                            className={`heap-filter-sidebar__tag-btn ${multiSelectEnabled ? "heap-filter-sidebar__tag-btn--active" : ""}`}
                            disabled={!heapParityEnabled}
                            style={!heapParityEnabled ? { opacity: 0.5, cursor: "not-allowed" } : undefined}
                            onClick={onToggleMultiSelect}
                        >
                            Multi
                        </button>
                    </div>
                    <input
                        id={searchInputId}
                        type="text"
                        placeholder="Include terms (comma separated) · Cmd/Ctrl+K"
                        className="heap-filter-sidebar__search"
                        value={filterTerm}
                        onChange={(e) => onFilterTermChange(e.target.value)}
                    />
                    <input
                        type="text"
                        placeholder="Exclude terms (NOT)"
                        className="heap-filter-sidebar__search"
                        style={{ marginTop: "0.5rem" }}
                        value={excludeTerm}
                        onChange={(e) => onExcludeTermChange(e.target.value)}
                    />

                    {/* Tag Cloud */}
                    {allTags.length > 0 && (
                        <div style={{ marginTop: "1rem", display: "flex", flexWrap: "wrap", gap: "6px", padding: "0 4px" }}>
                            {allTags.map(tag => (
                                <button
                                    key={tag}
                                    className={`heap-filter-sidebar__tag-btn ${selectedTags.includes(tag) ? "heap-filter-sidebar__tag-btn--active" : ""}`}
                                    onClick={() => onToggleTag(tag)}
                                >
                                    #{tag}
                                </button>
                            ))}
                        </div>
                    )}

                    {/* Page Link Facets */}
                    <div style={{ marginTop: "1rem" }}>
                        <input
                            type="text"
                            placeholder="Filter page links"
                            className="heap-filter-sidebar__search"
                            value={pageLinkTerm}
                            onChange={(e) => onPageLinkTermChange(e.target.value)}
                        />
                        {allPageLinks.length > 0 && (
                            <div style={{ marginTop: "0.65rem", display: "flex", flexWrap: "wrap", gap: "6px", padding: "0 4px" }}>
                                {allPageLinks
                                    .filter((pageLink) =>
                                        pageLinkTerm.trim().length === 0
                                            ? true
                                            : pageLink.toLowerCase().includes(pageLinkTerm.trim().toLowerCase())
                                    )
                                    .slice(0, 12)
                                    .map((pageLink) => (
                                        <button
                                            key={pageLink}
                                            className={`heap-filter-sidebar__tag-btn ${selectedPageLinks.includes(pageLink) ? "heap-filter-sidebar__tag-btn--active" : ""}`}
                                            onClick={() => onTogglePageLink(pageLink)}
                                            title={pageLink}
                                        >
                                            ⇢ {pageLink.slice(0, 8)}
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
