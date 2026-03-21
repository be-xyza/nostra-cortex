import type { SpatialShape } from "./spatialReplay.ts";

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

function validateNoteShape(shape: SpatialShape): string | null {
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

function validateArrowShape(shape: SpatialShape): string | null {
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

function validateShape(shape: SpatialShape): string | null {
  const baseError = validateBaseShape(shape);
  if (baseError) return baseError;

  if (shape.kind === "note") {
    return validateNoteShape(shape);
  }
  if (shape.kind === "arrow") {
    return validateArrowShape(shape);
  }
  return `unsupported shape kind: ${String(shape.kind)}`;
}

function shapeIdForTldraw(shapeId: string): string {
  return `shape:${shapeId}`;
}

function mapShape(shape: SpatialShape): Record<string, unknown> {
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

  return {
    id: tldrawId,
    type: "geo",
    x: shape.x,
    y: shape.y,
    props: {
      geo: "rectangle",
      w: shape.w ?? 170,
      h: shape.h ?? 84,
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
    mapped.push(mapShape(shape));
  }

  return { mapped, errors };
}
