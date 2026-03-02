import { useEffect, useMemo, useRef, useState } from "react";
import * as d3 from "d3";
import { CapabilityEdge, CapabilityNode } from "../contracts";

interface ForceNode extends d3.SimulationNodeDatum, CapabilityNode {
  radius: number;
  color: string;
  hasViolation: boolean;
  isLocked: boolean;
}

interface ForceEdge extends d3.SimulationLinkDatum<ForceNode>, CapabilityEdge {}

interface CapabilityMatrixMapProps {
  nodes: CapabilityNode[];
  edges: CapabilityEdge[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onNavigate?: (routeId: string) => void;
  currentRole?: string;
}

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

export function CapabilityMatrixMap({
  nodes,
  edges,
  selectedId,
  onSelect,
  onNavigate,
  currentRole = "operator",
}: CapabilityMatrixMapProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });

  useEffect(() => {
    if (!containerRef.current) return;
    const observer = new ResizeObserver((entries) => {
      if (entries.length > 0) {
        setDimensions({
          width: entries[0].contentRect.width,
          height: entries[0].contentRect.height,
        });
      }
    });
    observer.observe(containerRef.current);
    return () => observer.disconnect();
  }, []);

  const d3Nodes = useMemo<ForceNode[]>(() => {
    const actorRoleRank = roleRank(currentRole);
    return nodes.map((n) => {
      let color = "#6c7a89";
      let radius = 11;

      if (n.intent_type === "monitor") {
        color = "#4db6ac";
      } else if (n.intent_type === "execute") {
        color = "#64b5f6";
      } else if (n.intent_type === "mutate") {
        color = "#ef5350";
      } else if (n.intent_type === "configure") {
        color = "#ffb74d";
      } else if (n.intent_type === "navigate") {
        color = "#9575cd";
      }

      if (!n.route_id) {
        radius = 14;
      }

      const hasViolation = (n.invariant_violations?.length ?? 0) > 0;
      const requiredRoleRank = roleRank(n.required_role);
      const isLocked = requiredRoleRank > 0 && actorRoleRank < requiredRoleRank;

      return {
        ...n,
        radius,
        color,
        hasViolation,
        isLocked,
      };
    });
  }, [nodes, currentRole]);

  const d3Links = useMemo<ForceEdge[]>(() => {
    return edges.map((e) => ({
      ...e,
      source: e.from,
      target: e.to,
    })) as unknown as ForceEdge[];
  }, [edges]);

  useEffect(() => {
    if (!containerRef.current || d3Nodes.length === 0) return;

    const { width, height } = dimensions;
    const svg = d3.select(containerRef.current).select("svg");
    svg.selectAll("*").remove();

    const g = svg.append("g");

    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on("zoom", (event) => {
        g.attr("transform", event.transform);
      });

    svg.call(zoom as any);
    svg.call(
      zoom.transform as any,
      d3.zoomIdentity.translate(width / 2, height / 2).scale(0.8),
    );

    const simulation = d3
      .forceSimulation<ForceNode>(d3Nodes)
      .force(
        "link",
        d3
          .forceLink<ForceNode, ForceEdge>(d3Links)
          .id((d) => d.id)
          .distance((d) => (d.relationship === "follows" ? 120 : 85)),
      )
      .force("charge", d3.forceManyBody().strength(-300))
      .force(
        "collide",
        d3
          .forceCollide<ForceNode>()
          .radius((d) => d.radius + 10)
          .iterations(2),
      )
      .force("x", d3.forceX().strength(0.05))
      .force("y", d3.forceY().strength(0.05));

    const link = g
      .append("g")
      .attr("stroke", "#4a5568")
      .attr("stroke-opacity", 0.6)
      .selectAll("line")
      .data(d3Links)
      .join("line")
      .attr("stroke-width", (d: ForceEdge) => (d.relationship === "follows" ? 1.4 : 2.2))
      .attr("stroke-dasharray", (d: ForceEdge) => (d.relationship === "follows" ? "4 3" : null))
      .attr("marker-end", (d: ForceEdge) => (d.relationship === "follows" ? "" : "url(#arrowhead)"));

    svg
      .append("defs")
      .append("marker")
      .attr("id", "arrowhead")
      .attr("viewBox", "-0 -5 10 10")
      .attr("refX", 25)
      .attr("refY", 0)
      .attr("orient", "auto")
      .attr("markerWidth", 6)
      .attr("markerHeight", 6)
      .append("svg:path")
      .attr("d", "M 0,-5 L 10 ,0 L 0,5")
      .attr("fill", "#4a5568")
      .style("stroke", "none");

    const node = g
      .append("g")
      .selectAll("g")
      .data(d3Nodes)
      .join("g")
      .style("cursor", (d: ForceNode) =>
        !d.isLocked && d.route_id ? "pointer" : "default",
      )
      .call(drag(simulation) as any)
      .on("click", (event, d) => {
        event.stopPropagation();
        onSelect(d.id);
      })
      .on("dblclick", (event, d) => {
        event.stopPropagation();
        if (!d.isLocked && d.route_id && onNavigate) {
          onNavigate(d.route_id);
        }
      });

    node
      .append("circle")
      .attr("r", (d) => d.radius)
      .attr("fill", (d) => (d.id === selectedId ? "#ffffff" : d.color))
      .attr("fill-opacity", (d) => (d.isLocked ? 0.35 : 0.95))
      .attr("stroke", (d) => {
        if (d.id === selectedId) return "#63b3ed";
        if (d.hasViolation) return "#fc8181";
        if (d.isLocked) return "#a0aec0";
        return "#2d3748";
      })
      .attr("stroke-width", (d) => {
        if (d.id === selectedId) return 3;
        if (d.hasViolation || d.isLocked) return 2.4;
        return 1.5;
      })
      .attr("stroke-dasharray", (d) => (d.hasViolation && d.id !== selectedId ? "4 2" : null));

    node
      .append("text")
      .text((d) => d.title)
      .attr("x", 0)
      .attr("y", (d) => d.radius + 12)
      .attr("text-anchor", "middle")
      .attr("fill", (d) => (d.isLocked ? "#a0aec0" : "#e2e8f0"))
      .attr("font-size", "10px")
      .attr("font-family", "system-ui, sans-serif")
      .style("pointer-events", "none");

    simulation.on("tick", () => {
      link
        .attr("x1", (d) => (d.source as ForceNode).x ?? 0)
        .attr("y1", (d) => (d.source as ForceNode).y ?? 0)
        .attr("x2", (d) => (d.target as ForceNode).x ?? 0)
        .attr("y2", (d) => (d.target as ForceNode).y ?? 0);

      node.attr(
        "transform",
        (d) => `translate(${(d as ForceNode).x ?? 0},${(d as ForceNode).y ?? 0})`,
      );
    });

    return () => {
      simulation.stop();
    };
  }, [dimensions.width, dimensions.height, d3Nodes, d3Links, selectedId, onSelect, onNavigate]);

  function drag(simulation: d3.Simulation<ForceNode, undefined>) {
    return d3
      .drag<SVGGElement, ForceNode>()
      .on("start", (event, d) => {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
      })
      .on("drag", (event, d) => {
        d.fx = event.x;
        d.fy = event.y;
      })
      .on("end", (event, d) => {
        if (!event.active) simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
      });
  }

  if (!Array.isArray(nodes) || nodes.length === 0) {
    return <div className="placeholder">No capability nodes available.</div>;
  }

  return (
    <div
      ref={containerRef}
      style={{ width: "100%", height: "100%", overflow: "hidden", position: "relative" }}
    >
      <svg width={dimensions.width} height={dimensions.height} style={{ display: "block" }} />
    </div>
  );
}
