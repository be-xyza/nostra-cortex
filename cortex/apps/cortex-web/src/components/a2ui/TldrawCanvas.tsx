import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { mapShapesToTldraw } from "./spatialMapper";
import { emitA2uiEvent } from "./spatialEventContract";
import { SpatialBounds, SpatialPlanePayload, SpatialShape, replaySpatialCommands } from "./spatialReplay";

function readFeatureFlag(name: string): boolean {
  return String(import.meta.env[name] ?? "").toLowerCase() === "1";
}

function classifyReplayError(message: string): "contract_invalid" | "adapter_replay_failed" {
  return message.toLowerCase().includes("contract_invalid")
    ? "contract_invalid"
    : "adapter_replay_failed";
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
  const [tldrawModule, setTldrawModule] = useState<Record<string, any> | null>(null);
  const [adapterError, setAdapterError] = useState<string | null>(null);
  const surfaceClass = (payload.surface_class ?? "execution").toLowerCase();
  const commands = payload.commands ?? [];
  const focusBounds = payload.focus_bounds ?? { x: 0, y: 0, w: 1100, h: 640 };
  const shapes = useMemo(() => replaySpatialCommands(commands), [commands]);
  const replaySignature = useMemo(
    () =>
      JSON.stringify({
        commands,
        focusBounds,
        planeId: payload.plane_id ?? "unnamed"
      }),
    [commands, focusBounds, payload.plane_id]
  );
  const shapeList = useMemo(() => Array.from(shapes.values()), [shapes]);
  const lastReplaySignatureRef = useRef<string | null>(null);
  const editorRef = useRef<Record<string, any> | null>(null);

  const applyReplay = useCallback(
    (editor: Record<string, any>) => {
      if (lastReplaySignatureRef.current === replaySignature) {
        return;
      }
      try {
        replayIntoTldrawEditor(editor, shapeList, focusBounds);
        lastReplaySignatureRef.current = replaySignature;
        emitA2uiEvent("spatial_adapter_replay", {
          adapter: "tldraw",
          shapes: shapeList.length,
          replaySignature
        });
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        const reasonClass = classifyReplayError(message);
        emitA2uiEvent("spatial_adapter_replay_failed", {
          adapter: "tldraw",
          reasonClass,
          reason: message
        });
      }
    },
    [focusBounds, replaySignature, shapeList]
  );

  useEffect(() => {
    if (!tldrawAdapterEnabled) {
      setTldrawModule(null);
      setAdapterError(null);
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
        emitA2uiEvent("spatial_adapter_fallback", {
          adapter: "tldraw",
          reasonClass: "adapter_unavailable",
          reason: message
        });
      });

    return () => {
      cancelled = true;
    };
  }, [tldrawAdapterEnabled]);

  useEffect(() => {
    if (!tldrawAdapterEnabled || !tldrawModule?.Tldraw) {
      editorRef.current = null;
      return;
    }
    if (editorRef.current) {
      applyReplay(editorRef.current);
    }
  }, [applyReplay, tldrawAdapterEnabled, tldrawModule]);

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
        SpatialPlane blocked: only `execution` surface class may render mutating canvas commands.
      </div>
    );
  }

  const rendererMode = tldrawAdapterEnabled
    ? tldrawModule?.Tldraw
      ? "tldraw-runtime"
      : "svg-fallback (adapter unavailable)"
    : "svg-fallback";

  const TldrawComponent = tldrawModule?.Tldraw as React.ComponentType<Record<string, unknown>> | undefined;
  return (
    <div className="a2ui-spatial-plane">
      <div className="a2ui-spatial-plane__head">
        <span>plane={payload.plane_id ?? "unnamed"}</span>
        <span>commands={commands.length}</span>
        <span>renderer={rendererMode}</span>
      </div>
      {adapterError && <div className="a2ui-spatial-plane__adapter-note">adapter fallback reason: {adapterError}</div>}
      {TldrawComponent ? (
        <div className="a2ui-spatial-plane__host">
          {React.createElement(TldrawComponent, {
            inferDarkMode: true,
            onMount: (editor: Record<string, any>) => {
              editorRef.current = editor;
              applyReplay(editor);
            }
          })}
        </div>
      ) : (
        <svg
          className="a2ui-spatial-plane__svg"
          role="img"
          aria-label="A2UI spatial plane preview"
          viewBox={`${focusBounds.x} ${focusBounds.y} ${focusBounds.w} ${focusBounds.h}`}
        >
          <defs>
            <marker id="a2ui-arrow-head" markerWidth="10" markerHeight="8" refX="9" refY="4" orient="auto">
              <path d="M0,0 L10,4 L0,8 z" />
            </marker>
          </defs>
          {shapeList.map((shape) => {
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
                  onClick={() =>
                    emitA2uiEvent("spatial_shape_click", { shapeId: shape.id, shapeKind: "arrow" })
                  }
                />
              );
            }

            const width = shape.w ?? 170;
            const height = shape.h ?? 84;
            return (
              <g
                key={shape.id}
                onClick={() =>
                  emitA2uiEvent("spatial_shape_click", { shapeId: shape.id, shapeKind: "note" })
                }
              >
                <rect
                  className="a2ui-spatial-plane__note"
                  x={shape.x}
                  y={shape.y}
                  width={width}
                  height={height}
                  rx={12}
                />
                <text className="a2ui-spatial-plane__text" x={shape.x + 12} y={shape.y + 22}>
                  {shape.text ?? shape.id}
                </text>
              </g>
            );
          })}
        </svg>
      )}
    </div>
  );
}
