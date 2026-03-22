import React, { useMemo, useEffect } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  useNodesState,
  useEdgesState,
  MarkerType,
  Position,
  Handle,
  type Node,
  type Edge,
  type NodeProps,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import dagre from "@dagrejs/dagre";
import { WorkflowTopology, WorkflowTopologyNode } from "../../contracts";

const dagreGraph = new dagre.graphlib.Graph();
dagreGraph.setDefaultEdgeLabel(() => ({}));

const nodeWidth = 220;
const nodeHeight = 80;

const getLayoutedElements = (nodes: Node[], edges: Edge[], direction = "LR") => {
  const isHorizontal = direction === "LR";
  // Reset graph for each layout calculation
  const dg = new dagre.graphlib.Graph();
  dg.setGraph({ rankdir: direction, nodesep: 70, ranksep: 100 });
  dg.setDefaultEdgeLabel(() => ({}));

  nodes.forEach((node) => {
    dg.setNode(node.id, { width: nodeWidth, height: nodeHeight });
  });

  edges.forEach((edge) => {
    dg.setEdge(edge.source, edge.target);
  });

  dagre.layout(dg);

  const newNodes = nodes.map((node) => {
    const nodeWithPosition = dg.node(node.id);
    return {
      ...node,
      targetPosition: isHorizontal ? Position.Left : Position.Top,
      sourcePosition: isHorizontal ? Position.Right : Position.Bottom,
      position: {
        x: nodeWithPosition.x - nodeWidth / 2,
        y: nodeWithPosition.y - nodeHeight / 2,
      },
    };
  });

  return { nodes: newNodes, edges };
};

const WorkflowNode = ({ data }: NodeProps<Node & { label: string; status: WorkflowTopologyNode['status']; type: WorkflowTopologyNode['type']; metadata?: any }>) => {
  const statusColors = {
    pending: "border-cortex-border-strong bg-cortex-bg-panel/50 text-cortex-ink-faint",
    active: "border-cortex-accent bg-cortex-accent/10 text-cortex-accent shadow-[0_0_15px_rgba(59,130,246,0.3)]",
    completed: "border-cortex-ok bg-cortex-ok/10 text-cortex-ok",
    failed: "border-cortex-bad bg-cortex-bad/10 text-cortex-bad",
    skipped: "border-cortex-border-subtle bg-cortex-bg/20 text-cortex-ink-faint italic",
  };

  const typeIcons: Record<string, string> = {
    start: "◉",
    end: "◎",
    state: "⬚",
    decision: "◇",
    action: "⚡",
    gate: "🔒",
  };

  return (
    <div className={`flex flex-col rounded-cortex border px-4 py-2 min-w-[200px] backdrop-blur-sm transition-colors duration-500 ${statusColors[data.status as keyof typeof statusColors] || statusColors.pending}`}>
      <Handle type="target" position={Position.Left} className="w-2! h-2! bg-cortex-line! border-none" />
      <div className="flex items-center gap-2 mb-1">
        <span className="text-xs opacity-50">{typeIcons[(data as any).type] || "•"}</span>
        <span className="text-[10px] uppercase tracking-[0.15em] font-bold opacity-70">{(data as any).type}</span>
        {data.status === 'active' && (
          <div className="flex items-center gap-1.5 ml-auto">
             <span className="text-[9px] font-bold tracking-widest uppercase opacity-80">Execution</span>
             <span className="flex h-1.5 w-1.5 rounded-full bg-cortex-accent animate-pulse" />
          </div>
        )}
      </div>
      <div className="text-sm font-semibold tracking-tight truncate">{(data as any).label}</div>
      {(data as any).metadata?.model && (
        <div className="mt-1.5 text-[10px] font-mono opacity-80 bg-black/40 border border-white/5 px-1.5 py-0.5 rounded w-fit">
          {(data as any).metadata.model}
        </div>
      )}
      <Handle type="source" position={Position.Right} className="w-2! h-2! bg-cortex-line! border-none" />
    </div>
  );
};

const nodeTypes = {
  workflow: WorkflowNode,
};

export interface EvaluationDAGViewerProps {
  topology: WorkflowTopology;
  className?: string;
}

export function EvaluationDAGViewer({ topology, className }: EvaluationDAGViewerProps) {
  const initialNodes = useMemo(() => {
    return topology.nodes.map((n) => ({
      id: n.id,
      type: "workflow",
      data: { label: n.label, status: n.status, type: n.type, metadata: n.metadata },
      position: { x: 0, y: 0 }, // Will be set by layout
    }));
  }, [topology.nodes]);

  const initialEdges = useMemo(() => {
    return topology.edges.map((e) => {
      const isTraversed = e.status === 'traversed';
      const isBlocked = e.status === 'blocked';
      const sourceNode = topology.nodes.find(n => n.id === e.from);
      const isActive = sourceNode?.status === 'active';

      return {
        id: e.id,
        source: e.from,
        target: e.to,
        label: e.label,
        animated: isActive || isTraversed,
        style: {
          stroke: isTraversed ? "var(--ok)" : isBlocked ? "var(--bad)" : "var(--line)",
          strokeWidth: (isActive || isTraversed) ? 3 : 2,
          opacity: (isTraversed || isActive) ? 1 : (isBlocked ? 1 : 0.3),
        },
        markerEnd: {
          type: MarkerType.ArrowClosed,
          color: isTraversed ? "var(--ok)" : isBlocked ? "var(--bad)" : "var(--line)",
          width: 20,
          height: 20,
        },
        labelStyle: { fill: "rgba(255,255,255,0.8)", fontSize: "10px", fontWeight: 700 },
        labelBgPadding: [6, 3] as [number, number],
        labelBgBorderRadius: 4,
        labelBgStyle: { fill: "rgba(10, 18, 33, 0.9)", fillOpacity: 0.9 },
      };
    });
  }, [topology.edges, topology.nodes]);

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  useEffect(() => {
    if (initialNodes.length > 0) {
      const { nodes: layoutedNodes, edges: layoutedEdges } = getLayoutedElements(
        initialNodes as Node[],
        initialEdges as Edge[]
      );
      setNodes(layoutedNodes);
      setEdges(layoutedEdges);
    }
  }, [initialNodes, initialEdges, setNodes, setEdges]);

  return (
    <div className={`h-[500px] w-full rounded-cortex border border-cortex-line bg-cortex-950/40 backdrop-blur-xl overflow-hidden ${className}`}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
        minZoom={0.5}
        maxZoom={1.5}
        defaultEdgeOptions={{
          type: 'smoothstep',
        }}
      >
        <Background color="var(--line)" gap={24} size={1} />
        <Controls className="bg-cortex-bg-panel border-cortex-line fill-cortex-ink shadow-2xl" />
      </ReactFlow>
    </div>
  );
}
