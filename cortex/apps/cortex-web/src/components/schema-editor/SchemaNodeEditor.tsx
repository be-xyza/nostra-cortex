import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  ReactFlowProvider,
  Node,
  Edge,
  OnNodesChange,
  applyNodeChanges,
  ReactFlowInstance,
} from '@xyflow/react';

import { workbenchApi } from '../../api';
import type {
  OperationalFrequency,
  PlatformCapabilityCatalog,
  SpaceCapabilityGraph,
  SpaceCapabilityGraphUpsertResponse,
  SurfacingHeuristic,
} from '../../contracts';
import { useActiveSpaceContext, useAvailableSpaces } from '../../store/spacesRegistry';
import { useUiStore } from '../../store/uiStore';
import { SchemaSidebar } from './SchemaSidebar';
import { SchemaCanvas } from './SchemaCanvas';
import { SchemaNodeProps } from './SchemaNodeProps';
import {
  applyEditorOverride,
  buildCapabilityEditorGraph,
  buildPersistedCapabilityGraph,
  collectNodePositions,
  type CapabilityEditorNodeData,
} from './schemaEditorModel';

type SchemaNodeEditorProps = {
  spaceId?: string;
};

function defaultLineageRef(actorId?: string) {
  const normalized = actorId?.trim() || 'cortex-web';
  return `steward:${normalized}:capability-overlay-update`;
}

function resolveEditorSpaceId(
  explicitSpaceId: string | undefined,
  activeSpaceId: string,
  availableSpaceIds: string[],
) {
  if (explicitSpaceId?.trim()) return explicitSpaceId.trim();
  if (activeSpaceId && activeSpaceId !== 'meta') return activeSpaceId;
  return availableSpaceIds.find((spaceId) => spaceId !== 'meta') || '';
}

export const SchemaNodeEditor = ({ spaceId }: SchemaNodeEditorProps) => {
  const activeSpaceId = useActiveSpaceContext();
  const availableSpaces = useAvailableSpaces();
  const sessionUser = useUiStore((state) => state.sessionUser);
  const actorId = sessionUser?.actorId || 'cortex-web';
  const actorRole = sessionUser?.role || 'operator';

  const resolvedSpaceId = useMemo(
    () =>
      resolveEditorSpaceId(
        spaceId,
        activeSpaceId,
        availableSpaces.map((space) => space.id),
      ),
    [activeSpaceId, availableSpaces, spaceId],
  );

  const [catalog, setCatalog] = useState<PlatformCapabilityCatalog | null>(null);
  const [baselineGraph, setBaselineGraph] = useState<SpaceCapabilityGraph | null>(null);
  const [draftGraph, setDraftGraph] = useState<SpaceCapabilityGraph | null>(null);
  const [nodes, setNodes] = useState<Node<CapabilityEditorNodeData>[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [, setReactFlowInstance] =
    useState<ReactFlowInstance<Node<CapabilityEditorNodeData>, Edge> | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [lineageRef, setLineageRef] = useState(() => defaultLineageRef(actorId));
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [graphHash, setGraphHash] = useState<string | null>(null);
  const [planHash, setPlanHash] = useState<string | null>(null);
  const positionsRef = useRef<Record<string, { x: number; y: number }>>({});

  const selectedNode = useMemo(
    () => nodes.find((node) => node.id === selectedNodeId) || null,
    [nodes, selectedNodeId],
  );

  const loadEditorState = useCallback(async () => {
    if (!resolvedSpaceId) return;

    setLoading(true);
    setErrorMessage(null);

    try {
      const [nextCatalog, nextGraph, nextPlan, spaces] = await Promise.all([
        workbenchApi.getCapabilityCatalog(),
        workbenchApi.getSpaceCapabilityGraph(resolvedSpaceId),
        workbenchApi.getSpaceNavigationPlan(resolvedSpaceId, {
          actorRole,
          intent: 'navigate',
          density: 'comfortable',
        }),
        workbenchApi.getSpaces(),
      ]);

      const layout = buildCapabilityEditorGraph(nextCatalog, nextGraph, positionsRef.current);
      const registryRecord =
        spaces.items.find((record) => record.spaceId === resolvedSpaceId) || null;

      setCatalog(nextCatalog);
      setBaselineGraph(nextGraph);
      setDraftGraph(nextGraph);
      setNodes(layout.nodes);
      setEdges(layout.edges);
      setSelectedNodeId((current) =>
        current && layout.nodes.some((node) => node.id === current)
          ? current
          : layout.nodes[0]?.id || null,
      );
      setLineageRef(nextGraph.lineageRef?.trim() || defaultLineageRef(actorId));
      setGraphHash(registryRecord?.capabilityGraphHash || null);
      setPlanHash(nextPlan.planHash);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }, [actorId, actorRole, resolvedSpaceId]);

  useEffect(() => {
    void loadEditorState();
  }, [loadEditorState]);

  const dirty = useMemo(() => {
    if (!baselineGraph || !draftGraph) return false;
    return JSON.stringify(baselineGraph.nodes) !== JSON.stringify(draftGraph.nodes)
      || (baselineGraph.lineageRef || '') !== lineageRef.trim();
  }, [baselineGraph, draftGraph, lineageRef]);

  const onNodesChange: OnNodesChange<Node<CapabilityEditorNodeData>> = useCallback(
    (changes) =>
      setNodes((currentNodes) => {
        const nextNodes = applyNodeChanges<Node<CapabilityEditorNodeData>>(changes, currentNodes);
        const positions = collectNodePositions(nextNodes);
        positionsRef.current = positions;
        return nextNodes;
      }),
    [],
  );

  const onNodeClick = useCallback((_: React.MouseEvent, node: Node<CapabilityEditorNodeData>) => {
    setSelectedNodeId(node.id);
  }, []);

  const onUpdateNode = useCallback(
    (
      capabilityId: string,
      patch: {
        isActive?: boolean;
        localAlias?: string;
        localRequiredRole?: string;
        surfacingHeuristic?: SurfacingHeuristic;
        operationalFrequency?: OperationalFrequency;
      },
    ) => {
      setDraftGraph((currentGraph) => {
        if (!currentGraph) return currentGraph;
        return {
          ...currentGraph,
          nodes: applyEditorOverride(currentGraph.nodes, capabilityId, patch),
        };
      });
      setNodes((currentNodes) =>
        currentNodes.map((node) =>
          node.id === capabilityId
            ? {
                ...node,
                data: {
                  ...node.data,
                  ...patch,
                  effectiveRequiredRole:
                    'localRequiredRole' in patch
                      ? patch.localRequiredRole || node.data.canonicalRequiredRole
                      : node.data.localRequiredRole || node.data.canonicalRequiredRole,
                  effectiveSurfacingHeuristic:
                    'surfacingHeuristic' in patch
                      ? patch.surfacingHeuristic || node.data.canonicalSurfacingHeuristic
                      : node.data.surfacingHeuristic || node.data.canonicalSurfacingHeuristic,
                  effectiveOperationalFrequency:
                    'operationalFrequency' in patch
                      ? patch.operationalFrequency || node.data.canonicalOperationalFrequency
                      : node.data.operationalFrequency || node.data.canonicalOperationalFrequency,
                },
              }
            : node,
        ),
      );
      setStatusMessage(null);
      setErrorMessage(null);
    },
    [],
  );

  const onSave = useCallback(async () => {
    if (!draftGraph || !resolvedSpaceId) return;

    setSaving(true);
    setStatusMessage(null);
    setErrorMessage(null);

    try {
      const payload = buildPersistedCapabilityGraph(draftGraph, draftGraph.nodes, {
        updatedAt: new Date().toISOString(),
        updatedBy: actorId,
        lineageRef: lineageRef.trim(),
      });

      const receipt: SpaceCapabilityGraphUpsertResponse =
        await workbenchApi.putSpaceCapabilityGraph(
          resolvedSpaceId,
          payload,
          actorRole,
          actorId,
        );

      setGraphHash(receipt.capabilityGraphHash);
      setStatusMessage(`Saved steward overlay for ${receipt.spaceId}.`);
      await loadEditorState();
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }, [actorId, actorRole, draftGraph, lineageRef, loadEditorState, resolvedSpaceId]);

  if (!resolvedSpaceId) {
    return (
      <div className="flex h-[600px] w-full items-center justify-center rounded-xl border border-slate-800 bg-slate-950 p-8 text-center text-slate-400">
        Select a real Space to edit its capability overlay. The meta workbench does not own persisted capability graphs.
      </div>
    );
  }

  return (
    <div className="flex h-[600px] w-full bg-slate-950 border border-slate-800 rounded-xl overflow-hidden shadow-2xl">
      <ReactFlowProvider>
        <SchemaSidebar
          spaceId={resolvedSpaceId}
          actorRole={actorRole}
          catalogHash={catalog?.catalogHash || null}
          graphHash={graphHash}
          planHash={planHash}
          dirty={dirty}
          loading={loading}
          saving={saving}
          lineageRef={lineageRef}
          statusMessage={statusMessage}
          errorMessage={errorMessage}
          onLineageRefChange={setLineageRef}
          onRefresh={() => void loadEditorState()}
          onSave={() => void onSave()}
        />
        <SchemaCanvas
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onNodeClick={onNodeClick}
          setReactFlowInstance={setReactFlowInstance}
        />
        <SchemaNodeProps
          selectedNode={selectedNode}
          onUpdateNode={onUpdateNode}
        />
      </ReactFlowProvider>
    </div>
  );
};
