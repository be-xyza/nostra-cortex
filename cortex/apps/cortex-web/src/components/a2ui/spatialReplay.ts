export type SurfaceClass = "execution" | "constitutional";

export type SpatialNodeClass = "input" | "tool" | "procedure" | "output";
export type SpatialNodeStatus = "idle" | "running" | "blocked" | "done" | "error";
export type SpatialEdgeClass = "data" | "control" | "branch";
export type SpatialPortSide = "left" | "right" | "top" | "bottom";
export type SpatialPortDirection = "in" | "out" | "inout";

export type SpatialPort = {
  id: string;
  side: SpatialPortSide;
  direction: SpatialPortDirection;
  label?: string;
};

export type SpatialBounds = {
  x: number;
  y: number;
  w: number;
  h: number;
};

export type SpatialViewState = {
  zoom?: number;
  pan_x?: number;
  pan_y?: number;
};

export type SpatialBaseShape = {
  id: string;
  x: number;
  y: number;
  text?: string;
};

export type SpatialNoteShape = SpatialBaseShape & {
  kind: "note";
  w?: number;
  h?: number;
};

export type SpatialArrowShape = SpatialBaseShape & {
  kind: "arrow";
  to_x?: number;
  to_y?: number;
};

export type SpatialNodeShape = SpatialBaseShape & {
  kind: "node";
  w?: number;
  h?: number;
  node_class: SpatialNodeClass;
  status: SpatialNodeStatus;
  ports?: SpatialPort[];
};

export type SpatialEdgeShape = SpatialBaseShape & {
  kind: "edge";
  edge_class: SpatialEdgeClass;
  from_shape_id: string;
  to_shape_id: string;
  from_port_id?: string;
  to_port_id?: string;
};

export type SpatialGroupShape = SpatialBaseShape & {
  kind: "group";
  w?: number;
  h?: number;
  label?: string;
  member_ids: string[];
  collapsed?: boolean;
};

export type SpatialAnnotationShape = SpatialBaseShape & {
  kind: "annotation";
  w?: number;
  h?: number;
};

export type SpatialShape =
  | SpatialNoteShape
  | SpatialArrowShape
  | SpatialNodeShape
  | SpatialEdgeShape
  | SpatialGroupShape
  | SpatialAnnotationShape;

export type SpatialCommand = {
  op:
    | "create_shape"
    | "update_shape"
    | "delete_shape"
    | "focus_bounds"
    | "set_selection"
    | "set_view_state";
  shape?: SpatialShape;
  shape_id?: string;
  shape_ids?: string[];
  focus_bounds?: SpatialBounds;
  view_state?: SpatialViewState;
};

export type SpatialPlanePayload = {
  plane_id?: string;
  surface_class?: SurfaceClass | string;
  commands?: SpatialCommand[];
  focus_bounds?: SpatialBounds;
  view_state?: SpatialViewState;
  selection?: string[];
  layout_ref?: {
    view_spec_id?: string;
    workflow_id?: string;
    graph_hash?: string;
    space_id?: string;
  };
};

export type SpatialReplayState = {
  planeId?: string;
  surfaceClass: string;
  shapes: Map<string, SpatialShape>;
  focusBounds?: SpatialBounds;
  selection: string[];
  viewState?: SpatialViewState;
};

export type SpatialValidationIssue = {
  shapeId: string;
  reason: string;
};

export type SpatialPlaneLayoutV1 = {
  schema_version: string;
  plane_id: string;
  view_spec_id: string;
  space_id: string;
  revision: number;
  layout: {
    shape_positions: Record<string, { x: number; y: number }>;
    collapsed_groups: Record<string, boolean>;
    view_state?: SpatialViewState;
    selected_shape_ids?: string[];
  };
  lineage: {
    view_spec_id?: string;
    workflow_id?: string;
    graph_hash?: string;
    space_id?: string;
    updated_by: string;
    updated_at: string;
  };
};

function isMutatingOp(op: SpatialCommand["op"]): boolean {
  return op === "create_shape" || op === "update_shape" || op === "delete_shape";
}

function finiteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function validateShape(shape: SpatialShape): string[] {
  const issues: string[] = [];
  if (!shape.id?.trim()) {
    issues.push("shape.id is required");
  }
  if (!finiteNumber(shape.x) || !finiteNumber(shape.y)) {
    issues.push("shape.x and shape.y must be finite numbers");
  }

  if (shape.kind === "note" || shape.kind === "node" || shape.kind === "group" || shape.kind === "annotation") {
    if (shape.w !== undefined && (!finiteNumber(shape.w) || shape.w <= 0)) {
      issues.push("shape.w must be > 0 when provided");
    }
    if (shape.h !== undefined && (!finiteNumber(shape.h) || shape.h <= 0)) {
      issues.push("shape.h must be > 0 when provided");
    }
  }

  if (shape.kind === "arrow") {
    if (!finiteNumber(shape.to_x) || !finiteNumber(shape.to_y)) {
      issues.push("arrow requires to_x and to_y coordinates");
    }
  }

  if (shape.kind === "node") {
    const seenPorts = new Set<string>();
    for (const port of shape.ports ?? []) {
      if (!port.id?.trim()) {
        issues.push("node port id is required");
        continue;
      }
      if (seenPorts.has(port.id)) {
        issues.push(`duplicate port id '${port.id}'`);
      }
      seenPorts.add(port.id);
    }
  }

  if (shape.kind === "group") {
    if (!Array.isArray(shape.member_ids)) {
      issues.push("group.member_ids is required");
    }
  }

  if (shape.kind === "edge") {
    if (!shape.from_shape_id?.trim() || !shape.to_shape_id?.trim()) {
      issues.push("edge requires from_shape_id and to_shape_id");
    }
  }

  return issues;
}

export function replaySpatialPayload(payload: SpatialPlanePayload): SpatialReplayState {
  const shapes = new Map<string, SpatialShape>();
  let focusBounds = payload.focus_bounds;
  let selection = payload.selection ?? [];
  let viewState = payload.view_state;

  for (const command of payload.commands ?? []) {
    if (command.op === "create_shape" && command.shape) {
      shapes.set(command.shape.id, command.shape);
      continue;
    }
    if (command.op === "update_shape" && command.shape) {
      const current = shapes.get(command.shape.id);
      if (!current) continue;
      shapes.set(command.shape.id, { ...current, ...command.shape });
      continue;
    }
    if (command.op === "delete_shape" && command.shape_id) {
      shapes.delete(command.shape_id);
      continue;
    }
    if (command.op === "focus_bounds" && command.focus_bounds) {
      focusBounds = command.focus_bounds;
      continue;
    }
    if (command.op === "set_selection" && command.shape_ids) {
      selection = [...command.shape_ids];
      continue;
    }
    if (command.op === "set_view_state" && command.view_state) {
      viewState = { ...(viewState ?? {}), ...command.view_state };
    }
  }

  return {
    planeId: payload.plane_id,
    surfaceClass: String(payload.surface_class ?? "execution"),
    shapes,
    focusBounds,
    selection,
    viewState,
  };
}

export function replaySpatialCommands(commands: SpatialCommand[]): Map<string, SpatialShape> {
  return replaySpatialPayload({ commands }).shapes;
}

export function validateSpatialPayload(payload: SpatialPlanePayload): { errors: SpatialValidationIssue[] } {
  const errors: SpatialValidationIssue[] = [];
  const surfaceClass = String(payload.surface_class ?? "execution").toLowerCase();
  const commands = payload.commands ?? [];
  if (surfaceClass !== "execution" && commands.some((command) => isMutatingOp(command.op))) {
    errors.push({
      shapeId: payload.plane_id ?? "plane",
      reason: "mutating SpatialPlane commands require surface_class=execution",
    });
  }

  const replayed = replaySpatialPayload(payload);
  for (const shape of replayed.shapes.values()) {
    for (const issue of validateShape(shape)) {
      errors.push({ shapeId: shape.id, reason: issue });
    }
  }

  for (const shape of replayed.shapes.values()) {
    if (shape.kind === "edge") {
      const fromShape = replayed.shapes.get(shape.from_shape_id);
      const toShape = replayed.shapes.get(shape.to_shape_id);
      if (!fromShape || fromShape.kind !== "node") {
        errors.push({
          shapeId: shape.id,
          reason: `edge has unknown node reference '${shape.from_shape_id}'`,
        });
      }
      if (!toShape || toShape.kind !== "node") {
        errors.push({
          shapeId: shape.id,
          reason: `edge has unknown node reference '${shape.to_shape_id}'`,
        });
      }
    }
    if (shape.kind === "group") {
      for (const memberId of shape.member_ids) {
        if (!replayed.shapes.has(memberId)) {
          errors.push({
            shapeId: shape.id,
            reason: `group has unknown member '${memberId}'`,
          });
        }
      }
    }
  }

  return { errors };
}

export function applySpatialLayout(
  state: SpatialReplayState,
  layout: SpatialPlaneLayoutV1
): SpatialReplayState & { warnings: string[] } {
  const shapes = new Map<string, SpatialShape>();
  const warnings: string[] = [];

  for (const [shapeId, shape] of state.shapes.entries()) {
    let nextShape: SpatialShape = { ...shape } as SpatialShape;
    const position = layout.layout.shape_positions[shapeId];
    if (position) {
      nextShape = { ...nextShape, x: position.x, y: position.y } as SpatialShape;
    }
    if (nextShape.kind === "group" && shapeId in layout.layout.collapsed_groups) {
      nextShape = {
        ...nextShape,
        collapsed: Boolean(layout.layout.collapsed_groups[shapeId]),
      };
    }
    shapes.set(shapeId, nextShape);
  }

  for (const shapeId of Object.keys(layout.layout.shape_positions)) {
    if (!state.shapes.has(shapeId)) {
      warnings.push(shapeId);
    }
  }
  for (const shapeId of Object.keys(layout.layout.collapsed_groups)) {
    if (!state.shapes.has(shapeId)) {
      warnings.push(shapeId);
    }
  }

  const selectedShapeIds = (layout.layout.selected_shape_ids ?? []).filter((shapeId) => {
    if (!state.shapes.has(shapeId)) {
      warnings.push(shapeId);
      return false;
    }
    return true;
  });

  return {
    ...state,
    shapes,
    selection: selectedShapeIds,
    viewState: layout.layout.view_state ?? state.viewState,
    warnings,
  };
}
