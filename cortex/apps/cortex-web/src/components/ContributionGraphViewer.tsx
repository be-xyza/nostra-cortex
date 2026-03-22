import { useState } from "react";
import { ContributionGraph } from "../contracts";
import { ForceGraph } from "./ForceGraph";
import { TemporalTreeGraph } from "./TemporalTreeGraph";

type Props = {
  data: ContributionGraph;
  width?: number;
  height?: number;
};

export function ContributionGraphViewer({ data, width, height }: Props) {
  const [mode, setMode] = useState<"force" | "lineage">("force");
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const nodes = data?.nodes || [];
  const edges = data?.edges || [];

  return (
    <div className="relative w-full h-full flex flex-col min-h-[500px]">
      <div className="absolute top-4 right-4 z-10 flex gap-2 p-1 bg-black/40 backdrop-blur-md rounded-lg border border-white/10 shadow-xl">
        <button
          onClick={() => setMode("force")}
          className={`px-3 py-1.5 text-xs font-semibold rounded-md transition-all ${
            mode === "force"
              ? "bg-blue-600 text-white shadow-lg"
              : "text-white/60 hover:text-white hover:bg-white/10"
          }`}
        >
          Force
        </button>
        <button
          onClick={() => setMode("lineage")}
          className={`px-3 py-1.5 text-xs font-semibold rounded-md transition-all ${
            mode === "lineage"
              ? "bg-blue-600 text-white shadow-lg"
              : "text-white/60 hover:text-white hover:bg-white/10"
          }`}
        >
          Lineage
        </button>
      </div>
      
      <div className="flex-1 min-h-0">
        {mode === "force" ? (
          <ForceGraph 
            nodes={nodes} 
            edges={edges} 
            selectedId={selectedId} 
            onSelect={setSelectedId} 
          />
        ) : (
          <TemporalTreeGraph 
            nodes={nodes} 
            edges={edges} 
            selectedId={selectedId} 
            onSelect={setSelectedId} 
          />
        )}
      </div>
    </div>
  );
}
