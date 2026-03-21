import type { HeapBlockListItem } from "../../contracts";
import {
  estimateExploreBlockWeight,
  resolveExploreLaneCount,
  type ExploreSurfacePolicy,
} from "./exploreSurfacePolicy.ts";

export function resolveHeapLaneCount(
  width: number,
  policy: ExploreSurfacePolicy
): number {
  return resolveExploreLaneCount(width, policy);
}

export function buildHeapLanes(
  blocks: HeapBlockListItem[],
  laneCount: number,
  policy: ExploreSurfacePolicy
): HeapBlockListItem[][] {
  const resolvedLaneCount = Math.max(1, Math.trunc(laneCount || 1));
  const lanes = Array.from({ length: resolvedLaneCount }, () => [] as HeapBlockListItem[]);
  const laneWeights = Array.from({ length: resolvedLaneCount }, () => 0);

  blocks.forEach((block, index) => {
    if (index < resolvedLaneCount) {
      lanes[index]?.push(block);
      laneWeights[index] += estimateExploreBlockWeight(block, policy);
      return;
    }

    let targetLane = 0;
    for (let laneIndex = 1; laneIndex < resolvedLaneCount; laneIndex += 1) {
      if (laneWeights[laneIndex] < laneWeights[targetLane]) {
        targetLane = laneIndex;
      }
    }

    lanes[targetLane]?.push(block);
    laneWeights[targetLane] += estimateExploreBlockWeight(block, policy);
  });

  return lanes;
}
