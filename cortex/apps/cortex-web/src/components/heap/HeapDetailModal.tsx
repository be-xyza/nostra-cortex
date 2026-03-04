import React from "react";
import type { HeapBlockListItem } from "../../contracts";
import { PayloadRenderer } from "./PayloadRenderer";
import { surfaceToPayloadContent } from "./HeapBlockCard";
import "./heap.css";

interface HeapDetailModalProps {
    block: HeapBlockListItem;
    onClose: () => void;
}

const TYPE_COLOR: Record<string, string> = {
    scorecard: "red",
    note: "blue",
    media: "purple",
    task: "green",
};

export function HeapDetailModal({ block, onClose }: HeapDetailModalProps) {
    const { projection } = block;
    const blockType = projection.blockType || "note";
    const color = TYPE_COLOR[blockType] || "blue";
    const behaviors = (block.surfaceJson as Record<string, unknown>)?.behaviors as string[] || [];
    const attributes = projection.attributes || {};
    const payloadContent = surfaceToPayloadContent(block);

    return (
        <div className="heap-modal-backdrop" onClick={onClose}>
            <div className="heap-modal-content" onClick={(e) => e.stopPropagation()}>
                {/* Header */}
                <div className="heap-modal__header">
                    <div style={{ flex: 1 }}>
                        <div style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "0.5rem" }}>
                            <span className={`heap-badge heap-badge--outline heap-badge--${color}`}>{blockType}</span>
                            {behaviors.map(beh => (
                                <span key={beh} className={`heap-badge heap-badge--outline heap-badge--${beh === "urgent" ? "red" : beh === "pinned" ? "yellow" : "slate"}`}>
                                    {beh}
                                </span>
                            ))}
                        </div>
                        <h2 className="heap-modal__title">{projection.title}</h2>
                        <p className="heap-modal__meta">
                            {projection.artifactId} · {new Date(projection.emittedAt || projection.updatedAt).toLocaleString()}
                        </p>
                    </div>
                    <button className="heap-modal__close-btn" onClick={onClose}>✕</button>
                </div>

                {/* Body */}
                <div className="heap-modal__body heap-scroll">
                    {/* Attributes */}
                    {Object.keys(attributes).length > 0 && (
                        <div style={{ marginBottom: "1.25rem" }}>
                            <h3 className="heap-modal__section-label">Attributes</h3>
                            <div style={{ display: "flex", flexWrap: "wrap", gap: "0.5rem" }}>
                                {Object.entries(attributes).map(([k, v]) => (
                                    <div key={k} className="heap-attr-chip" style={{ fontSize: "0.75rem", padding: "4px 8px" }}>
                                        <span className="heap-attr-chip__key">{k}:</span>
                                        <span className="heap-attr-chip__value">{v}</span>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* Content (expanded — no line clamp) */}
                    <div style={{ marginBottom: "1.25rem" }}>
                        <h3 className="heap-modal__section-label">Content · {payloadContent.payload_type}</h3>
                        <PayloadRenderer content={payloadContent} expanded={true} />
                    </div>

                    {/* Relations */}
                    <div>
                        <h3 className="heap-modal__section-label">Relations</h3>
                        <div style={{ display: "flex", flexWrap: "wrap", gap: "0.5rem" }}>
                            {projection.tags?.map(t => (
                                <span key={t} className="heap-tag-chip" style={{ fontSize: "0.75rem", padding: "4px 8px", borderRadius: "9999px" }}>#{t}</span>
                            ))}
                            {projection.mentionsInline?.map(m => (
                                <span key={m} className="heap-mention-chip" style={{ padding: "4px 8px", borderRadius: "9999px" }}>
                                    🔗 {m}
                                </span>
                            ))}
                            {projection.pageLinks?.map((pageLink) => (
                                <span key={pageLink} className="heap-page-link-chip" style={{ padding: "4px 8px", borderRadius: "9999px" }}>
                                    ⇢ {pageLink}
                                </span>
                            ))}
                            {(!projection.tags?.length && !projection.mentionsInline?.length && !projection.pageLinks?.length) && (
                                <span style={{ fontSize: "0.75rem", color: "#64748b", fontStyle: "italic" }}>No relations.</span>
                            )}
                        </div>
                    </div>
                </div>

                {/* Footer Actions */}
                <div className="heap-modal__footer">
                    <button className="heap-modal__footer-btn">✎ Edit</button>
                    <button className="heap-modal__footer-btn">⇥ Move</button>
                    <button className="heap-modal__footer-btn heap-modal__footer-btn--accent">✦ Regenerate</button>
                </div>
            </div>
        </div>
    );
}
