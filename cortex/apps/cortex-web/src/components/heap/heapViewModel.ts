import type { HeapBlockListItem } from "../../contracts";

export type HeapPrimaryViewMode = "Explore" | "Inbox" | "Drafts" | "Tasks" | "Proposals" | "Activity" | "Pinned" | "Archive";
export type HeapViewCounts = Record<HeapPrimaryViewMode, number> & { Urgent: number };

const TASK_BLOCK_TYPES = new Set(["task", "checklist"]);
const PROPOSAL_BLOCK_TYPES = new Set(["action_plan", "compiled_plan"]);
export type HeapReviewLane = "private_review" | "public_review";

type HeapViewBlock = Pick<HeapBlockListItem, "projection" | "pinnedAt" | "surfaceJson" | "deletedAt">;

// Exported for UI components to iterate over
export const HEAP_PRIMARY_VIEW_MODES: HeapPrimaryViewMode[] = ["Explore", "Inbox", "Drafts", "Tasks", "Proposals", "Activity", "Pinned", "Archive"];

export function normalizeHeapPrimaryViewMode(value: string | null | undefined): HeapPrimaryViewMode {
  if (!value) return "Explore";

  const normalized = value.trim().toLowerCase();
  const aliasMap: Record<string, HeapPrimaryViewMode> = {
    all: "Explore",
    explore: "Explore",
    canvas: "Explore",
    inbox: "Inbox",
    unlinked: "Inbox", // Legacy map
    drafts: "Drafts",
    tasks: "Tasks",
    checklist: "Tasks",
    proposals: "Proposals",
    plans: "Proposals",
    activity: "Activity",
    sorted: "Activity", 
    pinned: "Pinned",
    pins: "Pinned",
    archive: "Archive",
    archived: "Archive",
  };

  return aliasMap[normalized] ?? "Explore";
}

export function heapPrimaryViewModeParam(viewMode: HeapPrimaryViewMode): string {
  if (viewMode === "Explore") return "all"; // Keep backwards compatibility in URL for now
  return viewMode.toLowerCase();
}

function blockBehaviors(block: HeapViewBlock): string[] {
  const surface = (block.surfaceJson as Record<string, unknown> | undefined) ?? {};
  const behaviors = surface.behaviors;
  return Array.isArray(behaviors) ? behaviors.filter((value): value is string => typeof value === "string") : [];
}

export function readHeapBlockReviewLane(block: HeapViewBlock): HeapReviewLane | null {
  const reviewLane = block.projection.attributes?.review_lane;
  if (reviewLane === "private_review" || reviewLane === "public_review") {
    return reviewLane;
  }
  return null;
}

export function isHeapBlockReviewWork(block: HeapViewBlock): boolean {
  return block.projection.blockType === "space_promotion_request" || readHeapBlockReviewLane(block) !== null;
}

export function hasHeapBlockRelations(block: HeapViewBlock): boolean {
  return Boolean(
    block.projection.tags?.length ||
      block.projection.pageLinks?.length ||
      block.projection.mentionsInline?.length,
  );
}

export function isHeapBlockInView(block: HeapViewBlock, viewMode: HeapPrimaryViewMode): boolean {
  // Archived blocks are ONLY visible in the Archive view
  if (block.deletedAt) {
    return viewMode === "Archive";
  }

  // If we are looking at any other view, hide archived blocks
  if (viewMode === "Archive") return false;

  const behaviors = blockBehaviors(block);
  const mentions = block.projection.mentionsInline || [];

  const blockType = block.projection.blockType ?? "";

  switch (viewMode) {
    case "Explore":
    case "Activity": // Activity view shows all active blocks, sorted by time
      return true;
    case "Inbox":
      // Inbox = Needs Attention (Mentions, Urgent, pending approvals)
      return mentions.length > 0 || behaviors.includes("urgent") || isHeapBlockReviewWork(block) || !hasHeapBlockRelations(block);
    case "Drafts":
      // Drafts = No formal governance or specific seed status
      return behaviors.includes("draft") || block.projection.status === "seed";
    case "Tasks":
      return TASK_BLOCK_TYPES.has(blockType) || behaviors.includes("actionable");
    case "Proposals":
      return PROPOSAL_BLOCK_TYPES.has(blockType) || behaviors.includes("awaiting_approval");
    case "Pinned":
      return behaviors.includes("pinned") || Boolean(block.pinnedAt);
    default:
      return false;
  }
}

export function buildHeapViewCounts<T extends HeapViewBlock>(blocks: T[]): HeapViewCounts {
  const counts: HeapViewCounts = {
    Explore: 0,
    Inbox: 0,
    Drafts: 0,
    Tasks: 0,
    Proposals: 0,
    Activity: 0, 
    Pinned: 0,
    Archive: 0,
    Urgent: 0,
  };

  for (const block of blocks) {
    const behaviors = blockBehaviors(block);
    const mentions = block.projection.mentionsInline || [];
    const blockType = block.projection.blockType ?? "";

    if (behaviors.includes("urgent")) counts.Urgent += 1;

    if (block.deletedAt) {
      counts.Archive += 1;
      continue; // Don't count deleted blocks in other active views
    }

    counts.Explore += 1;
    counts.Activity += 1;

    if (mentions.length > 0 || behaviors.includes("urgent") || isHeapBlockReviewWork(block) || !hasHeapBlockRelations(block)) {
      counts.Inbox += 1;
    }
    
    if (behaviors.includes("draft") || block.projection.status === "seed") {
      counts.Drafts += 1;
    }

    if (TASK_BLOCK_TYPES.has(blockType) || behaviors.includes("actionable")) {
      counts.Tasks += 1;
    }

    if (PROPOSAL_BLOCK_TYPES.has(blockType) || behaviors.includes("awaiting_approval")) {
      counts.Proposals += 1;
    }

    if (behaviors.includes("pinned") || block.pinnedAt) {
      counts.Pinned += 1;
    }
  }

  return counts;
}

export function filterHeapBlocksByView<T extends HeapViewBlock>(
  blocks: T[],
  viewMode: HeapPrimaryViewMode,
): T[] {
  return blocks.filter((block) => isHeapBlockInView(block, viewMode));
}

export function filterHeapBlocksByReviewLane<T extends HeapViewBlock>(
  blocks: T[],
  reviewLane: HeapReviewLane | null,
): T[] {
  if (!reviewLane) return blocks;
  return blocks.filter((block) => readHeapBlockReviewLane(block) === reviewLane);
}
