import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { workbenchApi } from "../../api";
import { useUiStore } from "../../store/uiStore";
import { mapShapesToTldraw } from "./spatialMapper";
import { emitA2uiEvent } from "./spatialEventContract";
import {
  SpatialBounds,
  SpatialEdgeShape,
  SpatialNodeShape,
  SpatialPlaneLayoutV1,
  SpatialPlanePayload,
  SpatialPort,
  SpatialReplayState,
  SpatialShape,
  applySpatialLayout,
  replaySpatialPayload,
  validateSpatialPayload,
} from "./spatialReplay";

type CanvasPoint = {
  x: number;
  y: number;
};

type DragState = {
  shapeId: string;
  pointerId: number;
  startPoint: CanvasPoint;
  originX: number;
  originY: number;
  moved: boolean;
};

type PendingEdgeStart = {
  shapeId: string;
  portId: string;
  direction: SpatialPort["direction"];
};

function readFeatureFlag(name: string): boolean {
  if (typeof window !== "undefined") {
    try {
      const override = window.localStorage.getItem(`cortex.feature.${name}`);
      if (override != null) {
        return override.trim().toLowerCase() === "1";
      }
    } catch {
      // ignore storage failures
    }
  }
  return String(import.meta.env[name] ?? "").toLowerCase() === "1";
}

function classifyReplayError(message: string): "contract_invalid" | "adapter_replay_failed" {
  return message.toLowerCase().includes("contract_invalid")
    ? "contract_invalid"
    : "adapter_replay_failed";
}

function roundCoordinate(value: number): number {
  return Math.round(value * 10) / 10;
}

function finiteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function shapeWidth(shape: SpatialShape): number {
  if (shape.kind === "node") return shape.w ?? 220;
  if (shape.kind === "group") return shape.w ?? 320;
  if (shape.kind === "annotation") return shape.w ?? 180;
  if (shape.kind === "note") return shape.w ?? 170;
  return 0;
}

function shapeHeight(shape: SpatialShape): number {
  if (shape.kind === "node") return shape.h ?? 140;
  if (shape.kind === "group") return shape.collapsed ? 56 : (shape.h ?? 220);
  if (shape.kind === "annotation") return shape.h ?? 90;
  if (shape.kind === "note") return shape.h ?? 84;
  return 0;
}

function readActorId(): string {
  if (typeof window === "undefined") return "cortex-web";
  try {
    const stored = window.localStorage.getItem("cortex.shell.actor.id");
    if (stored?.trim()) return stored.trim();
  } catch {
    // ignore storage failures
  }
  return "cortex-web";
}

function isDraggableShape(shape: SpatialShape): boolean {
  return shape.kind === "note" || shape.kind === "node" || shape.kind === "group" || shape.kind === "annotation";
}

function getNodePortPoint(
  node: SpatialNodeShape,
  portId: string | undefined,
  preference: "source" | "target"
): CanvasPoint {
  const width = shapeWidth(node);
  const height = shapeHeight(node);
  const ports = node.ports ?? [];
  const preferredPort =
    ports.find((port) => port.id === portId)
    ?? ports.find((port) => {
      if (preference === "source") return port.direction === "out" || port.direction === "inout";
      return port.direction === "in" || port.direction === "inout";
    })
    ?? ports[0];

  if (!preferredPort) {
    return {
      x: node.x + width / 2,
      y: node.y + height / 2,
    };
  }

  const sidePorts = ports.filter((port) => port.side === preferredPort.side);
  const index = Math.max(0, sidePorts.findIndex((port) => port.id === preferredPort.id));
  const count = Math.max(1, sidePorts.length);

  if (preferredPort.side === "left" || preferredPort.side === "right") {
    const step = height / (count + 1);
    return {
      x: preferredPort.side === "left" ? node.x : node.x + width,
      y: node.y + step * (index + 1),
    };
  }

  const step = width / (count + 1);
  return {
    x: node.x + step * (index + 1),
    y: preferredPort.side === "top" ? node.y : node.y + height,
  };
}

function edgeIdForConnection(
  fromShapeId: string,
  fromPortId: string | undefined,
  toShapeId: string,
  toPortId: string | undefined
): string {
  return `edge:${fromShapeId}:${fromPortId ?? "none"}:${toShapeId}:${toPortId ?? "none"}`;
}

function resolveEdgeEndpoints(
  start: PendingEdgeStart,
  next: PendingEdgeStart
):
  | {
      fromShapeId: string;
      fromPortId: string;
      toShapeId: string;
      toPortId: string;
    }
  | null {
  const startCanSource = start.direction === "out" || start.direction === "inout";
  const startCanTarget = start.direction === "in" || start.direction === "inout";
  const nextCanSource = next.direction === "out" || next.direction === "inout";
  const nextCanTarget = next.direction === "in" || next.direction === "inout";

  if (start.shapeId === next.shapeId && start.portId === next.portId) {
    return null;
  }
  if (startCanSource && nextCanTarget) {
    return {
      fromShapeId: start.shapeId,
      fromPortId: start.portId,
      toShapeId: next.shapeId,
      toPortId: next.portId,
    };
  }
  if (nextCanSource && startCanTarget) {
    return {
      fromShapeId: next.shapeId,
      fromPortId: next.portId,
      toShapeId: start.shapeId,
      toPortId: start.portId,
    };
  }
  return null;
}

function isWorkflowBacked(payload: SpatialPlanePayload): boolean {
  return Boolean(payload.layout_ref?.workflow_id?.trim() || payload.layout_ref?.graph_hash?.trim());
}

function currentViewState(state: SpatialReplayState, payload: SpatialPlanePayload) {
  return {
    zoom: finiteNumber(state.viewState?.zoom) ? state.viewState?.zoom : (finiteNumber(payload.view_state?.zoom) ? payload.view_state?.zoom : 1),
    pan_x: finiteNumber(state.viewState?.pan_x) ? state.viewState?.pan_x : (finiteNumber(payload.view_state?.pan_x) ? payload.view_state?.pan_x : 0),
    pan_y: finiteNumber(state.viewState?.pan_y) ? state.viewState?.pan_y : (finiteNumber(payload.view_state?.pan_y) ? payload.view_state?.pan_y : 0),
  };
}

function viewBoxForState(focusBounds: SpatialBounds, viewState: { zoom: number; pan_x: number; pan_y: number }) {
  const zoom = viewState.zoom > 0 ? viewState.zoom : 1;
  const width = focusBounds.w / zoom;
  const height = focusBounds.h / zoom;
  const centerX = focusBounds.x + focusBounds.w / 2 + viewState.pan_x;
  const centerY = focusBounds.y + focusBounds.h / 2 + viewState.pan_y;
  return {
    x: centerX - width / 2,
    y: centerY - height / 2,
    w: width,
    h: height,
  };
}

function canMoveShape(shape: SpatialShape | undefined): boolean {
  return Boolean(shape && isDraggableShape(shape));
}

function buildCollapsedMemberSet(shapes: SpatialShape[]): Set<string> {
  const hidden = new Set<string>();
  for (const shape of shapes) {
    if (shape.kind === "group" && shape.collapsed) {
      for (const memberId of shape.member_ids) {
        hidden.add(memberId);
      }
    }
  }
  return hidden;
}

function getEdgeGeometry(edge: SpatialEdgeShape, shapes: Map<string, SpatialShape>): {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
} {
  const fromShape = shapes.get(edge.from_shape_id);
  const toShape = shapes.get(edge.to_shape_id);

  if (fromShape?.kind === "node" && toShape?.kind === "node") {
    const start = getNodePortPoint(fromShape, edge.from_port_id, "source");
    const end = getNodePortPoint(toShape, edge.to_port_id, "target");
    return { x1: start.x, y1: start.y, x2: end.x, y2: end.y };
  }

  return {
    x1: edge.x,
    y1: edge.y,
    x2: edge.x + 180,
    y2: edge.y,
  };
}

function buildLayoutPayload(
  payload: SpatialPlanePayload,
  replayState: SpatialReplayState,
  revision: number,
  actorId: string,
  spaceId: string,
  viewSpecId: string
): SpatialPlaneLayoutV1 {
  const shapePositions: Record<string, { x: number; y: number }> = {};
  const collapsedGroups: Record<string, boolean> = {};

  for (const shape of replayState.shapes.values()) {
    if (shape.kind !== "edge" && shape.kind !== "arrow") {
      shapePositions[shape.id] = { x: shape.x, y: shape.y };
    }
    if (shape.kind === "group") {
      collapsedGroups[shape.id] = Boolean(shape.collapsed);
    }
  }

  return {
    schema_version: "1.0.0",
    plane_id: payload.plane_id ?? "unnamed",
    view_spec_id: viewSpecId,
    space_id: spaceId,
    revision,
    layout: {
      shape_positions: shapePositions,
      collapsed_groups: collapsedGroups,
      view_state: replayState.viewState,
      selected_shape_ids: replayState.selection,
    },
    lineage: {
      view_spec_id: viewSpecId,
      workflow_id: payload.layout_ref?.workflow_id,
      graph_hash: payload.layout_ref?.graph_hash,
      space_id: spaceId,
      updated_by: actorId,
      updated_at: new Date().toISOString(),
    },
  };
}

function replayIntoTldrawEditor(editor: Record<string, any>, shapes: SpatialShape[], focusBounds: SpatialBounds) {
  const { mapped, errors } = mapShapesToTldraw(shapes);
  if (errors.length > 0) {
    throw new Error(
      `contract_invalid:${errors
        .map((error) => `${error.shapeId}:${error.reason}`)
        .join(";")}`
    );
  }
  const listShapes = editor.getCurrentPageShapes;
  const deleteShapes = editor.deleteShapes;
  if (typeof listShapes === "function" && typeof deleteShapes === "function") {
    const current = listShapes.call(editor);
    const currentIds = Array.isArray(current)
      ? current
          .map((shape: Record<string, unknown>) => shape.id)
          .filter((id: unknown) => typeof id === "string")
      : [];
    if (currentIds.length > 0) {
      deleteShapes.call(editor, currentIds);
    }
  }

  if (mapped.length > 0) {
    const createShapes = editor.createShapes;
    if (typeof createShapes === "function") {
      createShapes.call(editor, mapped);
    } else {
      const createShape = editor.createShape;
      if (typeof createShape === "function") {
        for (const shape of mapped) createShape.call(editor, shape);
      }
    }
  }

  const zoomToBounds = editor.zoomToBounds;
  if (typeof zoomToBounds === "function") {
    zoomToBounds.call(editor, focusBounds, { inset: 32, animation: { duration: 0 } });
  }
}

export function TldrawCanvas({ payload }: { payload: SpatialPlanePayload }) {
  const spatialEnabled = readFeatureFlag("VITE_A2UI_SPATIAL_PLANE");
  const tldrawAdapterEnabled = readFeatureFlag("VITE_A2UI_TLDRAW_EXPERIMENT");
  const sessionUser = useUiStore((state) => state.sessionUser);
  const activeSpaceIds = useUiStore((state) => state.activeSpaceIds);
  const [tldrawModule, setTldrawModule] = useState<Record<string, any> | null>(null);
  const [adapterError, setAdapterError] = useState<string | null>(null);
  const [rendererMode, setRendererMode] = useState<"svg" | "runtime">("svg");
  const [interactiveReplay, setInteractiveReplay] = useState<SpatialReplayState>(() => replaySpatialPayload(payload));
  const [layoutStatus, setLayoutStatus] = useState<"idle" | "loading" | "loaded" | "empty" | "saving" | "saved" | "error">("idle");
  const [layoutWarnings, setLayoutWarnings] = useState<string[]>([]);
  const [layoutError, setLayoutError] = useState<string | null>(null);
  const [layoutRevision, setLayoutRevision] = useState<number>(0);
  const [isDirty, setIsDirty] = useState(false);
  const [pendingEdgeStart, setPendingEdgeStart] = useState<PendingEdgeStart | null>(null);
  const [pointerPreview, setPointerPreview] = useState<CanvasPoint | null>(null);
  const surfaceClass = (payload.surface_class ?? "execution").toLowerCase();
  const focusBounds = payload.focus_bounds ?? { x: 0, y: 0, w: 1100, h: 640 };
  const payloadSignature = useMemo(
    () =>
      JSON.stringify({
        planeId: payload.plane_id ?? "unnamed",
        surfaceClass,
        commands: payload.commands ?? [],
        focusBounds,
        viewState: payload.view_state ?? null,
        selection: payload.selection ?? [],
        layoutRef: payload.layout_ref ?? null,
      }),
    [focusBounds, payload.commands, payload.layout_ref, payload.plane_id, payload.selection, payload.view_state, surfaceClass]
  );
  const validation = useMemo(() => validateSpatialPayload(payload), [payloadSignature]);
  const baseReplay = useMemo(() => replaySpatialPayload(payload), [payloadSignature]);
  const layoutSpaceId = payload.layout_ref?.space_id?.trim() || activeSpaceIds[0]?.trim() || null;
  const layoutViewSpecId = payload.layout_ref?.view_spec_id?.trim() || null;
  const layoutScopeReady = Boolean(layoutSpaceId && layoutViewSpecId);
  const workflowBacked = isWorkflowBacked(payload);
  const shapeList = useMemo(() => Array.from(interactiveReplay.shapes.values()), [interactiveReplay.shapes]);
  const collapsedMemberIds = useMemo(() => buildCollapsedMemberSet(shapeList), [shapeList]);
  const visibleShapes = useMemo(
    () =>
      shapeList.filter((shape) => {
        if (shape.kind === "edge") {
          return !collapsedMemberIds.has(shape.from_shape_id) && !collapsedMemberIds.has(shape.to_shape_id);
        }
        return !collapsedMemberIds.has(shape.id);
      }),
    [collapsedMemberIds, shapeList]
  );
  const lastReplaySignatureRef = useRef<string | null>(null);
  const editorRef = useRef<Record<string, any> | null>(null);
  const dragStateRef = useRef<DragState | null>(null);
  const svgRef = useRef<SVGSVGElement | null>(null);

  const adapterSignature = useMemo(
    () =>
      JSON.stringify({
        shapes: shapeList,
        focusBounds,
        selection: interactiveReplay.selection,
        viewState: interactiveReplay.viewState ?? null,
      }),
    [focusBounds, interactiveReplay.selection, interactiveReplay.viewState, shapeList]
  );
  const effectiveViewState = useMemo(
    () => currentViewState(interactiveReplay, payload),
    [interactiveReplay, payload]
  );
  const svgViewBox = useMemo(
    () => viewBoxForState(focusBounds, effectiveViewState),
    [effectiveViewState, focusBounds]
  );
  const selectedShapes = useMemo(
    () => interactiveReplay.selection.map((shapeId) => interactiveReplay.shapes.get(shapeId)).filter(Boolean) as SpatialShape[],
    [interactiveReplay.selection, interactiveReplay.shapes]
  );
  const canDeleteSelection = useMemo(() => {
    if (selectedShapes.length === 0) return false;
    return !workflowBacked;
  }, [selectedShapes, workflowBacked]);

  const toSvgPoint = useCallback((event: React.PointerEvent<SVGSVGElement | SVGElement>): CanvasPoint | null => {
    const svg = svgRef.current;
    if (!svg) return null;
    const bounds = svg.getBoundingClientRect();
    if (bounds.width <= 0 || bounds.height <= 0) return null;
    const viewBox = svg.viewBox.baseVal;
    return {
      x: viewBox.x + ((event.clientX - bounds.left) / bounds.width) * viewBox.width,
      y: viewBox.y + ((event.clientY - bounds.top) / bounds.height) * viewBox.height,
    };
  }, []);

  const applyReplay = useCallback(
    (editor: Record<string, any>) => {
      if (lastReplaySignatureRef.current === adapterSignature) {
        return;
      }
      try {
        replayIntoTldrawEditor(editor, shapeList, svgViewBox);
        lastReplaySignatureRef.current = adapterSignature;
        emitA2uiEvent("spatial_adapter_replay", {
          adapter: "tldraw",
          shapes: shapeList.length,
          replaySignature: adapterSignature,
        });
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        const reasonClass = classifyReplayError(message);
        emitA2uiEvent("spatial_adapter_replay_failed", {
          adapter: "tldraw",
          reasonClass,
          reason: message,
        });
      }
    },
    [adapterSignature, shapeList, svgViewBox]
  );

  const updateViewState = useCallback((patch: Partial<{ zoom: number; pan_x: number; pan_y: number }>) => {
    setInteractiveReplay((current) => ({
      ...current,
      viewState: {
        ...currentViewState(current, payload),
        ...patch,
      },
    }));
    setIsDirty(true);
  }, [payload]);

  const handleDeleteSelection = useCallback(() => {
    if (interactiveReplay.selection.length === 0) {
      return;
    }
    if (!canDeleteSelection) {
      setLayoutError("Workflow-backed execution canvases keep topology read-only in this milestone.");
      return;
    }
    const selectedIds = new Set(interactiveReplay.selection);
    setInteractiveReplay((current) => {
      const shapes = new Map(current.shapes);
      for (const shapeId of selectedIds) {
        shapes.delete(shapeId);
      }
      for (const [shapeId, shape] of current.shapes.entries()) {
        if (shape.kind === "edge") {
          if (selectedIds.has(shapeId) || selectedIds.has(shape.from_shape_id) || selectedIds.has(shape.to_shape_id)) {
            shapes.delete(shapeId);
          }
        }
        if (shape.kind === "group" && selectedIds.has(shapeId)) {
          for (const memberId of shape.member_ids) {
            shapes.delete(memberId);
          }
        }
      }
      return {
        ...current,
        shapes,
        selection: [],
      };
    });
    setPendingEdgeStart(null);
    setPointerPreview(null);
    setLayoutError(null);
    setIsDirty(true);
    emitA2uiEvent("button_click", {
      action: "delete_selection",
      selectedShapeIds: interactiveReplay.selection,
      workflowBacked,
    });
  }, [canDeleteSelection, interactiveReplay.selection, workflowBacked]);

  useEffect(() => {
    if (!tldrawAdapterEnabled) {
      setTldrawModule(null);
      setAdapterError(null);
      setRendererMode("svg");
      return;
    }

    let cancelled = false;
    const moduleName = "tldraw";
    import(/* @vite-ignore */ moduleName)
      .then((mod) => {
        if (cancelled) return;
        setTldrawModule(mod as Record<string, any>);
        setAdapterError(null);
        emitA2uiEvent("spatial_adapter_loaded", { adapter: "tldraw", mode: "runtime" });
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        const message = err instanceof Error ? err.message : String(err);
        setAdapterError(message);
        setTldrawModule(null);
        setRendererMode("svg");
        emitA2uiEvent("spatial_adapter_fallback", {
          adapter: "tldraw",
          reasonClass: "adapter_unavailable",
          reason: message,
        });
      });

    return () => {
      cancelled = true;
    };
  }, [tldrawAdapterEnabled]);

  useEffect(() => {
    if (rendererMode !== "runtime" || !tldrawModule?.Tldraw) {
      editorRef.current = null;
      return;
    }
    if (editorRef.current) {
      applyReplay(editorRef.current);
    }
  }, [applyReplay, rendererMode, tldrawModule]);

  useEffect(() => {
    setPendingEdgeStart(null);
    setPointerPreview(null);

    if (validation.errors.length > 0 || surfaceClass !== "execution") {
      setInteractiveReplay(baseReplay);
      setLayoutStatus("idle");
      setLayoutWarnings([]);
      setLayoutError(null);
      setLayoutRevision(0);
      setIsDirty(false);
      return;
    }

    if (!layoutScopeReady || !layoutSpaceId || !layoutViewSpecId) {
      setInteractiveReplay(baseReplay);
      setLayoutStatus("empty");
      setLayoutWarnings([]);
      setLayoutError(null);
      setLayoutRevision(0);
      setIsDirty(false);
      return;
    }

    let cancelled = false;
    setLayoutStatus("loading");
    setLayoutWarnings([]);
    setLayoutError(null);
    setIsDirty(false);

    workbenchApi
      .getSpatialPlaneLayout(layoutSpaceId, layoutViewSpecId)
      .then((response) => {
        if (cancelled) return;
        const applied = applySpatialLayout(baseReplay, response.layout as SpatialPlaneLayoutV1);
        setInteractiveReplay(applied);
        setLayoutWarnings(applied.warnings);
        setLayoutRevision(response.layout.revision);
        setLayoutStatus("loaded");
      })
      .catch((err) => {
        if (cancelled) return;
        const message = err instanceof Error ? err.message : String(err);
        if (message.includes("404")) {
          setInteractiveReplay(baseReplay);
          setLayoutStatus("empty");
          setLayoutRevision(0);
          setLayoutError(null);
          return;
        }
        setInteractiveReplay(baseReplay);
        setLayoutStatus("error");
        setLayoutRevision(0);
        setLayoutError(message);
      });

    return () => {
      cancelled = true;
    };
  }, [baseReplay, layoutScopeReady, layoutSpaceId, layoutViewSpecId, surfaceClass, validation.errors.length]);

  const selectShape = useCallback((shapeId: string, shapeKind: string) => {
    setInteractiveReplay((current) => ({
      ...current,
      selection: [shapeId],
    }));
    emitA2uiEvent("spatial_shape_click", { shapeId, shapeKind });
  }, []);

  const handleShapePointerDown = useCallback(
    (event: React.PointerEvent<SVGElement>, shapeId: string, shapeKind: string) => {
      const shape = interactiveReplay.shapes.get(shapeId);
      if (!shape) return;
      event.stopPropagation();
      selectShape(shapeId, shapeKind);
      if (!canMoveShape(shape)) return;
      const point = toSvgPoint(event);
      if (!point) return;
      dragStateRef.current = {
        shapeId,
        pointerId: event.pointerId,
        startPoint: point,
        originX: shape.x,
        originY: shape.y,
        moved: false,
      };
      (event.currentTarget as SVGElement).setPointerCapture?.(event.pointerId);
    },
    [interactiveReplay.shapes, selectShape, toSvgPoint]
  );

  const handleCanvasPointerMove = useCallback(
    (event: React.PointerEvent<SVGSVGElement>) => {
      const point = toSvgPoint(event);
      if (pendingEdgeStart) {
        setPointerPreview(point);
      }

      const drag = dragStateRef.current;
      if (!drag || drag.pointerId !== event.pointerId || !point) {
        return;
      }

      const nextX = roundCoordinate(drag.originX + (point.x - drag.startPoint.x));
      const nextY = roundCoordinate(drag.originY + (point.y - drag.startPoint.y));
      drag.moved = drag.moved || nextX !== drag.originX || nextY !== drag.originY;

      setInteractiveReplay((current) => {
        const shape = current.shapes.get(drag.shapeId);
        if (!shape || !canMoveShape(shape)) return current;
        if (shape.x === nextX && shape.y === nextY) return current;
        const shapes = new Map(current.shapes);
        shapes.set(shape.id, {
          ...shape,
          x: nextX,
          y: nextY,
        } as SpatialShape);
        return {
          ...current,
          shapes,
          selection: [shape.id],
        };
      });
      setIsDirty(true);
    },
    [pendingEdgeStart, toSvgPoint]
  );

  const handleCanvasPointerUp = useCallback(
    (event: React.PointerEvent<SVGSVGElement>) => {
      const drag = dragStateRef.current;
      if (drag && drag.pointerId === event.pointerId) {
        const movedShape = interactiveReplay.shapes.get(drag.shapeId);
        if (drag.moved && movedShape) {
          emitA2uiEvent("spatial_shape_move", {
            shapeId: movedShape.id,
            shapeKind: movedShape.kind,
            x: movedShape.x,
            y: movedShape.y,
          });
        }
        dragStateRef.current = null;
      }
    },
    [interactiveReplay.shapes]
  );

  const handleCanvasPointerLeave = useCallback(() => {
    if (!pendingEdgeStart) {
      setPointerPreview(null);
    }
  }, [pendingEdgeStart]);

  const handleCanvasClick = useCallback(() => {
    setPendingEdgeStart(null);
    setPointerPreview(null);
    setInteractiveReplay((current) => ({ ...current, selection: [] }));
  }, []);

  const handlePortClick = useCallback(
    (event: React.PointerEvent<SVGCircleElement>, shapeId: string, port: SpatialPort) => {
      event.stopPropagation();
      if (workflowBacked) {
        setLayoutError("Workflow-backed execution canvases keep topology read-only in this milestone.");
        setPendingEdgeStart(null);
        setPointerPreview(null);
        return;
      }
      const nextEndpoint: PendingEdgeStart = {
        shapeId,
        portId: port.id,
        direction: port.direction,
      };

      if (!pendingEdgeStart) {
        setPendingEdgeStart(nextEndpoint);
        const point = toSvgPoint(event);
        setPointerPreview(point);
        return;
      }

      const endpoints = resolveEdgeEndpoints(pendingEdgeStart, nextEndpoint);
      if (!endpoints) {
        setPendingEdgeStart(nextEndpoint);
        const point = toSvgPoint(event);
        setPointerPreview(point);
        return;
      }

      const nextEdgeId = edgeIdForConnection(
        endpoints.fromShapeId,
        endpoints.fromPortId,
        endpoints.toShapeId,
        endpoints.toPortId
      );
      let action: "connect" | "disconnect" = "connect";

      setInteractiveReplay((current) => {
        const shapes = new Map(current.shapes);
        const existing = shapes.get(nextEdgeId);
        if (existing?.kind === "edge") {
          shapes.delete(nextEdgeId);
          action = "disconnect";
        } else {
          const fromShape = shapes.get(endpoints.fromShapeId);
          const start =
            fromShape?.kind === "node"
              ? getNodePortPoint(fromShape, endpoints.fromPortId, "source")
              : { x: 0, y: 0 };
          shapes.set(nextEdgeId, {
            id: nextEdgeId,
            kind: "edge",
            edge_class: "data",
            x: start.x,
            y: start.y,
            from_shape_id: endpoints.fromShapeId,
            to_shape_id: endpoints.toShapeId,
            from_port_id: endpoints.fromPortId,
            to_port_id: endpoints.toPortId,
            text: "Flow",
          });
        }
        return {
          ...current,
          shapes,
          selection: [nextEdgeId],
        };
      });

      setPendingEdgeStart(null);
      setPointerPreview(null);
      setIsDirty(true);
      emitA2uiEvent("spatial_edge_connect", {
        edgeId: nextEdgeId,
        action,
        fromShapeId: endpoints.fromShapeId,
        fromPortId: endpoints.fromPortId,
        toShapeId: endpoints.toShapeId,
        toPortId: endpoints.toPortId,
      });
    },
    [pendingEdgeStart, toSvgPoint, workflowBacked]
  );

  const handleToggleGroupCollapse = useCallback((event: React.MouseEvent<SVGGElement>, groupId: string) => {
    event.stopPropagation();
    setInteractiveReplay((current) => {
      const shape = current.shapes.get(groupId);
      if (!shape || shape.kind !== "group") return current;
      const shapes = new Map(current.shapes);
      shapes.set(groupId, {
        ...shape,
        collapsed: !shape.collapsed,
      });
      return {
        ...current,
        shapes,
        selection: [groupId],
      };
    });
    setIsDirty(true);
    setPendingEdgeStart(null);
    setPointerPreview(null);
  }, []);

  const handleSaveLayout = useCallback(async () => {
    if (!layoutScopeReady || !layoutSpaceId || !layoutViewSpecId) {
      setLayoutError("Spatial layout save requires layout_ref.space_id and layout_ref.view_spec_id.");
      return;
    }

    setLayoutStatus("saving");
    setLayoutError(null);
    try {
      const nextRevision = Math.max(1, layoutRevision + 1);
      const saved = await workbenchApi.saveSpatialPlaneLayout(
        layoutSpaceId,
        layoutViewSpecId,
        buildLayoutPayload(
          payload,
          interactiveReplay,
          nextRevision,
          sessionUser?.actorId?.trim() || readActorId(),
          layoutSpaceId,
          layoutViewSpecId
        )
      );
      const applied = applySpatialLayout(baseReplay, saved.layout as SpatialPlaneLayoutV1);
      setInteractiveReplay(applied);
      setLayoutWarnings(applied.warnings);
      setLayoutRevision(saved.layout.revision);
      setLayoutStatus("saved");
      setIsDirty(false);
    } catch (err) {
      setLayoutStatus("error");
      setLayoutError(err instanceof Error ? err.message : String(err));
    }
  }, [
    baseReplay,
    interactiveReplay,
    layoutRevision,
    layoutScopeReady,
    layoutSpaceId,
    layoutViewSpecId,
    payload,
    sessionUser?.actorId,
  ]);

  if (!spatialEnabled) {
    return (
      <div className="a2ui-spatial-plane a2ui-spatial-plane--disabled">
        SpatialPlane disabled. Set `VITE_A2UI_SPATIAL_PLANE=1` to enable this renderer.
      </div>
    );
  }

  if (surfaceClass !== "execution") {
    return (
      <div className="a2ui-spatial-plane a2ui-spatial-plane--blocked">
        SpatialPlane blocked: this execution canvas only renders `surface_class=execution`.
      </div>
    );
  }

  if (validation.errors.length > 0) {
    return (
      <div className="a2ui-spatial-plane a2ui-spatial-plane--blocked">
        <div>SpatialPlane blocked: contract validation failed.</div>
        <ul className="a2ui-spatial-plane__issue-list">
          {validation.errors.slice(0, 5).map((issue) => (
            <li key={`${issue.shapeId}:${issue.reason}`}>{issue.shapeId}: {issue.reason}</li>
          ))}
        </ul>
      </div>
    );
  }

  const TldrawComponent = tldrawModule?.Tldraw as React.ComponentType<Record<string, unknown>> | undefined;
  const runtimeAvailable = Boolean(TldrawComponent && tldrawAdapterEnabled);
  const rendererLabel =
    rendererMode === "runtime" && runtimeAvailable
      ? "tldraw-runtime"
      : runtimeAvailable
        ? "svg-fallback (runtime available)"
        : tldrawAdapterEnabled
          ? "svg-fallback (adapter unavailable)"
          : "svg-fallback";

  const groups = visibleShapes.filter((shape) => shape.kind === "group");
  const edges = visibleShapes.filter((shape) => shape.kind === "edge" || shape.kind === "arrow");
  const cards = visibleShapes.filter(
    (shape) => shape.kind === "note" || shape.kind === "node" || shape.kind === "annotation"
  );

  return (
    <div
      className="a2ui-spatial-plane"
      tabIndex={0}
      onKeyDown={(event) => {
        if ((event.key === "Delete" || event.key === "Backspace") && interactiveReplay.selection.length > 0) {
          event.preventDefault();
          handleDeleteSelection();
        }
      }}
    >
      <div className="a2ui-spatial-plane__head">
        <span>plane={payload.plane_id ?? "unnamed"}</span>
        <span>shapes={shapeList.length}</span>
        <span>selected={interactiveReplay.selection.length}</span>
        <span>renderer={rendererLabel}</span>
        <span>layout={isDirty ? "dirty" : layoutStatus}</span>
        <span>mode={workflowBacked ? "workflow-backed" : "labs-local"}</span>
        {runtimeAvailable && (
          <button
            type="button"
            className="a2ui-spatial-plane__toggle"
            onClick={() => setRendererMode((current) => (current === "svg" ? "runtime" : "svg"))}
          >
            {rendererMode === "svg" ? "Runtime Preview" : "SVG Authoring"}
          </button>
        )}
        <button
          type="button"
          className="a2ui-spatial-plane__toggle"
          onClick={() => updateViewState({ zoom: roundCoordinate(Math.min(2.5, effectiveViewState.zoom + 0.1)) })}
        >
          Zoom In
        </button>
        <button
          type="button"
          className="a2ui-spatial-plane__toggle"
          onClick={() => updateViewState({ zoom: roundCoordinate(Math.max(0.4, effectiveViewState.zoom - 0.1)) })}
        >
          Zoom Out
        </button>
        <button
          type="button"
          className="a2ui-spatial-plane__toggle"
          onClick={() => updateViewState({
            zoom: finiteNumber(payload.view_state?.zoom) ? payload.view_state.zoom : 1,
            pan_x: finiteNumber(payload.view_state?.pan_x) ? payload.view_state.pan_x : 0,
            pan_y: finiteNumber(payload.view_state?.pan_y) ? payload.view_state.pan_y : 0,
          })}
        >
          Reset View
        </button>
        <button
          type="button"
          className="a2ui-spatial-plane__toggle"
          onClick={() => handleDeleteSelection()}
          disabled={!canDeleteSelection}
        >
          Delete Selection
        </button>
        <button
          type="button"
          className="a2ui-spatial-plane__toggle"
          onClick={() => void handleSaveLayout()}
          disabled={!layoutScopeReady || layoutStatus === "saving" || !isDirty}
        >
          {layoutStatus === "saving" ? "Saving..." : "Save Layout"}
        </button>
      </div>
      {!layoutScopeReady && (
        <div className="a2ui-spatial-plane__adapter-note">
          Layout persistence is available once `layout_ref.space_id` and `layout_ref.view_spec_id` are provided.
        </div>
      )}
      {workflowBacked && (
        <div className="a2ui-spatial-plane__adapter-note">
          Workflow-backed mode keeps topology read-only. Move, select, collapse, and save layout are allowed; connect or delete topology is blocked.
        </div>
      )}
      {adapterError && (
        <div className="a2ui-spatial-plane__adapter-note">
          adapter fallback reason: {adapterError}
        </div>
      )}
      {layoutWarnings.length > 0 && (
        <div className="a2ui-spatial-plane__adapter-note">
          ignored stale layout ids: {layoutWarnings.join(", ")}
        </div>
      )}
      {layoutError && <div className="a2ui-spatial-plane__adapter-note">layout persistence error: {layoutError}</div>}
      {rendererMode === "runtime" && TldrawComponent ? (
        <div className="a2ui-spatial-plane__host">
          {React.createElement(TldrawComponent, {
            inferDarkMode: true,
            onMount: (editor: Record<string, any>) => {
              editorRef.current = editor;
              applyReplay(editor);
            },
          })}
        </div>
      ) : (
        <svg
          ref={svgRef}
          className="a2ui-spatial-plane__svg"
          role="img"
          aria-label="A2UI spatial plane preview"
          viewBox={`${svgViewBox.x} ${svgViewBox.y} ${svgViewBox.w} ${svgViewBox.h}`}
          onClick={handleCanvasClick}
          onPointerMove={handleCanvasPointerMove}
          onPointerUp={handleCanvasPointerUp}
          onPointerLeave={handleCanvasPointerLeave}
        >
          <defs>
            <marker id="a2ui-arrow-head" markerWidth="10" markerHeight="8" refX="9" refY="4" orient="auto">
              <path d="M0,0 L10,4 L0,8 z" />
            </marker>
          </defs>

          {groups.map((shape) => {
            const width = shapeWidth(shape);
            const height = shapeHeight(shape);
            const selected = interactiveReplay.selection.includes(shape.id);
            return (
              <g
                key={shape.id}
                className={selected ? "a2ui-spatial-plane__shape a2ui-spatial-plane__shape--selected" : "a2ui-spatial-plane__shape"}
                onPointerDown={(event) => handleShapePointerDown(event, shape.id, "group")}
              >
                <rect
                  className={shape.collapsed ? "a2ui-spatial-plane__group a2ui-spatial-plane__group--collapsed" : "a2ui-spatial-plane__group"}
                  x={shape.x}
                  y={shape.y}
                  width={width}
                  height={height}
                  rx={18}
                />
                <text className="a2ui-spatial-plane__label" x={shape.x + 16} y={shape.y + 24}>
                  {shape.label ?? shape.text ?? shape.id}
                </text>
                <text className="a2ui-spatial-plane__meta" x={shape.x + 16} y={shape.y + 42}>
                  {shape.collapsed ? `${shape.member_ids.length} hidden members` : "Execution group"}
                </text>
                <g onClick={(event) => handleToggleGroupCollapse(event, shape.id)}>
                  <rect
                    className="a2ui-spatial-plane__group-toggle"
                    x={shape.x + width - 88}
                    y={shape.y + 12}
                    width={72}
                    height={24}
                    rx={12}
                  />
                  <text className="a2ui-spatial-plane__group-toggle-text" x={shape.x + width - 52} y={shape.y + 28}>
                    {shape.collapsed ? "Expand" : "Collapse"}
                  </text>
                </g>
              </g>
            );
          })}

          {edges.map((shape) => {
            if (shape.kind === "arrow") {
              return (
                <line
                  key={shape.id}
                  className="a2ui-spatial-plane__arrow"
                  x1={shape.x}
                  y1={shape.y}
                  x2={shape.to_x ?? shape.x + 10}
                  y2={shape.to_y ?? shape.y + 10}
                  markerEnd="url(#a2ui-arrow-head)"
                  onClick={(event) => {
                    event.stopPropagation();
                    selectShape(shape.id, "arrow");
                  }}
                />
              );
            }

            const geometry = getEdgeGeometry(shape, interactiveReplay.shapes);
            const selected = interactiveReplay.selection.includes(shape.id);
            const labelX = geometry.x1 + (geometry.x2 - geometry.x1) / 2;
            const labelY = geometry.y1 + (geometry.y2 - geometry.y1) / 2 - 8;
            return (
              <g key={shape.id} onClick={(event) => {
                event.stopPropagation();
                selectShape(shape.id, "edge");
              }}>
                <line
                  className={[
                    "a2ui-spatial-plane__edge",
                    `a2ui-spatial-plane__edge--${shape.edge_class}`,
                    selected ? "a2ui-spatial-plane__edge--selected" : "",
                  ].filter(Boolean).join(" ")}
                  x1={geometry.x1}
                  y1={geometry.y1}
                  x2={geometry.x2}
                  y2={geometry.y2}
                  markerEnd="url(#a2ui-arrow-head)"
                />
                <text className="a2ui-spatial-plane__edge-label" x={labelX} y={labelY}>
                  {shape.text ?? shape.edge_class}
                </text>
              </g>
            );
          })}

          {cards.map((shape) => {
            const width = shapeWidth(shape);
            const height = shapeHeight(shape);
            const selected = interactiveReplay.selection.includes(shape.id);

            if (shape.kind === "node") {
              return (
                <g
                  key={shape.id}
                  className={selected ? "a2ui-spatial-plane__shape a2ui-spatial-plane__shape--selected" : "a2ui-spatial-plane__shape"}
                  onPointerDown={(event) => handleShapePointerDown(event, shape.id, "node")}
                >
                  <rect
                    className={`a2ui-spatial-plane__node a2ui-spatial-plane__node--${shape.node_class} a2ui-spatial-plane__node-status--${shape.status}`}
                    x={shape.x}
                    y={shape.y}
                    width={width}
                    height={height}
                    rx={16}
                  />
                  <text className="a2ui-spatial-plane__label" x={shape.x + 16} y={shape.y + 28}>
                    {shape.text ?? shape.id}
                  </text>
                  <text className="a2ui-spatial-plane__meta" x={shape.x + 16} y={shape.y + 48}>
                    {shape.node_class} · {shape.status}
                  </text>
                  {(shape.ports ?? []).map((port) => {
                    const point = getNodePortPoint(shape, port.id, port.direction === "in" ? "target" : "source");
                    const isPending =
                      pendingEdgeStart?.shapeId === shape.id && pendingEdgeStart?.portId === port.id;
                    return (
                      <g key={port.id}>
                        <circle
                          className={isPending ? "a2ui-spatial-plane__port a2ui-spatial-plane__port--pending" : "a2ui-spatial-plane__port"}
                          cx={point.x}
                          cy={point.y}
                          r={7}
                          onPointerDown={(event) => handlePortClick(event, shape.id, port)}
                        />
                        <text className="a2ui-spatial-plane__port-label" x={point.x} y={point.y - 10}>
                          {port.label ?? port.id}
                        </text>
                      </g>
                    );
                  })}
                </g>
              );
            }

            if (shape.kind === "annotation") {
              return (
                <g
                  key={shape.id}
                  className={selected ? "a2ui-spatial-plane__shape a2ui-spatial-plane__shape--selected" : "a2ui-spatial-plane__shape"}
                  onPointerDown={(event) => handleShapePointerDown(event, shape.id, "annotation")}
                >
                  <rect
                    className="a2ui-spatial-plane__annotation"
                    x={shape.x}
                    y={shape.y}
                    width={width}
                    height={height}
                    rx={16}
                  />
                  <text className="a2ui-spatial-plane__label" x={shape.x + 14} y={shape.y + 26}>
                    {shape.text ?? shape.id}
                  </text>
                  <text className="a2ui-spatial-plane__meta" x={shape.x + 14} y={shape.y + 46}>
                    annotation
                  </text>
                </g>
              );
            }

            return (
              <g
                key={shape.id}
                className={selected ? "a2ui-spatial-plane__shape a2ui-spatial-plane__shape--selected" : "a2ui-spatial-plane__shape"}
                onPointerDown={(event) => handleShapePointerDown(event, shape.id, "note")}
              >
                <rect
                  className="a2ui-spatial-plane__note"
                  x={shape.x}
                  y={shape.y}
                  width={width}
                  height={height}
                  rx={12}
                />
                <text className="a2ui-spatial-plane__label" x={shape.x + 12} y={shape.y + 24}>
                  {shape.text ?? shape.id}
                </text>
              </g>
            );
          })}

          {pendingEdgeStart && pointerPreview && (() => {
            const startShape = interactiveReplay.shapes.get(pendingEdgeStart.shapeId);
            if (startShape?.kind !== "node") return null;
            const start = getNodePortPoint(startShape, pendingEdgeStart.portId, "source");
            return (
              <line
                className="a2ui-spatial-plane__edge a2ui-spatial-plane__edge--preview"
                x1={start.x}
                y1={start.y}
                x2={pointerPreview.x}
                y2={pointerPreview.y}
              />
            );
          })()}
        </svg>
      )}
    </div>
  );
}
