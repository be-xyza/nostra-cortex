import { Suspense, lazy, useEffect } from "react";
import { useLocation, useNavigate } from "react-router-dom";

const WorkbenchAppShell = lazy(() =>
  import("./WorkbenchAppShell").then((module) => ({ default: module.WorkbenchAppShell })),
);
const TestHarnessApp = lazy(() =>
  import("./TestHarnessApp").then((module) => ({ default: module.TestHarnessApp })),
);

export function App() {
  const navigate = useNavigate();
  const location = useLocation();
  const isPlaywrightHarnessRoute = location.pathname.startsWith("/__test/");

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

  if (isPlaywrightHarnessRoute) {
    return (
      <Suspense fallback={<div className="min-h-screen bg-[#020202]" />}>
        <TestHarnessApp />
      </Suspense>
    );
  }

  return (
    <Suspense fallback={<div className="min-h-screen bg-[#020202]" />}>
      <WorkbenchAppShell />
    </Suspense>
  );
}
