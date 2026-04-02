export const SPACE_STUDIO_ROUTE = "/labs/space-studio";
export const EXECUTION_CANVAS_ROUTE = "/labs/execution-canvas";

export type SpaceStudioView = "draft" | "templates";

export function buildSpaceStudioRoute(view: SpaceStudioView = "draft"): string {
  if (view === "templates") {
    return `${SPACE_STUDIO_ROUTE}?view=templates`;
  }
  return SPACE_STUDIO_ROUTE;
}

export function isSpaceStudioPath(pathname: string): boolean {
  return pathname === SPACE_STUDIO_ROUTE || pathname.startsWith(`${SPACE_STUDIO_ROUTE}/`);
}

export function buildExecutionCanvasRoute(): string {
  return EXECUTION_CANVAS_ROUTE;
}

export function isExecutionCanvasPath(pathname: string): boolean {
  return pathname === EXECUTION_CANVAS_ROUTE || pathname.startsWith(`${EXECUTION_CANVAS_ROUTE}/`);
}
