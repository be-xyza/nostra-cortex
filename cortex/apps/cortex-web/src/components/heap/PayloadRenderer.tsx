import React, { useState } from "react";
import { workbenchApi } from "../../api";
import { A2UIChartRenderer, type A2UIChartData } from "./A2UIChartRenderer";
import { buildBenchmarkProjection } from "./benchmarkProjection";
import { buildGateSummaryRenderModel } from "./gateSummary";

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
        .replace(/`([^`]+)`/g, "<code class=\"bg-white/5 text-blue-300/90 px-1 py-0.5 rounded text-xs\">$1</code>")
        .replace(/- \[x\] (.*?)(?=\n|$)/g, "<label class=\"flex items-center gap-2 my-1\"><input type=\"checkbox\" checked disabled class=\"rounded bg-white/5 border-white/5 text-blue-500 focus:ring-blue-500 opacity-80\" /> <span class=\"text-slate-400 line-through\">$1</span></label>")
        .replace(/- \[ \] (.*?)(?=\n|$)/g, "<label class=\"flex items-center gap-2 my-1\"><input type=\"checkbox\" disabled class=\"rounded bg-white/5 border-white/5 text-blue-500 focus:ring-blue-500 appearance-none w-3.5 h-3.5 border\" /> <span>$1</span></label>")
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
    // Metadata and context
    meta?: Record<string, unknown>;
    // Pointer
    pointer?: string;
    // Warnings
    warnings?: any[];
}

interface PayloadRendererProps {
    content: PayloadContent;
    artifactId?: string;
    expanded?: boolean;
    showRaw?: boolean;
}

function JsonSchemaExplorer({ data, typeLabel }: { data: any; typeLabel?: string }) {
    if (!data || typeof data !== "object" || data === null) {
        return <div className="text-xs font-mono text-slate-400">{String(data)}</div>;
    }

    return (
        <div className="mt-2.5 p-2.5 rounded-lg bg-black/20 shadow-inner group/json">
            <div className="flex items-center justify-between mb-2">
                <span className="text-[9px] font-black uppercase tracking-widest text-slate-500 opacity-70">{typeLabel || "Schema Content"}</span>
            </div>
            <div className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 overflow-x-auto max-h-[300px] custom-scrollbar pb-1">
                {Object.entries(data).map(([key, value]) => (
                    <React.Fragment key={key}>
                        <div className="text-[10px] font-bold text-slate-500 text-right py-0.5 whitespace-nowrap">{key}:</div>
                        <div className="text-[10px] font-mono py-0.5 break-all">
                            {typeof value === "object" && value !== null ? (
                                <span className="text-slate-600 italic px-1.5 py-0.5 bg-white/5 rounded inline-block">
                                    {Array.isArray(value) ? `Array[${value.length}]` : `Object[${Object.keys(value as object).length}]`}
                                </span>
                            ) : typeof value === "boolean" ? (
                                <span className={value ? "text-emerald-400 font-bold" : "text-rose-400 font-bold"}>{String(value)}</span>
                            ) : typeof value === "number" ? (
                                <span className="text-amber-400">{value}</span>
                            ) : (
                                <span className="text-blue-300">{String(value)}</span>
                            )}
                        </div>
                    </React.Fragment>
                ))}
            </div>
        </div>
    );
}

function renderGateSummaryStructuredData(data: Record<string, unknown>) {
    const model = buildGateSummaryRenderModel(data);

    return (
        <div className="mt-2 rounded-xl bg-white/5 p-4 flex flex-col gap-4">
            <div className="flex items-start justify-between gap-3">
                <div className="flex flex-col gap-1">
                    <span className="text-xs uppercase tracking-widest text-slate-400 font-semibold">
                        {model.title}
                    </span>
                    <span className="text-sm text-slate-200 font-semibold">Run: {model.latestRunId}</span>
                    <span className="text-xs text-slate-500">Generated: {model.generatedAt}</span>
                </div>
                <span
                    className={`px-2 py-1 rounded text-xs font-semibold uppercase ${model.requiredGatesPass
                        ? "bg-emerald-500/15 text-emerald-300"
                        : "bg-rose-500/15 text-rose-300"
                        }`}
                >
                    {model.overallVerdict}
                </span>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                <div className="rounded bg-black/20 px-2 py-1.5">
                    <div className="text-[10px] uppercase text-slate-500">Required Gates Pass</div>
                    <div className={`text-sm font-semibold ${model.requiredGatesPass ? "text-emerald-300" : "text-rose-300"}`}>
                        {model.requiredGatesPass ? "true" : "false"}
                    </div>
                </div>
                {Object.entries(model.counts).map(([key, value]) => (
                    <div key={key} className="rounded bg-black/20 px-2 py-1.5">
                        <div className="text-[10px] uppercase text-slate-500">{key}</div>
                        <div className="text-sm font-semibold text-slate-200">{String(value)}</div>
                    </div>
                ))}
            </div>

            <div className="flex flex-wrap gap-2">
                <a
                    href={model.openWorkbenchHref}
                    className="text-xs rounded bg-blue-500/10 text-blue-300 px-2 py-1 hover:bg-blue-500/20"
                >
                    Open Workbench
                </a>
                {model.openLogsHref ? (
                    <a
                        href={model.openLogsHref}
                        className="text-xs rounded bg-cyan-500/10 text-cyan-300 px-2 py-1 hover:bg-cyan-500/20"
                    >
                        Open Logs
                    </a>
                ) : null}
            </div>

            <div className="rounded overflow-x-auto">
                <table className="min-w-full text-xs text-left">
                    <thead className="bg-black/20 text-slate-400 uppercase tracking-wide">
                        <tr>
                            <th className="px-2 py-1.5">Code</th>
                            <th className="px-2 py-1.5">Message</th>
                        </tr>
                    </thead>
                    <tbody className="">
                        {model.failuresPreview.length === 0 ? (
                            <tr>
                                <td colSpan={2} className="px-2 py-2 text-slate-500 italic">
                                    No failures
                                </td>
                            </tr>
                        ) : (
                            model.failuresPreview.map((row, index) => {
                                return (
                                    <tr key={`${row.code}:${index}`} className="bg-white/5">
                                        <td className="px-2 py-1.5 text-slate-300 font-mono">{row.code}</td>
                                        <td className="px-2 py-1.5 text-slate-300">{row.message}</td>
                                    </tr>
                                );
                            })
                        )}
                    </tbody>
                </table>
            </div>

            {model.failuresOverflow > 0 ? (
                <div className="text-[11px] text-slate-500">+{model.failuresOverflow} more failures</div>
            ) : null}
            <details className="rounded bg-black/20 p-2">
                <summary className="cursor-pointer text-xs text-slate-400">Raw JSON</summary>
                <JsonSchemaExplorer data={data} typeLabel="gate_summary_raw" />
            </details>
        </div>
    );
}

export function PayloadRenderer({ content, artifactId, expanded = false, showRaw = false }: PayloadRendererProps) {
    const renderContent = () => {
        switch (content.payload_type) {
            case "a2ui":
            case "chart":
            case "telemetry":
                return <A2UIPayloadRenderer content={content} expanded={expanded} artifactId={artifactId} />;
            case "rich_text":
            case "note":
            case "task":
                return renderRichText(content, expanded);
            case "media":
                return renderMedia(content);
            case "structured_data":
                return renderStructuredData(content);
            case "pointer":
                return renderPointer(content);
            default: {
                const fallbackData = content.data || content.structured_data || content.tree || content.a2ui || { type: content.payload_type };
                const textCandidate = content.text || content.plain_text;
                if (textCandidate) return renderRichText(content, expanded);
                return <JsonSchemaExplorer data={fallbackData} typeLabel={content.payload_type} />;
            }
        }
    };

    return (
        <div className="payload-renderer">
            {renderContent()}
            {showRaw && (
                <div className="mt-4 pt-4">
                    <JsonSchemaExplorer data={content} typeLabel="RAW_PAYLOAD" />
                </div>
            )}
        </div>
    );
}

function A2UIPayloadRenderer({ content, expanded, artifactId }: { content: PayloadContent; expanded?: boolean; artifactId?: string }) {
    const tree = content.tree || content.a2ui?.tree;
    const [submitting, setSubmitting] = useState(false);
    const [feedback, setFeedback] = useState("");

    const t = (tree || {}) as any;
    const chartData = t.chart_data || t.data?.chart_data || content.meta?.chart_data || content.data?.chart_data;

    // Chart telemetry pattern - check early
    // Also check for common chart properties at the top level of the tree
    const isChart = t.widget === "chart" ||
        t.type === "chart" ||
        content.payload_type === "chart" ||
        chartData ||
        (t.labels && t.datasets);

    if (isChart) {
        return <A2UIChartRenderer data={(chartData || tree || {}) as A2UIChartData} />;
    }

    if (!tree) {
        return (
            <div className="mt-2 p-4 rounded-xl bg-white/5 text-xs text-slate-500 flex items-center justify-center italic">
                Payload data missing (no tree found)
            </div>
        );
    }

    const handleSubmitFeedback = async (decision: string) => {
        if (!artifactId) return;
        setSubmitting(true);
        try {
            await workbenchApi.submitA2UIFeedback(artifactId, { decision, feedback });
            setFeedback("");
        } catch (err) {
            console.error("A2UI Feedback failed", err);
        } finally {
            setSubmitting(false);
        }
    };

    // AgentBenchmarkRecord pattern
    if (t.widget === "AgentBenchmarkRecord" || t.type === "benchmark_solicitation") {
        return (
            <div className="mt-2 p-3 rounded-lg flex flex-col gap-3 bg-purple-950/20">
                <div className="flex justify-between items-end">
                    <span className="font-bold text-sm text-purple-400">Agent Solicitation Pending</span>
                    <span className="text-xs font-semibold text-slate-400">{String(t.agent_role || "steward.code")}</span>
                </div>
                {Boolean(t.rationale) && (
                    <div className="text-sm text-slate-300 italic">{String(t.rationale)}</div>
                )}
                <textarea
                    value={feedback}
                    onChange={(e) => setFeedback(e.target.value)}
                    placeholder="Provide steering feedback or requirements (optional)..."
                    className="w-full bg-black/20 rounded px-2 py-1.5 text-xs text-slate-200 focus:outline-none min-h-[60px] border-none"
                />
                <div className="flex gap-2">
                    <button
                        onClick={() => handleSubmitFeedback("approved")}
                        disabled={submitting}
                        className={`flex-1 bg-green-500/20 text-green-400 text-xs font-semibold py-1.5 rounded hover:bg-green-500/30 transition-colors ${submitting ? "opacity-50" : ""}`}
                    >
                        Approve & Proceed
                    </button>
                    <button
                        onClick={() => handleSubmitFeedback("rejected")}
                        disabled={submitting}
                        className={`flex-1 bg-red-500/20 text-red-400 text-xs font-semibold py-1.5 rounded hover:bg-red-500/30 transition-colors ${submitting ? "opacity-50" : ""}`}
                    >
                        Reject
                    </button>
                </div>
            </div>
        );
    }

    // Activity Feed pattern
    if (t.widget === "ActivityFeed" || t.type === "activity_log") {
        const items = Array.isArray(t.items) ? t.items : [];
        const title = typeof t.title === "string" ? t.title : "Activity Feed";
        return (
            <div className="mt-3 flex flex-col gap-2">
                <span className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">{title}</span>
                <div className="flex flex-col ml-1.5 pl-3 gap-3 my-1">
                    {items.map((item: any, i: number) => (
                        <div key={i} className="flex flex-col gap-0.5 relative">
                            <div className="absolute -left-[15.5px] top-1.5 w-2 h-2 rounded-full bg-white/10 shadow-sm border border-white/5"></div>
                            <span className="text-[10px] font-bold text-slate-300">{String(item.action || "Event")}</span>
                            <span className="text-[10px] text-slate-500 leading-tight">{String(item.detail || item.message || "")}</span>
                            {item.timestamp && <span className="text-[8px] font-mono text-slate-600 uppercase">{String(item.timestamp)}</span>}
                        </div>
                    ))}
                </div>
            </div>
        );
    }

    // SiqScorecard pattern
    if (t.widget === "SiqScorecard") {
        const passing = t.passing === true;
        const score = t.score || 0;
        const violations = Array.isArray(t.violations) ? t.violations : [];

        return (
            <div className="mt-2 p-4 rounded-xl bg-white/5 flex flex-col gap-3">
                <div className="flex items-center justify-between">
                    <span className="text-xs uppercase tracking-widest text-slate-400 font-semibold">Siq Scorecard</span>
                    <span className={`px-2 py-1 rounded text-[10px] font-bold uppercase ${passing ? "bg-emerald-500/15 text-emerald-400" : "bg-rose-500/15 text-rose-400"}`}>
                        {passing ? "Passing" : "Failing"}
                    </span>
                </div>
                <div className="flex items-baseline gap-1">
                    <span className="text-2xl font-black text-slate-100">{score}</span>
                    <span className="text-xs text-slate-500">/100</span>
                </div>
                {violations.length > 0 && (
                    <div className="space-y-1">
                        {violations.map((v: any, i: number) => (
                            <div key={i} className="text-[10px] text-rose-300 bg-rose-500/5 px-2 py-1 rounded font-mono">
                                <span className="font-bold mr-2">{v.node}:</span>
                                {v.error}
                            </div>
                        ))}
                    </div>
                )}
            </div>
        );
    }

    // Generic A2UI tree — show as Rich Explorer
    return <JsonSchemaExplorer data={tree || {}} typeLabel={`A2UI: ${t.widget || "Generic"}`} />;
}

function renderRichText(content: PayloadContent, expanded: boolean) {
    const text = content.text || content.plain_text || "";
    if (!text) {
        return (
            <div className="mt-2 p-3 rounded-lg bg-white/5 text-[10px] text-slate-500 italic">
                No text representation available for this {content.payload_type || "block"}
            </div>
        );
    }

    return (
        <div
            className={`heap-rich-text text-sm text-slate-300 leading-relaxed mt-2 ${expanded ? "" : "line-clamp-6"} prose prose-invert prose-p:my-1 prose-a:text-blue-400 hover:prose-a:text-blue-300`}
            dangerouslySetInnerHTML={{ __html: renderMarkdown(text) }}
        />
    );
}

function renderMedia(content: PayloadContent) {
    const media = content.media;
    if (!media) return null;

    const url = media.url || `https://images.unsplash.com/photo-1451187580459-43490279c0fa?q=80&w=800&auto=format&fit=crop`;
    const hash = media.hash || "unknown";

    return (
        <div className="mt-2 rounded-xl relative overflow-hidden group cursor-zoom-in bg-slate-900 shadow-lg">
            <img 
                src={url} 
                alt="Media thumbnail" 
                className="w-full h-40 object-cover opacity-80 group-hover:opacity-100 transition-opacity"
                onError={(e) => {
                    const target = e.target as HTMLImageElement;
                    if (!target.src.includes('photo-1451187580459')) {
                        target.src = 'https://images.unsplash.com/photo-1451187580459-43490279c0fa?q=80&w=800';
                    }
                }}
            />
            <div className="absolute inset-x-0 bottom-0 bg-linear-to-t from-slate-950/90 to-transparent p-3 text-[10px] font-mono text-slate-300 opacity-0 group-hover:opacity-100 transition-opacity flex items-end">
                <span>{hash.substring(0, 16).toUpperCase()}</span>
            </div>
        </div>
    );
}

function renderStructuredData(content: PayloadContent) {
    const data = content.data || content.structured_data || {};
    
    if (data && typeof data === "object" && !Array.isArray(data)) {
        const typedData = data as Record<string, any>;
        
        if (typedData.schema_id === "nostra.heap.block.gate_summary.v1") {
            return renderGateSummaryStructuredData(typedData);
        }

        if (typedData.type === "self_optimization_proposal") {
            return (
                <div className="mt-2 p-3 rounded-lg flex flex-col gap-3 bg-purple-950/20 border border-purple-900/50">
                    <div className="flex justify-between items-start">
                        <div className="flex flex-col gap-1">
                            <span className="font-bold text-sm text-purple-400 font-mono">Self-Optimization Proposal</span>
                            <span className="text-xs font-semibold text-slate-400">{typedData.agent_id} • {typedData.domain}</span>
                        </div>
                        <span className="text-[10px] bg-purple-500/20 text-purple-300 px-2 py-1 rounded shadow-sm opacity-80 font-mono uppercase">
                            {typedData.proposed_changes?.action || "PROPOSAL"}
                        </span>
                    </div>
                    {typedData.rationale && (
                        <div className="text-sm text-slate-300 italic px-2 border-l-2 border-purple-500/30">
                            {typedData.rationale}
                        </div>
                    )}
                    {typedData.proposed_changes?.reason && (
                        <div className="text-xs text-rose-300/80 mt-1">
                            <strong>Trigger: </strong>{typedData.proposed_changes.reason}
                        </div>
                    )}
                    <details className="mt-2 text-xs">
                        <summary className="cursor-pointer text-slate-500 hover:text-slate-400">View Raw Data</summary>
                        <JsonSchemaExplorer data={data} typeLabel="self_optimization_proposal" />
                    </details>
                </div>
            );
        }
        
        if (typedData.type === "agent_execution_record") {
            const benchmark = buildBenchmarkProjection(typedData.benchmark);
            return (
                <div className="mt-2 p-3 rounded-lg flex flex-col gap-2 bg-indigo-950/20 border border-indigo-900/50">
                    <div className="flex items-center justify-between">
                        <span className="font-bold text-[11px] uppercase tracking-widest text-indigo-400">Execution Record</span>
                        <span className={`text-[10px] px-2 py-0.5 rounded font-bold uppercase ${
                            typedData.status === "completed" ? "bg-emerald-500/10 text-emerald-400" :
                            typedData.status === "failed" ? "bg-rose-500/10 text-rose-400" : 
                            "bg-slate-500/10 text-slate-400"
                        }`}>{typedData.status || "UNKNOWN"}</span>
                    </div>
                    
                    <div className="flex flex-col gap-1">
                        <div className="flex items-baseline justify-between text-xs">
                            <span className="text-slate-500 font-mono">Agent:</span>
                            <span className="text-slate-300 font-mono font-bold mr-auto ml-2">{typedData.agent_id}</span>
                            <span className="text-slate-500">Phase: <span className="text-slate-300 uppercase font-mono">{typedData.phase}</span></span>
                        </div>
                    </div>

                    {benchmark && (
                        <div className="mt-1 bg-black/20 p-2 rounded-md grid grid-cols-2 sm:grid-cols-4 gap-2 text-center items-center">
                            <div className="flex flex-col">
                                <span className="text-[9px] text-slate-500 uppercase">Grade</span>
                                <span className={`text-sm font-bold ${benchmark.grade === "PASS" ? "text-emerald-400" : benchmark.grade === "FAIL" ? "text-rose-400" : "text-amber-400"}`}>{benchmark.grade}</span>
                            </div>
                            <div className="flex flex-col border-x border-white/5">
                                <span className="text-[9px] text-slate-500 uppercase">Pass Rate</span>
                                <span className="text-xs font-mono text-slate-300">{benchmark.passRate === null ? "—" : `${Math.round(benchmark.passRate * 100)}%`}</span>
                            </div>
                            <div className="flex flex-col">
                                <span className="text-[9px] text-slate-500 uppercase">Latency</span>
                                <span className="text-xs font-mono text-slate-300">{benchmark.latencyMs === null ? "—" : `${Math.round(benchmark.latencyMs)}ms`}</span>
                            </div>
                            <div className="flex flex-col border-l border-white/5 sm:border-l-0">
                                <span className="text-[9px] text-slate-500 uppercase">Tokens</span>
                                <span className="text-xs font-mono text-slate-300">{benchmark.totalTokens === null ? "—" : benchmark.totalTokens.toLocaleString()}</span>
                            </div>
                        </div>
                    )}
                    {benchmark && (
                        <div className="text-[10px] text-slate-400 leading-snug">
                            {benchmark.summary}
                        </div>
                    )}
                    
                    <details className="mt-1 text-xs">
                        <summary className="cursor-pointer text-slate-500 hover:text-slate-400">View Details</summary>
                        <JsonSchemaExplorer data={data} typeLabel="agent_execution_record" />
                    </details>
                </div>
            );
        }

        if (typedData.type === "usage_report") {
            return (
                <div className="mt-2 p-3 rounded-lg flex flex-col gap-2 bg-emerald-950/10 border border-emerald-900/30">
                    <div className="flex justify-between items-center text-xs">
                        <span className="font-bold text-emerald-400/80 uppercase tracking-widest text-[10px]">Usage Report</span>
                        <span className="font-mono text-slate-500">{new Date(typedData.timestamp).toLocaleTimeString()}</span>
                    </div>
                    <div className="grid grid-cols-2 gap-2 mt-1">
                        <div className="bg-black/20 rounded p-2 flex flex-col justify-center items-center">
                            <span className="text-[10px] text-slate-500 uppercase">Tokens (Cycle)</span>
                            <span className="font-mono text-slate-200 text-sm">{typedData.tokens_consumed_this_cycle?.toLocaleString() || 0}</span>
                        </div>
                        <div className="bg-black/20 rounded p-2 flex flex-col justify-center items-center">
                            <span className="text-[10px] text-slate-500 uppercase">Cost (Cycle)</span>
                            <span className="font-mono text-slate-200 text-sm">${(typedData.cycle_cost_usd || 0).toFixed(4)}</span>
                        </div>
                    </div>
                    <div className="flex justify-between items-center text-[10px] text-slate-500 mt-1 px-1">
                        <span>Agent: <span className="text-slate-400 font-mono">{typedData.agent_id}</span></span>
                        <span>Total: {typedData.total_tokens_consumed?.toLocaleString() || 0}</span>
                    </div>
                </div>
            );
        }
    }
    
    return <JsonSchemaExplorer data={data} typeLabel="structured_data" />;
}

function renderPointer(content: PayloadContent) {
    return (
        <div className="text-[10px] font-mono bg-indigo-900/10 text-indigo-400 px-2 py-1 rounded-md inline-flex items-center gap-1.5 mt-2 shadow-sm font-bold">
            <span className="opacity-50">REF:</span> {content.pointer || "unknown reference"}
        </div>
    );
}
