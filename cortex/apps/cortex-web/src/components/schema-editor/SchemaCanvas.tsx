import React, { useCallback, useRef } from 'react';
import {
  ReactFlow,
  Controls,
  Background,
  applyNodeChanges,
  applyEdgeChanges,
  addEdge,
  Connection,
  Edge,
  Node,
  OnNodesChange,
  OnEdgesChange,
  OnConnect,
  ReactFlowInstance,
  Panel,
  BackgroundVariant,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { SchemaCustomNode, CapabilityNodeData } from './SchemaCustomNodes';

const nodeTypes = {
  capability: SchemaCustomNode,
};

type SchemaCanvasProps = {
  nodes: Node<any>[];
  edges: Edge[];
  onNodesChange: OnNodesChange<any>;
  onEdgesChange: OnEdgesChange;
  onConnect: OnConnect;
  onDrop: (event: React.DragEvent, reactFlowInstance: ReactFlowInstance | null) => void;
  onNodeClick: (event: React.MouseEvent, node: Node<any>) => void;
  setReactFlowInstance: (instance: ReactFlowInstance | null) => void;
};

export const SchemaCanvas = ({
  nodes,
  edges,
  onNodesChange,
  onEdgesChange,
  onConnect,
  onDrop,
  onNodeClick,
  setReactFlowInstance
}: SchemaCanvasProps) => {
  const reactFlowWrapper = useRef<HTMLDivElement>(null);

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  return (
    <div className="flex-1 h-full relative" ref={reactFlowWrapper}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onInit={setReactFlowInstance}
        onDrop={(e) => onDrop(e, null)} // Handler in parent
        onDragOver={onDragOver}
        onNodeClick={onNodeClick}
        nodeTypes={nodeTypes}
        fitView
        snapToGrid
        snapGrid={[15, 15]}
        defaultEdgeOptions={{
          animated: true,
          style: { stroke: '#64748b', strokeWidth: 2 }
        }}
        className="bg-[#0b0e14]"
      >
        <Background variant={BackgroundVariant.Dots} gap={15} size={1} color="#334155" />
        <Controls className="fill-slate-400 bg-slate-800 border-slate-700" />
        <Panel position="top-right" className="flex gap-2">
          <div className="bg-slate-800/80 backdrop-blur border border-slate-700 px-3 py-1 rounded-full text-[10px] text-slate-400 font-mono shadow-xl uppercase tracking-widest">
            A2UI Mode: Schematic
          </div>
        </Panel>
      </ReactFlow>
    </div>
  );
};
