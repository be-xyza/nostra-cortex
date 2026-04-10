// @ts-nocheck
import React, { useEffect, useState } from "react";
import { BookmarkPlus, X } from "lucide-react";

interface ExploreSavedViewModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: (label: string) => void;
    initialLabel?: string;
    description?: string;
}

export function ExploreSavedViewModal({
    isOpen,
    onClose,
    onConfirm,
    initialLabel = "",
    description = "Save the current Explore state as a personal sidebar shortcut.",
}: ExploreSavedViewModalProps) {
    const [label, setLabel] = useState(initialLabel);

    useEffect(() => {
        if (isOpen) {
            setLabel(initialLabel);
        }
    }, [initialLabel, isOpen]);

    if (!isOpen) {
        return null;
    }

    const handleSubmit = (event: React.FormEvent) => {
        event.preventDefault();
        const trimmed = label.trim();
        if (trimmed) {
            onConfirm(trimmed);
        }
    };

    return (
        <div className="fixed inset-0 z-[130] flex items-center justify-center bg-slate-950/75 px-4 py-8 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full max-w-md overflow-hidden rounded-[28px] border border-white/10 bg-[linear-gradient(180deg,rgba(15,23,42,0.98),rgba(2,6,23,0.96))] shadow-[0_32px_120px_-48px_rgba(0,0,0,0.8)]">
                <div className="flex items-center justify-between border-b border-white/8 px-5 py-4">
                    <div>
                        <div className="text-[10px] font-black uppercase tracking-[0.32em] text-cortex-500">Saved View</div>
                        <div className="mt-1 text-sm font-semibold text-white">Name this view</div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="rounded-full border border-white/10 bg-white/[0.04] p-2 text-white/70 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                        aria-label="Close save view dialog"
                    >
                        <X className="h-4 w-4" />
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="p-5">
                    <p className="text-sm leading-6 text-cortex-300/70">
                        {description}
                    </p>

                    <label htmlFor="saved-view-label" className="mt-4 block text-[10px] font-black uppercase tracking-[0.26em] text-cortex-500">
                        View label
                    </label>
                    <input
                        id="saved-view-label"
                        autoFocus
                        value={label}
                        onChange={(event) => setLabel(event.target.value)}
                        placeholder="Dense Research"
                        className="mt-2 w-full rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-3 text-sm text-white placeholder:text-cortex-500 outline-none transition focus:border-cyan-400/40 focus:ring-2 focus:ring-cyan-500/15"
                    />

                    <div className="mt-6 flex gap-3">
                        <button
                            type="button"
                            onClick={onClose}
                            className="flex-1 rounded-full border border-white/10 bg-white/[0.03] px-4 py-2.5 text-[11px] font-semibold uppercase tracking-[0.18em] text-cortex-200 transition hover:bg-white/[0.06]"
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            disabled={!label.trim()}
                            className="flex-1 rounded-full border border-cyan-400/20 bg-cyan-500/15 px-4 py-2.5 text-[11px] font-semibold uppercase tracking-[0.18em] text-cyan-100 transition hover:bg-cyan-500/20 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                            <span className="inline-flex items-center gap-2">
                                <BookmarkPlus className="h-4 w-4" />
                                Save View
                            </span>
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
