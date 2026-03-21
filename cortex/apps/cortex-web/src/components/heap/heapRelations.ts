import type { HeapBlockListItem } from "../../contracts";

type RelationBlock = Pick<HeapBlockListItem, "projection">;

export interface HeapRelationItem {
  id: string;
  title: string;
  subtitle?: string;
  isNavigable: boolean;
}

export interface HeapRelationIndex {
  outboundLinks: HeapRelationItem[];
  outboundMentions: HeapRelationItem[];
  backlinks: HeapRelationItem[];
  tagNeighbors: HeapRelationItem[];
}

export function resolveHeapRelationBlock<T extends RelationBlock>(
  artifactId: string,
  allBlocks: T[],
): T | null {
  return allBlocks.find((candidate) => candidate.projection.artifactId === artifactId) ?? null;
}

function toRelationItem<T extends RelationBlock>(
  artifactId: string,
  allBlocks: T[],
  subtitle: string,
): HeapRelationItem {
  const target = resolveHeapRelationBlock(artifactId, allBlocks);
  return {
    id: artifactId,
    title: target?.projection.title ?? artifactId,
    subtitle,
    isNavigable: Boolean(target),
  };
}

export function buildHeapRelationIndex<T extends RelationBlock>(
  block: T,
  allBlocks: T[],
): HeapRelationIndex {
  const artifactId = block.projection.artifactId;
  const tagSet = new Set(block.projection.tags ?? []);

  const outboundLinks = (block.projection.pageLinks ?? []).map((item) =>
    toRelationItem(item, allBlocks, "page link"),
  );
  const outboundMentions = (block.projection.mentionsInline ?? []).map((item) =>
    toRelationItem(item, allBlocks, "@ mention"),
  );
  const backlinks = allBlocks
    .filter((candidate) => {
      if (candidate.projection.artifactId === artifactId) return false;
      return (candidate.projection.pageLinks ?? []).includes(artifactId)
        || (candidate.projection.mentionsInline ?? []).includes(artifactId);
    })
    .map((candidate) => ({
      id: candidate.projection.artifactId,
      title: candidate.projection.title,
      subtitle: candidate.projection.blockType,
      isNavigable: true,
    }));
  const tagNeighbors = allBlocks
    .filter((candidate) => {
      if (candidate.projection.artifactId === artifactId) return false;
      return (candidate.projection.tags ?? []).some((tag) => tagSet.has(tag));
    })
    .map((candidate) => ({
      id: candidate.projection.artifactId,
      title: candidate.projection.title,
      subtitle: (candidate.projection.tags ?? []).join(", "),
      isNavigable: true,
    }));

  return {
    outboundLinks,
    outboundMentions,
    backlinks,
    tagNeighbors,
  };
}
