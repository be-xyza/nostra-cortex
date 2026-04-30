import React from "react";
import { Navigate, Route, Routes } from "react-router-dom";

import { ArtifactsWorkbenchHost } from "./components/artifacts/ArtifactsWorkbenchHost";
import { LayoutMatrixCatalogue } from "./components/a2ui/catalogue/LayoutMatrixCatalogue";
import { ErrorBoundary } from "./components/commons/ErrorBoundary";
import { ShellLayout } from "./components/commons/ShellLayout";
import { WorkbenchSurfaceView } from "./components/commons/WorkbenchSurfaceView";
import { ContributionsWorkbenchHost } from "./components/contributions/ContributionsWorkbenchHost";
import { ConversationsPage } from "./components/conversations/ConversationsPage";
import { HeapBlockGrid } from "./components/heap/HeapBlockGrid";
import { InboxPage } from "./components/inbox/InboxPage";
import { LogsPage } from "./components/live/LogsPage";
import { ExecutionContainmentHeader } from "./components/layout/ExecutionContainmentHeader";
import { SpaceDetailPage } from "./components/spaces/SpaceDetailPage";
import { ExecutionCanvasPage } from "./components/spaces/ExecutionCanvasPage";
import { SpacesPage } from "./components/spaces/SpacesPage";
import { SpaceStudioPage } from "./components/spaces/SpaceStudioPage";
import {
  EXECUTION_CANVAS_ROUTE,
  SPACE_STUDIO_ROUTE,
} from "./components/spaces/spaceStudioRoutes";
import { ProviderDashboard } from "./components/system/ProviderDashboard";
import { WorkflowWorkbenchHost } from "./components/workflows/WorkflowWorkbenchHost";

export function WorkbenchAppShell() {
  return (
    <div className="shell bg-cortex-bg text-cortex-ink min-h-screen">
      <ShellLayout>
        <ErrorBoundary>
          <Routes>
            <Route path="/" element={<Navigate to="/explore" replace />} />
            <Route path="/artifacts/*" element={<ArtifactsWorkbenchHost />} />
            <Route path="/workflows/*" element={<WorkflowWorkbenchHost />} />
            <Route path="/contributions/*" element={<ContributionsWorkbenchHost />} />
            <Route
              path={SPACE_STUDIO_ROUTE}
              element={
                <ExecutionContainmentHeader surfaceName="Space Studio">
                  <SpaceStudioPage />
                </ExecutionContainmentHeader>
              }
            />
            <Route
              path={EXECUTION_CANVAS_ROUTE}
              element={
                <ExecutionContainmentHeader surfaceName="Execution Canvas">
                  <ExecutionCanvasPage />
                </ExecutionContainmentHeader>
              }
            />
            <Route
              path="/labs/layout-catalogue"
              element={
                <ExecutionContainmentHeader surfaceName="A2UI Layout Catalogue">
                  <LayoutMatrixCatalogue />
                </ExecutionContainmentHeader>
              }
            />
            <Route
              path="/labs/*"
              element={
                <ExecutionContainmentHeader surfaceName="Labs">
                  <WorkbenchSurfaceView />
                </ExecutionContainmentHeader>
              }
            />
            <Route path="/spaces" element={<SpacesPage />} />
            <Route path="/spaces/:id" element={<SpaceDetailPage />} />
            <Route path="/inbox" element={<InboxPage />} />
            <Route path="/conversations" element={<ConversationsPage />} />
            <Route path="/explore" element={<HeapBlockGrid showFilterSidebar={true} />} />
            <Route path="/logs" element={<LogsPage />} />
            <Route
              path="/system/providers"
              element={
                <ExecutionContainmentHeader surfaceName="System Administration">
                  <ProviderDashboard />
                </ExecutionContainmentHeader>
              }
            />
            <Route path="/heap" element={<Navigate to="/explore" replace />} />
            <Route path="*" element={<WorkbenchSurfaceView />} />
          </Routes>
        </ErrorBoundary>
      </ShellLayout>
    </div>
  );
}
