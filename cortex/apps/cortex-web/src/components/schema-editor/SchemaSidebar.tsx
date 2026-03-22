import React from 'react';

type SchemaSidebarProps = {
  spaceId: string;
  actorRole: string;
  catalogHash?: string | null;
  graphHash?: string | null;
  planHash?: string | null;
  dirty: boolean;
  loading: boolean;
  saving: boolean;
  lineageRef: string;
  statusMessage?: string | null;
  errorMessage?: string | null;
  onLineageRefChange: (value: string) => void;
  onRefresh: () => void;
  onSave: () => void;
};

export const SchemaSidebar = ({
  actorRole,
  catalogHash,
  dirty,
  errorMessage,
  graphHash,
  lineageRef,
  loading,
  onLineageRefChange,
  onRefresh,
  onSave,
  planHash,
  saving,
  spaceId,
  statusMessage,
}: SchemaSidebarProps) => {
  const saveDisabled = loading || saving || !dirty || !lineageRef.trim();

  return (
    <div className="w-64 bg-slate-900 border-r border-slate-800 p-4 flex flex-col gap-4 overflow-y-auto">
      <div className="flex flex-col gap-1 mb-2">
        <h3 className="font-bold text-slate-200 text-sm uppercase tracking-wider">Space Capability Overlay</h3>
        <p className="text-[10px] text-slate-500">Edit steward-governed overrides over the canonical platform catalog.</p>
      </div>

      <div className="rounded-xl border border-slate-800 bg-slate-800/40 p-3 text-[11px] text-slate-300 space-y-3">
        <div>
          <div className="text-[9px] uppercase tracking-[0.24em] text-slate-500">Active Space</div>
          <div className="mt-1 font-mono break-all">{spaceId}</div>
        </div>
        <div className="grid gap-2">
          <div>
            <div className="text-[9px] uppercase tracking-[0.24em] text-slate-500">Actor Role</div>
            <div className="mt-1">{actorRole}</div>
          </div>
          <div>
            <div className="text-[9px] uppercase tracking-[0.24em] text-slate-500">Catalog Hash</div>
            <div className="mt-1 font-mono break-all text-[10px]">{catalogHash || "pending"}</div>
          </div>
          <div>
            <div className="text-[9px] uppercase tracking-[0.24em] text-slate-500">Graph Hash</div>
            <div className="mt-1 font-mono break-all text-[10px]">{graphHash || "available after save"}</div>
          </div>
          <div>
            <div className="text-[9px] uppercase tracking-[0.24em] text-slate-500">Plan Hash</div>
            <div className="mt-1 font-mono break-all text-[10px]">{planHash || "pending"}</div>
          </div>
          <div>
            <div className="text-[9px] uppercase tracking-[0.24em] text-slate-500">Draft State</div>
            <div className={`mt-1 font-semibold ${dirty ? 'text-amber-300' : 'text-emerald-300'}`}>
              {dirty ? 'Unsaved local overlay changes' : 'Persisted and in sync'}
            </div>
          </div>
        </div>
      </div>

      <div className="rounded-xl border border-slate-800 bg-slate-800/30 p-3 space-y-2">
        <label className="text-[9px] uppercase tracking-[0.24em] text-slate-500">Lineage Ref</label>
        <input
          value={lineageRef}
          onChange={(event) => onLineageRefChange(event.target.value)}
          className="w-full rounded-lg border border-slate-700 bg-slate-950 px-2 py-2 text-[11px] text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500"
          placeholder="decision:130:overlay-update"
        />
        <p className="text-[10px] text-slate-500">Required for steward structural updates.</p>
      </div>

      {statusMessage && (
        <div className="rounded-xl border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-[11px] text-emerald-200">
          {statusMessage}
        </div>
      )}

      {errorMessage && (
        <div className="rounded-xl border border-rose-500/30 bg-rose-500/10 px-3 py-2 text-[11px] text-rose-200">
          {errorMessage}
        </div>
      )}

      <div className="mt-auto pt-4 border-t border-slate-800 flex flex-col gap-2">
        <button
          onClick={onSave}
          disabled={saveDisabled}
          className="rounded-lg border border-cyan-400/40 bg-cyan-500/10 px-3 py-2 text-xs font-bold uppercase tracking-[0.2em] text-cyan-100 transition hover:bg-cyan-500/20 disabled:cursor-not-allowed disabled:opacity-40"
        >
          {saving ? 'Saving...' : 'Save Overlay'}
        </button>
        <button
          onClick={onRefresh}
          disabled={loading || saving}
          className="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-xs font-semibold text-slate-200 transition hover:border-slate-500 disabled:cursor-not-allowed disabled:opacity-40"
        >
          Refresh From Gateway
        </button>
      </div>
    </div>
  );
};
