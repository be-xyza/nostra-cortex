import { useEffect, useState } from "react";
import { Routes, Route, Navigate, useLocation, useNavigate } from "react-router-dom";
import { A2UIInterpreter, A2UINode } from "./components/a2ui/A2UIInterpreter";
import { ShellLayout } from "./components/commons/ShellLayout";
import { ErrorBoundary } from "./components/commons/ErrorBoundary";
import { ExecutionContainmentHeader } from "./components/layout/ExecutionContainmentHeader";
import { useUiStore } from "./store/uiStore";
import { gatewayBaseUrl } from "./api";

type RenderSurface = {
  type: string;
  surfaceId: string;
  title: string;
  components: Array<{
    id: string;
    type: string;
    props: Record<string, unknown>;
    children?: string[];
  }>;
  meta: Record<string, string>;
};

function surfaceToA2UITree(surface: RenderSurface): A2UINode {
  const componentMap = new Map<string, any>();
  surface.components.forEach(c => componentMap.set(c.id, c));

  function buildNode(component: any): A2UINode {
    const resolvedType = (component.props?.widgetType as string) || component.type;
    const explicitList: A2UINode[] = [];
    if (component.children && Array.isArray(component.children)) {
      for (const childId of component.children) {
        const childComponent = componentMap.get(childId);
        if (childComponent) {
          explicitList.push(buildNode(childComponent));
        }
      }
    }
    return {
      id: component.id,
      type: resolvedType,
      componentProperties: {
        [resolvedType]: component.props,
      },
      children: explicitList.length > 0 ? { explicitList } : undefined,
    };
  }

  const allChildren = new Set<string>();
  surface.components.forEach(c => {
    if (c.children && Array.isArray(c.children)) {
      c.children.forEach((id: string) => allChildren.add(id));
    }
  });

  const rootNodes = surface.components
    .filter(c => !allChildren.has(c.id))
    .map(c => buildNode(c));

  return {
    id: surface.surfaceId,
    type: "Column",
    componentProperties: { Column: {} },
    children: {
      explicitList: rootNodes,
    },
  };
}

function A2UIHostView() {
  const location = useLocation();
  const [a2uiNode, setA2uiNode] = useState<A2UINode | null>(null);
  const [error, setError] = useState<string | null>(null);
  const sessionUser = useUiStore((state) => state.sessionUser);

  useEffect(() => {
    setA2uiNode(null);
    setError(null);

    const targetRoute = location.pathname || "/";
    const query = new URLSearchParams();
    query.set("route", targetRoute);
    query.set("space_id", "nostra-governance-v0");
    const contextualParams = new URLSearchParams(location.search);
    const optionalContextKeys = ["intent", "density", "node_id"];
    for (const key of optionalContextKeys) {
      const value = contextualParams.get(key);
      if (value) {
        query.set(key, value);
      }
    }

    fetch(gatewayBaseUrl() + `/api/system/ux/workbench?${query.toString()}`, {
      headers: {
        "x-cortex-actor": sessionUser?.actorId || "unknown",
        "x-cortex-role": sessionUser?.role || "operator"
      }
    })
      .then(res => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json();
      })
      .then((data: RenderSurface) => {
        const tree = surfaceToA2UITree(data);
        setA2uiNode(tree);
      })
      .catch(err => setError(err.message));
  }, [location.pathname, location.search, sessionUser]);

  if (error) {
    return <div className="error-banner m-4">Failed to fetch surface: {error}</div>;
  }

  if (a2uiNode) {
    return <A2UIInterpreter node={a2uiNode} />;
  }

  return <div className="placeholder p-4 text-cortex-ink-muted">Loading Cortex Surface...</div>;
}

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
            <Route path="/" element={<Navigate to="/system" replace />} />
            <Route path="/labs/*" element={
              <ExecutionContainmentHeader surfaceName="Lab Workspace">
                <A2UIHostView />
              </ExecutionContainmentHeader>
            } />
            <Route path="*" element={<A2UIHostView />} />
          </Routes>
        </ErrorBoundary>
      </ShellLayout>
    </div>
  );
}
