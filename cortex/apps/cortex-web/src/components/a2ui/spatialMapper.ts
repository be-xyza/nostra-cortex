import type {
  SpatialAnnotationShape,
  SpatialArrowShape,
  SpatialEdgeShape,
  SpatialGroupShape,
  SpatialNodeShape,
  SpatialNoteShape,
  SpatialShape,
} from "./spatialReplay.ts";

export type SpatialMapperErrorClass = "contract_invalid";

export type SpatialMapperError = {
  errorClass: SpatialMapperErrorClass;
  shapeId: string;
  reason: string;
};

const MAX_ABS_COORDINATE = 20_000;
const MAX_ABS_DIMENSION = 10_000;

function finiteNumber(value: number | undefined): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function validateBaseShape(shape: SpatialShape): string | null {
  if (!shape.id || !shape.id.trim()) {
    return "shape.id is required";
  }
  if (!finiteNumber(shape.x) || !finiteNumber(shape.y)) {
    return "shape.x and shape.y must be finite numbers";
  }
  if (Math.abs(shape.x) > MAX_ABS_COORDINATE || Math.abs(shape.y) > MAX_ABS_COORDINATE) {
    return `shape coordinates exceed bounds (${MAX_ABS_COORDINATE})`;
  }
  return null;
}

function validateNoteShape(shape: SpatialNoteShape): string | null {
  const w = shape.w ?? 170;
  const h = shape.h ?? 84;
  if (!finiteNumber(w) || !finiteNumber(h)) {
    return "note dimensions must be finite numbers";
  }
  if (w <= 0 || h <= 0) {
    return "note dimensions must be > 0";
  }
  if (Math.abs(w) > MAX_ABS_DIMENSION || Math.abs(h) > MAX_ABS_DIMENSION) {
    return `note dimensions exceed bounds (${MAX_ABS_DIMENSION})`;
  }
  return null;
}

function validateArrowShape(shape: SpatialArrowShape): string | null {
  if (!finiteNumber(shape.to_x) || !finiteNumber(shape.to_y)) {
    return "arrow requires to_x and to_y coordinates";
  }
  if (
    Math.abs(shape.to_x) > MAX_ABS_COORDINATE ||
    Math.abs(shape.to_y) > MAX_ABS_COORDINATE
  ) {
    return `arrow end coordinates exceed bounds (${MAX_ABS_COORDINATE})`;
  }
  return null;
}

function validateNodeShape(shape: SpatialNodeShape): string | null {
  const w = shape.w ?? 220;
  const h = shape.h ?? 140;
  if (!finiteNumber(w) || !finiteNumber(h) || w <= 0 || h <= 0) {
    return "node dimensions must be > 0";
  }
  const seenPorts = new Set<string>();
  for (const port of shape.ports ?? []) {
    if (!port.id?.trim()) {
      return "node ports require id";
    }
    if (seenPorts.has(port.id)) {
      return `duplicate port id: ${port.id}`;
    }
    seenPorts.add(port.id);
  }
  return null;
}

function validateEdgeShape(shape: SpatialEdgeShape): string | null {
  if (!shape.from_shape_id?.trim() || !shape.to_shape_id?.trim()) {
    return "edge requires from_shape_id and to_shape_id";
  }
  return null;
}

function validateGroupShape(shape: SpatialGroupShape): string | null {
  const w = shape.w ?? 320;
  const h = shape.h ?? 220;
  if (!finiteNumber(w) || !finiteNumber(h) || w <= 0 || h <= 0) {
    return "group dimensions must be > 0";
  }
  if (!Array.isArray(shape.member_ids)) {
    return "group.member_ids is required";
  }
  return null;
}

function validateAnnotationShape(shape: SpatialAnnotationShape): string | null {
  const w = shape.w ?? 180;
  const h = shape.h ?? 90;
  if (!finiteNumber(w) || !finiteNumber(h) || w <= 0 || h <= 0) {
    return "annotation dimensions must be > 0";
  }
  return null;
}

function validateShape(shape: SpatialShape): string | null {
  const baseError = validateBaseShape(shape);
  if (baseError) return baseError;

  if (shape.kind === "note") {
    return validateNoteShape(shape);
  }
  if (shape.kind === "arrow") {
    return validateArrowShape(shape);
  }
  if (shape.kind === "node") {
    return validateNodeShape(shape);
  }
  if (shape.kind === "edge") {
    return validateEdgeShape(shape);
  }
  if (shape.kind === "group") {
    return validateGroupShape(shape);
  }
  if (shape.kind === "annotation") {
    return validateAnnotationShape(shape);
  }
  return null;
}

function shapeIdForTldraw(shapeId: string): string {
  return `shape:${shapeId}`;
}

function defaultWidth(shape: SpatialShape): number {
  if (shape.kind === "node") return shape.w ?? 220;
  if (shape.kind === "group") return shape.w ?? 320;
  if (shape.kind === "annotation") return shape.w ?? 180;
  if (shape.kind === "note") return shape.w ?? 170;
  return 0;
}

function defaultHeight(shape: SpatialShape): number {
  if (shape.kind === "node") return shape.h ?? 140;
  if (shape.kind === "group") return shape.collapsed ? 56 : (shape.h ?? 220);
  if (shape.kind === "annotation") return shape.h ?? 90;
  if (shape.kind === "note") return shape.h ?? 84;
  return 0;
}

function resolveNodePortPoint(shape: SpatialShape, portId: string | undefined, preference: "source" | "target"): { x: number; y: number } {
  if (shape.kind !== "node") {
    return { x: shape.x, y: shape.y };
  }

  const ports = shape.ports ?? [];
  const chosenPort =
    ports.find((port) => port.id === portId)
    ?? ports.find((port) => {
      if (preference === "source") return port.direction === "out" || port.direction === "inout";
      return port.direction === "in" || port.direction === "inout";
    })
    ?? ports[0];

  const width = defaultWidth(shape);
  const height = defaultHeight(shape);
  if (!chosenPort) {
    return {
      x: shape.x + width / 2,
      y: shape.y + height / 2,
    };
  }

  const sidePorts = ports.filter((port) => port.side === chosenPort.side);
  const index = Math.max(0, sidePorts.findIndex((port) => port.id === chosenPort.id));
  const count = Math.max(1, sidePorts.length);

  if (chosenPort.side === "left" || chosenPort.side === "right") {
    const step = height / (count + 1);
    return {
      x: chosenPort.side === "left" ? shape.x : shape.x + width,
      y: shape.y + step * (index + 1),
    };
  }

  const step = width / (count + 1);
  return {
    x: shape.x + step * (index + 1),
    y: chosenPort.side === "top" ? shape.y : shape.y + height,
  };
}

function mapShape(shape: SpatialShape, shapesById: Map<string, SpatialShape>): Record<string, unknown> {
  const tldrawId = shapeIdForTldraw(shape.id);
  if (shape.kind === "arrow") {
    const dx = (shape.to_x as number) - shape.x;
    const dy = (shape.to_y as number) - shape.y;
    return {
      id: tldrawId,
      type: "arrow",
      x: shape.x,
      y: shape.y,
      props: {
        start: { x: 0, y: 0 },
        end: { x: dx, y: dy },
        text: shape.text ?? ""
      }
    };
  }

  if (shape.kind === "edge") {
    const fromShape = shapesById.get(shape.from_shape_id);
    const toShape = shapesById.get(shape.to_shape_id);
    const start = fromShape
      ? resolveNodePortPoint(fromShape, shape.from_port_id, "source")
      : { x: shape.x, y: shape.y };
    const end = toShape
      ? resolveNodePortPoint(toShape, shape.to_port_id, "target")
      : { x: shape.x + 180, y: shape.y };
    return {
      id: tldrawId,
      type: "arrow",
      x: start.x,
      y: start.y,
      props: {
        start: { x: 0, y: 0 },
        end: { x: end.x - start.x, y: end.y - start.y },
        dash: shape.edge_class === "control" ? "dashed" : shape.edge_class === "branch" ? "dotted" : "solid",
        text: shape.text ?? `${shape.edge_class}:${shape.from_port_id ?? ""}->${shape.to_port_id ?? ""}`
      },
      meta: {
        spatialKind: "edge",
        edgeClass: shape.edge_class,
      }
    };
  }

  if (shape.kind === "group") {
    return {
      id: tldrawId,
      type: "geo",
      x: shape.x,
      y: shape.y,
      props: {
        geo: "rectangle",
        w: defaultWidth(shape),
        h: defaultHeight(shape),
        text: shape.label ?? shape.text ?? ""
      },
      meta: {
        spatialKind: "group",
        memberIds: shape.member_ids,
        collapsed: Boolean(shape.collapsed)
      }
    };
  }

  if (shape.kind === "annotation") {
    return {
      id: tldrawId,
      type: "geo",
      x: shape.x,
      y: shape.y,
      props: {
        geo: "rectangle",
        w: defaultWidth(shape),
        h: defaultHeight(shape),
        text: shape.text ?? ""
      },
      meta: {
        spatialKind: "annotation"
      }
    };
  }

  if (shape.kind === "node") {
    return {
      id: tldrawId,
      type: "geo",
      x: shape.x,
      y: shape.y,
      props: {
        geo: "rectangle",
        w: defaultWidth(shape),
        h: defaultHeight(shape),
        text: shape.text ?? ""
      },
      meta: {
        spatialKind: "node",
        nodeClass: shape.node_class,
        status: shape.status,
        ports: shape.ports ?? []
      }
    };
  }

  return {
    id: tldrawId,
    type: "geo",
    x: shape.x,
    y: shape.y,
    props: {
      geo: "rectangle",
      w: defaultWidth(shape),
      h: defaultHeight(shape),
      text: shape.text ?? ""
    }
  };
}

export function mapShapesToTldraw(shapes: SpatialShape[]): {
  mapped: Record<string, unknown>[];
  errors: SpatialMapperError[];
} {
  const mapped: Record<string, unknown>[] = [];
  const errors: SpatialMapperError[] = [];
  const shapesById = new Map(shapes.map((shape) => [shape.id, shape]));

  for (const shape of shapes) {
    const validationError = validateShape(shape);
    if (validationError) {
      errors.push({
        errorClass: "contract_invalid",
        shapeId: shape.id || "unknown",
        reason: validationError
      });
      continue;
    }
    mapped.push(mapShape(shape, shapesById));
  }

  return { mapped, errors };
}
