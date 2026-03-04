import React from "react";

/**
 * Renders a markdown-lite string to HTML.
 * Supports: **bold**, *italic*, `code`, and newlines.
 */
function renderMarkdown(text: string): string {
    return text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
        .replace(/\*(.+?)\*/g, "<em>$1</em>")
        .replace(/`([^`]+)`/g, "<code>$1</code>")
        .replace(/\n/g, "<br />");
}

export interface PayloadContent {
    payload_type: string;
    // Rich text
    text?: string;
    plain_text?: string;
    // Media
    media?: {
        hash?: string;
        mime_type?: string;
        url?: string;
    };
    // Structured data
    data?: Record<string, unknown>;
    structured_data?: Record<string, unknown>;
    // A2UI tree
    tree?: {
        widget?: string;
        passing?: boolean;
        score?: number;
        violations?: Array<{ node: string; error: string }>;
        [key: string]: unknown;
    };
    a2ui?: {
        tree?: Record<string, unknown>;
        [key: string]: unknown;
    };
    // Pointer
    pointer?: string;
}

interface PayloadRendererProps {
    content: PayloadContent;
    expanded?: boolean;
}

export function PayloadRenderer({ content, expanded = false }: PayloadRendererProps) {
    switch (content.payload_type) {
        case "a2ui":
            return renderA2UIPayload(content, expanded);
        case "rich_text":
            return renderRichText(content, expanded);
        case "media":
            return renderMedia(content);
        case "structured_data":
            return renderStructuredData(content);
        case "pointer":
            return renderPointer(content);
        default: {
            const fallbackData = content.data || content.structured_data || content.tree || content.a2ui || { type: content.payload_type };
            return (
                <div className="heap-structured-data">
                    <pre>{JSON.stringify(fallbackData, null, 2)}</pre>
                </div>
            );
        }
    }
}

function renderA2UIPayload(content: PayloadContent, _expanded: boolean) {
    const tree = content.tree || content.a2ui?.tree;
    if (!tree) {
        return <div className="heap-structured-data">Generic A2UI Widget</div>;
    }

    // SIQ Scorecard pattern
    if (tree.widget === "SiqScorecard" || tree.widget === "scorecard") {
        const score = typeof tree.score === "number" ? tree.score : 0;
        const passing = tree.passing !== false && score >= 80;
        const violations = Array.isArray(tree.violations) ? tree.violations : [];

        return (
            <div className={`heap-scorecard ${passing ? "heap-scorecard--passing" : ""}`}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-end", marginBottom: "0.5rem" }}>
                    <span style={{ color: passing ? "#4ade80" : "#f87171", fontWeight: 700, fontSize: "1.125rem" }}>
                        {score}/100
                    </span>
                    <span style={{ fontSize: "0.75rem", textTransform: "uppercase", fontWeight: 600, color: passing ? "#22c55e" : "#ef4444" }}>
                        {passing ? "Passing" : "Failing"}
                    </span>
                </div>
                <div className="heap-scorecard__bar-track">
                    <div
                        className={`heap-scorecard__bar-fill ${passing ? "heap-scorecard__bar-fill--passing" : "heap-scorecard__bar-fill--failing"}`}
                        style={{ width: `${score}%` }}
                    />
                </div>
                {violations.length > 0 && (
                    <ul style={{ listStyle: "none", padding: 0, margin: 0, display: "flex", flexDirection: "column", gap: "4px" }}>
                        {violations.map((v, i) => (
                            <li key={i} className="heap-scorecard__violation">
                                <span className="heap-scorecard__violation-node">[{v.node}]</span>
                                {v.error}
                            </li>
                        ))}
                    </ul>
                )}
                <button className="heap-scorecard__action-btn">Generate Auto-Fix</button>
            </div>
        );
    }

    // Generic A2UI tree — show as JSON
    return <div className="heap-structured-data"><pre>{JSON.stringify(tree, null, 2)}</pre></div>;
}

function renderRichText(content: PayloadContent, expanded: boolean) {
    const text = content.text || content.plain_text || "";
    if (!text) return null;

    return (
        <div
            className={`heap-rich-text ${expanded ? "" : "heap-rich-text--clamped"}`}
            dangerouslySetInnerHTML={{ __html: renderMarkdown(text) }}
        />
    );
}

function renderMedia(content: PayloadContent) {
    const media = content.media;
    if (!media) return null;

    const url = media.url || `https://images.unsplash.com/photo-1542831371-29b0f74f9713?q=80&w=800&auto=format&fit=crop`;
    const hash = media.hash || "unknown";

    return (
        <div className="heap-media-preview">
            <img src={url} alt="Media thumbnail" />
            <div className="heap-media-preview__overlay">
                <span className="heap-media-preview__hash">{hash.substring(0, 16)}...</span>
            </div>
        </div>
    );
}

function renderStructuredData(content: PayloadContent) {
    const data = content.data || content.structured_data || {};
    return (
        <div className="heap-structured-data">
            <pre>{JSON.stringify(data, null, 2)}</pre>
        </div>
    );
}

function renderPointer(content: PayloadContent) {
    return (
        <div className="heap-mention-chip" style={{ marginTop: "0.5rem" }}>
            → {content.pointer || "unknown reference"}
        </div>
    );
}
