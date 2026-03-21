import React, { useEffect, useState } from "react";
import { gatewayBaseUrl } from "../api";
import { A2UIComponentProps } from "./a2ui/WidgetRegistry";

// Add a specific type for the row structure so we aren't guessing
interface MatrixRow {
  routeId: string;
  patternId: string;
  requiredRole: string;
  approvalRequired: boolean;
  operatorCritical: boolean;
  promotionStatus: string;
}

interface CapabilityMatrixResponse {
  schemaVersion: string;
  generatedAt: string;
  matrix: MatrixRow[];
}

export function RulesMatrixWidget({ componentProperties }: A2UIComponentProps) {
  const [data, setData] = useState<CapabilityMatrixResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(true);

  // Read props securely if needed, but for now we just fetch the generic endpoint
  const dataSourceUrl = "/api/cortex/views/capability-matrix";

  useEffect(() => {
    setLoading(true);
    fetch(gatewayBaseUrl() + dataSourceUrl)
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP error ${res.status}`);
        return res.json();
      })
      .then((json: CapabilityMatrixResponse) => {
        setData(json);
        setLoading(false);
      })
      .catch((err) => {
        setError(err.message);
        setLoading(false);
      });
  }, [dataSourceUrl]);

  if (loading) {
    return <div className="p-4 text-cortex-ink-muted bg-cortex-bg-elev rounded-lg border border-cortex-line animate-pulse">Loading Governance Matrix...</div>;
  }

  if (error) {
    return <div className="p-4 text-cortex-bad bg-cortex-bad/10 rounded-lg border border-cortex-bad/20">Failed to load Matrix: {error}</div>;
  }

  if (!data || !data.matrix || data.matrix.length === 0) {
    return <div className="p-4 text-cortex-ink-faint italic border border-cortex-line rounded-lg bg-cortex-bg-elev">No governance rules found in the matrix.</div>;
  }

  return (
    <div className="flex flex-col gap-4 p-4 md:p-6 bg-cortex-bg-panel/40 backdrop-blur-md rounded-2xl border border-blue-500/20 shadow-lg text-slate-100">
      <div className="flex justify-between items-center mb-2">
        <div>
          <h3 className="text-[10px] font-black uppercase tracking-[0.2em] text-blue-400 mb-1">Governance Matrix</h3>
          <p className="text-sm text-slate-400">Mapping Nostra Patterns to Cortex Capabilities</p>
        </div>
        <div className="px-3 py-1 rounded-full bg-blue-500/10 border border-blue-500/20 text-[10px] font-mono text-blue-300">
          {data.matrix.length} Rules Active
        </div>
      </div>
      
      <div className="overflow-x-auto rounded-xl border border-white/5 bg-slate-950/40">
        <table className="w-full text-left text-sm">
          <thead className="bg-slate-900/60 uppercase text-[10px] font-bold tracking-wider text-slate-500">
            <tr>
              <th className="px-4 py-3 border-b border-white/5">Route ID (Execution)</th>
              <th className="px-4 py-3 border-b border-white/5">Pattern (Platform)</th>
              <th className="px-4 py-3 border-b border-white/5">Req. Role</th>
              <th className="px-4 py-3 border-b border-white/5 text-center">Critical</th>
              <th className="px-4 py-3 border-b border-white/5 text-center">Approval</th>
              <th className="px-4 py-3 border-b border-white/5">Status</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-white/5">
            {data.matrix.map((row, idx) => (
              <tr key={`${row.routeId}-${idx}`} className="hover:bg-white/5 transition-colors">
                <td className="px-4 py-3 font-mono text-xs text-blue-300">{row.routeId}</td>
                <td className="px-4 py-3">
                  <span className="px-2 py-0.5 rounded bg-slate-800 text-xs border border-white/10 text-slate-300">
                    {row.patternId}
                  </span>
                </td>
                <td className="px-4 py-3 text-xs">
                  <span className={`px-2 py-0.5 rounded font-mono ${row.requiredRole === 'admin' ? 'bg-red-500/10 text-red-400 border border-red-500/20' : 'bg-slate-800 text-slate-400 border border-white/5'}`}>
                    {row.requiredRole}
                  </span>
                </td>
                <td className="px-4 py-3 text-center">
                  {row.operatorCritical ? (
                    <span className="inline-block w-2 h-2 rounded-full bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)]" />
                  ) : (
                    <span className="inline-block w-2 h-2 rounded-full bg-white/10" />
                  )}
                </td>
                <td className="px-4 py-3 text-center">
                  {row.approvalRequired ? (
                    <span className="inline-block w-2 h-2 rounded-full bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)]" />
                  ) : (
                    <span className="inline-block w-2 h-2 rounded-full bg-white/10" />
                  )}
                </td>
                <td className="px-4 py-3 text-xs">
                  <span className={`px-2 py-0.5 rounded uppercase tracking-wider ${row.promotionStatus === 'production' ? 'text-green-400 bg-green-500/10' : 'text-amber-400 bg-amber-500/10'}`}>
                    {row.promotionStatus}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
