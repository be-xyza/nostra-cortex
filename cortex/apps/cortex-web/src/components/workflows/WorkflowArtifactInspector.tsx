import React from "react";

import { openGatewayApiArtifact } from "../../api";
import type {
  WorkflowCheckpointResponse,
  WorkflowDefinitionResponse,
  WorkflowDigestResponse,
  WorkflowProjectionResponse,
  WorkflowTraceResponse,
} from "../../contracts.ts";
import {
  parseWorkflowArtifactPath,
  type WorkflowArtifactPathDescriptor,
} from "./artifactRouting.ts";
import type { WorkflowArtifactInspectorState } from "./WorkflowArtifactInspectorContext.tsx";
import { WorkflowCheckpointPanel } from "./WorkflowCheckpointPanel.tsx";
import { WorkflowDefinitionProjectionTabs } from "./WorkflowDefinitionProjectionTabs.tsx";
import { WorkflowInstanceTracePanel } from "./WorkflowInstanceTracePanel.tsx";
import { WorkflowReplayDigestPanel } from "./WorkflowReplayDigestPanel.tsx";

function prettyTitle(descriptor: WorkflowArtifactPathDescriptor | null): string {
  if (!descriptor) return "Workflow Artifact Inspector";
  switch (descriptor.kind) {
    case "proposal_replay":
      return `Proposal Replay · ${descriptor.proposalId}`;
    case "proposal_digest":
      return `Proposal Digest · ${descriptor.proposalId}`;
    case "definition":
      return `Workflow Definition · ${descriptor.definitionId}`;
    case "definition_projection":
      return `Definition Projection · ${descriptor.definitionId}`;
    case "active_definition":
      return `Active Scope · ${descriptor.scopeKey}`;
    case "instance":
      return `Workflow Instance · ${descriptor.instanceId}`;
    case "instance_trace":
      return `Instance Trace · ${descriptor.instanceId}`;
    case "instance_checkpoints":
      return `Instance Checkpoints · ${descriptor.instanceId}`;
    case "instance_outcome":
      return `Instance Outcome · ${descriptor.instanceId}`;
  }
}

function GenericArtifactPanel({ payload }: { payload: unknown }) {
  return (
    <pre className="rounded-cortex border border-cortex-line bg-[#051325] p-4 text-xs text-cortex-ink-muted overflow-auto">
      {JSON.stringify(payload, null, 2)}
    </pre>
  );
}

function ArtifactBody({
  descriptor,
  payload,
}: {
  descriptor: WorkflowArtifactPathDescriptor | null;
  payload: unknown;
}) {
  if (!descriptor) return <GenericArtifactPanel payload={payload} />;

  if (descriptor.kind === "definition_projection") {
    return (
      <WorkflowDefinitionProjectionTabs
        definitionId={descriptor.definitionId}
        initialProjectionKind={descriptor.projectionKind}
        initialProjection={payload as WorkflowProjectionResponse}
      />
    );
  }
  if (descriptor.kind === "instance_trace") {
    return <WorkflowInstanceTracePanel response={payload as WorkflowTraceResponse} />;
  }
  if (descriptor.kind === "instance_checkpoints") {
    return <WorkflowCheckpointPanel response={payload as WorkflowCheckpointResponse} />;
  }
  if (descriptor.kind === "proposal_replay") {
    return <WorkflowReplayDigestPanel title="Replay Artifact" payload={payload} />;
  }
  if (descriptor.kind === "proposal_digest") {
    return (
      <WorkflowReplayDigestPanel
        title="Digest Artifact"
        payload={(payload as WorkflowDigestResponse).digest ?? payload}
      />
    );
  }
  if (descriptor.kind === "definition") {
    const definitionResponse = payload as WorkflowDefinitionResponse;
    return <GenericArtifactPanel payload={definitionResponse.definition ?? payload} />;
  }
  return <GenericArtifactPanel payload={payload} />;
}

export function WorkflowArtifactInspector({
  state,
  onClose,
}: {
  state: WorkflowArtifactInspectorState;
  onClose: () => void;
}) {
  const descriptor = state.path ? parseWorkflowArtifactPath(state.path) : null;
  return (
    <aside className="workflow-artifact-inspector rounded-[22px] border border-cortex-line bg-[linear-gradient(180deg,rgba(9,24,46,0.96),rgba(4,12,26,0.98))] shadow-[0_24px_64px_rgba(2,8,20,0.45)]">
      <div className="border-b border-cortex-line px-5 py-4">
        <div className="flex items-start justify-between gap-3">
          <div>
            <div className="text-[11px] uppercase tracking-[0.24em] text-cortex-ink-faint">
              Workflow Artifact Inspector
            </div>
            <h2 className="mt-2 text-lg font-semibold text-cortex-ink">
              {prettyTitle(descriptor)}
            </h2>
            <p className="mt-1 text-sm text-cortex-ink-muted">
              Inline artifact inspection for workflow governance and runtime evidence.
            </p>
          </div>
          <div className="flex gap-2">
            {state.path ? (
              <button
                className="rounded-full border border-cortex-line bg-cortex-bg px-3 py-2 text-xs uppercase tracking-[0.18em]"
                onClick={() => {
                  void openGatewayApiArtifact(state.path!, "new_tab");
                }}
              >
                Open Raw
              </button>
            ) : null}
            <button
              className="rounded-full border border-cortex-line bg-cortex-bg px-3 py-2 text-xs uppercase tracking-[0.18em]"
              onClick={onClose}
            >
              Close
            </button>
          </div>
        </div>
      </div>
      <div className="flex max-h-[calc(100vh-13rem)] min-h-[360px] flex-col gap-4 overflow-auto px-5 py-4">
        {!state.open || !state.path ? (
          <div className="rounded-[18px] border border-dashed border-cortex-line bg-cortex-bg px-4 py-5 text-sm text-cortex-ink-faint">
            Select a workflow replay, digest, projection, trace, or checkpoint action from
            the `/workflows` surface to inspect it here.
          </div>
        ) : null}
        {state.loading ? (
          <div className="rounded-[18px] border border-dashed border-cortex-line bg-cortex-bg px-4 py-5 text-sm text-cortex-ink-faint">
            Loading artifact from {state.path}...
          </div>
        ) : null}
        {state.error ? <div className="error-banner">{state.error}</div> : null}
        {state.payload && !state.loading ? (
          <ArtifactBody descriptor={descriptor} payload={state.payload} />
        ) : null}
      </div>
    </aside>
  );
}
