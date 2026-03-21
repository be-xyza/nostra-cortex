import type { WorkflowProjectionKind } from "../../contracts.ts";

export type WorkflowHrefKind = "internal_workbench" | "gateway_api" | "external";

export type WorkflowArtifactPathDescriptor =
  | {
      kind: "proposal_replay";
      proposalId: string;
      path: string;
    }
  | {
      kind: "proposal_digest";
      proposalId: string;
      path: string;
    }
  | {
      kind: "definition";
      definitionId: string;
      path: string;
    }
  | {
      kind: "definition_projection";
      definitionId: string;
      projectionKind: WorkflowProjectionKind;
      path: string;
    }
  | {
      kind: "active_definition";
      scopeKey: string;
      path: string;
    }
  | {
      kind: "instance";
      instanceId: string;
      path: string;
    }
  | {
      kind: "instance_trace";
      instanceId: string;
      path: string;
    }
  | {
      kind: "instance_checkpoints";
      instanceId: string;
      path: string;
    }
  | {
      kind: "instance_outcome";
      instanceId: string;
      path: string;
    };

function trimHash(value: string): string {
  const hashIndex = value.indexOf("#");
  return hashIndex >= 0 ? value.slice(0, hashIndex) : value;
}

export function normalizeWorkflowHref(href: string): string {
  const trimmed = href.trim();
  if (!trimmed) return "";
  try {
    if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
      const url = new URL(trimmed);
      return trimHash(`${url.pathname}${url.search}`);
    }
  } catch {
    return trimHash(trimmed);
  }
  return trimHash(trimmed);
}

export function isGatewayApiPath(path: string): boolean {
  const normalized = normalizeWorkflowHref(path);
  return normalized.startsWith("/api/cortex/workflow-");
}

export function classifyWorkbenchHref(href: string): WorkflowHrefKind {
  const trimmed = href.trim();
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    return "external";
  }
  const normalized = normalizeWorkflowHref(href);
  if (!normalized) return "external";
  if (isGatewayApiPath(normalized)) return "gateway_api";
  if (normalized.startsWith("/")) return "internal_workbench";
  return "external";
}

function decodeSegment(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

export function parseWorkflowArtifactPath(path: string): WorkflowArtifactPathDescriptor | null {
  const normalized = normalizeWorkflowHref(path);
  const proposalMatch = normalized.match(
    /^\/api\/cortex\/workflow-drafts\/proposals\/([^/]+)\/(replay|digest)$/
  );
  if (proposalMatch) {
    const proposalId = decodeSegment(proposalMatch[1] ?? "");
    const artifactKind = proposalMatch[2];
    if (artifactKind === "replay") {
      return { kind: "proposal_replay", proposalId, path: normalized };
    }
    return { kind: "proposal_digest", proposalId, path: normalized };
  }

  const definitionProjectionMatch = normalized.match(
    /^\/api\/cortex\/workflow-definitions\/([^/]+)\/projections\/([^/]+)$/
  );
  if (definitionProjectionMatch) {
    return {
      kind: "definition_projection",
      definitionId: decodeSegment(definitionProjectionMatch[1] ?? ""),
      projectionKind: decodeSegment(
        definitionProjectionMatch[2] ?? ""
      ) as WorkflowProjectionKind,
      path: normalized,
    };
  }

  const definitionMatch = normalized.match(/^\/api\/cortex\/workflow-definitions\/([^/]+)$/);
  if (definitionMatch) {
    return {
      kind: "definition",
      definitionId: decodeSegment(definitionMatch[1] ?? ""),
      path: normalized,
    };
  }

  const activeDefinitionMatch = normalized.match(
    /^\/api\/cortex\/workflow-definitions\/active\/([^/]+)$/
  );
  if (activeDefinitionMatch) {
    return {
      kind: "active_definition",
      scopeKey: decodeSegment(activeDefinitionMatch[1] ?? ""),
      path: normalized,
    };
  }

  const instanceMatch = normalized.match(/^\/api\/cortex\/workflow-instances\/([^/]+)$/);
  if (instanceMatch) {
    return {
      kind: "instance",
      instanceId: decodeSegment(instanceMatch[1] ?? ""),
      path: normalized,
    };
  }

  const instanceTraceMatch = normalized.match(
    /^\/api\/cortex\/workflow-instances\/([^/]+)\/trace$/
  );
  if (instanceTraceMatch) {
    return {
      kind: "instance_trace",
      instanceId: decodeSegment(instanceTraceMatch[1] ?? ""),
      path: normalized,
    };
  }

  const instanceCheckpointMatch = normalized.match(
    /^\/api\/cortex\/workflow-instances\/([^/]+)\/checkpoints$/
  );
  if (instanceCheckpointMatch) {
    return {
      kind: "instance_checkpoints",
      instanceId: decodeSegment(instanceCheckpointMatch[1] ?? ""),
      path: normalized,
    };
  }

  const instanceOutcomeMatch = normalized.match(
    /^\/api\/cortex\/workflow-instances\/([^/]+)\/outcome$/
  );
  if (instanceOutcomeMatch) {
    return {
      kind: "instance_outcome",
      instanceId: decodeSegment(instanceOutcomeMatch[1] ?? ""),
      path: normalized,
    };
  }

  return null;
}
