import type { HeapBlockListItem } from "../../contracts";

type RelationBlock = Pick<HeapBlockListItem, "projection" | "surfaceJson">;

export interface HeapRelationItem {
  id: string;
  title: string;
  subtitle?: string;
  relations?: string[];
  isNavigable: boolean;
}

export interface HeapRelationIndex {
  outboundLinks: HeapRelationItem[];
  outboundMentions: HeapRelationItem[];
  backlinks: HeapRelationItem[];
  tagNeighbors: HeapRelationItem[];
  semanticLineage: HeapRelationItem[];
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
  relation: string,
  subtitle: string,
): HeapRelationItem {
  const target = resolveHeapRelationBlock(artifactId, allBlocks);
  return {
    id: artifactId,
    title: target?.projection.title ?? artifactId,
    subtitle,
    relations: [relation],
    isNavigable: Boolean(target),
  };
}

export function describeHeapRelation(relation: string): string {
  switch (relation) {
    case "page_link":
      return "page link";
    case "mention":
      return "inline mention";
    case "inbound_page_link":
      return "referenced by page link";
    case "inbound_mention":
      return "referenced by mention";
    case "shared_tag":
      return "shared tag";
    case "parent_artifact":
      return "parent artifact";
    case "artifact":
      return "artifact reference";
    case "prompt_template":
      return "prompt template";
    case "prompt_template_revision":
      return "prompt template revision";
    case "prompt_execution":
      return "prompt execution";
    case "parent_run":
      return "parent run";
    case "run":
      return "run record";
    case "child_run":
      return "child run";
    case "child_artifact":
      return "child artifact";
    default:
      return relation.replace(/_/g, " ");
  }
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }
  return value as Record<string, unknown>;
}

function structuredData(block: RelationBlock): Record<string, unknown> | null {
  const surface = asRecord(block.surfaceJson);
  if (!surface) {
    return null;
  }
  return asRecord(surface.structured_data) ?? asRecord(surface.data) ?? surface;
}

function relationValue(block: RelationBlock, key: string): string | null {
  const data = structuredData(block);
  const value = data?.[key];
  if (typeof value !== "string") {
    return null;
  }
  const normalized = value.trim();
  return normalized.length ? normalized : null;
}

function relationValues(block: RelationBlock, key: string): string[] {
  const data = structuredData(block);
  const value = data?.[key];
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((entry) => (typeof entry === "string" ? entry.trim() : ""))
    .filter(Boolean);
}

export function buildHeapRelationIndex<T extends RelationBlock>(
  block: T,
  allBlocks: T[],
): HeapRelationIndex {
  const artifactId = block.projection.artifactId;
  const tagSet = new Set(block.projection.tags ?? []);

  const outboundLinks = (block.projection.pageLinks ?? []).map((item) =>
    toRelationItem(item, allBlocks, "page_link", "linked from this record"),
  );
  const outboundMentions = (block.projection.mentionsInline ?? []).map((item) =>
    toRelationItem(item, allBlocks, "mention", "mentioned inline in this record"),
  );
  const backlinks = allBlocks.flatMap((candidate) => {
    if (candidate.projection.artifactId === artifactId) return [];
    const relations: HeapRelationItem[] = [];
    if ((candidate.projection.pageLinks ?? []).includes(artifactId)) {
      relations.push({
        id: candidate.projection.artifactId,
        title: candidate.projection.title,
        subtitle: "references this record through a page link",
        relations: ["inbound_page_link"],
        isNavigable: true,
      });
    }
    if ((candidate.projection.mentionsInline ?? []).includes(artifactId)) {
      relations.push({
        id: candidate.projection.artifactId,
        title: candidate.projection.title,
        subtitle: "references this record through an inline mention",
        relations: ["inbound_mention"],
        isNavigable: true,
      });
    }
    return relations;
  });
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
  const semanticLineageMap = new Map<string, HeapRelationItem>();
  const appendSemanticItem = (item: HeapRelationItem) => {
    const existing = semanticLineageMap.get(item.id);
    if (!existing) {
      semanticLineageMap.set(item.id, item);
      return;
    }
    const relations = new Set([...(existing.relations ?? []), ...(item.relations ?? [])]);
    const mergedSubtitle = [existing.subtitle, item.subtitle].filter(Boolean).join(" · ");
    semanticLineageMap.set(item.id, {
      ...existing,
      title: existing.title || item.title,
      subtitle: mergedSubtitle || existing.subtitle || item.subtitle,
      relations: Array.from(relations),
      isNavigable: existing.isNavigable || item.isNavigable,
    });
  };

  const semanticTargets = [
    { key: "parent_artifact_id", relation: "parent_artifact", subtitle: "parent artifact referenced by this record" },
    { key: "artifact_id", relation: "artifact", subtitle: "artifact reference for this record" },
    { key: "prompt_template_artifact_id", relation: "prompt_template", subtitle: "prompt template used for this execution" },
    { key: "prompt_template_revision_id", relation: "prompt_template_revision", subtitle: "prompt template revision used here" },
    { key: "prompt_execution_artifact_id", relation: "prompt_execution", subtitle: "resolved execution snapshot for this prompt" },
    { key: "parent_run_id", relation: "parent_run", subtitle: "parent run in the chain" },
    { key: "run_id", relation: "run", subtitle: "current run record" },
  ] as const;

  for (const target of semanticTargets) {
    const targetId = relationValue(block, target.key);
    if (!targetId || targetId === artifactId) {
      continue;
    }
    appendSemanticItem(toRelationItem(targetId, allBlocks, target.relation, target.subtitle));
  }

  for (const runId of relationValues(block, "child_run_ids")) {
    appendSemanticItem({
      id: runId,
      title: runId,
      subtitle: "child run spawned from this record",
      relations: ["child_run"],
      isNavigable: false,
    });
  }

  for (const candidate of allBlocks) {
    if (candidate.projection.artifactId === artifactId) {
      continue;
    }
    if (relationValue(candidate, "parent_artifact_id") === artifactId) {
      appendSemanticItem({
        id: candidate.projection.artifactId,
        title: candidate.projection.title,
        subtitle: "child artifact that points back to this record",
        relations: ["child_artifact"],
        isNavigable: true,
      });
    }
  }

  const semanticLineage = Array.from(semanticLineageMap.values());

  return {
    outboundLinks,
    outboundMentions,
    backlinks,
    tagNeighbors,
    semanticLineage,
  };
}
