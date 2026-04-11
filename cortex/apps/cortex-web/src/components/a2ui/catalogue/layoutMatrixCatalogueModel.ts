import type { A2UIBlock } from "../SpatialHeapGrid";

export type ExperimentalLayoutFamily =
  | "lane_board"
  | "spatial_bsp"
  | "force_graph"
  | "time_indexed";

export const EXPERIMENTAL_LAYOUT_FAMILIES = [
  "lane_board",
  "spatial_bsp",
  "force_graph",
  "time_indexed",
] as const satisfies readonly ExperimentalLayoutFamily[];

export const EXPERIMENTAL_LAYOUT_LABELS: Record<ExperimentalLayoutFamily, string> = {
  lane_board: "Lane Board",
  spatial_bsp: "Spatial BSP",
  force_graph: "Force Graph",
  time_indexed: "Time Indexed",
};

export const EXPERIMENTAL_LAYOUT_DESCRIPTIONS: Record<ExperimentalLayoutFamily, string> = {
  lane_board: "Collection lanes for queueing, triage, and throughput.",
  spatial_bsp: "Recursive spatial packing for exploratory scanning.",
  force_graph: "Relationship-first topology for connected clusters.",
  time_indexed: "Chronological sequencing for provenance and review.",
};

export const LAYOUT_MATRIX_SAMPLE_BLOCKS: A2UIBlock[] = [
  {
    id: "b1",
    type: "claim",
    title: "Caffeine improves short-term recall",
    accent: "var(--ui-accent-purple)",
  },
  {
    id: "b2",
    type: "question",
    title: "Does consciousness require a period of genuine solitude?",
    accent: "var(--ui-accent-orange)",
  },
  {
    id: "b3",
    type: "entity",
    title: "Nostra Platform",
    accent: "var(--ui-accent-blue)",
  },
  {
    id: "b4",
    type: "task",
    title: "Review papers on distributed consensus",
    accent: "var(--ui-accent-green)",
  },
  {
    id: "b5",
    type: "idea",
    title: "A2UI spatial mapping logic",
    accent: "var(--ui-accent-yellow)",
  },
  {
    id: "b6",
    type: "quote",
    title: "Attention is the rarest form of generosity - Simone Weil",
    accent: "var(--ui-accent-slate)",
  },
  {
    id: "b7",
    type: "thesis",
    title: "Spatial architectures require explicit topology contracts.",
    accent: "var(--ui-accent-gold)",
  },
];
