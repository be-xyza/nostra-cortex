import React from 'react';
import {
  ReactFlow,
  Controls,
  Background,
  Edge,
  Node,
  OnNodesChange,
  ReactFlowInstance,
  BackgroundVariant,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { SchemaCustomNode } from './SchemaCustomNodes';
import type { CapabilityEditorNodeData } from './schemaEditorModel';

const nodeTypes = {
  capability: SchemaCustomNode,
};

type SchemaCanvasProps = {
  nodes: Node<CapabilityEditorNodeData>[];
  edges: Edge[];
  onNodesChange: OnNodesChange<Node<CapabilityEditorNodeData>>;
  onNodeClick: (event: React.MouseEvent, node: Node<CapabilityEditorNodeData>) => void;
  setReactFlowInstance: (
    instance: ReactFlowInstance<Node<CapabilityEditorNodeData>, Edge> | null,
  ) => void;
};

export const SchemaCanvas = ({
  nodes,
  edges,
  onNodesChange,
  onNodeClick,
  setReactFlowInstance
}: SchemaCanvasProps) => {
  return (
    <div className="flex-1 h-full relative">
      <ReactFlow<Node<CapabilityEditorNodeData>, Edge>
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onInit={setReactFlowInstance}
        onNodeClick={onNodeClick}
        nodeTypes={nodeTypes}
        fitView
        snapToGrid
        snapGrid={[15, 15]}
        nodesConnectable={false}
        edgesFocusable={false}
        defaultEdgeOptions={{
          style: { stroke: '#64748b', strokeWidth: 2 }
        }}
        className="bg-[#0b0e14]"
      >
        <Background variant={BackgroundVariant.Dots} gap={15} size={1} color="#334155" />
        <Controls className="fill-slate-400 bg-slate-800 border-slate-700" />
      </ReactFlow>
    </div>
  );
};
