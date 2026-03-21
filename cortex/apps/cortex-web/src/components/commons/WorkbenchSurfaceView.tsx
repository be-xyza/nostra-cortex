import { useEffect, useState } from "react";
import { useLocation } from "react-router-dom";

import { gatewayBaseUrl } from "../../api";
import { useUiStore } from "../../store/uiStore";
import { useActiveSpaceContext } from "../../store/spacesRegistry";
import { A2UIInterpreter, A2UINode } from "../a2ui/A2UIInterpreter";

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

export function surfaceToA2UITree(surface: RenderSurface): A2UINode {
  const componentMap = new Map<string, RenderSurface["components"][number]>();
  surface.components.forEach((component) => componentMap.set(component.id, component));

  function buildNode(component: RenderSurface["components"][number]): A2UINode {
    const resolvedType = (component.props?.widgetType as string) || component.type;
    const explicitList: A2UINode[] = [];
    if (component.children && Array.isArray(component.children)) {
      for (const childId of component.children) {
        const childComponent = componentMap.get(childId);
        if (childComponent) explicitList.push(buildNode(childComponent));
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
  surface.components.forEach((component) => {
    if (component.children && Array.isArray(component.children)) {
      component.children.forEach((id) => allChildren.add(id));
    }
  });

  const rootNodes = surface.components
    .filter((component) => !allChildren.has(component.id))
    .map((component) => buildNode(component));

  return {
    id: surface.surfaceId,
    type: "Column",
    componentProperties: { Column: {} },
    children: {
      explicitList: rootNodes,
    },
  };
}

export function WorkbenchSurfaceView({
  routeOverride,
  className,
}: {
  routeOverride?: string;
  className?: string;
}) {
  const location = useLocation();
  const activeSpaceId = useActiveSpaceContext();
  const sessionUser = useUiStore((state) => state.sessionUser);
  const [a2uiNode, setA2uiNode] = useState<A2UINode | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setA2uiNode(null);
    setError(null);

    const targetRoute = routeOverride || location.pathname || "/";
    const query = new URLSearchParams();
    query.set("route", targetRoute);
    query.set("space_id", activeSpaceId);
    const contextualParams = new URLSearchParams(location.search);
    const optionalContextKeys = ["intent", "density", "node_id", "run_id", "contribution_id"];
    for (const key of optionalContextKeys) {
      const value = contextualParams.get(key);
      if (value) query.set(key, value);
    }

    fetch(gatewayBaseUrl() + `/api/system/ux/workbench?${query.toString()}`, {
      headers: {
        "x-cortex-actor": sessionUser?.actorId || "unknown",
        "x-cortex-role": sessionUser?.role || "operator",
      },
    })
      .then((response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        return response.json();
      })
      .then((data: RenderSurface) => {
        setA2uiNode(surfaceToA2UITree(data));
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : String(err));
      });
  }, [location.pathname, location.search, routeOverride, sessionUser]);

  if (error) {
    return <div className="error-banner m-4">Failed to fetch surface: {error}</div>;
  }

  return (
    <div className={className}>
      {a2uiNode ? (
        <A2UIInterpreter node={a2uiNode} />
      ) : (
        <div className="placeholder p-4 text-cortex-ink-muted">Loading Cortex Surface...</div>
      )}
    </div>
  );
}
