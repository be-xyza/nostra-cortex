import type {
  CompiledNavigationPlan,
  NavigationEntrySpec,
  ShellLayoutSpec,
} from "../../contracts";

function normalizeRouteId(routeId: string): string {
  if (!routeId) {
    return "/";
  }
  return routeId.startsWith("/") ? routeId : `/${routeId}`;
}

function normalizeRole(role: string | undefined): string {
  return role?.trim().toLowerCase() || "viewer";
}

export function resolveShellEntries(input: {
  layoutSpec: ShellLayoutSpec | null;
  compiledPlan: CompiledNavigationPlan | null;
  actorRole: string;
  playgroundEnabled: boolean;
  preferBaseEntries?: boolean;
}): NavigationEntrySpec[] {
  const baseEntries = (input.layoutSpec?.navigationGraph?.entries ?? []).filter(
    (entry) => input.playgroundEnabled || entry.routeId !== "/playground",
  );

  if (input.preferBaseEntries) {
    return baseEntries;
  }

  if (!input.compiledPlan?.entries?.length) {
    return baseEntries;
  }

  if (
    normalizeRole(input.compiledPlan.actorRole) !== normalizeRole(input.actorRole)
  ) {
    return baseEntries;
  }

  const rankByRoute = new Map<string, number>();
  for (const entry of input.compiledPlan.entries) {
    rankByRoute.set(normalizeRouteId(entry.routeId), entry.rank);
  }

  const filtered = baseEntries.filter((entry) =>
    rankByRoute.has(normalizeRouteId(entry.routeId)),
  );

  filtered.sort((left, right) => {
    const leftRank =
      rankByRoute.get(normalizeRouteId(left.routeId)) ?? Number.MAX_SAFE_INTEGER;
    const rightRank =
      rankByRoute.get(normalizeRouteId(right.routeId)) ?? Number.MAX_SAFE_INTEGER;
    if (leftRank !== rightRank) {
      return leftRank - rightRank;
    }
    return left.label.localeCompare(right.label);
  });

  return filtered;
}
