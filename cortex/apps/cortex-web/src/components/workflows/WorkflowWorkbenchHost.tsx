import React from "react";

import { WorkbenchSurfaceView } from "../commons/WorkbenchSurfaceView";
import { WorkflowArtifactInspector } from "./WorkflowArtifactInspector.tsx";
import {
  WorkflowArtifactInspectorProvider,
  useWorkflowArtifactInspector,
} from "./WorkflowArtifactInspectorContext.tsx";

function WorkflowWorkbenchHostBody() {
  const inspector = useWorkflowArtifactInspector();
  if (!inspector) {
    return <WorkbenchSurfaceView />;
  }

  return (
    <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_440px]">
      <div className="min-w-0 rounded-[22px] border border-cortex-line bg-[linear-gradient(180deg,rgba(9,24,46,0.92),rgba(4,12,26,0.92))] p-4 shadow-[0_20px_56px_rgba(2,8,20,0.35)]">
        <WorkbenchSurfaceView className="workflow-cockpit-surface min-w-0" />
      </div>
      <div className="min-w-0 xl:sticky xl:top-4 xl:self-start">
        <WorkflowArtifactInspector
          state={inspector.state}
          onClose={inspector.closeArtifact}
        />
      </div>
    </div>
  );
}

export function WorkflowWorkbenchHost() {
  return (
    <WorkflowArtifactInspectorProvider>
      <WorkflowWorkbenchHostBody />
    </WorkflowArtifactInspectorProvider>
  );
}
