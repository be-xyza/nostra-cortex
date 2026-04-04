import { create } from "zustand";

export interface SavedCustomView {
  id: string;
  label: string;
  href: string;
  description: string;
  createdAt: string;
  updatedAt: string;
}

interface CustomViewsState {
  cache: Record<string, SavedCustomView[]>;
  getViews: (spaceId: string) => SavedCustomView[];
  saveView: (spaceId: string, input: { label: string; href: string; description?: string }) => SavedCustomView;
  removeView: (spaceId: string, viewId: string) => void;
  updateView: (spaceId: string, viewId: string, patch: Partial<Pick<SavedCustomView, "label" | "href" | "description">>) => void;
}

const STORAGE_PREFIX = "cortex.shell.customViews.";

function storageKey(spaceId: string): string {
  return `${STORAGE_PREFIX}${spaceId}`;
}

function readFromStorage(spaceId: string): SavedCustomView[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(storageKey(spaceId));
    const parsed = raw ? (JSON.parse(raw) as SavedCustomView[]) : [];
    return Array.isArray(parsed) ? parsed.filter(isSavedCustomView) : [];
  } catch {
    return [];
  }
}

function writeToStorage(spaceId: string, views: SavedCustomView[]): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(storageKey(spaceId), JSON.stringify(views));
  } catch {
    // Best-effort persistence only.
  }
}

function isSavedCustomView(value: unknown): value is SavedCustomView {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }
  const record = value as Record<string, unknown>;
  return (
    typeof record.id === "string" &&
    typeof record.label === "string" &&
    typeof record.href === "string" &&
    typeof record.description === "string" &&
    typeof record.createdAt === "string" &&
    typeof record.updatedAt === "string"
  );
}

function createViewId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `custom-view-${crypto.randomUUID()}`;
  }
  return `custom-view-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

function normalizeLabel(value: string): string {
  return value.trim().replace(/\s+/g, " ");
}

export const useCustomViewsStore = create<CustomViewsState>((set, get) => ({
  cache: {},

  getViews: (spaceId: string): SavedCustomView[] => {
    const { cache } = get();
    if (cache[spaceId]) {
      return cache[spaceId];
    }
    const loaded = readFromStorage(spaceId);
    set((state) => ({
      cache: {
        ...state.cache,
        [spaceId]: loaded,
      },
    }));
    return loaded;
  },

  saveView: (spaceId, input) => {
    const now = new Date().toISOString();
    const nextView: SavedCustomView = {
      id: createViewId(),
      label: normalizeLabel(input.label),
      href: input.href,
      description: input.description?.trim() || "Saved from the current view state.",
      createdAt: now,
      updatedAt: now,
    };

    set((state) => {
      const current = state.cache[spaceId] ?? readFromStorage(spaceId);
      const next = [...current, nextView];
      writeToStorage(spaceId, next);
      return {
        cache: {
          ...state.cache,
          [spaceId]: next,
        },
      };
    });

    return nextView;
  },

  removeView: (spaceId, viewId) => {
    set((state) => {
      const current = state.cache[spaceId] ?? readFromStorage(spaceId);
      const next = current.filter((view) => view.id !== viewId);
      writeToStorage(spaceId, next);
      return {
        cache: {
          ...state.cache,
          [spaceId]: next,
        },
      };
    });
  },

  updateView: (spaceId, viewId, patch) => {
    set((state) => {
      const current = state.cache[spaceId] ?? readFromStorage(spaceId);
      const next = current.map((view) => {
        if (view.id !== viewId) {
          return view;
        }
        return {
          ...view,
          label: patch.label !== undefined ? normalizeLabel(patch.label) : view.label,
          href: patch.href !== undefined ? patch.href : view.href,
          description: patch.description !== undefined ? patch.description.trim() : view.description,
          updatedAt: new Date().toISOString(),
        };
      });
      writeToStorage(spaceId, next);
      return {
        cache: {
          ...state.cache,
          [spaceId]: next,
        },
      };
    });
  },
}));
