import React from "react";
import { CheckCircle2, XCircle, AlertCircle, Clock, ExternalLink, Activity, ShieldCheck, Database } from "lucide-react";

interface GateSummaryProps {
  payload: {
    artifactId?: string;
    title?: string;
    updatedAt?: string;
    structured_data?: {
      counts: {
        pass: number;
        fail: number;
        warn: number;
        pending: number;
      };
      overall_verdict: string;
      latest_run_id: string;
      required_gates_pass: boolean;
      generated_at: string;
      render_hints?: {
        log_stream_id: string;
        primary_route: string;
      };
    };
  };
}

export function GateSummary({ payload }: GateSummaryProps) {
  let data = payload.structured_data || (payload as any).data || payload;
  
  if (typeof data === 'string') {
    try {
      data = JSON.parse(data);
    } catch {
      // ignore parsing error
    }
  }
  
  if (!data || (!data.counts && !data.overall_verdict)) {
    return <div className="p-8 text-slate-500 italic">No structure data available for Gate Summary.</div>;
  }

  const { counts, overall_verdict, latest_run_id, required_gates_pass, generated_at } = data;

  return (
    <div className="flex flex-col gap-6 p-6">
      {/* Header / Big Verdict */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-6 rounded-3xl border border-white/10 bg-white/3 backdrop-blur-xl shadow-2xl">
        <div className="flex items-center gap-4">
          <div className={`flex h-16 w-16 items-center justify-center rounded-2xl shadow-lg border-2 ${
            required_gates_pass ? "bg-emerald-500/20 border-emerald-500/50 text-emerald-400" : "bg-rose-500/20 border-rose-500/50 text-rose-400"
          }`}>
            {required_gates_pass ? <ShieldCheck className="w-9 h-9" /> : <XCircle className="w-9 h-9" />}
          </div>
          <div>
            <h2 className="text-2xl font-bold text-white tracking-tight leading-tight">
              {required_gates_pass ? "Gates Passed" : "Gates Failed"}
            </h2>
            <div className="flex items-center gap-2 mt-1">
              <span className={`text-xs font-bold uppercase tracking-widest px-2 py-0.5 rounded ${
                overall_verdict === "ready" ? "bg-emerald-500 text-white" : "bg-rose-500 text-white"
              }`}>
                {overall_verdict}
              </span>
              <span className="text-xs text-slate-400 font-mono">
                {latest_run_id.slice(0, 16)}...
              </span>
            </div>
          </div>
        </div>
        
        <div className="flex flex-col items-end gap-1">
          <div className="text-[10px] font-mono text-slate-500 uppercase tracking-tighter">Generated At</div>
          <div className="text-sm font-semibold text-slate-300">{new Date(generated_at).toLocaleString()}</div>
        </div>
      </div>

      {/* Metrics Row */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricBox 
          label="Passed" 
          value={counts.pass} 
          icon={<CheckCircle2 className="w-4 h-4 text-emerald-400" />} 
          color="emerald" 
        />
        <MetricBox 
          label="Failed" 
          value={counts.fail} 
          icon={<XCircle className="w-4 h-4 text-rose-400" />} 
          color="rose" 
        />
        <MetricBox 
          label="Warnings" 
          value={counts.warn} 
          icon={<AlertCircle className="w-4 h-4 text-amber-400" />} 
          color="amber" 
        />
        <MetricBox 
          label="Pending" 
          value={counts.pending} 
          icon={<Clock className="w-4 h-4 text-slate-400" />} 
          color="slate" 
        />
      </div>

      {/* Details Section */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="p-5 rounded-2xl border border-white/5 bg-white/2 flex flex-col gap-3">
          <div className="flex items-center gap-2 text-xs font-bold text-slate-400 uppercase tracking-wider">
            <Activity className="w-3.5 h-3.5" />
            Execution Context
          </div>
          <div className="grid grid-cols-2 gap-y-3 gap-x-4">
            <DataRow label="Run ID" value={latest_run_id} mono />
            <DataRow label="Schema" value="gate_summary.v1" mono secondary />
            <DataRow label="Source" value="Local System" secondary />
          </div>
        </div>

        <div className="p-5 rounded-2xl border border-white/5 bg-white/2 flex flex-col justify-between overflow-hidden">
          <div className="flex items-center gap-2 text-xs font-bold text-slate-400 uppercase tracking-wider">
             <Database className="w-3.5 h-3.5" />
             Diagnostic Paths
          </div>
          
          <div className="flex flex-col gap-2 mt-4">
            <a 
              href={`/testing?run_id=${latest_run_id}`}
              className="flex items-center justify-between p-3 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 transition-all group"
            >
              <span className="text-sm font-medium text-slate-200">View Catalog Analysis</span>
              <ExternalLink className="w-4 h-4 text-cyan-400 opacity-0 group-hover:opacity-100 transition-opacity" />
            </a>
            <a 
              href={`/logs?node_id=log_stream:${data.render_hints?.log_stream_id || "unknown"}`}
              className="flex items-center justify-between p-3 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 transition-all group"
            >
              <span className="text-sm font-medium text-slate-200">Inspect Log Stream</span>
              <ExternalLink className="w-4 h-4 text-cyan-400 opacity-0 group-hover:opacity-100 transition-opacity" />
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}

function MetricBox({ label, value, icon, color }: { label: string; value: number; icon: React.ReactNode; color: string }) {
  const colorMap: Record<string, string> = {
    emerald: "text-emerald-400 bg-emerald-400/5",
    rose: "text-rose-400 bg-rose-400/5",
    amber: "text-amber-400 bg-amber-400/5",
    slate: "text-slate-400 bg-slate-400/5"
  };

  return (
    <div className={`flex flex-col gap-1 p-4 rounded-2xl border border-white/10 ${colorMap[color]} shadow-xl`}>
      <div className="flex items-center justify-between">
        <span className="text-[10px] font-bold uppercase tracking-widest opacity-70">{label}</span>
        {icon}
      </div>
      <span className="text-2xl font-bold tracking-tight">{value}</span>
    </div>
  );
}

function DataRow({ label, value, mono = false, secondary = false }: { label: string; value: string; mono?: boolean; secondary?: boolean }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-[9px] font-bold text-slate-500 uppercase tracking-tighter">{label}</span>
      <span className={`text-xs truncate ${mono ? "font-mono" : "font-medium"} ${secondary ? "text-slate-400" : "text-slate-200"}`} title={value}>
        {value}
      </span>
    </div>
  );
}
