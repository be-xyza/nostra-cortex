import { useEffect, useMemo, useRef } from "react";
import * as d3 from "d3";
import { ContributionEdge, ContributionNode } from "../contracts";
import { statusColor, layerColor } from "./ContributionGraphUtils";

type Props = {
  nodes: ContributionNode[];
  edges: ContributionEdge[];
  selectedId: string | null;
  onSelect: (id: string) => void;
};

export function ForceGraph({ nodes, edges, selectedId, onSelect }: Props) {
  const containerRef = useRef<HTMLDivElement | null>(null);

  const graph = useMemo(
    () => ({
      nodes: nodes.map((node) => ({ ...node })),
      links: edges.map((edge) => ({
        source: edge.from,
        target: edge.to,
        explicit: edge.is_explicit ?? false,
        confidence: edge.confidence ?? 0,
        kind: edge.edge_kind ?? "depends_on"
      }))
    }),
    [nodes, edges]
  );

  useEffect(() => {
    const host = containerRef.current;
    if (!host) return;

    host.innerHTML = "";
    const width = Math.max(host.clientWidth, 840);
    const height = Math.max(host.clientHeight, 520);

    const svg = d3
      .select(host)
      .append("svg")
      .attr("width", width)
      .attr("height", height)
      .attr("viewBox", `0 0 ${width} ${height}`)
      .style("display", "block");

    const root = svg.append("g");
    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.25, 3])
      .filter((event) => {
        if (event.type === "wheel") {
          return !!event.ctrlKey || !!event.metaKey;
        }
        return !event.button;
      })
      .on("zoom", (event) => {
        root.attr("transform", event.transform.toString());
      });
    svg.call(zoom);

    const simulation = d3
      .forceSimulation(graph.nodes as d3.SimulationNodeDatum[])
      .force(
        "link",
        d3
          .forceLink(graph.links as d3.SimulationLinkDatum<d3.SimulationNodeDatum>[])
          .id((d) => (d as ContributionNode).id)
          .distance(90)
      )
      .force("charge", d3.forceManyBody().strength(-220))
      .force("center", d3.forceCenter(width / 2, height / 2));

    const links = root
      .append("g")
      .attr("stroke-linecap", "round")
      .selectAll("line")
      .data(graph.links)
      .enter()
      .append("line")
      .attr("stroke", (d) => (d.explicit ? "var(--ink-muted)" : "var(--ink-faint)"))
      .attr("stroke-width", (d) => (d.explicit ? 1.4 : 1.1))
      .attr("stroke-dasharray", (d) => (d.explicit ? null : "6 4"))
      .attr("opacity", 0.65);

    const node = root
      .append("g")
      .selectAll("g")
      .data(graph.nodes)
      .enter()
      .append("g")
      .on("click", (_, d) => onSelect(d.id));

    node
      .append("circle")
      .attr("r", (d) => (d.id === selectedId ? 13 : 10))
      .attr("fill", (d) => statusColor(d.status))
      .attr("stroke", (d) => layerColor(d.layer))
      .attr("stroke-width", (d) => (d.id === selectedId ? 3.8 : 2.4));

    node
      .append("text")
      .attr("font-size", "10")
      .attr("font-weight", "700")
      .attr("fill", "var(--ink)")
      .attr("text-anchor", "middle")
      .attr("dy", 4)
      .text((d) => d.id);

    simulation.on("tick", () => {
      links
        .attr("x1", (d) => (d.source as d3.SimulationNodeDatum).x ?? 0)
        .attr("y1", (d) => (d.source as d3.SimulationNodeDatum).y ?? 0)
        .attr("x2", (d) => (d.target as d3.SimulationNodeDatum).x ?? 0)
        .attr("y2", (d) => (d.target as d3.SimulationNodeDatum).y ?? 0);

      node.attr("transform", (d) => {
        const simNode = d as unknown as d3.SimulationNodeDatum;
        return `translate(${simNode.x ?? 0},${simNode.y ?? 0})`;
      });
    });

    return () => {
      simulation.stop();
    };
  }, [graph, onSelect, selectedId]);

  return <div className="graph-host w-full h-full min-h-[500px]" ref={containerRef} />;
}
