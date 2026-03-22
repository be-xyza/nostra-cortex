import type {
  WorkflowProjectionKind,
  WorkflowProjectionResponse,
} from "../../contracts.ts";

export type WorkflowProjectionTab = {
  key: WorkflowProjectionKind;
  label: string;
};

const FALLBACK_PROJECTION_TABS: WorkflowProjectionTab[] = [
  { key: "flow_graph_v1", label: "Graph" },
  { key: "execution_topology_v1", label: "Topology" },
  { key: "a2ui_surface_v1", label: "A2UI" },
  { key: "serverless_workflow_v0_8", label: "SW" },
  { key: "normalized_graph_v1", label: "Normalized" },
];

export function resolveWorkflowProjectionTabs(
  response: WorkflowProjectionResponse | null,
  activeKind: WorkflowProjectionKind
): WorkflowProjectionTab[] {
  const emittedTabs = Array.isArray(response?.available_projections)
    ? response.available_projections
        .filter(
          (entry): entry is { kind: WorkflowProjectionKind; label: string } =>
            Boolean(
              entry &&
                typeof entry.kind === "string" &&
                typeof entry.label === "string" &&
                entry.label.trim().length > 0
            )
        )
        .map((entry) => ({
          key: entry.kind,
          label: entry.label,
        }))
    : [];

  if (emittedTabs.length > 0) {
    return emittedTabs.some((tab) => tab.key === activeKind)
      ? emittedTabs
      : [...emittedTabs, { key: activeKind, label: activeKind }];
  }

  return FALLBACK_PROJECTION_TABS;
}
