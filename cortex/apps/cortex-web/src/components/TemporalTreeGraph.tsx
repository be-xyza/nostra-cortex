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

interface HierarchyNode extends ContributionNode {
  children?: HierarchyNode[];
}

export function TemporalTreeGraph({ nodes, edges, selectedId, onSelect }: Props) {
  const containerRef = useRef<HTMLDivElement | null>(null);

  const forest = useMemo(() => {
    // 1. Map nodes by ID
    const nodeMap = new Map<string, HierarchyNode>();
    nodes.forEach((n) => nodeMap.set(n.id, { ...n, children: [] }));

    // 2. Identify "supersedes" relationships
    const supersedesEdges = edges.filter((e) => e.edge_kind === "supersedes");
    const hasParent = new Set<string>();

    supersedesEdges.forEach((edge) => {
      const parent = nodeMap.get(edge.to); // "to" is the predecessor/parent in lineage
      const child = nodeMap.get(edge.from); // "from" is the new version/child
      if (parent && child) {
        parent.children = parent.children || [];
        parent.children.push(child);
        hasParent.add(child.id);
      }
    });

    // 3. Roots are nodes that are not children of any "supersedes" edge
    return Array.from(nodeMap.values()).filter((n) => !hasParent.has(n.id));
  }, [nodes, edges]);

  useEffect(() => {
    const host = containerRef.current;
    if (!host || forest.length === 0) return;

    host.innerHTML = "";
    const width = Math.max(host.clientWidth, 840);
    const height = Math.max(host.clientHeight, 600);

    const svg = d3
      .select(host)
      .append("svg")
      .attr("width", width)
      .attr("height", height)
      .attr("viewBox", `0 0 ${width} ${height}`)
      .style("display", "block");

    const g = svg.append("g");
    
    // Zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>().on("zoom", (event) => {
      g.attr("transform", event.transform);
    });
    svg.call(zoom);

    // Layout
    const treeLayout = d3.tree<HierarchyNode>().nodeSize([60, 160]); // [vertical, horizontal]

    let currentY = 50;
    const allRoots = forest.map((rootNode) => {
      const root = d3.hierarchy(rootNode);
      treeLayout(root);
      
      // Offset each tree in the forest
      root.each((d: any) => {
        d.y += currentY;
        d.x += 100; // Left margin
      });
      
      // Calculate bounding box for next tree
      const descendants = root.descendants();
      const maxY = d3.max(descendants, (d: any) => d.x) || 0;
      currentY += maxY + 100;
      
      return root;
    });

    // Render edges
    allRoots.forEach((root) => {
      g.append("g")
        .attr("fill", "none")
        .attr("stroke", "var(--ink-faint)")
        .attr("stroke-width", 1.5)
        .selectAll("path")
        .data(root.links())
        .join("path")
        .attr("d", d3.linkHorizontal<any, any>()
          .x((d: any) => d.y)
          .y((d: any) => d.x)
        );
    });

    // Render nodes
    allRoots.forEach((root) => {
      const node = g.append("g")
        .selectAll("g")
        .data(root.descendants())
        .join("g")
        .attr("transform", (d: any) => `translate(${d.y},${d.x})`)
        .on("click", (_, d) => onSelect(d.data.id))
        .style("cursor", "pointer");

      node.append("circle")
        .attr("r", (d) => (d.data.id === selectedId ? 8 : 6))
        .attr("fill", (d) => statusColor(d.data.status))
        .attr("stroke", (d) => layerColor(d.data.layer))
        .attr("stroke-width", (d) => (d.data.id === selectedId ? 3 : 1.5));

      node.append("text")
        .attr("dy", "0.31em")
        .attr("x", (d) => (d.data.children && d.data.children.length > 0 ? -12 : 12))
        .attr("text-anchor", (d) => (d.data.children && d.data.children.length > 0 ? "end" : "start"))
        .attr("font-size", "10px")
        .attr("font-weight", "500")
        .attr("fill", "var(--ink)")
        .text((d) => d.data.id)
        .clone(true).lower()
        .attr("stroke", "var(--canvas)")
        .attr("stroke-width", 3);
        
      node.append("title")
        .text(d => `${d.data.id}: ${d.data.title}`);
    });

    // Initial view set
    svg.call(zoom.transform, d3.zoomIdentity.translate(20, 20).scale(0.8));

  }, [forest, selectedId, onSelect]);

  return <div className="graph-host w-full h-full min-h-[500px]" ref={containerRef} />;
}
