import React, { useEffect, useRef, useState, useCallback } from "react";
import type { ContributionGraph } from "../../contracts";
import { workbenchApi } from "../../api";
import { useUserPreferences, MotionStyle, GraphVariant as GVariant } from "../../store/userPreferences";
import { useAvailableSpaces, DEFAULT_GRAPH_PHYSICS } from "../../store/spacesRegistry";

type GraphVariant = GVariant;

interface Props {
    visible?: boolean;
    variant?: GraphVariant;
    spaceId: string;
}

import ForceGraph2D from "react-force-graph-2d";
import ForceGraph3D from "react-force-graph-3d";
// @ts-ignore — three.js examples lack type declarations
import { UnrealBloomPass } from "three/examples/jsm/postprocessing/UnrealBloomPass";

/**
 * Ambient background graph that renders the Space contribution graph
 * at ultra-low opacity behind the primary heap content.
 *
 * Supports 2D (Canvas) and 3D (WebGL/ThreeJS) variants for comparison.
 * All interactions are disabled — this is pure visual context.
 * 
 * Resolution Logic: Accessibility > Space Theme > User Pref
 */
export function AmbientGraphBackground({ visible: propVisible, variant: propVariant, spaceId }: Props) {
    const containerRef = useRef<HTMLDivElement>(null);
    const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
    const [graphData, setGraphData] = useState<{ nodes: any[]; links: any[] } | null>(null);
    const graphRef = useRef<any>(null);

    // Settings Primitive (Global)
    const { 
        ambientGraphVariant: globalVariant, 
        ambientGraphMotion: globalMotion,
        reduceMotion: globalReduceMotion 
    } = useUserPreferences();
    
    // Space Discovery (Local Override)
    const spaces = useAvailableSpaces();
    const activeSpace = spaces.find(s => s.id === spaceId);
    const spaceOverride = activeSpace?.metadata?.theme;

    // --- Resolution Logic ---
    // 1. Visibility & Variant
    const resolvedVariant = spaceOverride?.ambientGraphVariant || propVariant || globalVariant;
    const isVisible = (propVisible !== false) && resolvedVariant !== "off";

    // 2. Motion
    let resolvedMotion: MotionStyle = spaceOverride?.ambientGraphMotion || globalMotion;
    if (globalReduceMotion) {
        resolvedMotion = "static";
    }

    // 3. Physics Primitives (Principally aligned: Nostra defines, Cortex runs)
    const physics = spaceOverride?.graphPhysics || DEFAULT_GRAPH_PHYSICS;

    // --- UI Execution & Standard Forces ---
    
    // Apply Standardized Graph Physics (Nostra Primitives)
    useEffect(() => {
        if (!graphRef.current || !graphData || !isVisible) return;

        const fg = graphRef.current;
        
        // Universal branching pattern configuration
        fg.d3Force("charge").strength(physics.repulsionStrength);
        fg.d3Force("link").distance(physics.linkDistance);
        
        // Gentle center gravity to keep the cluster centered but expanded
        if (fg.d3Force("center")) {
            fg.d3Force("center").strength(physics.centerGravity);
        }

        // Apply bloom effect to 3D graph if needed
        if (resolvedVariant === "3d") {
            const composer = fg.postProcessingComposer();
            if (composer.passes.length <= 1) {
                const bloomPass = new UnrealBloomPass();
                bloomPass.strength = 1.2;
                bloomPass.radius = 0.3;
                bloomPass.threshold = 0.1;
                composer.addPass(bloomPass);
            }
        }
    }, [resolvedVariant, graphData, isVisible, physics]);

    // Observe container size
    useEffect(() => {
        const el = containerRef.current;
        if (!el) return;

        const obs = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                setDimensions({ width: Math.round(width), height: Math.round(height) });
            }
        });
        obs.observe(el);
        return () => obs.disconnect();
    }, []);

    // Fetch contribution graph data
    useEffect(() => {
        if (!isVisible) return;
        let cancelled = false;
        workbenchApi.getGraph(spaceId).then((graph: ContributionGraph) => {
            if (cancelled) return;
            setGraphData({
                nodes: graph.nodes.map((n) => ({
                    id: n.id,
                    name: n.title,
                    status: n.status,
                    layer: n.layer,
                    val: 1,
                })),
                links: graph.edges.map((e) => ({
                    source: e.from,
                    target: e.to,
                })),
            });

            // Trigger Zoom-to-fit once data is loaded
            setTimeout(() => {
                if (graphRef.current) {
                    graphRef.current.zoomToFit(800, 300);
                }
            }, 800);
        }).catch(() => {});
        return () => { cancelled = true; };
    }, [isVisible, spaceId, resolvedVariant]);

    // --- Living System Motion Effects ---
    useEffect(() => {
        if (!graphRef.current || resolvedMotion === "static") return;

        let animationFrame: number;
        let angle = 0;

        const animate = () => {
            if (!graphRef.current) return;

            if (resolvedVariant === "2d" && resolvedMotion === "drift") {
                 // Principally aligned drift: pulse alpha to keep simulation alive 
                 // while gently nudging the center to avoid a static appearance.
                 const fg = graphRef.current;
                 
                 // Fix: Access simulation instance via d3ForceSimulation()
                 const simulation = fg?.d3ForceSimulation?.();
                 if (simulation) {
                    simulation.alphaTarget(0.02);
                 }
                 
                 const driftX = Math.sin(Date.now() / 4000) * 0.4;
                 const driftY = Math.cos(Date.now() / 5000) * 0.4;
                 
                 const center = fg?.d3Force()?.("center");
                 if (center) {
                    center.x(dimensions.width / 2 + driftX);
                    center.y(dimensions.height / 2 + driftY);
                 }
            }

            if (resolvedVariant === "3d" && resolvedMotion === "orbit") {
                // Slow rotation in 3D
                angle += 0.001;
                const distance = 800;
                graphRef.current.cameraPosition({
                    x: distance * Math.sin(angle),
                    z: distance * Math.cos(angle)
                });
            }

            animationFrame = requestAnimationFrame(animate);
        };

        animate();
        return () => cancelAnimationFrame(animationFrame);
    }, [resolvedVariant, resolvedMotion, dimensions]);

    // Custom node painting for 2D — ultra-faint dots
    const paintNode2D = useCallback((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
        const r = 8 / globalScale; 
        ctx.beginPath();
        ctx.arc(node.x, node.y, r, 0, 2 * Math.PI);
        const color = nodeColor(node.layer, 0.9);
        ctx.shadowBlur = 15;
        ctx.shadowColor = color;
        ctx.fillStyle = color;
        ctx.fill();
        ctx.shadowBlur = 0;
    }, []);

    // Custom link painting for 2D — ultra-faint lines
    const paintLink2D = useCallback((link: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
        const start = link.source;
        const end = link.target;
        if (!start?.x || !end?.x) return;
        ctx.beginPath();
        ctx.moveTo(start.x, start.y);
        ctx.lineTo(end.x, end.y);
        ctx.strokeStyle = "rgba(148, 163, 184, 0.6)"; 
        ctx.lineWidth = 3 / globalScale; 
        ctx.stroke();
    }, []);

    const { width, height } = dimensions;
    const hasSize = width > 0 && height > 0;

    if (!isVisible) return null;

    return (
        <div
            ref={containerRef}
            className="absolute inset-0 z-5 pointer-events-none overflow-hidden"
            style={{ opacity: 1 }}
        >
            <GraphErrorBoundary>
                {hasSize && graphData && (
                    <React.Fragment>
                        {resolvedVariant === "2d" ? (
                            <ForceGraph2D
                                key="force-graph-2d"
                                ref={graphRef}
                                graphData={graphData}
                                width={width}
                                height={height}
                                backgroundColor="rgba(0,0,0,0)"
                                nodeCanvasObject={paintNode2D}
                                nodeCanvasObjectMode={() => "replace" as const}
                                linkCanvasObject={paintLink2D}
                                linkCanvasObjectMode={() => "replace" as const}
                                enableNodeDrag={false}
                                enableZoomInteraction={false}
                                enablePanInteraction={false}
                                enablePointerInteraction={false}
                                cooldownTicks={120}
                                d3AlphaDecay={0.01}
                                d3VelocityDecay={0.1}
                                onEngineStop={() => {
                                    // Final precise framing once simulation settles
                                    graphRef.current?.zoomToFit(400, 300);
                                }}
                            />
                        ) : (
                            <ForceGraph3D
                                key="force-graph-3d"
                                ref={graphRef}
                                graphData={graphData}
                                width={width}
                                height={height}
                                backgroundColor="rgba(0,0,0,0)"
                                nodeColor={(node: any) => nodeColor(node.layer, 0.7)}
                                nodeOpacity={0.8}
                                nodeResolution={6}
                                nodeVal={2.0}
                                linkColor={() => "rgba(148, 163, 184, 0.6)"}
                                linkOpacity={0.6}
                                linkWidth={1.5}
                                enableNodeDrag={false}
                                enableNavigationControls={false}
                                enablePointerInteraction={false}
                                cooldownTicks={120}
                                showNavInfo={false}
                                d3AlphaDecay={0.01}
                                d3VelocityDecay={0.1}
                                onEngineStop={() => {
                                    // Final precise framing once simulation settles
                                    graphRef.current?.zoomToFit(400, 300);
                                }}
                            />
                        )}
                    </React.Fragment>
                )}
            </GraphErrorBoundary>
        </div>
    );
}

/**
 * Localized ErrorBoundary to prevent background visualization 
 * failures from crashing the entire Workbench shell.
 */
class GraphErrorBoundary extends React.Component<{ children: React.ReactNode }, { hasError: boolean }> {
    constructor(props: any) {
        super(props);
        this.state = { hasError: false };
    }

    static getDerivedStateFromError() {
        return { hasError: true };
    }

    componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
        console.error("Ambient Graph Visualization Failure:", error, errorInfo);
    }

    render() {
        if (this.state.hasError) return null; // Silently fail for ambient bg
        return this.props.children;
    }
}

function nodeColor(layer: string, alpha: number): string {
    switch (layer) {
        case "protocol": return `rgba(168, 85, 247, ${alpha})`; // Purple
        case "runtime": return `rgba(34, 197, 94, ${alpha})`;  // Green
        case "host": return `rgba(236, 72, 153, ${alpha})`;    // Pink
        case "adapter": return `rgba(234, 179, 8, ${alpha})`;  // Yellow/Gold
        default: return `rgba(59, 130, 246, ${alpha})`;        // Blue
    }
}
