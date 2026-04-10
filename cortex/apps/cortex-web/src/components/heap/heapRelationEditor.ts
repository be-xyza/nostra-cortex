import type { EmitHeapBlockRequest, HeapBlockListItem } from "../../contracts";

export interface HeapRelationDraftMention {
  artifactId: string;
  label?: string;
}

export interface HeapRelationDraft {
  tags: string[];
  mentions: HeapRelationDraftMention[];
  pageLinks: string[];
}

interface BuildHeapRelationUpsertRequestArgs {
  block: HeapBlockListItem;
  relationDraft: HeapRelationDraft;
  emittedAt?: string;
  agentId?: string;
}

export function createInitialHeapRelationDraft(
  block: Pick<HeapBlockListItem, "projection">,
): HeapRelationDraft {
  return {
    tags: [...(block.projection.tags ?? [])],
    mentions: (block.projection.mentionsInline ?? []).map((artifactId) => ({
      artifactId,
      label: artifactId,
    })),
    pageLinks: [...(block.projection.pageLinks ?? [])],
  };
}

export function buildHeapRelationUpsertRequest({
  block,
  relationDraft,
  emittedAt = new Date().toISOString(),
  agentId = "cortex-web",
}: BuildHeapRelationUpsertRequestArgs): EmitHeapBlockRequest {
  const surface = (block.surfaceJson ?? {}) as Record<string, unknown>;
  const projection = block.projection as unknown as Record<string, unknown>;
  const projectionWorkspaceId =
    typeof projection.workspaceId === "string" ? projection.workspaceId : undefined;

  return {
    schema_version: "1.0.0",
    mode: "heap",
    space_id:
      block.projection.spaceId
      ?? projectionWorkspaceId
      ?? (typeof surface.space_id === "string" ? surface.space_id : ""),
    source: {
      agent_id: agentId,
      emitted_at: emittedAt,
    },
    block: {
      id: block.projection.blockId,
      type: block.projection.blockType,
      title: block.projection.title,
      attributes: block.projection.attributes,
    },
    content: deriveHeapBlockContent(block),
    relations: {
      tags: dedupeStrings(relationDraft.tags).map((toBlockId) => ({
        to_block_id: toBlockId,
      })),
      mentions: dedupeMentions(relationDraft.mentions).map((mention) => ({
        to_block_id: mention.artifactId,
        label: mention.label,
      })),
      page_links: dedupeStrings(relationDraft.pageLinks).map((toBlockId) => ({
        to_block_id: toBlockId,
      })),
    },
    crdt_projection: {
      artifact_id: block.projection.artifactId,
    },
  };
}

export function buildMinimalHeapBlockRequest(
  artifactId: string,
  spaceId: string,
  emittedAt = new Date().toISOString(),
  agentId = "cortex-web",
): EmitHeapBlockRequest {
  return {
    schema_version: "1.0.0",
    mode: "heap",
    space_id: spaceId,
    source: {
      agent_id: agentId,
      emitted_at: emittedAt,
    },
    block: {
      type: "note",
      title: artifactId,
    },
    content: {
      payload_type: "rich_text",
      rich_text: {
        plain_text: `Placeholder block for ${artifactId}`,
      },
    },
    crdt_projection: {
      artifact_id: artifactId,
    },
  };
}

function dedupeStrings(values: string[]): string[] {
  const seen = new Set<string>();
  const next: string[] = [];

  for (const value of values) {
    const normalized = value.trim();
    if (!normalized || seen.has(normalized)) continue;
    seen.add(normalized);
    next.push(normalized);
  }

  return next;
}

function dedupeMentions(values: HeapRelationDraftMention[]): HeapRelationDraftMention[] {
  const seen = new Set<string>();
  const next: HeapRelationDraftMention[] = [];

  for (const value of values) {
    const artifactId = value.artifactId.trim();
    if (!artifactId || seen.has(artifactId)) continue;
    seen.add(artifactId);
    next.push({
      artifactId,
      label: value.label?.trim() || artifactId,
    });
  }

  return next;
}

function deriveHeapBlockContent(
  block: HeapBlockListItem,
): EmitHeapBlockRequest["content"] {
  const surface = (block.surfaceJson ?? {}) as Record<string, unknown>;

  if (typeof surface.pointer === "string" && surface.pointer.trim()) {
    return {
      payload_type: "pointer",
      pointer: surface.pointer,
    };
  }

  if (surface.structured_data && typeof surface.structured_data === "object") {
    return {
      payload_type: "structured_data",
      structured_data: surface.structured_data as Record<string, unknown>,
    };
  }

  if (surface.data && typeof surface.data === "object") {
    return {
      payload_type: "structured_data",
      structured_data: surface.data as Record<string, unknown>,
    };
  }

  const plainText = extractPlainTextCandidate(surface, block.projection.title);
  return {
    payload_type: "rich_text",
    rich_text: {
      plain_text: plainText,
    },
  };
}

function extractPlainTextCandidate(
  surface: Record<string, unknown>,
  titleFallback: string,
): string {
  if (typeof surface.plain_text === "string" && surface.plain_text.trim()) {
    return surface.plain_text;
  }
  if (typeof surface.text === "string" && surface.text.trim()) {
    return surface.text;
  }

  const components = Array.isArray(surface.components) ? surface.components : [];
  const description = components
    .map((component) => {
      if (!component || typeof component !== "object") return null;
      const props = (component as Record<string, unknown>).props;
      if (!props || typeof props !== "object") return null;
      const value = (props as Record<string, unknown>).description;
      return typeof value === "string" && value.trim() ? value : null;
    })
    .find(Boolean);

  if (description) {
    return description;
  }

  return titleFallback || "Heap block";
}
