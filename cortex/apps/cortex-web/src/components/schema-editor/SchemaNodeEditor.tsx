import React, { useState, useCallback, useMemo } from 'react';
import {
  ReactFlowProvider,
  Node,
  Edge,
  OnNodesChange,
  OnEdgesChange,
  OnConnect,
  applyNodeChanges,
  applyEdgeChanges,
  addEdge,
  ReactFlowInstance,
  Connection,
} from '@xyflow/react';

import { SchemaSidebar } from './SchemaSidebar';
import { SchemaCanvas } from './SchemaCanvas';
import { SchemaNodeProps } from './SchemaNodeProps';
import { CapabilityNodeData } from './SchemaCustomNodes';
import { PLATFORM_CAPABILITY_CATALOG } from '../../store/seedData';

const initialNodes: Node<CapabilityNodeData>[] = PLATFORM_CAPABILITY_CATALOG.nodes.map((n, i) => ({
  id: typeof n.id === 'string' ? n.id : n.id[0],
  type: 'capability',
  position: { x: 100 + (i % 3) * 250, y: 100 + Math.floor(i / 3) * 150 },
  data: {
    title: n.name,
    description: n.description,
    surfacing_heuristic: n.surfacingHeuristic,
    intent_type: n.intentType,
  },
}));

const initialEdges: Edge[] = PLATFORM_CAPABILITY_CATALOG.edges.map((e, i) => ({
  id: `e-${i}`,
  source: typeof e.source === 'string' ? e.source : e.source[0],
  target: typeof e.target === 'string' ? e.target : e.target[0],
  label: e.relationship,
  animated: true,
}));

export const SchemaNodeEditor = () => {
  const [nodes, setNodes] = useState<Node<CapabilityNodeData>[]>(initialNodes);
  const [edges, setEdges] = useState<Edge[]>(initialEdges);
  const [reactFlowInstance, setReactFlowInstance] = useState<ReactFlowInstance | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);

  const onNodesChange: OnNodesChange<Node<CapabilityNodeData>> = useCallback(
    (changes) => setNodes((nds) => applyNodeChanges(changes, nds)),
    []
  );

  const onEdgesChange: OnEdgesChange = useCallback(
    (changes) => setEdges((eds) => applyEdgeChanges(changes, eds)),
    []
  );

  const onConnect: OnConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge({ ...params, animated: true, label: 'depends_on' }, eds)),
    []
  );

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      if (!reactFlowInstance) return;

      const nodeType = event.dataTransfer.getData('application/reactflow');
      if (!nodeType) return;

      const position = reactFlowInstance.screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      const newNodeId = `new-cap-${Date.now()}`;
      const newNode: Node<CapabilityNodeData> = {
        id: newNodeId,
        type: 'capability',
        position,
        data: {
          title: 'New Capability',
          description: 'Define the purpose of this capability...',
          intent_type: nodeType,
          surfacing_heuristic: 'Secondary',
        },
      };

      setNodes((nds) => nds.concat(newNode));
      setSelectedNodeId(newNodeId);
    },
    [reactFlowInstance]
  );

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node) => {
    setSelectedNodeId(node.id);
  }, []);

  const onUpdateNode = useCallback((nodeId: string, newData: Partial<CapabilityNodeData>) => {
    setNodes((nds) =>
      nds.map((node) => {
        if (node.id === nodeId) {
          return { ...node, data: { ...node.data, ...newData } };
        }
        return node;
      })
    );
  }, []);

  const onDeleteNode = useCallback((nodeId: string) => {
    setNodes((nds) => nds.filter((n) => n.id !== nodeId));
    setEdges((eds) => eds.filter((e) => e.source !== nodeId && e.target !== nodeId));
    setSelectedNodeId(null);
  }, []);

  const selectedNode = useMemo(
    () => nodes.find((n) => n.id === selectedNodeId) || null,
    [nodes, selectedNodeId]
  );

  return (
    <div className="flex h-[600px] w-full bg-slate-950 border border-slate-800 rounded-xl overflow-hidden shadow-2xl">
      <ReactFlowProvider>
        <SchemaSidebar />
        <SchemaCanvas
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onDrop={onDrop}
          onNodeClick={onNodeClick}
          setReactFlowInstance={setReactFlowInstance}
        />
        <SchemaNodeProps
          selectedNode={selectedNode}
          onUpdateNode={onUpdateNode}
          onDeleteNode={onDeleteNode}
        />
      </ReactFlowProvider>
    </div>
  );
};
