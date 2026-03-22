import React from "react";

import { workbenchApi } from "../../api";
import type {
  WorkflowDefinitionResponse,
  WorkflowProjectionKind,
  WorkflowProjectionResponse,
} from "../../contracts.ts";
import { resolveWorkflowProjectionTabs } from "./workflowProjectionTabs.ts";
import { EvaluationDAGViewer } from "../evaluation/EvaluationDAGViewer.tsx";
import { WorkflowTopology } from "../../contracts.ts";

function readDefinitionField(
  response: WorkflowDefinitionResponse | null,
  field: string
): string | null {
  if (!response?.definition || typeof response.definition !== "object") return null;
  const value = (response.definition as Record<string, unknown>)[field];
  return typeof value === "string" && value.trim().length > 0 ? value : null;
}

export function WorkflowDefinitionProjectionTabs({
  definitionId,
  initialProjectionKind,
  initialProjection,
}: {
  definitionId: string;
  initialProjectionKind: WorkflowProjectionKind;
  initialProjection: WorkflowProjectionResponse | null;
}) {
  const [activeTab, setActiveTab] =
    React.useState<WorkflowProjectionKind>(initialProjectionKind);
  const [definition, setDefinition] = React.useState<WorkflowDefinitionResponse | null>(null);
  const [projections, setProjections] = React.useState<
    Partial<Record<WorkflowProjectionKind, WorkflowProjectionResponse>>
  >(initialProjection ? { [initialProjectionKind]: initialProjection } : {});
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    let cancelled = false;
    void workbenchApi
      .getWorkflowDefinition(definitionId)
      .then((payload) => {
        if (!cancelled) setDefinition(payload);
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [definitionId]);

  React.useEffect(() => {
    if (projections[activeTab]) return;
    let cancelled = false;
    setLoading(true);
    setError(null);
    void workbenchApi
      .getWorkflowDefinitionProjection(definitionId, activeTab)
      .then((payload) => {
        if (cancelled) return;
        setProjections((current) => ({ ...current, [activeTab]: payload }));
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [activeTab, definitionId, projections]);

  const activeProjection = projections[activeTab] || null;
  const projectionTabs = resolveWorkflowProjectionTabs(activeProjection, activeTab);
  const digest = readDefinitionField(definition, "digest");
  const motifKind = readDefinitionField(definition, "motif_kind");

  return (
    <div className="flex flex-col gap-4">
      <div className="rounded-cortex border border-cortex-line bg-cortex-bg px-3 py-3">
        <div className="text-[11px] uppercase tracking-[0.18em] text-cortex-ink-faint">
          Definition
        </div>
        <div className="mt-2 flex flex-wrap gap-3 text-sm text-cortex-ink-muted">
          <span className="font-medium text-cortex-ink">{definitionId}</span>
          {motifKind ? <span>motif={motifKind}</span> : null}
          {digest ? <span>digest={digest}</span> : null}
        </div>
      </div>
      <div className="flex flex-wrap gap-2">
        {projectionTabs.map((tab) => (
          <button
            key={tab.key}
            className={`rounded-full border px-3 py-2 text-xs uppercase tracking-[0.18em] ${
              activeTab === tab.key
                ? "border-cortex-accent bg-cortex-bg text-cortex-ink"
                : "border-cortex-line bg-cortex-bg-panel text-cortex-ink-muted"
            }`}
            onClick={() => setActiveTab(tab.key)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      {error ? <div className="error-banner">{error}</div> : null}
      {loading && !activeProjection ? (
        <div className="rounded-cortex border border-dashed border-cortex-line bg-cortex-bg px-3 py-4 text-sm text-cortex-ink-faint">
          Loading {activeTab} projection...
        </div>
      ) : null}
      {activeProjection ? (
        activeTab === "execution_topology_v1" ? (
          <EvaluationDAGViewer 
            topology={activeProjection.projection as unknown as WorkflowTopology} 
            className="min-h-[600px] border border-cortex-line rounded-cortex"
          />
        ) : (
          <pre className="rounded-cortex border border-cortex-line bg-[#051325] p-4 text-xs text-cortex-ink-muted overflow-auto">
            {JSON.stringify(activeProjection.projection, null, 2)}
          </pre>
        )
      ) : null}
    </div>
  );
}
