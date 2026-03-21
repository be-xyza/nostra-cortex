import React, { useState } from "react";
import { X, Save } from "lucide-react";

interface WorkbenchNamingModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: (name: string) => void;
    initialName?: string;
    description?: string;
}

export function WorkbenchNamingModal({
    isOpen,
    onClose,
    onConfirm,
    initialName = "",
    description = "You've combined multiple spaces. Give this Workbench session a name before saving your artifact."
}: WorkbenchNamingModalProps) {
    const [name, setName] = useState(initialName || "");

    if (!isOpen) return null;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (name.trim()) {
            onConfirm(name.trim());
        }
    };

    return (
        <div className="fixed inset-0 z-100 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="w-full max-w-md overflow-hidden rounded-2xl border border-white/10 bg-slate-900 shadow-2xl animate-in zoom-in-95 duration-200">
                <div className="flex items-center justify-between border-b border-white/5 px-6 py-4">
                    <h3 className="text-sm font-black uppercase tracking-widest text-slate-400">Name Workbench session</h3>
                    <button onClick={onClose} className="text-slate-500 hover:text-slate-300 transition-colors">
                        <X size={18} />
                    </button>
                </div>

                <form onSubmit={handleSubmit} className="p-6">
                    <p className="mb-4 text-xs text-slate-400 leading-relaxed">
                        {description}
                    </p>

                    <div className="space-y-2">
                        <label htmlFor="session-name" className="text-[10px] font-bold uppercase tracking-wider text-slate-500">
                            Workbench Name
                        </label>
                        <input
                            id="session-name"
                            type="text"
                            autoFocus
                            placeholder="e.g., Q3 Roadmap Draft"
                            className="w-full rounded-xl border border-white/10 bg-slate-950/50 px-4 py-3 text-sm text-slate-100 placeholder:text-slate-600 outline-none focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 transition-all"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                        />
                    </div>

                    <div className="mt-8 flex gap-3">
                        <button
                            type="button"
                            onClick={onClose}
                            className="flex-1 rounded-xl border border-white/5 bg-white/5 py-2.5 text-xs font-semibold text-slate-300 hover:bg-white/10 transition-all"
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            disabled={!name.trim()}
                            className="flex-2 flex items-center justify-center gap-2 rounded-xl bg-blue-600 py-2.5 text-xs font-bold text-white hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-blue-900/20 transition-all"
                        >
                            <Save size={14} />
                            Create Workbench
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}
