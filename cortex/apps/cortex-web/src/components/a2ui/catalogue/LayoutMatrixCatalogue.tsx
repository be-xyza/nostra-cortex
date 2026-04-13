import React, { useState } from "react";
import { SpatialHeapGrid } from "../SpatialHeapGrid";
import {
  EXPERIMENTAL_LAYOUT_DESCRIPTIONS,
  EXPERIMENTAL_LAYOUT_FAMILIES,
  EXPERIMENTAL_LAYOUT_LABELS,
  LAYOUT_MATRIX_SAMPLE_BLOCKS,
  type ExperimentalLayoutFamily,
} from "./layoutMatrixCatalogueModel";

export function LayoutMatrixCatalogue() {
  const [mode, setMode] = useState<ExperimentalLayoutFamily>("spatial_bsp");

  const renderLayout = () => {
    switch (mode) {
      case "spatial_bsp":
        return <SpatialHeapGrid blocks={LAYOUT_MATRIX_SAMPLE_BLOCKS} />;

      case "time_indexed":
        return (
          <div className="mx-auto flex max-w-3xl flex-col gap-4 px-8 py-10">
            {LAYOUT_MATRIX_SAMPLE_BLOCKS.map((block, index) => (
              <div key={block.id} className="relative pl-10">
                <div className="absolute left-3 top-2 h-full w-px bg-white/10" />
                <div
                  className="absolute left-0 top-1.5 flex h-6 w-6 items-center justify-center rounded-full border border-white/10 bg-slate-900 text-[10px] font-black uppercase text-slate-300"
                  style={{ boxShadow: `0 0 0 2px ${block.accent ?? "var(--ui-accent-blue)"}` }}
                >
                  {index + 1}
                </div>
                <div
                  className="rounded-2xl border border-white/10 bg-slate-900/50 p-4"
                  style={{ borderLeft: `4px solid ${block.accent}` }}
                >
                  <div className="mb-2 flex items-center justify-between gap-3">
                    <div
                      className="font-mono text-[10px] uppercase tracking-widest"
                      style={{ color: block.accent }}
                    >
                      {block.type}
                    </div>
                    <div className="text-[10px] uppercase tracking-[0.3em] text-slate-500">
                      T + {index}
                    </div>
                  </div>
                  <div className="text-sm text-slate-100">{block.title}</div>
                </div>
              </div>
            ))}
          </div>
        );

      case "lane_board":
        return (
          <div className="flex gap-4 p-8 overflow-x-auto">
            {["Intake", "Linking", "Thesis"].map((column, index) => (
              <div
                key={column}
                className="w-80 shrink-0 rounded-2xl border border-white/5 bg-slate-900/40 p-4"
              >
                <h3 className="mb-1 text-xs font-bold uppercase text-slate-400">{column}</h3>
                <p className="mb-4 text-[11px] uppercase tracking-[0.25em] text-slate-600">
                  {index === 0 ? "Queue" : index === 1 ? "Active relation work" : "Promoted outputs"}
                </p>
                <div className="flex flex-col gap-3">
                  {LAYOUT_MATRIX_SAMPLE_BLOCKS.slice(index, index + 3).map((block) => (
                    <div
                      key={`${column}-${block.id}`}
                      className="rounded-lg border border-white/5 bg-slate-950 p-3 text-sm"
                    >
                      <div
                        className="mb-1 font-mono text-[10px] uppercase tracking-widest"
                        style={{ color: block.accent }}
                      >
                        {block.type}
                      </div>
                      {block.title}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        );

      case "force_graph":
        return (
          <div className="m-8 flex h-full items-center justify-center rounded-3xl border border-dashed border-white/10 text-sm font-mono text-slate-500">
            Force-graph family remains experimental. Use SpatialPlane for durable spatial layout contracts.
          </div>
        );
    }
  };

  return (
    <div className="flex flex-col h-screen w-full bg-[#020202] text-slate-200">
      <header className="flex items-center justify-between px-6 py-4 border-b border-white/10 shrink-0">
        <div className="max-w-2xl">
          <h1 className="text-lg font-black tracking-widest uppercase">Layout Matrix Catalogue</h1>
          <p className="mt-1 text-xs text-slate-500">
            Experimental Cortex Labs vocabulary only. Heap settings and shared ViewSpec contracts remain unchanged in this phase.
          </p>
          <p className="mt-3 text-[11px] uppercase tracking-[0.25em] text-slate-600">
            {EXPERIMENTAL_LAYOUT_DESCRIPTIONS[mode]}
          </p>
        </div>
        <div className="flex bg-slate-900 rounded-lg p-1 border border-white/5">
          {EXPERIMENTAL_LAYOUT_FAMILIES.map((family) => (
            <button
              key={family}
              onClick={() => setMode(family)}
              className={`px-4 py-1.5 text-xs font-bold uppercase tracking-wider rounded-md transition-all ${mode === family ? 'bg-slate-700 text-white shadow-lg' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800'}`}
              data-layout-family={family}
            >
              {EXPERIMENTAL_LAYOUT_LABELS[family]}
            </button>
          ))}
        </div>
      </header>

      <main className="flex-1 overflow-hidden relative">
        {renderLayout()}
      </main>
    </div>
  );
}
