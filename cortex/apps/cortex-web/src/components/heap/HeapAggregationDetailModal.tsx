import React from "react";
import type { HeapAggregationGroup } from "./heapAggregation.ts";

interface HeapAggregationDetailModalProps {
    group: HeapAggregationGroup;
    onClose: () => void;
    onOpenBlock: (artifactId: string) => void;
}

export function HeapAggregationDetailModal({
    group,
    onClose,
    onOpenBlock,
}: HeapAggregationDetailModalProps) {
    return (
        <div
            className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 p-4 backdrop-blur-sm"
            onClick={onClose}
        >
            <div
                className="relative flex max-h-[90vh] w-full max-w-6xl flex-col overflow-hidden rounded-3xl border border-white/6 bg-slate-950 shadow-[0_36px_120px_rgba(0,0,0,0.55)]"
                onClick={(event) => event.stopPropagation()}
            >
                <header className="border-b border-white/6 bg-slate-900/85 px-6 py-5 backdrop-blur-xl">
                    <div className="flex items-start justify-between gap-4">
                        <div>
                            <div className="text-[10px] font-black uppercase tracking-[0.32em] text-cortex-500">Details</div>
                            <h2 className="mt-2 text-2xl font-semibold tracking-tight text-cortex-50">{group.label}</h2>
                            <p className="mt-2 max-w-3xl text-sm leading-6 text-cortex-300/75">{group.description}</p>
                        </div>
                        <button
                            type="button"
                            onClick={onClose}
                            className="rounded-full border border-white/8 bg-white/5 p-2 text-cortex-400 transition-colors hover:text-white"
                            aria-label="Close view details"
                        >
                            ✕
                        </button>
                    </div>
                    <div className="mt-4 flex flex-wrap gap-2">
                        <span className="rounded-full border border-cyan-500/20 bg-cyan-500/10 px-3 py-1 text-[11px] font-semibold text-cyan-200">
                            {group.count} grouped blocks
                        </span>
                        <span className="rounded-full border border-white/8 bg-white/[0.04] px-3 py-1 text-[11px] text-cortex-300/80">
                            Grouped list
                        </span>
                    </div>
                </header>

                <div className="overflow-auto px-6 py-6">
                    <div className="overflow-hidden rounded-2xl border border-white/6 bg-slate-900/50">
                        <table className="min-w-full border-collapse text-left">
                            <thead className="bg-white/[0.03]">
                                <tr>
                                    <th className="px-4 py-3 text-[10px] font-black uppercase tracking-[0.28em] text-cortex-500">Block</th>
                                    {group.columns.map((column) => (
                                        <th
                                            key={column.key}
                                            className="px-4 py-3 text-[10px] font-black uppercase tracking-[0.28em] text-cortex-500"
                                        >
                                            {column.label}
                                        </th>
                                    ))}
                                    <th className="px-4 py-3 text-[10px] font-black uppercase tracking-[0.28em] text-cortex-500">Summary</th>
                                    <th className="px-4 py-3 text-[10px] font-black uppercase tracking-[0.28em] text-cortex-500">Updated</th>
                                </tr>
                            </thead>
                            <tbody>
                                {group.items.map((item) => (
                                    <tr
                                        key={item.artifactId}
                                        className="cursor-pointer border-t border-white/6 transition-colors hover:bg-white/[0.04]"
                                        onClick={() => {
                                            onOpenBlock(item.artifactId);
                                            onClose();
                                        }}
                                    >
                                        <td className="px-4 py-3 align-top">
                                            <div className="min-w-[220px]">
                                                <div className="text-sm font-semibold text-cortex-50">{item.title}</div>
                                                <div className="mt-1 text-[11px] uppercase tracking-[0.2em] text-cortex-500">{item.blockType}</div>
                                            </div>
                                        </td>
                                        {group.columns.map((column) => (
                                            <td key={column.key} className="px-4 py-3 align-top text-sm text-cortex-300/85">
                                                {item.fields[column.key] ?? "n/a"}
                                            </td>
                                        ))}
                                        <td className="max-w-[420px] px-4 py-3 align-top text-sm leading-6 text-cortex-300/75">
                                            {item.summary}
                                        </td>
                                        <td className="whitespace-nowrap px-4 py-3 align-top text-xs text-cortex-500">
                                            {formatUpdatedAt(item.updatedAt)}
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    );
}

function formatUpdatedAt(value: string): string {
    return new Date(value).toLocaleString(undefined, {
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
    });
}
