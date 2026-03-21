import type { DpubBlastRadiusResponse } from "../../contracts.ts";

export type ContributionFocusKind =
  | "idle"
  | "agent_run"
  | "graph_run"
  | "contribution";

export interface ContributionSelectionState {
  focusKind: ContributionFocusKind;
  selectedContributionId: string | null;
  selectedAgentRunId: string | null;
  selectedGraphRunId: string | null;
}

export type ContributionFocusRelationship =
  | "dependsOn"
  | "dependedBy"
  | "invalidates"
  | "invalidatedBy"
  | "supersedes"
  | "supersededBy"
  | "references"
  | "referencedBy";

export interface ContributionFocusGraphNode {
  id: string;
  role: "focus" | "related";
}

export interface ContributionFocusGraphEdge {
  source: string;
  target: string;
  relationship: ContributionFocusRelationship;
}

export interface ContributionFocusGraphGroup {
  key: ContributionFocusRelationship;
  label: string;
  items: string[];
}

export interface ContributionFocusGraphModel {
  nodes: ContributionFocusGraphNode[];
  edges: ContributionFocusGraphEdge[];
  groups: ContributionFocusGraphGroup[];
}

const CONTRIBUTION_RELATION_GROUPS: Array<{
  key: ContributionFocusRelationship;
  label: string;
}> = [
  { key: "dependsOn", label: "Depends On" },
  { key: "dependedBy", label: "Depended By" },
  { key: "invalidates", label: "Invalidates" },
  { key: "invalidatedBy", label: "Invalidated By" },
  { key: "supersedes", label: "Supersedes" },
  { key: "supersededBy", label: "Superseded By" },
  { key: "references", label: "References" },
  { key: "referencedBy", label: "Referenced By" },
];

function normalizeToken(value: string | null): string | null {
  const trimmed = value?.trim();
  return trimmed ? trimmed : null;
}

export function normalizeContributionSelection(
  searchParams: URLSearchParams
): ContributionSelectionState {
  const nodeId = normalizeToken(searchParams.get("node_id"));
  const contributionId = normalizeToken(searchParams.get("contribution_id"));
  const graphRunId = normalizeToken(searchParams.get("run_id"));

  if (nodeId?.startsWith("agent_run:")) {
    return {
      focusKind: "agent_run",
      selectedContributionId: contributionId,
      selectedAgentRunId: normalizeToken(nodeId.slice("agent_run:".length)),
      selectedGraphRunId: graphRunId,
    };
  }
  if (nodeId?.startsWith("contribution:")) {
    return {
      focusKind: "contribution",
      selectedContributionId: normalizeToken(nodeId.slice("contribution:".length)),
      selectedAgentRunId: null,
      selectedGraphRunId: graphRunId,
    };
  }
  if (graphRunId) {
    return {
      focusKind: "graph_run",
      selectedContributionId: contributionId,
      selectedAgentRunId: null,
      selectedGraphRunId: graphRunId,
    };
  }
  if (contributionId) {
    return {
      focusKind: "contribution",
      selectedContributionId: contributionId,
      selectedAgentRunId: null,
      selectedGraphRunId: null,
    };
  }
  return {
    focusKind: "idle",
    selectedContributionId: null,
    selectedAgentRunId: null,
    selectedGraphRunId: null,
  };
}

export function pipelineModeRequiresApproval(mode: string): boolean {
  return new Set(["full", "ingest", "doctor", "simulate", "publish"]).has(
    mode.trim().toLowerCase()
  );
}

export function buildContributionFocusGraphModel(
  contributionId: string,
  blastRadius: DpubBlastRadiusResponse | null
): ContributionFocusGraphModel {
  if (!contributionId) {
    return {
      nodes: [],
      edges: [],
      groups: [],
    };
  }

  const nodes = new Map<string, ContributionFocusGraphNode>();
  const edges: ContributionFocusGraphEdge[] = [];
  nodes.set(contributionId, { id: contributionId, role: "focus" });

  for (const relation of CONTRIBUTION_RELATION_GROUPS) {
    const items = [...new Set((blastRadius?.[relation.key] ?? []).filter(Boolean))];
    for (const item of items) {
      nodes.set(item, { id: item, role: "related" });
      if (
        relation.key === "dependedBy" ||
        relation.key === "invalidatedBy" ||
        relation.key === "supersededBy" ||
        relation.key === "referencedBy"
      ) {
        edges.push({
          source: item,
          target: contributionId,
          relationship: relation.key,
        });
      } else {
        edges.push({
          source: contributionId,
          target: item,
          relationship: relation.key,
        });
      }
    }
  }

  return {
    nodes: [...nodes.values()],
    edges,
    groups: CONTRIBUTION_RELATION_GROUPS.map((relation) => ({
      key: relation.key,
      label: relation.label,
      items: [...new Set((blastRadius?.[relation.key] ?? []).filter(Boolean))],
    })).filter((group) => group.items.length > 0),
  };
}
