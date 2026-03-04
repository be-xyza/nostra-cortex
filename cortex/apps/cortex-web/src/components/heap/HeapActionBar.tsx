import React from "react";
import { workbenchApi } from "../../api";
import "./heap.css";

interface HeapActionBarProps {
    selectedBlockIds: string[];
    onDeselect: () => void;
    onPinToggled: () => void;
    onDeleted: () => void;
    onRegenerate: () => void;
    onContextBundle: () => void;
    onExport: () => void;
    onHistory: () => void;
    onPublish: () => void;
}

export function HeapActionBar({
    selectedBlockIds,
    onDeselect,
    onPinToggled,
    onDeleted,
    onRegenerate,
    onContextBundle,
    onExport,
    onHistory,
    onPublish
}: HeapActionBarProps) {
    const selectedCount = selectedBlockIds.length;

    const handlePin = async () => {
        try {
            await Promise.all(selectedBlockIds.map((artifactId) => workbenchApi.pinHeapBlock(artifactId)));
            onPinToggled();
        } catch (err) {
            console.error("Failed to pin block:", err);
        }
    };

    const handleDelete = async () => {
        try {
            await Promise.all(selectedBlockIds.map((artifactId) => workbenchApi.deleteHeapBlock(artifactId)));
            onDeleted();
        } catch (err) {
            console.error("Failed to delete block:", err);
        }
    };

    return (
        <div className="heap-action-bar">
            <div className="heap-action-bar__inner">
                {/* Block Info */}
                <div className="heap-action-bar__info">
                    <span className="heap-action-bar__info-label">{selectedCount} Block{selectedCount > 1 ? "s" : ""}</span>
                    <span className="heap-action-bar__info-id">
                        {selectedCount === 1
                            ? `${selectedBlockIds[0]?.substring(0, 12)}...`
                            : `${selectedBlockIds[0]?.substring(0, 8)}... +${selectedCount - 1}`}
                    </span>
                </div>

                {/* Actions */}
                <div className="heap-action-bar__actions">
                    <button className="heap-action-bar__btn heap-action-bar__btn--regen" onClick={onRegenerate} title="Regenerate">
                        ✦ <span style={{ fontSize: "0.75rem" }}>Regen</span>
                    </button>
                    <div className="heap-action-bar__divider" />
                    <button className="heap-action-bar__btn" onClick={onContextBundle} title="Send selected context bundle to agent">⊞ Send to Agent</button>
                    <button className="heap-action-bar__btn" onClick={onExport} title="Export">⤓ Export</button>
                    <button className="heap-action-bar__btn" onClick={onHistory} title="History">🕘 History</button>
                    <button className="heap-action-bar__btn heap-action-bar__btn--publish" onClick={onPublish} title="Publish via Steward Gate">⇪ Publish</button>
                    <button className="heap-action-bar__btn heap-action-bar__btn--pin" onClick={handlePin} title="Toggle Pin">📌</button>
                    <div className="heap-action-bar__divider" />
                    <button className="heap-action-bar__btn heap-action-bar__btn--delete" onClick={handleDelete} title="Delete Block">🗑</button>
                </div>

                {/* Close */}
                <button className="heap-action-bar__close" onClick={onDeselect} title="Deselect">✕</button>
            </div>
        </div>
    );
}
