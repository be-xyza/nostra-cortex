import React, { memo } from 'react';
import { Handle, Position, NodeProps, Node } from '@xyflow/react';
import { CapabilityNode, SurfacingHeuristic } from '../../contracts';

export type CapabilityNodeData = {
  title: string;
  description: string;
  surfacing_heuristic?: SurfacingHeuristic;
  intent_type: string;
  onSelect?: (nodeId: string) => void;
};

const DEFAULT_NODE_WIDTH = 200;

export const SchemaCustomNode = memo(({ data, selected }: NodeProps<Node<CapabilityNodeData>>) => {
  const { title, surfacing_heuristic, intent_type } = data;

  const getSurfacingStyles = (heuristic?: SurfacingHeuristic) => {
    switch (heuristic) {
      case 'PrimaryCore':
        return 'bg-blue-500/20 border-blue-500 text-blue-300';
      case 'Secondary':
        return 'bg-purple-500/20 border-purple-500 text-purple-300';
      case 'ContextualDeep':
        return 'bg-amber-500/20 border-amber-500 text-amber-300';
      case 'Hidden':
        return 'bg-gray-500/20 border-gray-500 text-gray-400';
      default:
        return 'bg-slate-800/20 border-slate-700 text-slate-300';
    }
  };

  const style = getSurfacingStyles(surfacing_heuristic);

  return (
    <div 
      className={`px-4 py-3 rounded-lg border-2 shadow-xl transition-all duration-200 ${style} ${selected ? 'ring-2 ring-white scale-105 shadow-white/10' : ''}`}
      style={{ width: DEFAULT_NODE_WIDTH }}
    >
      <Handle type="target" position={Position.Top} className="w-3 h-3 bg-white/50 border-none hover:bg-white" />
      
      <div className="flex flex-col gap-1">
        <div className="text-[10px] font-mono uppercase tracking-wider opacity-60 mb-1 flex justify-between">
          <span>{intent_type}</span>
          {surfacing_heuristic && (
            <span className="font-bold">{surfacing_heuristic}</span>
          )}
        </div>
        <div className="font-bold text-sm truncate">{title}</div>
      </div>

      <Handle type="source" position={Position.Bottom} className="w-3 h-3 bg-white/50 border-none hover:bg-white" />
    </div>
  );
});

SchemaCustomNode.displayName = 'SchemaCustomNode';
