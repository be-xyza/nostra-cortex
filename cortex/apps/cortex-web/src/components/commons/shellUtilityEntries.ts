import type { NavigationEntrySpec } from "../../contracts";

export const CONVERSATIONS_ROUTE = "/conversations";
export const SYSTEM_PROVIDERS_ROUTE = "/system/providers";

export function appendShellUtilityEntries(entries: NavigationEntrySpec[]): NavigationEntrySpec[] {
  const nextEntries = [...entries];

  if (!nextEntries.some((entry) => entry.routeId === CONVERSATIONS_ROUTE)) {
    nextEntries.push({
      routeId: CONVERSATIONS_ROUTE,
      label: "Conversations",
      icon: "messages-square",
      category: "bridge",
      requiredRole: "operator",
      navSlot: "secondary_agents",
    });
  }

  if (!nextEntries.some((entry) => entry.routeId === SYSTEM_PROVIDERS_ROUTE)) {
    nextEntries.push({
      routeId: SYSTEM_PROVIDERS_ROUTE,
      label: "Providers",
      icon: "brain-circuit",
      category: "system",
      requiredRole: "operator",
      navSlot: "secondary_admin",
    });
  }

  return nextEntries;
}
