import React from "react";
import type { HeapBlockListItem } from "../../contracts";
import { PayloadRenderer, PayloadContent } from "./PayloadRenderer";
import { A2UIInterpreter, type A2UINode } from "../a2ui/A2UIInterpreter";
import { NdlMetadataBlock } from "../ndl/NdlMetadataBlock";
import "./heap.css";

interface HeapBlockCardProps {
    block: HeapBlockListItem;
    isSelected: boolean;
    onClick: (event: React.MouseEvent<HTMLDivElement>) => void;
    onDoubleClick: () => void;
    isRegenerating?: boolean;
    children?: React.ReactNode;
}

const BEHAVIOR_BADGE_COLOR: Record<string, string> = {
    urgent: "red",
    pinned: "yellow",
    completed: "green",
    "read-only": "slate",
    collapsed: "slate",
};

const TYPE_COLOR: Record<string, string> = {
    scorecard: "red",
    note: "blue",
    media: "purple",
    task: "green",
};

function surfaceToPayloadContent(block: HeapBlockListItem): PayloadContent {
    const surface = block.surfaceJson || {};
    const payloadType = (surface as Record<string, unknown>).payload_type as string
        || (surface as Record<string, unknown>).payloadType as string
        || block.projection.blockType
        || "structured_data";

    return {
        payload_type: payloadType,
        text: (surface as Record<string, unknown>).text as string | undefined,
        plain_text: (surface as Record<string, unknown>).plain_text as string | undefined,
        media: (surface as Record<string, unknown>).media as PayloadContent["media"],
        data: (surface as Record<string, unknown>).data as Record<string, unknown> | undefined,
        structured_data: (surface as Record<string, unknown>).structured_data as Record<string, unknown> | undefined,
        tree: (surface as Record<string, unknown>).tree as PayloadContent["tree"],
        a2ui: (surface as Record<string, unknown>).a2ui as PayloadContent["a2ui"],
        pointer: (surface as Record<string, unknown>).pointer as string | undefined,
    };
}

function surfaceToNestedNode(block: HeapBlockListItem): A2UINode | null {
    const surface = block.surfaceJson as Record<string, unknown> | undefined;
    const candidate = surface?.nestedA2uiTree;
    if (!candidate || typeof candidate !== "object" || Array.isArray(candidate)) {
        return null;
    }
    return candidate as A2UINode;
}

export function HeapBlockCard({
    block,
    isSelected,
    onClick,
    onDoubleClick,
    isRegenerating,
    children,
}: HeapBlockCardProps) {
    const { projection } = block;
    const blockType = projection.blockType || "note";
    const surface = (block.surfaceJson as Record<string, unknown>) || {};
    const behaviors = surface.behaviors as string[] || [];
    const behaviorBadges = block.pinnedAt && !behaviors.includes("pinned")
        ? [...behaviors, "pinned"]
        : behaviors;
    const blockColor = TYPE_COLOR[blockType] || "blue";
    const attributes = projection.attributes || {};
    const isCollapsed = behaviors.includes("collapsed");
    const emittedAt = projection.emittedAt || projection.updatedAt;

    const cardClass = [
        "heap-block-card",
        isSelected ? "heap-block-card--selected" : "",
        isCollapsed ? "heap-block-card--collapsed" : "",
        isRegenerating ? "heap-shimmer" : "",
    ].filter(Boolean).join(" ");

    const payloadContent = surfaceToPayloadContent(block);
    const nestedNode = surfaceToNestedNode(block);

    return (
        <div className={cardClass} onClick={onClick} onDoubleClick={(e) => { e.stopPropagation(); onDoubleClick(); }}>
            <div className="heap-block-card__header">
                <div style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}>
                    <span className={`heap-badge heap-badge--outline heap-badge--${blockColor}`}>{blockType}</span>
                    <span style={{ fontSize: "10px", color: "#64748b", fontFamily: "JetBrains Mono, monospace" }}>
                        {formatTime(emittedAt)}
                    </span>
                </div>
                <div style={{ display: "flex", gap: "4px", flexWrap: "wrap", justifyContent: "flex-end", maxWidth: "65%" }}>
                    {behaviorBadges.map((behavior) => (
                        <span
                            key={behavior}
                            className={`heap-badge heap-badge--outline heap-badge--${BEHAVIOR_BADGE_COLOR[behavior] || "slate"}`}
                        >
                            {behavior}
                        </span>
                    ))}
                </div>
            </div>

            <NdlMetadataBlock
                typeIndicator={blockType.toUpperCase()}
                versionChain={String(surface.version || "v1.0")}
                phase={String(surface.phase || "Alpha")}
                confidence={typeof surface.confidence === "number" ? surface.confidence : 50}
                authorityScope={String(surface.authority_scope || "Local")}
            />

            {/* Body */}
            <div className="heap-block-card__body">
                <h3 className="heap-block-card__title">{projection.title}</h3>

                {/* Attributes */}
                {!isCollapsed && Object.keys(attributes).length > 0 && (
                    <div style={{ display: "flex", flexWrap: "wrap", gap: "0.5rem", margin: "0.75rem 0" }}>
                        {Object.entries(attributes).map(([k, v]) => (
                            <div key={k} className="heap-attr-chip">
                                <span className="heap-attr-chip__key">{k}:</span>
                                <span className="heap-attr-chip__value">{v}</span>
                            </div>
                        ))}
                    </div>
                )}

                {/* Payload */}
                {!isCollapsed && <PayloadRenderer content={payloadContent} />}
                {!isCollapsed && nestedNode && (
                    <div className="heap-nested-tree">
                        <A2UIInterpreter node={nestedNode} />
                    </div>
                )}
                {!isCollapsed && children && (
                    <div className="heap-nested-tree">{children}</div>
                )}
            </div>

            {/* Footer */}
            <div className="heap-block-card__footer">
                <span className="heap-payload-label">{payloadContent.payload_type}</span>
                {projection.pageLinks?.map((pageLink) => (
                    <span key={pageLink} className="heap-page-link-chip">⇢ {pageLink.substring(0, 8)}</span>
                ))}
                {projection.mentionsInline?.map(m => (
                    <span key={m} className="heap-mention-chip">🔗 {m.substring(0, 8)}</span>
                ))}
                {projection.tags?.map(t => (
                    <span key={t} className="heap-tag-chip">#{t}</span>
                ))}
            </div>
        </div>
    );
}

function formatTime(isoString: string): string {
    try {
        return new Date(isoString).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    } catch {
        return isoString;
    }
}

export { surfaceToPayloadContent };
