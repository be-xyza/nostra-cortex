import React from "react";

import { WidgetRegistry } from "../WidgetRegistry";

const REGISTRY_HARNESS_BLOCKS = [
  {
    id: "registry-1",
    type: "claim",
    title: "Registry contract block one",
    accent: "#5eead4",
  },
  {
    id: "registry-2",
    type: "question",
    title: "Registry contract block two",
    accent: "#f59e0b",
  },
  {
    id: "registry-3",
    type: "entity",
    title: "Registry contract block three",
    accent: "#60a5fa",
  },
] as const;

export function LayoutRegistryTestHarness() {
  const SpatialHeapGridWidget = WidgetRegistry.SpatialHeapGrid;

  return (
    <div className="min-h-screen bg-[#020202] px-6 py-8 text-slate-200">
      <div className="mx-auto flex max-w-6xl flex-col gap-4">
        <header className="max-w-2xl">
          <h1 className="text-lg font-black uppercase tracking-widest">
            Layout Registry Harness
          </h1>
          <p className="mt-1 text-xs text-slate-500">
            Dedicated browser test route for WidgetRegistry resolution without shell bootstrap.
          </p>
        </header>
        <div className="min-h-[70vh] rounded-3xl border border-white/10 bg-slate-950/40 p-3">
          <SpatialHeapGridWidget
            id="layout-registry-harness"
            componentProperties={{
              SpatialHeapGrid: {
                blocks: REGISTRY_HARNESS_BLOCKS,
              },
            }}
          />
        </div>
      </div>
    </div>
  );
}
