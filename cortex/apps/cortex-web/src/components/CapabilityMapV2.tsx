import React, { useEffect, useMemo, useState, useRef } from "react";
import {
  Background,
  Controls,
  type Edge as FlowEdge,
  Handle,
  MarkerType,
  MiniMap,
  type Node as FlowNode,
  Position,
  ReactFlow,
  useEdgesState,
  useNodesState,
  useReactFlow,
  ReactFlowProvider
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import type { CapabilityEdge, CapabilityLegend, CapabilityLayoutHints, CapabilityNode } from "../contracts";

type CapabilityMapV2Props = {
  nodes: CapabilityNode[];
  edges: CapabilityEdge[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onNavigate?: (routeId: string) => void;
  currentRole?: string;
  layoutHints?: CapabilityLayoutHints;
  legend?: CapabilityLegend;
};

type CapabilityNodeData = {
  node: CapabilityNode;
  color: string;
  isLocked: boolean;
  lockReason?: string;
  onNavigate?: (routeId: string) => void;
};

import dagre from "@dagrejs/dagre";

const dagreGraph = new dagre.graphlib.Graph();
dagreGraph.setDefaultEdgeLabel(() => ({}));



const COLORS = {
  healthy: "#22c55e",
  warning: "#f59e0b",
  critical: "#ef4444",
  muted: "#64748b",
  border: "rgba(255,255,255,0.1)",
  background: "#020617",
  intent: {
    monitor: "#38bdf8",
    execute: "#22c55e",
    mutate: "#f97316",
    configure: "#f59e0b",
    navigate: "#a78bfa",
    default: "#94a3b8"
  }
};

function healthGlow(health?: string): string {
  switch ((health ?? "").toLowerCase()) {
    case "healthy":
    case "pass":
      return `0 0 15px ${COLORS.healthy}33`;
    case "warning":
      return `0 0 15px ${COLORS.warning}40`;
    case "critical":
    case "fail":
      return `0 0 20px ${COLORS.critical}59`;
    default:
      return "none";
  }
}

function healthColor(health?: string): string {
  switch ((health ?? "").toLowerCase()) {
    case "healthy":
    case "pass":
      return COLORS.healthy;
    case "warning":
      return COLORS.warning;
    case "critical":
    case "fail":
      return COLORS.critical;
    default:
      return COLORS.muted;
  }
}

const CapabilityNodeCard = ({ data, selected }: { data: CapabilityNodeData; selected: boolean }) => {
  const { node, color, isLocked, lockReason, onNavigate } = data;
  const health = node.health || "healthy";
  
  const healthEffect = healthGlow(health);
  const borderColor = health === "healthy" && !selected ? "rgba(255,255,255,0.1)" : healthColor(health);

  return (
    <div
      style={{
        width: nodeWidth,
        height: nodeHeight,
        padding: "16px",
        borderRadius: "16px",
        background: isLocked ? "rgba(15, 23, 42, 0.9)" : "rgba(30, 41, 59, 0.7)",
        backdropFilter: "blur(16px)",
        border: `0.5px solid ${selected ? color : borderColor}`,
        boxShadow: selected ? `0 0 40px ${color}33` : healthEffect,
        display: "flex",
        flexDirection: "column",
        gap: "12px",
        color: "#f8fafc",
        position: "relative",
        overflow: "hidden",
        transition: "all 0.3s cubic-bezier(0.4, 0, 0.2, 1)",
        opacity: isLocked ? 0.6 : 1,
      }}
    >
      <Handle type="target" position={Position.Top} style={{ background: "rgba(148, 163, 184, 0.5)", border: 0, width: 8, height: 4, borderRadius: 2 }} />
      
      {/* Intent Accent Line */}
      <div style={{ position: "absolute", top: 0, left: 0, width: "2px", height: "100%", background: color }} />
      
      {/* Content Header */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 12 }}>
        <div style={{ flex: 1 }}>
          <div style={{ fontSize: "10px", textTransform: "uppercase", letterSpacing: "0.1em", color: "#94a3b8", fontWeight: 700, marginBottom: 4 }}>
            {node.intent_type} {node.promotion_status ? `• ${node.promotion_status}` : ""}
          </div>
          <div style={{ fontSize: "15px", fontWeight: 800, lineHeight: 1.2, color: "#f1f5f9" }}>{node.title}</div>
        </div>
        
        {/* Health Indicator Pips */}
        <div 
          style={{ 
            width: 8, 
            height: 8, 
            borderRadius: "50%", 
            background: healthColor(health),
            boxShadow: `0 0 10px ${healthColor(health)}88`
          }} 
        />
      </div>

      <div style={{ fontSize: "11px", color: "#94a3b8", lineHeight: 1.5, display: "-webkit-box", WebkitLineClamp: 2, WebkitBoxOrient: "vertical", overflow: "hidden" }}>
        {node.description}
      </div>

      {isLocked ? (
        <div style={{ marginTop: "auto", display: "flex", alignItems: "center", gap: 6, fontSize: "10px", color: "#ef4444", fontWeight: 600 }}>
          <span style={{ fontSize: 12 }}>🔒</span> {lockReason}
        </div>
      ) : (
        <button
          className="nodrag"
          onClick={(e) => {
            e.stopPropagation();
            if (node.route_id) onNavigate?.(node.route_id);
          }}
          disabled={!node.route_id}
          style={{
            marginTop: "auto",
            padding: "8px 12px",
            borderRadius: "8px",
            background: "rgba(255,255,255,0.05)",
            border: "1px solid rgba(255,255,255,0.1)",
            color: "#fff",
            fontSize: "11px",
            fontWeight: 600,
            cursor: node.route_id ? "pointer" : "default",
            transition: "all 0.2s",
            textAlign: "center"
          }}
        >
          {node.route_id ? "Launch Capability" : "View Details"}
        </button>
      )}

      {/* Decorative Gradient Overlay */}
      <div 
        style={{ 
          position: "absolute", 
          bottom: "-20%", 
          right: "-10%", 
          width: "60%", 
          height: "60%", 
          background: `radial-gradient(circle, ${color}11 0%, transparent 70%)`,
          pointerEvents: "none"
        }} 
      />
      
      <Handle type="source" position={Position.Bottom} style={{ background: "#475569", border: 0, width: 8, height: 4, borderRadius: 2 }} />
    </div>
  );
};

const nodeWidth = 260;
const nodeHeight = 150;

function CapabilityGroupNode({ data }: { data: { label: string; color: string } }) {
  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        borderRadius: 20,
        backgroundColor: "rgba(15, 23, 42, 0.4)",
        border: `1.5px dashed ${data.color}44`,
        padding: "12px 16px",
        pointerEvents: "none",
        position: "relative"
      }}
    >
      <div
        style={{
          position: "absolute",
          top: -12,
          left: 16,
          fontSize: 11,
          fontWeight: 700,
          textTransform: "uppercase",
          letterSpacing: "0.1em",
          color: data.color,
          backgroundColor: "#020617",
          padding: "2px 8px",
          borderRadius: 4,
          border: `1px solid ${data.color}33`,
          boxShadow: `0 0 15px ${data.color}11`
        }}
      >
        {data.label}
      </div>
    </div>
  );
}

const nodeTypes = {
  capabilityNode: CapabilityNodeCard,
  capabilityGroup: CapabilityGroupNode
};

function roleRank(role?: string): number {
  switch ((role ?? "").toLowerCase()) {
    case "viewer":
      return 1;
    case "editor":
      return 2;
    case "operator":
      return 3;
    case "steward":
      return 4;
    default:
      return 0;
  }
}

function intentColor(intentType: string, legend?: CapabilityLegend): string {
  const fromLegend = legend?.intent_type_colors?.[intentType];
  if (fromLegend) return fromLegend;
  
  return COLORS.intent[intentType as keyof typeof COLORS.intent] || COLORS.intent.default;
}

// Projection types and mode selector
type ProjectionMode = "functional" | "pattern";

interface CapabilityProjectionToggleProps {
  mode: ProjectionMode;
  onChange: (mode: ProjectionMode) => void;
}

function CapabilityProjectionToggle({ mode, onChange }: CapabilityProjectionToggleProps) {
  return (
    <div
      style={{
        position: "absolute",
        top: 20,
        left: 20,
        zIndex: 1000,
        background: "rgba(15,23,42,0.85)",
        backdropFilter: "blur(12px)",
        padding: "4px",
        borderRadius: "10px",
        border: "1px solid rgba(255,255,255,0.1)",
        display: "flex",
        gap: "4px",
        boxShadow: "0 10px 30px rgba(0,0,0,0.3)"
      }}
    >
      <button
        onClick={() => onChange("functional")}
        style={{
          padding: "6px 14px",
          borderRadius: "7px",
          fontSize: "10px",
          fontWeight: 700,
          textTransform: "uppercase",
          letterSpacing: "0.05em",
          border: 0,
          cursor: "pointer",
          transition: "all 0.2s cubic-bezier(0.4, 0, 0.2, 1)",
          background: mode === "functional" ? "#3b82f6" : "transparent",
          color: mode === "functional" ? "#fff" : "#94a3b8"
        }}
      >
        Functional
      </button>
      <button
        onClick={() => onChange("pattern")}
        style={{
          padding: "6px 14px",
          borderRadius: "7px",
          fontSize: "10px",
          fontWeight: 700,
          textTransform: "uppercase",
          letterSpacing: "0.05em",
          border: 0,
          cursor: "pointer",
          transition: "all 0.2s cubic-bezier(0.4, 0, 0.2, 1)",
          background: mode === "pattern" ? "#3b82f6" : "transparent",
          color: mode === "pattern" ? "#fff" : "#94a3b8"
        }}
      >
        Pattern
      </button>
    </div>
  );
}

function getLayoutedElements(
  graphNodes: CapabilityNode[],
  graphEdges: CapabilityEdge[],
  actorRole: string,
  onNavigate: ((routeId: string) => void) | undefined,
  layoutHints: CapabilityLayoutHints | undefined,
  legend: CapabilityLegend | undefined,
  direction = "TB",
  clusterBy: "domain" | "pattern" = "domain"
): FlowNode[] {
  const role = roleRank(actorRole);
  
  dagreGraph.setGraph({ rankdir: direction, nodesep: 100, ranksep: 140 });

  // 1. Discover groups for zoning
  const groups = new Set<string>();
  graphNodes.forEach(n => {
    const groupKey = clusterBy === "domain" ? n.domain : n.pattern_id;
    if (groupKey) groups.add(groupKey);
  });

  // 2. Setup nodes in dagre
  graphNodes.forEach((node) => {
    dagreGraph.setNode(node.id, { width: nodeWidth, height: nodeHeight });
  });

  graphEdges.forEach((edge) => {
    dagreGraph.setEdge(edge.from, edge.to);
  });

  dagre.layout(dagreGraph);

  // 3. Build Flow Nodes
  const nodes: FlowNode[] = graphNodes.map((node) => {
    const nodeWithPosition = dagreGraph.node(node.id);
    const requiredRole = node.required_role ?? node.inspector?.required_role;
    const locked = roleRank(requiredRole) > role;
    const groupKey = clusterBy === "domain" ? node.domain : node.pattern_id;

    return {
      id: node.id,
      type: "capabilityNode",
      position: {
        x: nodeWithPosition.x - nodeWidth / 2,
        y: nodeWithPosition.y - nodeHeight / 2
      },
      parentId: groupKey ? `zone:${groupKey}` : undefined,
      extent: groupKey ? "parent" : undefined,
      data: {
        node,
        color: intentColor(node.intent_type, legend),
        isLocked: locked,
        lockReason: locked ? node.locked_reason ?? `Requires ${requiredRole} role` : undefined,
        onNavigate
      }
    };
  });

  // 4. Build Zone Nodes (Parents)
  const zoneNodes: FlowNode[] = Array.from(groups).map(groupKey => {
    const groupNodes = nodes.filter(n => n.parentId === `zone:${groupKey}`);
    
    // Calculate bounding box for the domain nodes
    const minX = Math.min(...groupNodes.map(n => n.position.x)) - 60;
    const minY = Math.min(...groupNodes.map(n => n.position.y)) - 80;
    const maxX = Math.max(...groupNodes.map(n => n.position.x + nodeWidth)) + 60;
    const maxY = Math.max(...groupNodes.map(n => n.position.y + nodeHeight)) + 60;

    const groupColor = layoutHints?.groups.find(g => g.key === groupKey || g.key === `zone:${groupKey}`)?.color 
      || '#94a3b8';

    return {
      id: `zone:${groupKey}`,
      type: "capabilityGroup",
      position: { x: minX, y: minY },
      style: { width: maxX - minX, height: maxY - minY },
      data: { label: groupKey, color: groupColor },
      zIndex: -1
    };
  });

  // 5. Adjust child positions to be relative to parent
  nodes.forEach(node => {
    if (node.parentId) {
      const parent = zoneNodes.find(z => z.id === node.parentId);
      if (parent) {
        node.position.x -= parent.position.x;
        node.position.y -= parent.position.y;
      }
    }
  });

  return [...zoneNodes, ...nodes];
}

function buildEdges(graphEdges: CapabilityEdge[]): FlowEdge[] {
  return graphEdges.map((edge) => {
    const isFollows = edge.relationship === "follows";
    const isDrillDown = edge.relationship === "drill_down";
    
    return {
      id: `${edge.from}->${edge.to}:${edge.relationship}`,
      source: edge.from,
      target: edge.to,
      data: { ...edge } as Record<string, unknown>,
      animated: isFollows,
      label: edge.relationship_label || edge.relationship,
      labelStyle: { fill: "#94a3b8", fontSize: 9, fontWeight: 600, textTransform: "uppercase" },
      labelBgPadding: [4, 2],
      labelBgBorderRadius: 4,
      labelBgStyle: { fill: "#0f172a", fillOpacity: 0.8 },
      markerEnd: {
        type: MarkerType.ArrowClosed,
        color: "#64748b",
        width: 20,
        height: 20
      },
      style: {
        stroke: isDrillDown ? "#3b82f6" : "#64748b",
        strokeWidth: isDrillDown ? 2 : 1.2,
        strokeDasharray: isFollows ? "6 4" : undefined,
        opacity: isFollows ? 0.6 : 0.8,
      }
    };
  });
}

function CapabilityMapV2Inner({
  nodes,
  edges,
  selectedId,
  onSelect,
  onNavigate,
  currentRole = "operator",
  layoutHints,
  legend
}: CapabilityMapV2Props) {
  const [projectionMode, setProjectionMode] = useState<ProjectionMode>("functional");

  // Pre-filter nodes to eliminate governance patterns (which are shown in the RulesMatrixWidget)
  const { filteredNodes, filteredEdges } = useMemo(() => {
    const fNodes = nodes.filter(n => n.domain !== "pattern");
    const validIds = new Set(fNodes.map(n => n.id));
    const fEdges = edges.filter(e => validIds.has(e.from) && validIds.has(e.to));
    return { filteredNodes: fNodes, filteredEdges: fEdges };
  }, [nodes, edges]);

  const flowNodes = useMemo(() => {
    const clusterBy = projectionMode === "functional" ? "domain" : "pattern";
    const deterministic = getLayoutedElements(filteredNodes, filteredEdges, currentRole, onNavigate, layoutHints, legend, "TB", clusterBy);
    return deterministic.map((node) => ({ ...node, selected: node.id === selectedId }));
  }, [filteredNodes, filteredEdges, currentRole, onNavigate, layoutHints, legend, selectedId, projectionMode]);
  
  const flowEdges = useMemo(() => buildEdges(filteredEdges), [filteredEdges]);

  const [rfNodes, setRfNodes, onNodesChange] = useNodesState(flowNodes as FlowNode[]);
  const [rfEdges, setRfEdges, onEdgesChange] = useEdgesState(flowEdges as FlowEdge[]);
  const [selectedEdgeId, setSelectedEdgeId] = useState<string | null>(null);
  const [focusTrigger, setFocusTrigger] = useState(0);

  const { fitView } = useReactFlow();
  const lastSelectedIdRef = React.useRef<string | null>(null);
  const lastFocusTriggerRef = React.useRef<number>(0);

  // Spatial Keyboard Navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Check if user is typing in an input
      if (["INPUT", "TEXTAREA"].includes((e.target as HTMLElement).tagName)) return;

      const directions: Record<string, { dx: number; dy: number }> = {
        ArrowUp: { dx: 0, dy: -1 },
        ArrowDown: { dx: 0, dy: 1 },
        ArrowLeft: { dx: -1, dy: 0 },
        ArrowRight: { dx: 1, dy: 0 }
      };

      const move = directions[e.key];
      if (move) {
        e.preventDefault();
        const currentNode = rfNodes.find(n => n.id === selectedId && n.type === "capabilityNode");
        
        const getAbsPos = (node: FlowNode) => {
          if (!node.parentId) return node.position;
          const parent = rfNodes.find(n => n.id === node.parentId);
          return {
            x: node.position.x + (parent?.position.x ?? 0),
            y: node.position.y + (parent?.position.y ?? 0)
          };
        };

        if (!currentNode) {
          const first = rfNodes.find(n => n.type === "capabilityNode");
          if (first) {
            onSelect(first.id);
            setFocusTrigger(prev => prev + 1);
          }
          return;
        }

        const currentPos = getAbsPos(currentNode);

        // Spatial Search
        let bestNode: FlowNode | null = null;
        let minScore = Infinity;

        rfNodes.forEach(node => {
          if (node.id === currentNode.id || node.type !== "capabilityNode") return;

          const nodePos = getAbsPos(node);
          const dx = nodePos.x - currentPos.x;
          const dy = nodePos.y - currentPos.y;

          // Check if node is in the correct directional half-plane
          const isCorrectDirection = (move.dx > 0 && dx > 20) || 
                                     (move.dx < 0 && dx < -20) || 
                                     (move.dy > 0 && dy > 20) || 
                                     (move.dy < 0 && dy < -20);

          if (isCorrectDirection) {
            const distance = Math.sqrt(dx * dx + dy * dy);
            const lateralDistance = Math.abs(move.dx !== 0 ? dy : dx);
            const score = distance + lateralDistance * 2.0; // Higher penalty for lateral drift

            if (score < minScore) {
              minScore = score;
              bestNode = node;
            }
          }
        });

        if (bestNode) {
          onSelect((bestNode as FlowNode).id);
          setFocusTrigger(prev => prev + 1);
        }
      }

      if (e.key === "Enter" && selectedId) {
        const node = nodes.find(n => n.id === selectedId);
        if (node?.route_id) {
          e.preventDefault();
          onNavigate?.(node.route_id);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [rfNodes, selectedId, onSelect, fitView, onNavigate, nodes]);

  // Immersive Selection Focus (Cinematic Zoom)
  const selectedNodePos = useMemo(() => {
    if (!selectedId) return null;
    const node = rfNodes.find(n => n.id === selectedId);
    return node ? { x: node.position.x, y: node.position.y } : null;
  }, [selectedId, rfNodes]);

  useEffect(() => {
    // Only move camera if:
    // 1. We have an explicit target (selectedId)
    // 2. AND EITHER the selection actually changed OR a manual focus trigger was fired
    // 3. AND the node is actually available in the React Flow store
    const idChanged = selectedId !== lastSelectedIdRef.current;
    const triggerFired = focusTrigger !== lastFocusTriggerRef.current;
    const modeChanged = projectionMode !== (lastSelectedIdRef as any).currentMode;

    if (selectedId && (idChanged || triggerFired || modeChanged)) {
      const exists = rfNodes.some(n => n.id === selectedId);
      if (exists) {
        // Update refs to prevent redundant fires on the next store update
        lastSelectedIdRef.current = selectedId;
        lastFocusTriggerRef.current = focusTrigger;
        (lastSelectedIdRef as any).currentMode = projectionMode;

        fitView({ 
          nodes: [{ id: selectedId }], 
          duration: 900, 
          minZoom: 0.8,
          maxZoom: 1.2,
          // Shift the viewpoint to the left to account for the right-side inspector
          // This creates a "fixed" feel where the card is centered in the remaining space
          padding: { top: 100, right: 480, bottom: 100, left: 100 }
        });
      }
    } else if (!selectedId) {
      // Sync the ref when nothing is selected
      lastSelectedIdRef.current = null;
    }
  }, [selectedId, focusTrigger, fitView, rfNodes, projectionMode]);

  useEffect(() => {
    setRfNodes(flowNodes as FlowNode[]);
  }, [flowNodes, setRfNodes]);

  useEffect(() => {
    setRfEdges(flowEdges as FlowEdge[]);
  }, [flowEdges, setRfEdges]);

  const selectedEdge = useMemo(() => {
    if (!selectedEdgeId) return null;
    const edge = rfEdges.find((item) => item.id === selectedEdgeId);
    return (edge?.data as CapabilityEdge | undefined) ?? null;
  }, [rfEdges, selectedEdgeId]);

  return (
    <div style={{ position: "relative", width: "100%", height: "100%", minHeight: 640 }}>
      {/* Cinematic Dimmer Layer */}
      <div 
        style={{ 
          position: "absolute",
          inset: 0,
          background: "rgba(2, 6, 23, 0.4)",
          backdropFilter: "none",
          opacity: selectedId ? 1 : 0,
          pointerEvents: "none",
          transition: "opacity 0.6s ease",
          zIndex: 5
        }}
      />
      
      <CapabilityProjectionToggle mode={projectionMode} onChange={setProjectionMode} />
      <ReactFlow
        nodes={rfNodes}
        edges={rfEdges}
        nodeTypes={nodeTypes as any}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        nodesConnectable={false}
        nodesDraggable={false}
        elementsSelectable={true}
        minZoom={0.25}
        maxZoom={1.5}
        onNodeClick={(_event, node: FlowNode) => {
          setSelectedEdgeId(null);
          onSelect(node.id);
          setFocusTrigger(prev => prev + 1);
        }}
        onNodeDoubleClick={(_event, node: FlowNode) => {
          const routeId = (node.data as CapabilityNodeData).node?.route_id;
          if (routeId) onNavigate?.(routeId);
        }}
        onEdgeClick={(_event, edge: FlowEdge) => {
          setSelectedEdgeId(edge.id);
        }}
        onPaneClick={() => {
          setSelectedEdgeId(null);
          onSelect(null);
        }}
        colorMode="dark"
        style={{
          borderRadius: 20,
          border: "1px solid rgba(255,255,255,0.05)",
          background:
            "radial-gradient(circle at 12% 10%, rgba(30,64,175,0.08), transparent 40%), radial-gradient(circle at 90% 18%, rgba(124,58,237,0.06), transparent 30%), #020617"
        }}
      >
        <MiniMap
          position="bottom-left"
          pannable
          zoomable
          style={{ 
            backgroundColor: "rgba(2,6,23,0.8)", 
            border: "1px solid rgba(255,255,255,0.05)", 
            borderRadius: 12,
            bottom: 20,
            left: 20
          }}
          nodeColor={(node: any) => ((node.data as CapabilityNodeData)?.color || "#475569")}
        />
        <Controls 
          position="bottom-left" 
          style={{ 
            marginBottom: 20,
            marginLeft: 220, // Offset from MiniMap
            background: "rgba(2,6,23,0.8)", 
            border: "1px solid rgba(255,255,255,0.05)", 
            borderRadius: 8, 
            overflow: "hidden" 
          }} 
        />
        <Background gap={24} size={1} color="rgba(51,65,85,0.2)" />
      </ReactFlow>

      {selectedEdge ? (
        <div
          style={{
            position: "absolute",
            right: 20,
            bottom: 20,
            width: 320,
            borderRadius: 16,
            border: "1px solid rgba(255,255,255,0.1)",
            background: "rgba(15,23,42,0.95)",
            backdropFilter: "blur(12px)",
            color: "#e2e8f0",
            padding: 16,
            boxShadow: "0 20px 50px rgba(0,0,0,0.5)",
            display: "grid",
            gap: 8,
            zIndex: 1000
          }}
        >
          <div style={{ fontSize: 10, letterSpacing: "0.1em", textTransform: "uppercase", color: "#60a5fa", fontWeight: 700 }}>
            Relationship Inspector
          </div>
          <div style={{ fontSize: 14, fontWeight: 700, color: "#f8fafc" }}>{selectedEdge.relationship_label ?? selectedEdge.relationship}</div>
          <div style={{ fontSize: 11.5, color: "#94a3b8", lineHeight: 1.5 }}>{selectedEdge.rationale ?? "No formal rationale archived for this relationship."}</div>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginTop: 4 }}>
            {typeof selectedEdge.confidence === "number" ? (
              <span style={{ fontSize: 9, padding: "3px 8px", borderRadius: 6, background: "rgba(59,130,246,0.1)", color: "#93c5fd", border: "1px solid rgba(59,130,246,0.2)" }}>
                CONFIDENCE {selectedEdge.confidence}%
              </span>
            ) : null}
            {selectedEdge.policy_ref ? (
              <span style={{ fontSize: 9, padding: "3px 8px", borderRadius: 6, background: "rgba(148,163,184,0.1)", color: "#cbd5e1", border: "1px solid rgba(255,255,255,0.1)" }}>
                {selectedEdge.policy_ref}
              </span>
            ) : null}
          </div>
        </div>
      ) : null}
    </div>
  );
}

export function CapabilityMapV2(props: CapabilityMapV2Props) {
  return (
    <ReactFlowProvider>
      <CapabilityMapV2Inner {...props} />
    </ReactFlowProvider>
  );
}
