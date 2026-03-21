import { create } from "zustand";

// ─── Contracts ───────────────────────────────────────────────────────────────

/** Order override for a single UI surface (nav section list, tab row, etc.) */
export interface SurfaceOrder {
  /** Ordered list of item IDs – only items present in the source are kept. */
  itemOrder: string[];
  /** Items the user chose to hide from this surface. */
  hidden: string[];
}

/** Per-space layout preferences covering every rearrangeable surface. */
export interface LayoutPreferences {
  /** Section slot order (keys = slot names from navSections.ts). */
  navSections?: SurfaceOrder;
  /** Item order *within* each nav section.  Key = slot name. */
  navItems?: Record<string, SurfaceOrder>;
  /** Modal tab ordering. */
  modalTabs?: SurfaceOrder;
  /** Selection ActionBar action ordering. */
  actionBar?: SurfaceOrder;
}

// ─── Pure helpers ────────────────────────────────────────────────────────────

export const EMPTY_PREFS: LayoutPreferences = {};

const STORAGE_PREFIX = "cortex.layout.";

function storageKey(spaceId: string): string {
  return `${STORAGE_PREFIX}${spaceId}`;
}

function readFromStorage(spaceId: string): LayoutPreferences {
  if (typeof window === "undefined") return {};
  try {
    const raw = window.localStorage.getItem(storageKey(spaceId));
    return raw ? (JSON.parse(raw) as LayoutPreferences) : {};
  } catch {
    return {};
  }
}

function writeToStorage(spaceId: string, prefs: LayoutPreferences): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(storageKey(spaceId), JSON.stringify(prefs));
  } catch {
    // quota or private-mode – silent ignore
  }
}

/**
 * Merge a backend-supplied item list with user overrides.
 *
 * Resolution:
 * 1. Items in `overrides.itemOrder` are placed first (preserving user order).
 * 2. Remaining source items are appended in their original order.
 * 3. Items in `overrides.hidden` are filtered out.
 * 4. Items in `overrides.itemOrder` that are no longer in `source` are dropped.
 */
export function applyOrder(
  source: string[],
  overrides: SurfaceOrder | undefined,
): string[] {
  if (!overrides) return source;

  const sourceSet = new Set(source);
  const hiddenSet = new Set(overrides.hidden);

  // User-ordered items that still exist in the source
  const ordered = overrides.itemOrder.filter(
    (id) => sourceSet.has(id) && !hiddenSet.has(id),
  );
  const orderedSet = new Set(ordered);

  // Append any source items the user hasn't explicitly positioned
  for (const id of source) {
    if (!orderedSet.has(id) && !hiddenSet.has(id)) {
      ordered.push(id);
    }
  }

  return ordered;
}

// ─── Store ───────────────────────────────────────────────────────────────────

interface LayoutPreferencesState {
  /** Preferences cache keyed by spaceId. */
  cache: Record<string, LayoutPreferences>;

  /** Whether there are unsaved (pending) changes for the active space. */
  pendingSpaceId: string | null;
  pendingSnapshot: LayoutPreferences | null;

  /** Load prefs for a space (reads localStorage on first access). */
  getPreferences: (spaceId: string) => LayoutPreferences;

  /** Stage a change without persisting (for optimistic drag preview). */
  stageChange: (
    spaceId: string,
    updater: (current: LayoutPreferences) => LayoutPreferences,
  ) => void;

  /** Commit pending changes to localStorage. */
  commitPending: () => void;

  /** Revert pending changes to last-saved state. */
  revertPending: () => void;
}

export const useLayoutPreferences = create<LayoutPreferencesState>(
  (set, get) => ({
    cache: {},
    pendingSpaceId: null,
    pendingSnapshot: null,

    getPreferences: (spaceId: string): LayoutPreferences => {
      const { cache } = get();
      if (cache[spaceId]) return cache[spaceId];
      const loaded = readFromStorage(spaceId);
      set((state) => ({
        cache: { ...state.cache, [spaceId]: loaded },
      }));
      return loaded;
    },

    stageChange: (spaceId, updater) => {
      const state = get();
      const current = state.cache[spaceId] ?? readFromStorage(spaceId);
      const snapshot =
        state.pendingSpaceId === spaceId && state.pendingSnapshot
          ? state.pendingSnapshot
          : current;
      const next = updater(current);
      set({
        cache: { ...state.cache, [spaceId]: next },
        pendingSpaceId: spaceId,
        pendingSnapshot: snapshot,
      });
    },

    commitPending: () => {
      const { pendingSpaceId, cache } = get();
      if (!pendingSpaceId) return;
      const prefs = cache[pendingSpaceId];
      if (prefs) writeToStorage(pendingSpaceId, prefs);
      set({ pendingSpaceId: null, pendingSnapshot: null });
    },

    revertPending: () => {
      const { pendingSpaceId, pendingSnapshot, cache } = get();
      if (!pendingSpaceId || !pendingSnapshot) {
        set({ pendingSpaceId: null, pendingSnapshot: null });
        return;
      }
      set({
        cache: { ...cache, [pendingSpaceId]: pendingSnapshot },
        pendingSpaceId: null,
        pendingSnapshot: null,
      });
    },
  }),
);
