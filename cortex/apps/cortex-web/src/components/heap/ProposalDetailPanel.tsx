import React, { useMemo } from "react";
import { CheckCircle2, XCircle, MessageSquare, ClipboardCheck, Loader2 } from "lucide-react";
import type { HeapBlockListItem } from "../../contracts";

interface ProposalStep {
    id: string;
    label: string;
    description?: string;
    status?: "pending" | "approved" | "rejected";
}

interface ProposalDetailPanelProps {
    block: HeapBlockListItem;
    onApprove: () => void;
    onReject: () => void;
    onRequestChanges?: () => void;
    isProcessing?: boolean;
}

function extractSteps(block: HeapBlockListItem): ProposalStep[] {
    const surface = (block.surfaceJson as Record<string, unknown>) || {};
    const raw = surface.execution_steps ?? surface.steps ?? surface.plan_steps;
    if (!Array.isArray(raw)) return [];
    return raw.map((step: unknown, idx: number) => {
        if (typeof step === "string") {
            return { id: `step-${idx}`, label: step, status: "pending" as const };
        }
        const s = step as Record<string, unknown>;
        return {
            id: String(s.id ?? `step-${idx}`),
            label: String(s.label ?? s.title ?? s.name ?? `Step ${idx + 1}`),
            description: s.description ? String(s.description) : undefined,
            status: (s.status as ProposalStep["status"]) ?? "pending",
        };
    });
}

export function ProposalDetailPanel({
    block,
    onApprove,
    onReject,
    onRequestChanges,
    isProcessing = false,
}: ProposalDetailPanelProps) {
    const steps = useMemo(() => extractSteps(block), [block]);
    const surface = (block.surfaceJson as Record<string, unknown>) || {};
    const summary = String(surface.summary ?? surface.rationale ?? "");

    return (
        <div className="proposal-detail-panel bg-cortex-surface-panel/60 backdrop-blur-xl rounded-xl border border-white/5 p-5 space-y-5">
            {/* Header */}
            <div className="flex items-center gap-2">
                <ClipboardCheck className="w-5 h-5 text-amber-400" />
                <h3 className="text-sm font-bold text-cortex-50 tracking-tight">
                    Execution Proposal
                </h3>
                <span className="ml-auto text-[10px] font-mono bg-amber-500/10 text-amber-300 px-2 py-0.5 rounded-full">
                    {steps.length} steps
                </span>
            </div>

            {/* Summary */}
            {summary && (
                <p className="text-xs text-cortex-400 leading-relaxed border-l-2 border-amber-500/20 pl-3">
                    {summary}
                </p>
            )}

            {/* Step List */}
            {steps.length > 0 && (
                <div className="space-y-2">
                    {steps.map((step, idx) => (
                        <div
                            key={step.id}
                            className="flex items-start gap-3 p-3 rounded-lg bg-cortex-surface-base/40 border border-white/5"
                        >
                            <span className="text-[10px] font-mono text-cortex-600 mt-0.5 shrink-0 w-5 text-right">
                                {idx + 1}.
                            </span>
                            <div className="flex-1 min-w-0">
                                <span className="text-xs font-semibold text-cortex-200 block">
                                    {step.label}
                                </span>
                                {step.description && (
                                    <span className="text-[10px] text-cortex-500 mt-0.5 block">
                                        {step.description}
                                    </span>
                                )}
                            </div>
                            <span
                                className={`text-[9px] font-mono px-1.5 py-0.5 rounded-sm ${
                                    step.status === "approved"
                                        ? "bg-green-500/10 text-green-400"
                                        : step.status === "rejected"
                                        ? "bg-red-500/10 text-red-400"
                                        : "bg-slate-800/60 text-slate-500"
                                }`}
                            >
                                {step.status}
                            </span>
                        </div>
                    ))}
                </div>
            )}

            {/* Action Buttons */}
            <div className="flex items-center gap-2 pt-2 border-t border-white/5">
                <button
                    onClick={onApprove}
                    disabled={isProcessing}
                    className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-green-500/10 text-green-400 hover:bg-green-500/20 transition-all text-xs font-bold tracking-tight disabled:opacity-40"
                >
                    {isProcessing ? (
                        <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    ) : (
                        <CheckCircle2 className="w-3.5 h-3.5" />
                    )}
                    Approve
                </button>
                {onRequestChanges && (
                    <button
                        onClick={onRequestChanges}
                        disabled={isProcessing}
                        className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-amber-500/10 text-amber-400 hover:bg-amber-500/20 transition-all text-xs font-bold tracking-tight disabled:opacity-40"
                    >
                        <MessageSquare className="w-3.5 h-3.5" />
                        Discuss
                    </button>
                )}
                <button
                    onClick={onReject}
                    disabled={isProcessing}
                    className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-all text-xs font-bold tracking-tight disabled:opacity-40"
                >
                    <XCircle className="w-3.5 h-3.5" />
                    Reject
                </button>
            </div>
        </div>
    );
}
