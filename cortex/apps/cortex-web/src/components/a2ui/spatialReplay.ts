export type SurfaceClass = "execution" | "constitutional";

export type SpatialBounds = {
  x: number;
  y: number;
  w: number;
  h: number;
};

export type SpatialShape = {
  id: string;
  kind: "note" | "arrow";
  x: number;
  y: number;
  w?: number;
  h?: number;
  to_x?: number;
  to_y?: number;
  text?: string;
};

export type SpatialCommand = {
  op: "create_shape" | "update_shape" | "delete_shape" | "focus_bounds";
  shape?: SpatialShape;
  shape_id?: string;
  focus_bounds?: SpatialBounds;
};

export type SpatialPlanePayload = {
  plane_id?: string;
  surface_class?: SurfaceClass | string;
  commands?: SpatialCommand[];
  focus_bounds?: SpatialBounds;
};

export function replaySpatialCommands(commands: SpatialCommand[]): Map<string, SpatialShape> {
  const shapes = new Map<string, SpatialShape>();

  for (const command of commands) {
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
  }

  return shapes;
}
