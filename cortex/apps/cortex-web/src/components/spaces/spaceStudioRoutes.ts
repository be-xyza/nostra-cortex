export const SPACE_STUDIO_ROUTE = "/labs/space-studio";

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
