import React from 'react';
import { Node } from '@xyflow/react';
import { CapabilityNodeData } from './SchemaCustomNodes';
import { SurfacingHeuristic, OperationalFrequency } from '../../contracts';

type SchemaNodePropsProps = {
  selectedNode: Node<CapabilityNodeData> | null;
  onUpdateNode: (nodeId: string, data: Partial<CapabilityNodeData>) => void;
  onDeleteNode: (nodeId: string) => void;
};

const SURFACING_HEURISTICS: SurfacingHeuristic[] = ['PrimaryCore', 'Secondary', 'ContextualDeep', 'Hidden'];
const OPERATIONAL_FREQUENCIES: OperationalFrequency[] = ['Continuous', 'Daily', 'AdHoc', 'Rare'];

export const SchemaNodeProps = ({ selectedNode, onUpdateNode, onDeleteNode }: SchemaNodePropsProps) => {
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
          <p className="text-[10px] text-slate-500 font-mono">ID: {selectedNode.id}</p>
        </div>
        <button 
          onClick={() => onDeleteNode(selectedNode.id)}
          className="p-1 hover:bg-red-500/20 rounded transition-colors text-red-500/60 hover:text-red-500"
          title="Delete Node"
        >
          🗑️
        </button>
      </div>

      <div className="flex flex-col gap-4">
        {/* Title */}
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Title</label>
          <input 
            type="text" 
            className="bg-slate-800 border border-slate-700 rounded p-2 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500"
            value={data.title}
            onChange={(e) => onUpdateNode(selectedNode.id, { title: e.target.value })}
          />
        </div>

        {/* Description */}
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Description</label>
          <textarea 
            className="bg-slate-800 border border-slate-700 rounded p-2 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500 min-h-20 resize-none"
            value={data.description}
            onChange={(e) => onUpdateNode(selectedNode.id, { description: e.target.value })}
          />
        </div>

        {/* Surfacing Heuristic */}
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Surfacing Heuristic</label>
          <select 
            className="bg-slate-800 border border-slate-700 rounded p-2 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500 appearance-none"
            value={data.surfacing_heuristic}
            onChange={(e) => onUpdateNode(selectedNode.id, { surfacing_heuristic: e.target.value as SurfacingHeuristic })}
          >
            {SURFACING_HEURISTICS.map(h => (
              <option key={h} value={h}>{h}</option>
            ))}
          </select>
        </div>

        {/* Intent Type */}
        <div className="flex flex-col gap-1.5">
          <label className="text-[10px] font-bold text-slate-400 uppercase tracking-tighter">Intent Type</label>
          <input 
            type="text" 
            className="bg-slate-800 border border-slate-700 rounded p-2 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500"
            value={data.intent_type}
            onChange={(e) => onUpdateNode(selectedNode.id, { intent_type: e.target.value })}
            readOnly
          />
        </div>
      </div>

      <div className="mt-auto">
        <div className="p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg text-[10px] text-blue-400 italic">
          Changes are buffered locally. Synchronize with Platform registry to publish.
        </div>
      </div>
    </div>
  );
};
