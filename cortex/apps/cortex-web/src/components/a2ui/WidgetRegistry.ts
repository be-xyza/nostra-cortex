import React from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { TldrawCanvas } from "./TldrawCanvas";
import { SpatialPlanePayload } from "./spatialReplay";
import { emitA2uiEvent } from "./spatialEventContract";
import { SPACE_ID, workbenchApi, gatewayBaseUrl } from "../../api";
import { CapabilityMatrixMap } from "../CapabilityMatrixMap";
import { PlatformCapabilityGraph } from "../../contracts";
import { HeapBlockCard as LegacyHeapBlockCard } from "./HeapBlockCard";
import { A2UISynthesisSpace } from "./A2UISynthesisSpace";
import { HeapBlockGrid } from "../heap/HeapBlockGrid";
import { useUiStore } from "../../store/uiStore";

export type A2UIComponentProps = {
  id: string;
  componentProperties: Record<string, unknown>;
  children?: React.ReactNode;
};

function readProps(componentProperties: Record<string, unknown>, componentType: string): Record<string, unknown> {
  const typed = componentProperties[componentType];
  if (typed && typeof typed === "object" && !Array.isArray(typed)) {
    return typed as Record<string, unknown>;
  }
  return componentProperties;
}

type ActionExecutionResult = {
  ok: boolean;
  message: string;
  code: string;
};

function parseActionDescriptor(action: string): { name: string; params: URLSearchParams } {
  const [name, query = ""] = action.split("?", 2);
  return {
    name,
    params: new URLSearchParams(query)
  };
}

async function executeWidgetAction(action: unknown, props: Record<string, unknown>): Promise<ActionExecutionResult> {
  if (typeof action !== "string" || !action.trim()) {
    return {
      ok: false,
      message: "Unsupported action: missing action descriptor.",
      code: "unsupported_action_missing_descriptor"
    };
  }

  const { name, params } = parseActionDescriptor(action.trim());
  type ActionHandler = (params: URLSearchParams, props: Record<string, unknown>) => Promise<ActionExecutionResult>;
  const actionHandlers: Record<string, ActionHandler> = {
    provisionSpace: async (_queryParams, actionProps) => {
      const initialSpaceId = typeof actionProps.space_id === "string" ? actionProps.space_id : "";
      const enteredSpaceId = window.prompt("Space identifier", initialSpaceId);
      const space_id = (enteredSpaceId ?? "").trim();
      if (!space_id) {
        return {
          ok: false,
          message: "Space provisioning cancelled: no space identifier provided.",
          code: "space_provision_cancelled_missing_space_id"
        };
      }

      const response = await workbenchApi.createSpace({
        space_id,
        creation_mode: typeof actionProps.creation_mode === "string"
          ? (actionProps.creation_mode as "blank" | "import" | "template")
          : "blank",
        owner: typeof actionProps.owner === "string" ? actionProps.owner : "cortex-web-operator",
        reference_uri: typeof actionProps.reference_uri === "string" ? actionProps.reference_uri : undefined,
        template_id: typeof actionProps.template_id === "string" ? actionProps.template_id : undefined
      });
      return {
        ok: true,
        message: response.message ?? `Space '${response.space_id}' provisioned.`,
        code: "space_provisioned"
      };
    },
    startAgentInitiative: async (queryParams) => {
      const spaceId = queryParams.get("spaceId") || SPACE_ID;
      const initiativeId = queryParams.get("initiativeId") || `workbench-${Date.now()}`;
      const response = await workbenchApi.startAgentInitiative(spaceId, initiativeId);
      return {
        ok: true,
        message: `Initiative run started: ${response.runId}`,
        code: "agent_initiative_started"
      };
    }
  };
  const handler = actionHandlers[name];
  if (!handler) {
    return {
      ok: false,
      message: `Unsupported action: ${name}`,
      code: "unsupported_action"
    };
  }
  return handler(params, props);
}

function CapabilityMapWidget({ componentProperties }: A2UIComponentProps): React.ReactElement {
  const navigate = useNavigate();
  const location = useLocation();
  const actorRole = useUiStore((state) => state.sessionUser?.role || "operator");
  const props = readProps(componentProperties, "CapabilityMap");
  const dataSourceUrl = String(props.dataSourceUrl ?? "/api/system/capability-graph");

  const [graphData, setGraphData] = React.useState<PlatformCapabilityGraph | null>(null);
  const [selectedId, setSelectedId] = React.useState<string | null>(null);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    fetch(gatewayBaseUrl() + dataSourceUrl)
      .then(res => res.json())
      .then(data => setGraphData(data))
      .catch(err => setError(err.message));
  }, [dataSourceUrl]);

  React.useEffect(() => {
    const selectedFromQuery = new URLSearchParams(location.search).get("node_id");
    if (selectedFromQuery) {
      setSelectedId(selectedFromQuery);
    }
  }, [location.search]);

  if (error) {
    return React.createElement("div", { className: "error-banner" }, `Failed to load graph data: ${error}`);
  }

  if (!graphData || !graphData.nodes || !graphData.edges) {
    return React.createElement("div", { className: "placeholder" }, "Loading capability graph...");
  }

  const selectedNode = graphData.nodes.find((node) => node.id === selectedId) || null;
  const canNavigate = Boolean(selectedNode?.route_id);

  const handleSelectNode = (nodeId: string) => {
    setSelectedId(nodeId);
    const params = new URLSearchParams(location.search);
    params.set("node_id", nodeId);
    if (!params.get("intent")) params.set("intent", "navigate");
    if (!params.get("density")) params.set("density", "comfortable");
    navigate(`${location.pathname}?${params.toString()}`, { replace: true });
  };

  return React.createElement(
    "div",
    { className: "flex flex-col lg:flex-row gap-4 min-h-[540px]" },
    [
      React.createElement("div", { key: "graph", className: "flex-1 min-h-[520px] border border-cortex-line rounded-cortex bg-cortex-bg-panel" },
        React.createElement(CapabilityMatrixMap, {
          nodes: graphData.nodes,
          edges: graphData.edges,
          selectedId,
          currentRole: actorRole,
          onSelect: handleSelectNode,
          onNavigate: (routeId: string) => navigate(routeId)
        })
      ),
      React.createElement(
        "aside",
        { key: "inspector", className: "w-full lg:w-[320px] p-4 border border-cortex-line rounded-cortex bg-cortex-bg-elev flex flex-col gap-2" },
        [
          React.createElement("h4", { key: "title", className: "text-base font-semibold text-cortex-ink" }, "Capability Inspector"),
          selectedNode
            ? React.createElement(
                React.Fragment,
                { key: "content" },
                [
                  React.createElement("div", { key: "name", className: "text-sm text-cortex-ink font-medium" }, selectedNode.title),
                  React.createElement("div", { key: "desc", className: "text-xs text-cortex-ink-muted" }, selectedNode.description),
                  React.createElement("div", { key: "intent", className: "text-xs text-cortex-ink-muted" }, `intent=${selectedNode.intent_type}`),
                  selectedNode.required_role
                    ? React.createElement("div", { key: "role", className: "text-xs text-cortex-ink-muted" }, `required_role=${selectedNode.required_role}`)
                    : null,
                  selectedNode.pattern_id
                    ? React.createElement("div", { key: "pattern", className: "text-xs text-cortex-ink-muted" }, `pattern=${selectedNode.pattern_id}`)
                    : null,
                  selectedNode.promotion_status
                    ? React.createElement("div", { key: "status", className: "text-xs text-cortex-ink-muted" }, `promotion_status=${selectedNode.promotion_status}`)
                    : null,
                  canNavigate
                    ? React.createElement(
                        "button",
                        {
                          key: "navigate",
                          className: "mt-2 px-3 py-2 rounded-cortex border border-cortex-line bg-cortex-bg text-cortex-ink hover:bg-cortex-bg-panel text-sm",
                          onClick: () => navigate(String(selectedNode.route_id))
                        },
                        `Open ${selectedNode.route_id}`
                      )
                    : React.createElement("div", { key: "hint", className: "text-xs text-cortex-ink-faint" }, "No route binding for this capability.")
                ]
              )
            : React.createElement("div", { key: "empty", className: "text-xs text-cortex-ink-faint" }, "Select a node to inspect metadata."),
        ]
      )
    ]
  );
}

export const WidgetRegistry: Record<string, React.FC<A2UIComponentProps>> = {
  Heading: ({ componentProperties }) => {
    const props = readProps(componentProperties, "Heading");
    return React.createElement("h3", { className: "text-lg font-bold mt-2 text-cortex-ink" }, String(props.text ?? ""));
  },

  Text: ({ componentProperties }) => {
    const props = readProps(componentProperties, "Text");
    return React.createElement("p", { className: "text-sm text-cortex-ink-muted" }, String(props.text ?? ""));
  },

  Button: ({ componentProperties }) => {
    const props = readProps(componentProperties, "Button");
    const [pending, setPending] = React.useState(false);
    const [result, setResult] = React.useState<ActionExecutionResult | null>(null);
    return React.createElement(
      "div",
      { className: "a2ui-action-button flex flex-col gap-2" },
      React.createElement(
        "button",
        {
          disabled: pending,
          className: "px-3 py-2 rounded-cortex border border-cortex-line bg-cortex-bg-elev text-cortex-ink hover:bg-cortex-bg disabled:opacity-60",
          onClick: async () => {
            setPending(true);
            setResult(null);
            const action = props.action;
            let executionResult: ActionExecutionResult | null = null;
            try {
              executionResult = await executeWidgetAction(action, props);
              setResult(executionResult);
            } catch (error) {
              executionResult = {
                ok: false,
                message: error instanceof Error ? error.message : String(error),
                code: "execution_error"
              };
              setResult(executionResult);
            } finally {
              setPending(false);
            }
            emitA2uiEvent("button_click", {
              action: props.action,
              label: props.label,
              executionCode: executionResult?.code,
              executionStatus: executionResult?.ok ? "success" : "error",
              executionMessage: executionResult?.message
            });
          }
        },
        pending ? "Running..." : String(props.label ?? "Action")
      ),
      result
        ? React.createElement(
            "div",
            {
              className: `text-xs ${result.ok ? "text-cortex-ok" : "text-cortex-bad"}`
            },
            result.message
          )
        : null
    );
  },

  Container: ({ children }) =>
    React.createElement(
      "div",
      { className: "a2ui-container flex flex-col gap-3 p-4 border border-cortex-line rounded-cortex bg-cortex-bg-panel" },
      children
    ),

  Row: ({ children }) => React.createElement("div", { className: "a2ui-row flex flex-row gap-2" }, children),

  Column: ({ children }) =>
    React.createElement("div", { className: "a2ui-column flex flex-col gap-2" }, children),

  ApprovalControls: ({ componentProperties }) => {
    const props = readProps(componentProperties, "ApprovalControls");
    const runId = String(props.runId ?? "");
    const spaceId = String(props.spaceId ?? SPACE_ID);
    const decisionRef = String(props.decisionRef ?? `DEC-${Date.now()}`);

    async function submitApproval(decision: "approved" | "rejected") {
      if (!runId) {
        console.warn("ApprovalControls missing runId");
        return;
      }
      try {
        await workbenchApi.approveAgentRun(spaceId, runId, {
          decision,
          actor: "cortex-web-operator",
          rationale: decision === "approved"
            ? "Operator approved simulation output"
            : "Operator rejected simulation output",
          decisionRef
        });
        emitA2uiEvent("approval", {
          decision,
          status: "success",
          scenarioId: props.scenarioId,
          runId,
          spaceId
        });
      } catch (error) {
        emitA2uiEvent("approval", {
          decision,
          status: "error",
          runId,
          spaceId,
          error: error instanceof Error ? error.message : String(error)
        });
      }
    }

    return React.createElement("div", { className: "flex gap-4 mt-4 border-t border-cortex-line pt-4" }, [
      React.createElement(
        "button",
        {
          key: "approve",
          className: "chip ok px-6 py-2 rounded-cortex font-bold uppercase",
          onClick: async () => {
            await submitApproval("approved");
          }
        },
        "Approve Changes"
      ),
      React.createElement(
        "button",
        {
          key: "reject",
          className: "chip bad px-6 py-2 rounded-cortex font-bold uppercase",
          onClick: async () => {
            await submitApproval("rejected");
          }
        },
        "Reject Initiative"
      )
    ]);
  },

  DiffViewer: ({ componentProperties }) => {
    const props = readProps(componentProperties, "DiffViewer");
    return React.createElement(
      "pre",
      { className: "p-4 mt-2 bg-cortex-bg text-cortex-ok text-sm overflow-auto rounded-cortex border border-cortex-line max-h-64 font-mono" },
      String(props.diffText ?? "")
    );
  },

  SpatialPlane: ({ componentProperties }) => {
    const props = readProps(componentProperties, "SpatialPlane") as SpatialPlanePayload;
    return React.createElement(TldrawCanvas, { payload: props });
  },

  AlertBanner: ({ componentProperties }) => {
    const props = readProps(componentProperties, "AlertBanner");
    const severity = String(props.severity || "info");
    const message = String(props.message || "");
    const title = props.title ? String(props.title) : undefined;

    let bgColorClass = "bg-cortex-bg-elev";
    let textColorClass = "text-cortex-accent";
    let borderColorClass = "border-cortex-accent";

    if (severity === "error" || severity === "critical") {
      bgColorClass = "bg-[#ff7d7920]";
      textColorClass = "text-cortex-bad";
      borderColorClass = "border-cortex-bad";
    } else if (severity === "warning") {
      bgColorClass = "bg-[#f8c25820]";
      textColorClass = "text-cortex-warn";
      borderColorClass = "border-cortex-warn";
    } else if (severity === "success") {
      bgColorClass = "bg-[#35cb8b20]";
      textColorClass = "text-cortex-ok";
      borderColorClass = "border-cortex-ok";
    }

    return React.createElement("div", {
      className: `p-4 my-2 border rounded-cortex ${bgColorClass} ${textColorClass} ${borderColorClass}`
    }, [
      title && React.createElement("h4", { key: "title", className: "font-semibold mb-1" }, title),
      React.createElement("span", { key: "msg", className: "text-sm" }, message)
    ]);
  },

  MetricCard: ({ componentProperties }) => {
    const props = readProps(componentProperties, "MetricCard");
    return React.createElement("div", {
      className: "metric flex flex-col p-4 border border-cortex-line rounded-cortex bg-cortex-bg-elev shadow"
    }, [
      React.createElement("span", { key: "label", className: "text-xs font-medium text-cortex-ink-muted uppercase tracking-wider mb-1" }, String(props.label || "")),
      React.createElement("strong", { key: "value", className: "text-2xl font-bold text-cortex-ink" }, String(props.value || "")),
      props.trend ? React.createElement("span", {
        key: "trend",
        className: `text-xs mt-2 ${String(props.trend).startsWith("-") || String(props.trend).includes("down") ? "text-cortex-bad" : "text-cortex-ok"}`
      }, String(props.trend)) : null
    ]);
  },

  DataTable: ({ componentProperties }) => {
    const props = readProps(componentProperties, "DataTable");
    const explicitColumns = Array.isArray(props.columns) ? props.columns.map((column) => String(column)) : [];
    const rawRows = props.rows ?? props.data;
    const rows = Array.isArray(rawRows)
      ? rawRows
          .map((row) => {
            if (row && typeof row === "object" && !Array.isArray(row)) {
              return row as Record<string, unknown>;
            }
            if (Array.isArray(row)) {
              const mapped: Record<string, unknown> = {};
              row.forEach((cell, index) => {
                const columnName = explicitColumns[index] ?? `Column ${index + 1}`;
                mapped[columnName] = cell;
              });
              return mapped;
            }
            return null;
          })
          .filter((row): row is Record<string, unknown> => row !== null)
      : [];
    const columns = explicitColumns.length > 0 ? explicitColumns : (rows[0] ? Object.keys(rows[0]) : []);

    if (columns.length === 0 || rows.length === 0) {
      return React.createElement("div", { className: "text-sm text-cortex-ink-faint italic p-4" }, "No data to display");
    }

    return React.createElement("div", { className: "overflow-x-auto my-4 border border-cortex-line rounded-cortex" },
      React.createElement("table", { className: "min-w-full text-sm text-left text-cortex-ink" }, [
        React.createElement("thead", { key: "thead", className: "text-xs text-cortex-ink-muted uppercase bg-cortex-bg border-b border-cortex-line" },
          React.createElement("tr", {},
            columns.map((col): React.ReactNode => React.createElement("th", { key: col, className: "px-4 py-3 font-medium" }, col))
          )
        ),
        React.createElement("tbody", { key: "tbody", className: "divide-y divide-cortex-line" },
          rows.map((row, idx): React.ReactNode => React.createElement("tr", { key: idx, className: "bg-cortex-bg-panel hover:bg-cortex-bg-elev" },
            columns.map((col): React.ReactNode => React.createElement("td", { key: `${idx}-${col}`, className: "px-4 py-3" }, String(row[col] || "")))
          ))
        )
      ])
    );
  },

  HeapBlockCard: LegacyHeapBlockCard,
  HeapBoard: ({ componentProperties }) => {
    const props = componentProperties["HeapBoard"] as Record<string, unknown> || {};
    return React.createElement(HeapBlockGrid, {
      filterDefaults: { spaceId: String(props.spaceId || "") },
      showFilterSidebar: false,
    });
  },
  HeapCanvas: () => React.createElement(HeapBlockGrid, { showFilterSidebar: true }),
  A2UISynthesisSpace: ({ componentProperties }) => React.createElement(A2UISynthesisSpace, { id: "synthesis_space", componentProperties }),

  CapabilityMap: CapabilityMapWidget,
  BrandingLabsWidget: () =>
    React.createElement("div", { className: "p-4 border border-cortex-line rounded-cortex bg-cortex-bg-panel" }, [
      React.createElement("h4", { key: "title", className: "text-base font-semibold text-cortex-ink" }, "Branding Labs"),
      React.createElement(
        "p",
        { key: "text", className: "text-sm text-cortex-ink-muted mt-1" },
        "Brand policy experiments, identity styling, and UX lab governance checkpoints are available in this surface."
      )
    ]),

  Card: ({ componentProperties, children }) => {
    const props = readProps(componentProperties, "Card");
    return React.createElement("div", { className: "p-4 border border-cortex-line rounded-cortex bg-cortex-bg-elev shadow flex flex-col gap-2" }, [
      props.text ? React.createElement("div", { key: "text", className: "text-cortex-ink font-medium" }, String(props.text)) : null,
      children
    ]);
  },
  TextField: ({ componentProperties }) => {
      const props = readProps(componentProperties, "TextField");
      return React.createElement("div", { className: "flex flex-col gap-1" }, [
        React.createElement("label", { key: "label", className: "text-xs font-medium text-cortex-ink-muted uppercase tracking-wider" }, String(props.label || "")),
        React.createElement("input", { key: "input", className: "bg-cortex-bg p-2 text-sm text-cortex-ink border border-cortex-line focus:outline-none focus:border-cortex-accent disabled:opacity-50" })
      ]);
  },
  Grid: ({ children }) => {
      return React.createElement("div", { className: "grid grid-cols-2 md:grid-cols-4 gap-4 w-full" }, children);
  },
  Tabs: ({ componentProperties }) => {
    const props = readProps(componentProperties, "Tabs");
    const items = props.tabItems as Array<{ title: string, child: string }> || [];
    return React.createElement("div", { className: "w-full border-t border-cortex-line pt-4" }, [
      React.createElement("div", { key: "tab-header", className: "flex space-x-4 border-b border-cortex-line pb-2 mb-4" },
        items.map((item, i) => React.createElement("div", { key: i, className: "px-2 py-1 text-sm text-cortex-ink-muted font-medium cursor-pointer hover:text-cortex-ink" }, item.title))
      ),
      React.createElement("div", { key: "tab-content", className: "p-2 bg-cortex-bg-panel rounded-cortex" },
        React.createElement("span", { className: "text-xs text-cortex-ink-faint italic" }, "Tab content mapped to children")
      )
    ]);
  },
  Markdown: ({ componentProperties }) => {
    const props = readProps(componentProperties, "Markdown");
    const content = String(props.content || "");
    return React.createElement("pre", { className: "p-4 bg-cortex-bg text-cortex-ink text-sm overflow-auto rounded-cortex font-mono whitespace-pre-wrap border border-cortex-line" }, content);
  }
};
