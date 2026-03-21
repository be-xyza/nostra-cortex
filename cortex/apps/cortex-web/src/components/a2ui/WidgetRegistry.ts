import React from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { TldrawCanvas } from "./TldrawCanvas";
import { SpatialPlanePayload } from "./spatialReplay";
import { emitA2uiEvent } from "./spatialEventContract";
import {
  gatewayBaseUrl,
  openGatewayApiArtifact,
  resolveWorkbenchSpaceId,
  workbenchApi,
} from "../../api";
import { CapabilityMatrixMap } from "../CapabilityMatrixMap";
import { CapabilityMapV2 } from "../CapabilityMapV2";
import type { HeapBlockListItem, PlatformCapabilityGraph } from "../../contracts";
import { HeapBlockGrid } from "../heap/HeapBlockGrid";
import { HeapBlockCard as CanonicalHeapBlockCard } from "../heap/HeapBlockCard";
import { useUiStore } from "../../store/uiStore";
import { ForceGraph } from "../ForceGraph";
import { projectDataTable } from "./dataTable";
import { classifyWorkbenchHref } from "../workflows/artifactRouting.ts";
import { useWorkflowArtifactInspector } from "../workflows/WorkflowArtifactInspectorContext.tsx";
import {
  WorkflowInstanceTimeline,
  WorkflowProjectionPreview,
  WorkflowStatusBadge,
  WorkflowSummaryStrip,
} from "../workflows/workflowWidgets.tsx";
import { CapabilityInspectorBlock } from "../CapabilityInspectorBlock";
import { RulesMatrixWidget } from "../RulesMatrixWidget";

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

function readString(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function readStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value
    .map((item) => (typeof item === "string" ? item.trim() : ""))
    .filter((item) => item.length > 0);
}

function readStringRecord(value: unknown): Record<string, string> | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) return undefined;
  const next: Record<string, string> = {};
  for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
    if (item === undefined || item === null) continue;
    next[key] = String(item);
  }
  return Object.keys(next).length > 0 ? next : undefined;
}

function normalizeWidgetPayloadType(props: Record<string, unknown>): "a2ui" | "rich_text" | "media" | "structured_data" | "pointer" {
  const candidate = readString(props.payload_type)?.toLowerCase();
  switch (candidate) {
    case "a2ui":
    case "rich_text":
    case "media":
    case "structured_data":
    case "pointer":
      return candidate;
    default:
      break;
  }
  return typeof props.contentPrefix === "string" || typeof props.content === "string" ? "rich_text" : "structured_data";
}

function projectWidgetHeapCard(props: Record<string, unknown>): HeapBlockListItem {
  const nowIso = new Date().toISOString();
  const title = readString(props.title) || readString(props.blockType) || "Heap Block";
  const artifactId =
    readString(props.artifactId) ||
    readString(props.id) ||
    `widget_${title.toLowerCase().replace(/[^a-z0-9]+/g, "_").replace(/^_+|_+$/g, "") || "heap"}`;
  const blockType = readString(props.blockType) || "note";
  const payloadType = normalizeWidgetPayloadType(props);
  const emittedAt = readString(props.timestamp) || readString(props.createdAtUtc) || nowIso;
  const attributes = readStringRecord(props.attributes);
  const tags = readStringArray(props.tags);
  const mentionsInline = readStringArray(props.mentions);
  const pageLinks = readStringArray(props.pageLinks ?? props.page_links);
  const behaviors = readStringArray(props.behaviors);
  const confidenceValue = typeof props.confidence === "number" ? props.confidence : Number(props.confidence);
  const confidence = Number.isFinite(confidenceValue) ? confidenceValue : 50;
  const providedSurface = props.surfaceJson;
  const hasProvidedSurface =
    providedSurface && typeof providedSurface === "object" && !Array.isArray(providedSurface);

  const surfaceJson: Record<string, unknown> = hasProvidedSurface
    ? { ...(providedSurface as Record<string, unknown>) }
    : {
      payload_type: payloadType,
      version: readString(props.version) || "v1.0",
      phase: readString(props.phase) || "Alpha",
      confidence,
      authority_scope: readString(props.authority_scope) || "Local",
      behaviors,
    };

  if (!hasProvidedSurface) {
    if (payloadType === "rich_text") {
      surfaceJson.text = readString(props.contentPrefix) || readString(props.content) || "";
    } else if (payloadType === "pointer") {
      surfaceJson.pointer = readString(props.pointer) || readString(props.content) || "";
    } else if (payloadType === "structured_data") {
      const structuredSource = props.content ?? props.attributes ?? {};
      surfaceJson.data =
        structuredSource && typeof structuredSource === "object" && !Array.isArray(structuredSource)
          ? structuredSource
          : { value: String(structuredSource ?? "") };
    } else if (payloadType === "media") {
      const media = props.media;
      if (media && typeof media === "object" && !Array.isArray(media)) {
        surfaceJson.media = media;
      }
    }
  }

  if (!("payload_type" in surfaceJson)) {
    surfaceJson.payload_type = payloadType;
  }

  return {
    projection: {
      artifactId,
      title,
      blockType,
      updatedAt: emittedAt,
      emittedAt,
      tags,
      mentionsInline,
      pageLinks,
      attributes,
      hasFiles: false,
    },
    surfaceJson,
    warnings: [],
  };
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
  const state = useUiStore.getState();
  const activeSpaceIds = state.activeSpaceIds;
  const activeSpaceId = activeSpaceIds[0] || "";
  const wasWorkbenchNamedManually = state.wasWorkbenchNamedManually;

  // Deferred Naming Gate for multi-space sessions
  const gatedActions = ["emitGateSummaryToHeap", "startAgentContribution"];
  if (activeSpaceIds.length > 1 && !wasWorkbenchNamedManually && gatedActions.includes(name)) {
    state.setPendingWorkbenchAction(() => executeWidgetAction(action, props));
    state.setNamingModalOpen(true);
    return {
      ok: true, // Returning true to avoid error UI while modal is open
      message: "Workbench naming required...",
      code: "workbench_naming_gate_active"
    };
  }

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
    startAgentContribution: async (queryParams) => {
      const spaceId = resolveWorkbenchSpaceId(queryParams.get("spaceId") ?? undefined);
      const contributionId = queryParams.get("contributionId") || `workbench-${Date.now()}`;
      const response = await workbenchApi.startAgentContribution(spaceId, contributionId);
      return {
        ok: true,
        message: `Contribution run started: ${response.runId}`,
        code: "agent_contribution_started"
      };
    },
    emitGateSummaryToHeap: async (queryParams, actionProps) => {
      const kindCandidate = (queryParams.get("kind") || "").trim().toLowerCase();
      if (kindCandidate !== "siq" && kindCandidate !== "testing") {
        return {
          ok: false,
          message: `Invalid gate summary kind: ${kindCandidate || "missing"}.`,
          code: "invalid_gate_summary_kind"
        };
      }
      const spaceId =
        (queryParams.get("spaceId") || queryParams.get("workspaceId") || "").trim() ||
        (typeof actionProps.spaceId === "string" ? actionProps.spaceId.trim() : "") ||
        (typeof actionProps.space_id === "string" ? actionProps.space_id.trim() : "") ||
        activeSpaceId ||
        "";
      if (!spaceId) {
        return {
          ok: false,
          message: "spaceId is required for gate summary emission.",
          code: "missing_space_id"
        };
      }

      const response = await workbenchApi.emitGateSummaryHeapBlock({
        schemaVersion: "1.0.0",
        spaceId,
        kind: kindCandidate as "siq" | "testing"
      });
      return {
        ok: true,
        message: `Saved to Heap as ${response.artifactId}.`,
        code: "gate_summary_saved"
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
  const envV2Flag =
    ((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_CAPABILITY_GRAPH_V2_ENABLED as
      | string
      | undefined) ?? "false";
  const graphV2Enabled = String(props.graphV2Enabled ?? envV2Flag).toLowerCase() === "true";

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

  const handleSelectNode = (nodeId: string | null) => {
    setSelectedId(nodeId);
    const params = new URLSearchParams(location.search);
    if (nodeId) {
      params.set("node_id", nodeId);
      if (!params.get("intent")) params.set("intent", "navigate");
      if (!params.get("density")) params.set("density", "comfortable");
    } else {
      params.delete("node_id");
      params.delete("intent");
      params.delete("density");
    }
    navigate(`${location.pathname}?${params.toString()}`, { replace: true });
  };

  return React.createElement(
    "div",
    { className: "relative w-full h-[85vh] min-h-[700px] animate-in fade-in zoom-in-95 duration-700" },
    [
      React.createElement("div", { key: "graph", className: "absolute inset-0 border border-white/5 rounded-3xl bg-slate-950/20 shadow-2xl overflow-hidden" },
        [
          React.createElement(
            "div",
            {
              key: "graph-version-badge",
              className: "absolute top-6 left-6 z-20 px-3 py-1.5 text-[10px] font-black uppercase tracking-widest rounded-lg border border-white/5 bg-slate-900/60 backdrop-blur-md text-slate-400"
            },
            graphV2Enabled
              ? `Graph V2 • ${graphData.capabilities_version ?? "runtime"}`
              : "Graph V1 (fallback)"
          ),
          graphV2Enabled
            ? React.createElement(CapabilityMapV2, {
              key: "graph-v2",
              nodes: graphData.nodes,
              edges: graphData.edges,
              selectedId,
              currentRole: actorRole,
              layoutHints: graphData.layout_hints,
              legend: graphData.legend,
              onSelect: handleSelectNode,
              onNavigate: (routeId: string) => navigate(routeId)
            })
            : React.createElement(CapabilityMatrixMap, {
              key: "graph-v1",
              nodes: graphData.nodes,
              edges: graphData.edges,
              selectedId,
              currentRole: actorRole,
              onSelect: handleSelectNode,
              onNavigate: (routeId: string) => navigate(routeId)
            })
        ]
      ),
      React.createElement(
        "aside",
        { 
          key: "inspector", 
          className: `absolute right-6 top-6 bottom-6 w-[400px] z-30 p-6 border border-white/10 rounded-3xl bg-slate-900/40 backdrop-blur-2xl flex flex-col gap-5 overflow-y-auto shadow-2xl transition-all duration-500 ease-out custom-scrollbar ${selectedNode ? "translate-x-0 opacity-100" : "translate-x-12 opacity-0 pointer-events-none"}` 
        },
        [
          React.createElement("div", { key: "header", className: "flex items-center justify-between mb-2 shrink-0" }, [
            React.createElement("div", { key: "title-group", className: "flex flex-col gap-1" }, [
              React.createElement("h4", { key: "title", className: "text-[10px] font-black uppercase tracking-[0.24em] text-slate-500" }, "Capability Inspector"),
              selectedNode && React.createElement("div", { key: "node-title", className: "text-sm font-bold text-slate-100" }, selectedNode.title)
            ]),
            selectedNode && React.createElement("span", { key: "badge", className: "px-2 py-0.5 rounded-md bg-blue-500/10 text-blue-400 text-[10px] font-black border border-blue-500/20" }, "LIVE")
          ]),
          selectedNode 
            ? React.createElement(CapabilityInspectorBlock, { key: "block", node: selectedNode })
            : null
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
    const navigate = useNavigate();
    const workflowInspector = useWorkflowArtifactInspector();
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
            const href = typeof props.href === "string" ? props.href.trim() : "";
            let executionResult: ActionExecutionResult | null = null;
            try {
              if (typeof action === "string" && action.trim().length > 0) {
                executionResult = await executeWidgetAction(action, props);
              } else if (href) {
                const hrefKind = classifyWorkbenchHref(href);
                if (hrefKind === "gateway_api") {
                  if (workflowInspector) {
                    await workflowInspector.openArtifact(href);
                  } else {
                    await openGatewayApiArtifact(href, "new_tab");
                  }
                } else if (hrefKind === "internal_workbench") {
                  navigate(href);
                } else {
                  window.location.assign(href);
                }
                executionResult = {
                  ok: true,
                  message: `Opened ${href}`,
                  code: "navigation_opened"
                };
              } else {
                executionResult = {
                  ok: false,
                  message: "Button requires either action or href.",
                  code: "button_missing_action_or_href"
                };
              }
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
              href: props.href,
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
    const spaceId = resolveWorkbenchSpaceId(
      typeof props.spaceId === "string" ? props.spaceId : undefined
    );
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

    return React.createElement("div", {
      className: "flex gap-4 mt-4 border-t border-cortex-line pt-4",
      role: "region",
      "aria-label": "Approval Gate Controls",
      "data-testid": "approval-controls"
    }, [
      React.createElement(
        "button",
        {
          key: "approve",
          className: "chip ok px-6 py-2 rounded-cortex font-bold uppercase",
          "aria-label": "Approve contribution changes",
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
          "aria-label": "Reject contribution changes",
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
    const navigate = useNavigate();
    const workflowInspector = useWorkflowArtifactInspector();
    const { columns, rows, rowHrefField, rowKeyField } = projectDataTable(props);

    if (columns.length === 0 || rows.length === 0) {
      return React.createElement("div", { className: "text-sm text-cortex-ink-faint italic p-4" }, "No data to display");
    }

    const handleRowNavigation = async (href: string) => {
      const hrefKind = classifyWorkbenchHref(href);
      if (hrefKind === "gateway_api") {
        if (workflowInspector) {
          await workflowInspector.openArtifact(href);
          return;
        }
        await openGatewayApiArtifact(href, "new_tab");
        return;
      }
      if (hrefKind === "internal_workbench") {
        navigate(href);
        return;
      }
      window.location.assign(href);
    };

    return React.createElement("div", { className: "overflow-x-auto my-4 border border-cortex-line rounded-cortex" },
      React.createElement("table", { className: "min-w-full text-sm text-left text-cortex-ink" }, [
        React.createElement("thead", { key: "thead", className: "text-xs text-cortex-ink-muted uppercase bg-cortex-bg border-b border-cortex-line" },
          React.createElement("tr", {},
            columns.map((col): React.ReactNode => React.createElement("th", { key: col, className: "px-4 py-3 font-medium" }, col))
          )
        ),
        React.createElement("tbody", { key: "tbody", className: "divide-y divide-cortex-line" },
          rows.map((row, idx): React.ReactNode => {
            const href =
              typeof row[rowHrefField] === "string" ? String(row[rowHrefField]) : "";
            const key =
              typeof row[rowKeyField] === "string" && String(row[rowKeyField]).trim()
                ? String(row[rowKeyField])
                : `row-${idx}`;
            const clickable = href.trim().length > 0;
            return React.createElement("tr", {
              key,
              className: `bg-cortex-bg-panel hover:bg-cortex-bg-elev ${clickable ? "cursor-pointer" : ""}`,
              onClick: clickable ? () => void handleRowNavigation(href) : undefined,
              role: clickable ? "link" : undefined,
              tabIndex: clickable ? 0 : undefined,
              onKeyDown: clickable
                ? (event: React.KeyboardEvent<HTMLTableRowElement>) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      void handleRowNavigation(href);
                    }
                  }
                : undefined,
            },
              columns.map((col): React.ReactNode => React.createElement("td", { key: `${key}-${col}`, className: "px-4 py-3" }, String(row[col] || "")))
            );
          })
        )
      ])
    );
  },

  HeapBlockCard: ({ componentProperties, children }) => {
    const props = componentProperties["HeapBlockCard"] as Record<string, unknown> || {};
    const projected = projectWidgetHeapCard(props);
    return React.createElement(CanonicalHeapBlockCard, {
      block: projected,
      isSelected: false,
      isRegenerating: false,
      onClick: () => undefined,
      onDoubleClick: () => undefined
    }, children);
  },
  HeapBoard: ({ componentProperties }) => {
    const props = componentProperties["HeapBoard"] as Record<string, unknown> || {};
    return React.createElement(HeapBlockGrid, {
      filterDefaults: { spaceId: String(props.spaceId || "") },
      showFilterSidebar: false,
    });
  },
  Canvas: () => React.createElement(HeapBlockGrid, { showFilterSidebar: true }),
  HeapCanvas: () => React.createElement(HeapBlockGrid, { showFilterSidebar: true }),

  ContributionGraph: ({ componentProperties }) => {
    const props = componentProperties["ContributionGraph"] as Record<string, unknown> || {};
    return React.createElement(ForceGraph, {
      nodes: Array.isArray(props.nodes) ? props.nodes as any[] : [],
      edges: Array.isArray(props.edges) ? props.edges as any[] : [],
      selectedId: typeof props.selectedId === "string" ? props.selectedId : null,
      onSelect: () => undefined,
    });
  },

  CapabilityMap: CapabilityMapWidget,
  RulesMatrixWidget: RulesMatrixWidget,
  WorkbenchSummary: () => {
    const activeWorkbenchSession = useUiStore((state) => state.activeWorkbenchSession);
    const activeSpaceIds = useUiStore((state) => state.activeSpaceIds);
    const wasWorkbenchNamedManually = useUiStore((state) => state.wasWorkbenchNamedManually);

    return React.createElement(
      "div",
      { className: "p-6 border border-blue-500/20 rounded-cortex bg-cortex-bg-panel/40 backdrop-blur-xl flex flex-col gap-4 shadow-lg animate-in fade-in zoom-in-95 duration-500" },
      [
        React.createElement("div", { key: "header", className: "flex justify-between items-start" }, [
          React.createElement("div", { key: "title-group" }, [
            React.createElement("h2", { key: "title", className: "text-xl font-black text-cortex-ink uppercase tracking-tight" }, activeWorkbenchSession?.name || "Untitled Workbench"),
            React.createElement("div", { key: "badge", className: "flex items-center gap-2 mt-1" }, [
              React.createElement("div", { key: "dot", className: "w-2 h-2 rounded-full bg-blue-500 animate-pulse" }),
              React.createElement("span", { key: "status", className: "text-[10px] font-bold text-blue-400 uppercase tracking-widest" }, wasWorkbenchNamedManually ? "Live Session" : "Drafting"),
            ])
          ]),
          React.createElement("div", { key: "count", className: "px-3 py-1 rounded-full border border-cortex-line bg-cortex-bg-elev text-xs font-mono text-cortex-ink-muted" }, `${activeSpaceIds.length} Spaces`)
        ]),
        React.createElement("div", { key: "content", className: "grid grid-cols-1 md:grid-cols-2 gap-4" }, [
          React.createElement("div", { key: "spaces-list", className: "flex flex-col gap-2" }, [
            React.createElement("span", { key: "label", className: "text-[10px] font-bold text-cortex-ink-faint uppercase tracking-widest" }, "Aggregated Contexts"),
            React.createElement("div", { key: "list", className: "flex flex-wrap gap-1.5" }, 
              activeSpaceIds.map(id => React.createElement("span", { key: id, className: "px-2 py-0.5 rounded border border-white/5 bg-white/5 text-[9px] font-medium text-cortex-ink-muted" }, id))
            )
          ]),
          React.createElement("div", { key: "summary", className: "flex flex-col gap-2" }, [
            React.createElement("span", { key: "label", className: "text-[10px] font-bold text-cortex-ink-faint uppercase tracking-widest" }, "Intelligence state"),
            React.createElement("p", { key: "text", className: "text-xs text-cortex-ink-muted leading-relaxed" }, 
              activeSpaceIds.length > 1 
                ? "Multiple spaces are bound into this synthetic workbench. Intelligence kernels are currently cross-referencing capabilities across the union."
                : "Standard sovereign space context. Local intelligence kernels are active."
            )
          ])
        ])
      ]
    );
  },
  WorkflowSummaryStrip: ({ componentProperties }) =>
    React.createElement(WorkflowSummaryStrip, { componentProperties }),
  WorkflowStatusBadge: ({ componentProperties }) =>
    React.createElement(WorkflowStatusBadge, { componentProperties }),
  WorkflowProjectionPreview: ({ componentProperties }) =>
    React.createElement(WorkflowProjectionPreview, { componentProperties }),
  WorkflowInstanceTimeline: ({ componentProperties }) =>
    React.createElement(WorkflowInstanceTimeline, { componentProperties }),
  BrandingLabsWidget: () =>
    React.createElement("div", { className: "p-4 border border-cortex-line rounded-cortex bg-cortex-bg-panel" }, [
      React.createElement("h4", { key: "title", className: "text-base font-semibold text-cortex-ink" }, "Labs"),
      React.createElement(
        "p",
        { key: "text", className: "text-sm text-cortex-ink-muted mt-1" },
        "Draft new ideas here before they become live spaces, templates, or other governed surfaces."
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
