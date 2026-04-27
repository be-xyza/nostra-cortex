import React, { useEffect, useState } from "react";
import { connectWorkbenchStream } from "../../api";
import { AgentEvent, parseAgentActivityEvent } from "./agentActivity";

export interface AgentActivityPanelProps {
    spaceId: string;
    onSolicit?: () => void;
}

const AgentEventRow = React.memo(({ evt }: { evt: AgentEvent }) => (
    <div className="flex flex-col gap-1 px-3 py-2 bg-slate-950/50 hover:bg-slate-800/50 rounded transition-colors group">
        <div className="flex justify-between items-start gap-4">
            <span className="text-xs font-mono text-purple-400 font-semibold flex items-center gap-2">
                {evt.agent}
                {evt.details?.includes("provider:codex_subscription") && (
                    <span className="px-1.5 py-0.5 rounded text-[8px] font-black tracking-widest bg-amber-500/20 text-amber-500 uppercase border border-amber-500/30">
                        LIVE COGNITION
                    </span>
                )}
            </span>
            <span className="text-[10px] font-mono text-slate-500 shrink-0">
                {new Date(evt.timestamp).toLocaleTimeString()}
            </span>
        </div>
        <div className="flex items-center gap-2">
            <span className={`text-[10px] px-1.5 py-0.5 rounded uppercase font-bold tracking-wider ${evt.status === "started" ? "bg-blue-500/20 text-blue-400 border border-blue-500/30" :
                    evt.status === "running" ? "bg-indigo-500/20 text-indigo-400 border border-indigo-500/30" :
                        evt.status === "completed" ? "bg-green-500/20 text-green-400 border border-green-500/30" :
                            evt.status === "failed" ? "bg-red-500/20 text-red-400 border border-red-500/30" :
                                "bg-slate-800 text-slate-400 border border-slate-700"
                }`}>
                {evt.action}
            </span>
            <span className="text-xs text-slate-300 truncate opacity-80 group-hover:opacity-100">
                {evt.details && evt.details !== "{}" ? evt.details : "Working..."}
            </span>
        </div>
    </div>
));

export function AgentActivityPanel({ spaceId, onSolicit }: AgentActivityPanelProps) {
    const [events, setEvents] = useState<AgentEvent[]>([]);
    const [activeAgentCount, setActiveAgentCount] = useState(0);
    const [streamUnavailable, setStreamUnavailable] = useState(false);

    useEffect(() => {
        if (!spaceId) return;

        setStreamUnavailable(false);

        const stream = connectWorkbenchStream(
            "global/events",
            spaceId,
            (data: any) => {
                const parsed = parseAgentActivityEvent(data);
                if (!parsed) return;

                setEvents((prev) => {
                    const updated = [parsed, ...prev].slice(0, 50); // Keep last 50

                    // compute active
                    const activeCount = updated.filter(e => e.status === "started" || e.status === "running").length;
                    setActiveAgentCount(activeCount);

                    return updated;
                });
            },
            () => setStreamUnavailable(true)
        );

        return () => {
            stream.close();
        };
    }, [spaceId]);

    if (events.length === 0 || streamUnavailable) {
        return null;
    }

    return (
        <div className="flex flex-col border-b border-white/5 bg-slate-900 shadow-sm max-h-48 overflow-y-auto custom-scrollbar">
            <div className="sticky top-0 bg-slate-900/95 backdrop-blur px-4 py-2 flex items-center justify-between border-b border-white/5">
                <div className="flex items-center gap-2">
                    <span className="relative flex h-2 w-2">
                        {activeAgentCount > 0 ? (
                            <>
                                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
                                <span className="relative inline-flex rounded-full h-2 w-2 bg-blue-500"></span>
                            </>
                        ) : (
                            <span className="relative inline-flex rounded-full h-2 w-2 bg-slate-600"></span>
                        )}
                    </span>
                    <span className="text-xs font-semibold text-slate-300 uppercase tracking-wider">
                        Live activity
                    </span>
                </div>
                <div className="flex items-center gap-3">
                    <button 
                        onClick={onSolicit}
                        className="text-[10px] font-bold tracking-widest bg-indigo-500/20 text-indigo-300 hover:bg-indigo-500/30 px-2 py-1 rounded border border-indigo-500/30 uppercase transition-colors"
                        title="Request review from the active agent loop"
                    >
                        Request Review
                    </button>
                    <span className="text-[10px] font-mono text-slate-500">{events.length} trace(s)</span>
                </div>
            </div>

            <div className="flex flex-col p-2 gap-1">
                {events.map((evt) => (
                    <AgentEventRow key={evt.id} evt={evt} />
                ))}
            </div>
        </div>
    );
}
