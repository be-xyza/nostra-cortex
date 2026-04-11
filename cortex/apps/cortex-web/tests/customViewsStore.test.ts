import test from "node:test";
import assert from "node:assert/strict";

test("customViewsStore saves, renames, and removes views per space", async () => {
  let useCustomViewsStore: any = null;
  const globalWindow = globalThis as typeof globalThis & {
    window?: {
      localStorage: {
        getItem: (key: string) => string | null;
        setItem: (key: string, value: string) => void;
        removeItem: (key: string) => void;
      };
    };
  };
  const originalWindow = globalWindow.window;
  const storage = new Map<string, string>();

  globalWindow.window = {
    localStorage: {
      getItem: (key) => storage.get(key) ?? null,
      setItem: (key, value) => {
        storage.set(key, value);
      },
      removeItem: (key) => {
        storage.delete(key);
      },
    },
  };

  try {
    ({ useCustomViewsStore } = await import("../src/store/customViewsStore.ts"));
    useCustomViewsStore.setState({ cache: {} });

    const saved = useCustomViewsStore.getState().saveView("space-alpha", {
      label: "Dense Research",
      href: "/explore?view=density",
    });

    assert.equal(saved.label, "Dense Research");
    assert.equal(useCustomViewsStore.getState().getViews("space-alpha").length, 1);

    useCustomViewsStore.getState().updateView("space-alpha", saved.id, {
      label: "Renamed Research",
    });
    assert.equal(useCustomViewsStore.getState().getViews("space-alpha")[0]?.label, "Renamed Research");

    useCustomViewsStore.getState().removeView("space-alpha", saved.id);
    assert.equal(useCustomViewsStore.getState().getViews("space-alpha").length, 0);
  } finally {
    useCustomViewsStore?.setState({ cache: {} });
    globalWindow.window = originalWindow;
  }
});
