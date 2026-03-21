import React from "react";

import { useLocation } from "react-router-dom";
import { openGatewayApiArtifact } from "../../api";

export type WorkflowArtifactInspectorState = {
  open: boolean;
  path: string | null;
  payload: unknown;
  loading: boolean;
  error: string | null;
};

type WorkflowArtifactInspectorContextValue = {
  state: WorkflowArtifactInspectorState;
  openArtifact: (path: string) => Promise<void>;
  closeArtifact: () => void;
};

const WorkflowArtifactInspectorContext =
  React.createContext<WorkflowArtifactInspectorContextValue | null>(null);

export function WorkflowArtifactInspectorProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const location = useLocation();
  const [state, setState] = React.useState<WorkflowArtifactInspectorState>({
    open: false,
    path: null,
    payload: null,
    loading: false,
    error: null,
  });

  const openArtifact = React.useCallback(async (path: string) => {
    setState({
      open: true,
      path,
      payload: null,
      loading: true,
      error: null,
    });
    try {
      const payload = await openGatewayApiArtifact(path, "inline");
      setState({
        open: true,
        path,
        payload,
        loading: false,
        error: null,
      });
    } catch (error) {
      setState({
        open: true,
        path,
        payload: null,
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }, []);

  const closeArtifact = React.useCallback(() => {
    setState({
      open: false,
      path: null,
      payload: null,
      loading: false,
      error: null,
    });
  }, []);

  // Sync inspector with node_id query parameter
  // Use a ref to track the last URL-synced nodeId to avoid re-triggering when
  // openArtifact is called programmatically (e.g. from timeline button clicks)
  const lastSyncedNodeIdRef = React.useRef<string | null>(null);
  
  React.useEffect(() => {
    const params = new URLSearchParams(location.search);
    const nodeId = params.get("node_id");
    
    // Only react when nodeId actually changes in the URL
    if (nodeId === lastSyncedNodeIdRef.current) return;
    lastSyncedNodeIdRef.current = nodeId;
    
    if (!nodeId) {
      closeArtifact();
      return;
    }

    // Map common node_id prefixes to gateway API paths
    let resolvedPath = nodeId.startsWith("/api/")
      ? nodeId
      : `/api/cortex/${nodeId.replace(":", "/")}`;

    // Normalize known prefixes to match gateway requirements (plural + hyphen)
    // Use regex to catch all variations and ensure pluralization
    resolvedPath = resolvedPath
      .replace(/workflow_definition(s)?/g, "workflow-definitions")
      .replace(/workflow_instance(s)?/g, "workflow-instances");

    void openArtifact(resolvedPath);
  }, [location.search, openArtifact, closeArtifact]);

  return (
    <WorkflowArtifactInspectorContext.Provider
      value={{ state, openArtifact, closeArtifact }}
    >
      {children}
    </WorkflowArtifactInspectorContext.Provider>
  );
}

export function useWorkflowArtifactInspector() {
  return React.useContext(WorkflowArtifactInspectorContext);
}
