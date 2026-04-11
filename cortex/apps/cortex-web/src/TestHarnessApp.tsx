import React, { Suspense, lazy } from "react";
import { Navigate, Route, Routes } from "react-router-dom";

import { ErrorBoundary } from "./components/commons/ErrorBoundary";

const LayoutCatalogueTestHarness = lazy(() =>
  import("./components/a2ui/catalogue/LayoutCatalogueTestHarness").then((module) => ({
    default: module.LayoutCatalogueTestHarness,
  })),
);
const LayoutRegistryTestHarness = lazy(() =>
  import("./components/a2ui/catalogue/LayoutRegistryTestHarness").then((module) => ({
    default: module.LayoutRegistryTestHarness,
  })),
);

export function TestHarnessApp() {
  return (
    <div className="shell bg-cortex-bg text-cortex-ink min-h-screen">
      <ErrorBoundary>
        <Suspense fallback={<div className="min-h-screen bg-[#020202]" />}>
          <Routes>
            <Route path="/__test/layout-catalogue" element={<LayoutCatalogueTestHarness />} />
            <Route path="/__test/layout-registry" element={<LayoutRegistryTestHarness />} />
            <Route path="*" element={<Navigate to="/__test/layout-catalogue" replace />} />
          </Routes>
        </Suspense>
      </ErrorBoundary>
    </div>
  );
}
