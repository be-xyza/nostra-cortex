import React from "react";
import { WorkbenchSurfaceView } from "../commons/WorkbenchSurfaceView";
import { WorkflowArtifactInspector } from "../workflows/WorkflowArtifactInspector";
import {
  WorkflowArtifactInspectorProvider,
  useWorkflowArtifactInspector,
} from "../workflows/WorkflowArtifactInspectorContext";

function ArtifactsWorkbenchHostBody() {
  const inspector = useWorkflowArtifactInspector();
  if (!inspector) {
    return <WorkbenchSurfaceView routeOverride="/artifacts" />;
  }

  return (
    <>
      <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
        <WorkbenchSurfaceView routeOverride="/artifacts" className="artifact-cockpit-surface min-w-0 flex-1" />
      </div>
      {Boolean(inspector.state.open) && (
        <WorkflowArtifactInspector state={inspector.state} onClose={inspector.closeArtifact} />
      )}
    </>
  );
}

export function ArtifactsWorkbenchHost() {
  return (
    <WorkflowArtifactInspectorProvider>
      <ArtifactsWorkbenchHostBody />
    </WorkflowArtifactInspectorProvider>
  );
}
