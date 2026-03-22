import React from 'react';
import { Node } from '@xyflow/react';
import type { CapabilityEditorNodeData } from './schemaEditorModel';
import { SurfacingHeuristic, OperationalFrequency } from '../../contracts';

type SchemaNodePropsProps = {
  selectedNode: Node<CapabilityEditorNodeData> | null;
  onUpdateNode: (
    capabilityId: string,
    data: {
      isActive?: boolean;
      localAlias?: string;
      localRequiredRole?: string;
      surfacingHeuristic?: SurfacingHeuristic;
      operationalFrequency?: OperationalFrequency;
    },
  ) => void;
};

const SURFACING_HEURISTICS: SurfacingHeuristic[] = ['PrimaryCore', 'Secondary', 'ContextualDeep', 'Hidden'];
const OPERATIONAL_FREQUENCIES: OperationalFrequency[] = ['Continuous', 'Daily', 'AdHoc', 'Rare'];
const ROLE_OPTIONS = ['', 'viewer', 'editor', 'operator', 'steward', 'admin'];

export const SchemaNodeProps = ({ selectedNode, onUpdateNode }: SchemaNodePropsProps) => {
  if (!selectedNode) {
    return (
      <div className="w-80 bg-slate-900 border-l border-slate-800 p-6 flex flex-col items-center justify-center text-slate-500">
        <div className="text-4xl mb-4">🔍</div>
        <p className="text-center text-sm">Select a node to inspect its properties.</p>
      </div>
    );
  }

  const { data } = selectedNode;

  return (
    <div className="w-80 bg-slate-900 border-l border-slate-800 p-6 flex flex-col gap-6 overflow-y-auto">
      <div className="flex justify-between items-start">
        <div className="flex flex-col gap-1">
          <h3 className="font-bold text-slate-200 text-sm uppercase tracking-wider">Properties</h3>
          <p className="text-[10px] text-slate-500 font-mono">Capability: {selectedNode.id}</p>
        </div>
      </div>

      <div className="flex flex-col gap-3 rounded-xl border border-slate-800 bg-slate-800/30 p-4">
        <div className="text-[10px] uppercase tracking-[0.24em] text-slate-500">Canonical Metadata</div>
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Title</label>
          <div className="bg-slate-950 border border-slate-700 rounded p-2 text-xs text-white">{data.title}</div>
        </div>
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Description</label>
          <div className="bg-slate-950 border border-slate-700 rounded p-2 text-xs text-slate-300 leading-relaxed">{data.description}</div>
        </div>
        <div className="grid grid-cols-2 gap-3 text-xs">
          <div>
            <div className="text-[10px] uppercase tracking-widest text-slate-500">Intent</div>
            <div className="mt-1 text-slate-200">{data.intentType}</div>
          </div>
          <div>
            <div className="text-[10px] uppercase tracking-widest text-slate-500">Route</div>
            <div className="mt-1 text-slate-200">{data.routeId || "n/a"}</div>
          </div>
          <div>
            <div className="text-[10px] uppercase tracking-widest text-slate-500">Category</div>
            <div className="mt-1 text-slate-200">{data.category || "n/a"}</div>
          </div>
          <div>
            <div className="text-[10px] uppercase tracking-widest text-slate-500">Platform Role Floor</div>
            <div className="mt-1 text-slate-200">{data.canonicalRequiredRole || "viewer"}</div>
          </div>
        </div>
      </div>

      <div className="flex flex-col gap-4 rounded-xl border border-slate-800 bg-slate-800/30 p-4">
        <div className="text-[10px] uppercase tracking-[0.24em] text-slate-500">Space Overlay</div>
        <label className="flex items-center justify-between rounded-lg border border-slate-700 bg-slate-950 px-3 py-2">
          <span className="text-xs text-slate-200">Capability Active</span>
          <input
            type="checkbox"
            checked={data.isActive}
            onChange={(event) => onUpdateNode(selectedNode.id, { isActive: event.target.checked })}
          />
        </label>
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Local Alias</label>
          <input 
            type="text" 
            className="bg-slate-800 border border-slate-700 rounded p-2 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500 appearance-none"
            value={data.localAlias || ''}
            onChange={(e) => onUpdateNode(selectedNode.id, { localAlias: e.target.value || undefined })}
            placeholder="Optional space-specific label"
          />
        </div>
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Local Required Role</label>
          <select 
            className="bg-slate-800 border border-slate-700 rounded p-2 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500 appearance-none"
            value={data.localRequiredRole || ''}
            onChange={(e) => onUpdateNode(selectedNode.id, { localRequiredRole: e.target.value || undefined })}
          >
            {ROLE_OPTIONS.map(role => (
              <option key={role || 'inherit-role'} value={role}>
                {role || 'Inherit platform role'}
              </option>
            ))}
          </select>
        </div>
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Surfacing Heuristic</label>
          <select 
            className="bg-slate-800 border border-slate-700 rounded p-2 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500 appearance-none"
            value={data.surfacingHeuristic || ''}
            onChange={(e) => onUpdateNode(selectedNode.id, { surfacingHeuristic: (e.target.value || undefined) as SurfacingHeuristic | undefined })}
          >
            <option value="">Inherit platform heuristic</option>
            {SURFACING_HEURISTICS.map(h => (
              <option key={h} value={h}>{h}</option>
            ))}
          </select>
        </div>
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Operational Frequency</label>
          <select 
            className="bg-slate-800 border border-slate-700 rounded p-2 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500 appearance-none"
            value={data.operationalFrequency || ''}
            onChange={(e) => onUpdateNode(selectedNode.id, { operationalFrequency: (e.target.value || undefined) as OperationalFrequency | undefined })}
          >
            <option value="">Inherit platform frequency</option>
            {OPERATIONAL_FREQUENCIES.map(h => (
              <option key={h} value={h}>{h}</option>
            ))}
          </select>
        </div>
      </div>

      <div className="mt-auto grid grid-cols-1 gap-3 text-[10px] text-slate-400">
        <div className="p-3 bg-slate-950 border border-slate-700 rounded-lg">
          Effective required role: <span className="text-slate-200">{data.effectiveRequiredRole || data.canonicalRequiredRole || 'viewer'}</span>
        </div>
        <div className="p-3 bg-slate-950 border border-slate-700 rounded-lg">
          Effective surfacing: <span className="text-slate-200">{data.effectiveSurfacingHeuristic || data.canonicalSurfacingHeuristic || 'inherit'}</span>
        </div>
        <div className="p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg text-blue-300 italic">
          Catalog structure is read-only here. This surface edits only the space overlay that Initiative 130 persists.
        </div>
      </div>
    </div>
  );
};
