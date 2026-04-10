// @ts-nocheck
import type {
  CompiledNavigationPlan,
  NavigationEntrySpec,
} from "../../contracts";
import {
  applyOrder,
  type LayoutPreferences,
} from "../../store/layoutPreferences.ts";

export interface NavigationSection {
  slot: string;
  label: string;
  entries: NavigationEntrySpec[];
}

export const NAV_SLOT_LABELS: Record<string, string> = {
  primary_focus: "Focus",
  primary_platform: "Platform",
  primary_attention: "Inbox",
  primary_workspace: "Explore",
  primary_execute: "Execute",
  secondary_ops: "Ops",
  secondary_build: "Build",
  secondary_agents: "Agents",
  secondary_conversations: "Conversations",
  secondary_admin: "Admin",
  labs: "Labs",
};

export function buildNavigationSections(
  entries: NavigationEntrySpec[],
  compiledPlan: CompiledNavigationPlan | null,
  layoutPrefs?: LayoutPreferences,
): NavigationSection[] {
  const slotByRoute = new Map<string, string>();
  for (const entry of compiledPlan?.entries ?? []) {
    slotByRoute.set(entry.routeId, entry.navSlot);
  }

  const grouped = new Map<string, NavigationEntrySpec[]>();
  const order: string[] = [];
  for (const entry of entries) {
    const slot =
      slotByRoute.get(entry.routeId) ??
      entry.navSlot ??
      "secondary_ops";
    if (!grouped.has(slot)) {
      grouped.set(slot, []);
      order.push(slot);
    }
    grouped.get(slot)?.push(entry);
  }

  // Apply user section-level ordering
  const sectionOrder = applyOrder(order, layoutPrefs?.navSections);

  return sectionOrder
    .map((slot) => {
      const rawEntries = grouped.get(slot) ?? [];
      // Apply user item-level ordering within each section
      const itemIds = rawEntries.map((e) => e.routeId);
      const orderedIds = applyOrder(itemIds, layoutPrefs?.navItems?.[slot]);
      const orderedEntries = orderedIds
        .map((id) => rawEntries.find((e) => e.routeId === id))
        .filter(Boolean) as NavigationEntrySpec[];

      return {
        slot,
        label: NAV_SLOT_LABELS[slot] ?? slot,
        entries: orderedEntries,
      };
    })
    .filter((section) => section.entries.length > 0);
}
