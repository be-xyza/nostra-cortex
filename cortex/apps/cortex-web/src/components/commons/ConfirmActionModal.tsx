import React from "react";
import { AlertTriangle, X } from "lucide-react";

interface ConfirmActionModalProps {
    isOpen: boolean;
    title: string;
    description: React.ReactNode;
    confirmLabel: string;
    onClose: () => void;
    onConfirm: () => void;
    confirmTone?: "danger" | "neutral";
}

export function ConfirmActionModal({
    isOpen,
    title,
    description,
    confirmLabel,
    onClose,
    onConfirm,
    confirmTone = "danger",
}: ConfirmActionModalProps) {
    if (!isOpen) {
        return null;
    }

    return (
        <div className="fixed inset-0 z-[130] flex items-center justify-center bg-slate-950/75 px-4 py-8 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full max-w-md overflow-hidden rounded-[28px] border border-white/10 bg-[linear-gradient(180deg,rgba(15,23,42,0.98),rgba(2,6,23,0.96))] shadow-[0_32px_120px_-48px_rgba(0,0,0,0.8)]">
                <div className="flex items-center justify-between border-b border-white/8 px-5 py-4">
                    <div className="flex items-center gap-3">
                        <div className={`flex h-9 w-9 items-center justify-center rounded-2xl border ${confirmTone === "danger" ? "border-red-400/20 bg-red-500/10 text-red-200" : "border-white/10 bg-white/[0.04] text-white/80"}`}>
                            <AlertTriangle className="h-4 w-4" />
                        </div>
                        <div>
                            <div className="text-[10px] font-black uppercase tracking-[0.32em] text-cortex-500">Confirm action</div>
                            <div className="mt-1 text-sm font-semibold text-white">{title}</div>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="rounded-full border border-white/10 bg-white/[0.04] p-2 text-white/70 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                        aria-label="Close confirmation dialog"
                    >
                        <X className="h-4 w-4" />
                    </button>
                </div>

                <div className="p-5">
                    <div className="text-sm leading-6 text-cortex-300/70">{description}</div>

                    <div className="mt-6 flex gap-3">
                        <button
                            type="button"
                            onClick={onClose}
                            className="flex-1 rounded-full border border-white/10 bg-white/[0.03] px-4 py-2.5 text-[11px] font-semibold uppercase tracking-[0.18em] text-cortex-200 transition hover:bg-white/[0.06]"
                        >
                            Cancel
                        </button>
                        <button
                            type="button"
                            onClick={onConfirm}
                            className={`flex-1 rounded-full px-4 py-2.5 text-[11px] font-semibold uppercase tracking-[0.18em] transition ${confirmTone === "danger" ? "border border-red-400/20 bg-red-500/15 text-red-100 hover:bg-red-500/20" : "border border-cyan-400/20 bg-cyan-500/15 text-cyan-100 hover:bg-cyan-500/20"}`}
                        >
                            {confirmLabel}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
