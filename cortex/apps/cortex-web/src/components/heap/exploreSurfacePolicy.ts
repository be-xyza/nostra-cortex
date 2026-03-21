import type { HeapBlockListItem } from "../../contracts";
import { resolveSpaceArchetypeProfile } from "../spaces/spaceArchetypeProfiles.ts";

export type ExploreProjectionMode = "list" | "graph";
export type ExploreProjectionIntent =
  | "overview"
  | "story"
  | "density"
  | "lineage";

export interface ExploreSurfacePolicy {
  policyId: string;
  projectionMode: ExploreProjectionMode;
  projectionIntent: ExploreProjectionIntent;
  laneBreakpoints: Array<{
    minWidth: number;
    laneCount: number;
  }>;
  laneWeightByBlockType: Record<string, number>;
  textWeightDivisor: number;
  textWeightCap: number;
  warningWeightMultiplier: number;
  warningWeightCap: number;
}

export interface ExploreSurfacePolicyContext {
  spaceId?: string;
  surfaceId?: string;
  spaceArchetype?: string;
}

type ExploreListPolicyOverrides = Partial<
  Omit<ExploreSurfacePolicy, "policyId" | "projectionMode" | "projectionIntent">
>;

const DEFAULT_LANE_BREAKPOINTS: ExploreSurfacePolicy["laneBreakpoints"] = [
  { minWidth: 1600, laneCount: 4 },
  { minWidth: 1280, laneCount: 3 },
  { minWidth: 768, laneCount: 2 },
  { minWidth: 0, laneCount: 1 },
];

const DEFAULT_LANE_WEIGHT_BY_BLOCK_TYPE: Record<string, number> = {
  chart: 8,
  telemetry: 7,
  structured_data: 6,
  gate_summary: 6,
  note: 5,
  task: 5,
  media: 4,
  a2ui: 4,
  widget: 4,
};

function createExploreListPolicy(
  policyId: string,
  projectionIntent: ExploreProjectionIntent,
  overrides: ExploreListPolicyOverrides = {}
): ExploreSurfacePolicy {
  return {
    policyId,
    projectionMode: "list",
    projectionIntent,
    laneBreakpoints: overrides.laneBreakpoints ?? DEFAULT_LANE_BREAKPOINTS,
    laneWeightByBlockType:
      overrides.laneWeightByBlockType ?? DEFAULT_LANE_WEIGHT_BY_BLOCK_TYPE,
    textWeightDivisor: overrides.textWeightDivisor ?? 180,
    textWeightCap: overrides.textWeightCap ?? 2,
    warningWeightMultiplier: overrides.warningWeightMultiplier ?? 0.5,
    warningWeightCap: overrides.warningWeightCap ?? 1.5,
  };
}

export const DEFAULT_EXPLORE_LIST_POLICY = createExploreListPolicy(
  "explore.list.default.v1",
  "overview"
);

export const STORY_EXPLORE_LIST_POLICY = createExploreListPolicy(
  "explore.list.story.v1",
  "story",
  {
    laneBreakpoints: [
      { minWidth: 1600, laneCount: 3 },
      { minWidth: 1120, laneCount: 2 },
      { minWidth: 0, laneCount: 1 },
    ],
    laneWeightByBlockType: {
      ...DEFAULT_LANE_WEIGHT_BY_BLOCK_TYPE,
      note: 6,
      task: 5.5,
      chart: 7,
      telemetry: 6,
    },
    textWeightDivisor: 150,
    textWeightCap: 2.5,
  }
);

export const DENSITY_EXPLORE_LIST_POLICY = createExploreListPolicy(
  "explore.list.density.v1",
  "density",
  {
    laneBreakpoints: [
      { minWidth: 1440, laneCount: 4 },
      { minWidth: 1080, laneCount: 3 },
      { minWidth: 720, laneCount: 2 },
      { minWidth: 0, laneCount: 1 },
    ],
    laneWeightByBlockType: {
      ...DEFAULT_LANE_WEIGHT_BY_BLOCK_TYPE,
      chart: 7,
      telemetry: 6.5,
      media: 3.5,
      a2ui: 3.5,
      widget: 3.5,
    },
    textWeightDivisor: 220,
    textWeightCap: 1.75,
  }
);

export const LINEAGE_EXPLORE_LIST_POLICY = createExploreListPolicy(
  "explore.list.lineage.v1",
  "lineage",
  {
    laneBreakpoints: [
      { minWidth: 1600, laneCount: 3 },
      { minWidth: 1180, laneCount: 2 },
      { minWidth: 0, laneCount: 1 },
    ],
    laneWeightByBlockType: {
      ...DEFAULT_LANE_WEIGHT_BY_BLOCK_TYPE,
      structured_data: 7,
      gate_summary: 7,
      note: 4.5,
      media: 3.5,
    },
    warningWeightMultiplier: 0.65,
    warningWeightCap: 2,
  }
);

export const EXPLORE_SURFACE_POLICIES_BY_ID: Record<string, ExploreSurfacePolicy> = {
  [DEFAULT_EXPLORE_LIST_POLICY.policyId]: DEFAULT_EXPLORE_LIST_POLICY,
  [STORY_EXPLORE_LIST_POLICY.policyId]: STORY_EXPLORE_LIST_POLICY,
  [DENSITY_EXPLORE_LIST_POLICY.policyId]: DENSITY_EXPLORE_LIST_POLICY,
  [LINEAGE_EXPLORE_LIST_POLICY.policyId]: LINEAGE_EXPLORE_LIST_POLICY,
};

export function resolveExploreSurfacePolicy(
  context: ExploreSurfacePolicyContext = {}
): ExploreSurfacePolicy {
  const profile = resolveSpaceArchetypeProfile(context.spaceArchetype);
  return (
    EXPLORE_SURFACE_POLICIES_BY_ID[profile.explorePolicyId] ??
    DEFAULT_EXPLORE_LIST_POLICY
  );
}

export function resolveExploreLaneCount(
  width: number,
  policy: ExploreSurfacePolicy
): number {
  for (const breakpoint of policy.laneBreakpoints) {
    if (width >= breakpoint.minWidth) {
      return breakpoint.laneCount;
    }
  }
  return 1;
}

export function estimateExploreBlockWeight(
  block: HeapBlockListItem,
  policy: ExploreSurfacePolicy
): number {
  const blockType = String(block.projection.blockType || "").toLowerCase();
  const surface = (block.surfaceJson || {}) as Record<string, unknown>;
  const text = String(
    surface.text || surface.plain_text || block.projection.title || ""
  );
  const warningsWeight = Math.min(
    (block.warnings?.length ?? 0) * policy.warningWeightMultiplier,
    policy.warningWeightCap
  );
  const textWeight = Math.min(
    text.length / policy.textWeightDivisor,
    policy.textWeightCap
  );

  return (
    (policy.laneWeightByBlockType[blockType] ?? 4.5) +
    textWeight +
    warningsWeight
  );
}
