import type { Edge, Node } from "@xyflow/react";

import type {
  OperationalFrequency,
  PlatformCapabilityCatalog,
  SpaceCapabilityGraph,
  SpaceCapabilityNodeOverride,
  SurfacingHeuristic,
} from "../../contracts";

export interface CapabilityEditorNodeData extends Record<string, unknown> {
  capabilityId: string;
  title: string;
  description: string;
  intentType: string;
  routeId?: string;
  category?: string;
  canonicalRequiredRole?: string;
  localRequiredRole?: string;
  effectiveRequiredRole?: string;
  canonicalSurfacingHeuristic?: SurfacingHeuristic;
  surfacingHeuristic?: SurfacingHeuristic;
  effectiveSurfacingHeuristic?: SurfacingHeuristic;
  canonicalOperationalFrequency?: OperationalFrequency;
  operationalFrequency?: OperationalFrequency;
  effectiveOperationalFrequency?: OperationalFrequency;
  localAlias?: string;
  isActive: boolean;
}

export interface CapabilityEditorGraphModel {
  nodes: Node<CapabilityEditorNodeData>[];
  edges: Edge[];
}

export function normalizeCapabilityValue(
  value: { 0: string } | string | undefined,
): string {
  if (!value) return "";
  return typeof value === "string" ? value : value[0];
}

function defaultNodePosition(index: number) {
  return {
    x: 120 + (index % 3) * 280,
    y: 100 + Math.floor(index / 3) * 190,
  };
}

export function buildCapabilityEditorGraph(
  catalog: PlatformCapabilityCatalog,
  graph: SpaceCapabilityGraph,
  positions: Record<string, { x: number; y: number }> = {},
): CapabilityEditorGraphModel {
  const overrides = new Map(
    graph.nodes.map((node) => [normalizeCapabilityValue(node.capabilityId), node]),
  );

  return {
    nodes: catalog.nodes.map((node, index) => {
      const capabilityId = normalizeCapabilityValue(node.id);
      const override = overrides.get(capabilityId);
      return {
        id: capabilityId,
        type: "capability",
        position: positions[capabilityId] ?? defaultNodePosition(index),
        data: {
          capabilityId,
          title: node.name,
          description: node.description,
          intentType: node.intentType,
          routeId: node.routeId,
          category: node.category,
          canonicalRequiredRole: node.requiredRole,
          localRequiredRole: override?.localRequiredRole,
          effectiveRequiredRole: override?.localRequiredRole ?? node.requiredRole,
          canonicalSurfacingHeuristic: node.surfacingHeuristic,
          surfacingHeuristic: override?.surfacingHeuristic,
          effectiveSurfacingHeuristic:
            override?.surfacingHeuristic ?? node.surfacingHeuristic,
          canonicalOperationalFrequency: node.operationalFrequency,
          operationalFrequency: override?.operationalFrequency,
          effectiveOperationalFrequency:
            override?.operationalFrequency ?? node.operationalFrequency,
          localAlias: override?.localAlias,
          isActive: override?.isActive ?? true,
        },
      };
    }),
    edges: catalog.edges.map((edge, index) => ({
      id: `capability-edge-${index}`,
      source: normalizeCapabilityValue(edge.source),
      target: normalizeCapabilityValue(edge.target),
      label: edge.relationship,
      animated: false,
      selectable: false,
    })),
  };
}

export function collectNodePositions(
  nodes: Node<CapabilityEditorNodeData>[],
): Record<string, { x: number; y: number }> {
  return Object.fromEntries(
    nodes.map((node) => [node.id, { x: node.position.x, y: node.position.y }]),
  );
}

export function applyEditorOverride(
  overrides: SpaceCapabilityNodeOverride[],
  capabilityId: string,
  patch: Partial<SpaceCapabilityNodeOverride>,
): SpaceCapabilityNodeOverride[] {
  let matched = false;
  const next = overrides.map((override) => {
    if (normalizeCapabilityValue(override.capabilityId) !== capabilityId) {
      return override;
    }
    matched = true;
    return {
      ...override,
      ...patch,
      capabilityId: override.capabilityId,
    };
  });

  if (matched) {
    return next;
  }

  return next.concat({
    capabilityId,
    isActive: true,
    ...patch,
  });
}

export function buildPersistedCapabilityGraph(
  baseGraph: SpaceCapabilityGraph,
  overrides: SpaceCapabilityNodeOverride[],
  meta: { updatedAt: string; updatedBy: string; lineageRef: string },
): SpaceCapabilityGraph {
  return {
    ...baseGraph,
    nodes: overrides.map((override) => ({ ...override })),
    updatedAt: meta.updatedAt,
    updatedBy: meta.updatedBy,
    lineageRef: meta.lineageRef,
  };
}
