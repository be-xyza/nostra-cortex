import { useEffect } from "react";
import { Routes, Route, Navigate, useNavigate } from "react-router-dom";
import { ShellLayout } from "./components/commons/ShellLayout";
import { ErrorBoundary } from "./components/commons/ErrorBoundary";
import { ExecutionContainmentHeader } from "./components/layout/ExecutionContainmentHeader";
import { WorkbenchSurfaceView } from "./components/commons/WorkbenchSurfaceView";
import { ArtifactsWorkbenchHost } from "./components/artifacts/ArtifactsWorkbenchHost";
import { WorkflowWorkbenchHost } from "./components/workflows/WorkflowWorkbenchHost";
import { HeapBlockGrid } from "./components/heap/HeapBlockGrid";
import { SpacesPage } from "./components/spaces/SpacesPage";
import { SpaceDetailPage } from "./components/spaces/SpaceDetailPage";
import { SpaceStudioPage } from "./components/spaces/SpaceStudioPage";
import { ExecutionCanvasPage } from "./components/spaces/ExecutionCanvasPage";
import { EXECUTION_CANVAS_ROUTE, SPACE_STUDIO_ROUTE } from "./components/spaces/spaceStudioRoutes";
import { ContributionsWorkbenchHost } from "./components/contributions/ContributionsWorkbenchHost";
import { LogsPage } from "./components/live/LogsPage";
import { ProviderDashboard } from "./components/system/ProviderDashboard";

export function App() {
  const navigate = useNavigate();

  // Global Nostra Scheme Interceptor
  useEffect(() => {
    const handleGlobalClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      const anchor = target.closest('a');

      if (anchor && anchor.href) {
        try {
          const url = new URL(anchor.href);
          if (url.protocol === 'nostra:') {
            e.preventDefault();
            // Translate nostra://resource/type/id to /type/id locally
            const routePath = url.pathname + url.search;
            navigate(routePath);
          }
        } catch (err) {
          // Ignore invalid URLs
        }
      }
    };

    document.addEventListener('click', handleGlobalClick);
    return () => document.removeEventListener('click', handleGlobalClick);
  }, [navigate]);

  return (
    <div className="shell bg-cortex-bg text-cortex-ink min-h-screen">
      <ShellLayout>
        <ErrorBoundary>
          <Routes>
            <Route path="/" element={<Navigate to="/explore" replace />} />
            <Route path="/artifacts/*" element={<ArtifactsWorkbenchHost />} />
            <Route path="/workflows/*" element={<WorkflowWorkbenchHost />} />
            <Route path="/contributions/*" element={<ContributionsWorkbenchHost />} />
            <Route path={SPACE_STUDIO_ROUTE} element={
              <ExecutionContainmentHeader surfaceName="Space Studio">
                <SpaceStudioPage />
              </ExecutionContainmentHeader>
            } />
            <Route path={EXECUTION_CANVAS_ROUTE} element={
              <ExecutionContainmentHeader surfaceName="Execution Canvas">
                <ExecutionCanvasPage />
              </ExecutionContainmentHeader>
            } />
            <Route path="/labs/*" element={
              <ExecutionContainmentHeader surfaceName="Labs">
                <WorkbenchSurfaceView />
              </ExecutionContainmentHeader>
            } />
            <Route path="/spaces" element={<SpacesPage />} />
            <Route path="/spaces/:id" element={<SpaceDetailPage />} />
            <Route path="/explore" element={<HeapBlockGrid showFilterSidebar={true} />} />
            <Route path="/logs" element={<LogsPage />} />
            <Route path="/system/providers" element={
              <ExecutionContainmentHeader surfaceName="System Administration">
                <ProviderDashboard />
              </ExecutionContainmentHeader>
            } />
            <Route path="/heap" element={<Navigate to="/explore" replace />} />
            <Route path="*" element={<WorkbenchSurfaceView />} />
          </Routes>
        </ErrorBoundary>
      </ShellLayout>
    </div>
  );
}
