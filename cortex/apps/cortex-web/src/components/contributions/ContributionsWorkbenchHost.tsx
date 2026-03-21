import React, { useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";

import { workbenchApi } from "../../api";
import type {
  AgentRunRecord,
  AgentRunSummary,
  DpubBlastRadiusResponse,
  DpubPipelineRunReport,
  DpubStewardPacketExportResponse,
} from "../../contracts";
import { useActiveSpaceContext } from "../../store/spacesRegistry";
import { useUiStore } from "../../store/uiStore";
import { A2UIRenderSpace } from "../A2UIRenderSpace";
import { ContributionFocusMap } from "./ContributionFocusMap";
import {
  normalizeContributionSelection,
  pipelineModeRequiresApproval,
} from "./contributionsRouteState";

const PIPELINE_MODES = [
  "validate",
  "ingest",
  "doctor",
  "path",
  "simulate",
  "publish",
  "diff",
  "full",
] as const;

function panelShell(extraClassName?: string): string {
  return [
    "min-w-0 rounded-[24px] border border-cortex-line",
    "bg-[linear-gradient(180deg,rgba(8,20,40,0.94),rgba(3,10,24,0.94))]",
    "shadow-[0_24px_60px_rgba(2,8,20,0.38)] backdrop-blur-xl",
    extraClassName ?? "",
  ]
    .join(" ")
    .trim();
}

function normalizeIso(iso?: string): string {
  if (!iso) return "n/a";
  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? iso : date.toLocaleString();
}

function historyButtonClass(active: boolean, tone: "graph" | "agent"): string {
  const toneClass =
    tone === "graph"
      ? active
        ? "border-cyan-300/40 bg-cyan-300/14"
        : "border-white/10 bg-white/4 hover:border-cyan-300/20 hover:bg-cyan-300/8"
      : active
        ? "border-emerald-300/40 bg-emerald-300/14"
        : "border-white/10 bg-white/4 hover:border-emerald-300/20 hover:bg-emerald-300/8";

  return [
    "w-full rounded-[20px] border px-4 py-3 text-left transition",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-300/50",
    toneClass,
  ].join(" ");
}

export function ContributionsWorkbenchHost() {
  const location = useLocation();
  const navigate = useNavigate();
  const activeSpaceId = useActiveSpaceContext();
  const sessionUser = useUiStore((state) => state.sessionUser);

  const selection = normalizeContributionSelection(new URLSearchParams(location.search));
  const [graphRuns, setGraphRuns] = useState<DpubPipelineRunReport[]>([]);
  const [agentRuns, setAgentRuns] = useState<AgentRunSummary[]>([]);
  const [selectedGraphRun, setSelectedGraphRun] = useState<DpubPipelineRunReport | null>(null);
  const [selectedAgentRun, setSelectedAgentRun] = useState<AgentRunRecord | null>(null);
  const [blastRadius, setBlastRadius] = useState<DpubBlastRadiusResponse | null>(null);
  const [packetResponse, setPacketResponse] =
    useState<DpubStewardPacketExportResponse | null>(null);
  const [pipelineResponse, setPipelineResponse] =
    useState<DpubPipelineRunReport | null>(null);
  const [pipelineError, setPipelineError] = useState<string | null>(null);
  const [launchRequestToken, setLaunchRequestToken] = useState<number | null>(null);
  const [manualContributionId, setManualContributionId] = useState(
    selection.selectedContributionId ?? ""
  );
  const [pipelineForm, setPipelineForm] = useState({
    mode: "validate",
    goal: "stable-cortex-domain",
    scenarioTemplateId: "",
    publishVersion: "",
    fromVersion: "",
    toVersion: "",
    approvalRationale: "",
    approvalDecisionRef: "",
  });

  const selectedContributionId =
    selection.selectedContributionId ??
    selectedAgentRun?.contributionId ??
    (manualContributionId.trim() || null);
  const selectedAgentRunSummary = agentRuns.find(
    (run) => run.runId === selection.selectedAgentRunId
  );
  const liveRunId = selectedAgentRun?.runId ?? selectedAgentRunSummary?.runId ?? null;
  const launchContributionId =
    launchRequestToken !== null && selectedContributionId && !liveRunId
      ? selectedContributionId
      : null;
  const focusBadgeLabel =
    selection.focusKind === "agent_run"
      ? `Agent run ${selection.selectedAgentRunId ?? "pending"}`
      : selection.focusKind === "graph_run"
        ? `Graph run ${selection.selectedGraphRunId ?? "pending"}`
        : selection.focusKind === "contribution"
          ? `Contribution ${selectedContributionId ?? "pending"}`
          : "Manual";
  const blastRadiusEntries: Array<[string, string[]]> = blastRadius
    ? [
        ["Depends On", blastRadius.dependsOn],
        ["Depended By", blastRadius.dependedBy],
        ["Invalidates", blastRadius.invalidates],
        ["Referenced By", blastRadius.referencedBy],
      ]
    : [];

  function updateSelection(params: Record<string, string | null>) {
    const next = new URLSearchParams(location.search);
    for (const [key, value] of Object.entries(params)) {
      if (!value) {
        next.delete(key);
      } else {
        next.set(key, value);
      }
    }
    navigate({
      pathname: "/contributions",
      search: next.toString() ? `?${next.toString()}` : "",
    });
  }

  useEffect(() => {
    setManualContributionId(selection.selectedContributionId ?? "");
  }, [selection.selectedContributionId]);

  useEffect(() => {
    setLaunchRequestToken(null);
  }, [selectedContributionId, liveRunId]);

  function launchLiveExecution() {
    if (!selectedContributionId || liveRunId) {
      return;
    }
    setPipelineError(null);
    setLaunchRequestToken(Date.now());
  }

  useEffect(() => {
    let cancelled = false;
    workbenchApi
      .getRuns(activeSpaceId)
      .then((runs) => {
        if (cancelled) return;
        setGraphRuns(runs);
        setSelectedGraphRun(
          selection.selectedGraphRunId
            ? runs.find((run) => run.runId === selection.selectedGraphRunId) ?? null
            : null
        );
      })
      .catch((error) => {
        if (!cancelled) {
          setPipelineError(error instanceof Error ? error.message : String(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [activeSpaceId, selection.selectedGraphRunId]);

  useEffect(() => {
    let cancelled = false;
    workbenchApi
      .getSystemAgentRuns(activeSpaceId)
      .then((runs) => {
        if (!cancelled) {
          setAgentRuns(runs);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setPipelineError(error instanceof Error ? error.message : String(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [activeSpaceId]);

  useEffect(() => {
    if (!selection.selectedAgentRunId) {
      setSelectedAgentRun(null);
      return;
    }

    const matchingSummary = agentRuns.find(
      (run) => run.runId === selection.selectedAgentRunId
    );
    if (!matchingSummary) {
      setSelectedAgentRun(null);
      return;
    }

    let cancelled = false;
    workbenchApi
      .getSystemAgentRun(activeSpaceId, matchingSummary.runId)
      .then((run) => {
        if (!cancelled) {
          setSelectedAgentRun(run);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setSelectedAgentRun(null);
          setPipelineError(error instanceof Error ? error.message : String(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [activeSpaceId, agentRuns, selection.selectedAgentRunId]);

  useEffect(() => {
    if (!selectedContributionId) {
      setBlastRadius(null);
      return;
    }

    let cancelled = false;
    workbenchApi
      .getContributionBlastRadius(selectedContributionId, activeSpaceId)
      .then((value) => {
        if (!cancelled) {
          setBlastRadius(value);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setPipelineError(error instanceof Error ? error.message : String(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [activeSpaceId, selectedContributionId]);

  const approvalRequired = pipelineModeRequiresApproval(pipelineForm.mode);

  async function submitPipeline(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPipelineError(null);
    try {
      const approval = approvalRequired
        ? {
            approvedBy: sessionUser?.actorId || "cortex-web-operator",
            rationale: pipelineForm.approvalRationale || "Approved from contributions cockpit",
            approvedAt: new Date().toISOString(),
            decisionRef:
              pipelineForm.approvalDecisionRef || `DEC-${Date.now().toString(36).toUpperCase()}`,
          }
        : undefined;

      const report = await workbenchApi.runPipeline(
        {
          mode: pipelineForm.mode,
          goal: pipelineForm.goal || undefined,
          scenarioTemplateId: pipelineForm.scenarioTemplateId || undefined,
          publishVersion: pipelineForm.publishVersion || undefined,
          fromVersion: pipelineForm.fromVersion || undefined,
          toVersion: pipelineForm.toVersion || undefined,
          approval,
        },
        sessionUser?.role || "operator",
        sessionUser?.actorId || "cortex-web-operator",
        activeSpaceId
      );
      setPipelineResponse(report);
      updateSelection({
        run_id: report.runId,
        node_id: null,
      });
    } catch (error) {
      setPipelineError(error instanceof Error ? error.message : String(error));
    }
  }

  async function submitStewardPacket() {
    setPipelineError(null);
    try {
      const response = await workbenchApi.exportStewardPacket(
        {
          goal: pipelineForm.goal || undefined,
          fromVersion: pipelineForm.fromVersion || undefined,
          toVersion: pipelineForm.toVersion || undefined,
          approval: {
            approvedBy: sessionUser?.actorId || "cortex-web-operator",
            rationale:
              pipelineForm.approvalRationale || "Approved for steward packet export",
            approvedAt: new Date().toISOString(),
            decisionRef:
              pipelineForm.approvalDecisionRef || `DEC-${Date.now().toString(36).toUpperCase()}`,
          },
        },
        sessionUser?.role || "operator",
        sessionUser?.actorId || "cortex-web-operator",
        activeSpaceId
      );
      setPacketResponse(response);
    } catch (error) {
      setPipelineError(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <div className="flex flex-col gap-5">
      <section className={panelShell("overflow-hidden")}>
        <div className="flex flex-col gap-3 border-b border-white/8 bg-[radial-gradient(circle_at_top_left,rgba(62,163,255,0.18),transparent_42%),linear-gradient(90deg,rgba(7,18,35,0.98),rgba(9,24,46,0.92))] px-6 py-5">
          <div className="flex flex-wrap items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.28em] text-cyan-200/80">
            <span>Steward Cockpit</span>
            <span className="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-3 py-1 text-[10px] text-cyan-100">
              Space {activeSpaceId}
            </span>
            {selectedContributionId ? (
              <span className="rounded-full border border-amber-300/20 bg-amber-300/10 px-3 py-1 text-[10px] text-amber-100">
                Focus {selectedContributionId}
              </span>
            ) : null}
          </div>
          <div className="flex flex-col gap-2 xl:flex-row xl:items-end xl:justify-between">
            <div className="max-w-4xl">
              <h1 className="font-mono text-2xl font-semibold tracking-tight text-white">
                Contributions Cockpit
              </h1>
              <p className="mt-1 max-w-3xl text-sm leading-6 text-slate-300">
                Governed contribution execution, operator review, and steward packet preparation now live in a single route instead of being split between placeholder surfaces and detached lifecycle controls.
              </p>
            </div>
            <div className="grid grid-cols-2 gap-3 text-xs text-slate-300 sm:grid-cols-4">
              <div className="rounded-2xl border border-white/8 bg-white/5 px-3 py-2">
                <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Graph Runs</div>
                <div className="mt-1 text-lg font-semibold text-white">{graphRuns.length}</div>
              </div>
              <div className="rounded-2xl border border-white/8 bg-white/5 px-3 py-2">
                <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Agent Runs</div>
                <div className="mt-1 text-lg font-semibold text-white">{agentRuns.length}</div>
              </div>
              <div className="rounded-2xl border border-white/8 bg-white/5 px-3 py-2">
                <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Needs Review</div>
                <div className="mt-1 text-lg font-semibold text-white">
                  {agentRuns.filter((run) => run.requiresReview).length}
                </div>
              </div>
              <div className="rounded-2xl border border-white/8 bg-white/5 px-3 py-2">
                <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Live Run</div>
                <div className="mt-1 truncate text-lg font-semibold text-white">
                  {liveRunId ?? "idle"}
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1.15fr)_minmax(320px,0.9fr)_minmax(320px,1fr)]">
        <section
          className={panelShell("overflow-hidden")}
          role="region"
          aria-label="Contributions Summary and History"
          data-testid="contributions-summary-pane"
        >
          <div className="border-b border-white/8 px-5 py-4">
            <h2 className="font-mono text-sm uppercase tracking-[0.26em] text-slate-300">
              Summary and History
            </h2>
          </div>
          <div className="grid gap-4 p-4">
            <div className="rounded-[20px] border border-white/8 bg-white/4 p-4">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <div className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
                    Route Contract
                  </div>
                  <div className="mt-1 text-sm text-white">
                    Graph runs, agent runs, and contribution focus are rendered directly in the
                    host so steward history remains usable even if backend summary surfaces lag
                    behind deployment.
                  </div>
                </div>
                <div className="rounded-full border border-white/10 bg-slate-950/60 px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-slate-300">
                  {focusBadgeLabel}
                </div>
              </div>
            </div>

            <div
              className="rounded-[20px] border border-white/8 bg-white/4 p-4"
              data-testid="contributions-graph-history"
            >
              <div className="flex items-center justify-between gap-3">
                <div>
                  <h3 className="font-mono text-sm text-white">Contribution Graph Runs</h3>
                  <p className="mt-1 text-sm leading-6 text-slate-300">
                    Pick a graph run to inspect its mode, status, and timing without leaving the
                    cockpit.
                  </p>
                </div>
                <div className="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-cyan-100">
                  {graphRuns.length}
                </div>
              </div>
              <div className="mt-4 grid gap-3">
                {graphRuns.length > 0 ? (
                  graphRuns.map((run) => {
                    const active = run.runId === selection.selectedGraphRunId;
                    return (
                      <button
                        key={run.runId}
                        type="button"
                        className={historyButtonClass(active, "graph")}
                        onClick={() =>
                          updateSelection({
                            run_id: run.runId,
                            node_id: null,
                          })
                        }
                      >
                        <div className="flex items-start justify-between gap-3">
                          <div>
                            <div className="font-mono text-sm text-white">{run.runId}</div>
                            <div className="mt-1 text-xs uppercase tracking-[0.18em] text-slate-400">
                              {run.mode} · {run.status}
                            </div>
                          </div>
                          <div className="text-right text-xs text-slate-400">
                            {normalizeIso(run.startedAt)}
                          </div>
                        </div>
                      </button>
                    );
                  })
                ) : (
                  <div className="rounded-[18px] border border-dashed border-white/10 bg-slate-950/50 px-4 py-4 text-sm leading-6 text-slate-400">
                    No contribution-graph runs are recorded yet. The cockpit stays usable through
                    agent history and manual contribution focus while the graph artifact catches up.
                  </div>
                )}
              </div>
            </div>

            <div
              className="rounded-[20px] border border-white/8 bg-white/4 p-4"
              data-testid="contributions-agent-history"
            >
              <div className="flex items-center justify-between gap-3">
                <div>
                  <h3 className="font-mono text-sm text-white">Agent Run History</h3>
                  <p className="mt-1 text-sm leading-6 text-slate-300">
                    Resume stewardable live runs from here, even when graph history is empty.
                  </p>
                </div>
                <div className="rounded-full border border-emerald-300/20 bg-emerald-300/10 px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-emerald-100">
                  {agentRuns.length}
                </div>
              </div>
              <div className="mt-4 grid gap-3">
                {agentRuns.length > 0 ? (
                  agentRuns.map((run) => {
                    const active = run.runId === selection.selectedAgentRunId;
                    return (
                      <button
                        key={run.runId}
                        type="button"
                        className={historyButtonClass(active, "agent")}
                        onClick={() =>
                          updateSelection({
                            node_id: `agent_run:${run.runId}`,
                            contribution_id: run.contributionId ?? null,
                          })
                        }
                      >
                        <div className="flex items-start justify-between gap-3">
                          <div>
                            <div className="font-mono text-sm text-white">{run.runId}</div>
                            <div className="mt-1 text-xs uppercase tracking-[0.18em] text-slate-400">
                              {run.status} · {run.contributionId ?? "no contribution"}
                            </div>
                          </div>
                          <div className="text-right text-xs text-slate-400">
                            {normalizeIso(run.updatedAt)}
                          </div>
                        </div>
                      </button>
                    );
                  })
                ) : (
                  <div className="rounded-[18px] border border-dashed border-white/10 bg-slate-950/50 px-4 py-4 text-sm leading-6 text-slate-400">
                    No agent runs are visible yet. Start from contribution focus to launch a new
                    governed run when the system is ready.
                  </div>
                )}
              </div>
            </div>
          </div>
        </section>

        <section
          className={panelShell("overflow-hidden")}
          role="region"
          aria-label="Contributions Steward Tools"
          data-testid="contributions-steward-pane"
        >
          <div className="border-b border-white/8 px-5 py-4">
            <h2 className="font-mono text-sm uppercase tracking-[0.26em] text-slate-300">
              Steward Tools
            </h2>
          </div>
          <div className="flex flex-col gap-4 p-4">
            <div className="rounded-[20px] border border-white/8 bg-white/4 p-4">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <div className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
                    Selection
                  </div>
                  <div className="mt-1 font-mono text-sm text-white">
                    {focusBadgeLabel}
                  </div>
                </div>
                <button
                  type="button"
                  className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-slate-200"
                  onClick={() =>
                    updateSelection({
                      contribution_id: null,
                      run_id: null,
                      node_id: null,
                    })
                  }
                >
                  Clear
                </button>
              </div>
              <div className="mt-4 flex gap-2">
                <input
                  value={manualContributionId}
                  onChange={(event) => setManualContributionId(event.target.value)}
                  placeholder="proposal-alpha"
                  className="min-w-0 flex-1 rounded-2xl border border-white/10 bg-slate-950/70 px-3 py-2 text-sm text-white outline-none placeholder:text-slate-500"
                />
                <button
                  type="button"
                  className="rounded-2xl border border-cyan-300/30 bg-cyan-300/10 px-4 py-2 text-xs font-semibold uppercase tracking-[0.22em] text-cyan-100"
                  onClick={() =>
                    updateSelection({
                      contribution_id: manualContributionId.trim() || null,
                      node_id: manualContributionId.trim()
                        ? `contribution:${manualContributionId.trim()}`
                        : null,
                    })
                  }
                >
                  Focus
                </button>
              </div>
              {selection.focusKind === "contribution" ? (
                <p className="mt-3 text-sm leading-6 text-cyan-100">
                  Start from contribution focus. The live execution pane will launch or resume the governed flow for the contribution currently in focus.
                </p>
              ) : null}
              {selection.focusKind === "graph_run" && !selectedContributionId ? (
                <p className="mt-3 text-sm leading-6 text-amber-100">
                  Graph run focus does not yet include contribution lineage. Enter a contribution ID to unlock blast-radius inspection and steward packet export.
                </p>
              ) : null}
            </div>

            <div className="rounded-[20px] border border-white/8 bg-white/4 p-4">
              <div className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
                Selected Graph Run
              </div>
              {selectedGraphRun ? (
                <dl className="mt-3 grid grid-cols-2 gap-3 text-sm text-slate-200">
                  <div>
                    <dt className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Run</dt>
                    <dd className="mt-1 font-mono text-white">{selectedGraphRun.runId}</dd>
                  </div>
                  <div>
                    <dt className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Mode</dt>
                    <dd className="mt-1 text-white">{selectedGraphRun.mode}</dd>
                  </div>
                  <div>
                    <dt className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Status</dt>
                    <dd className="mt-1 text-white">{selectedGraphRun.status}</dd>
                  </div>
                  <div>
                    <dt className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Started</dt>
                    <dd className="mt-1 text-white">{normalizeIso(selectedGraphRun.startedAt)}</dd>
                  </div>
                </dl>
              ) : (
                <p className="mt-3 text-sm leading-6 text-slate-400">
                  No graph run selected yet. Choose a contribution-graph run from the history pane to inspect its mode, status, and timeline in context.
                </p>
              )}
            </div>

            <div className="rounded-[20px] border border-white/8 bg-white/4 p-4">
              <div className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
                Selected Agent Run
              </div>
              {selectedAgentRun ? (
                <dl className="mt-3 grid grid-cols-2 gap-3 text-sm text-slate-200">
                  <div>
                    <dt className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Run</dt>
                    <dd className="mt-1 font-mono text-white">{selectedAgentRun.runId}</dd>
                  </div>
                  <div>
                    <dt className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Status</dt>
                    <dd className="mt-1 text-white">{selectedAgentRun.status}</dd>
                  </div>
                  <div>
                    <dt className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Contribution</dt>
                    <dd className="mt-1 break-all text-white">{selectedAgentRun.contributionId}</dd>
                  </div>
                  <div>
                    <dt className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Updated</dt>
                    <dd className="mt-1 text-white">{normalizeIso(selectedAgentRun.updatedAt)}</dd>
                  </div>
                </dl>
              ) : (
                <p className="mt-3 text-sm leading-6 text-slate-400">
                  No live agent run selected yet. Select an agent run from the history pane to resume its live A2UI execution surface and steward it in place.
                </p>
              )}
            </div>

            <div className="rounded-[20px] border border-white/8 bg-white/4 p-4">
              <div className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
                Blast Radius
              </div>
              {blastRadius ? (
                <div className="mt-3 grid grid-cols-2 gap-3">
                  {blastRadiusEntries.map(([label, values]) => (
                    <div key={label} className="rounded-2xl border border-white/8 bg-slate-950/60 p-3">
                      <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500">
                        {label}
                      </div>
                      <div className="mt-2 text-sm text-white">
                        {values.length > 0 ? values.join(", ") : "None"}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="mt-3 text-sm leading-6 text-slate-400">
                  Blast radius is unavailable until a contribution is in focus. Choose or enter a contribution ID to inspect dependency, invalidation, and reference edges before steward action.
                </p>
              )}
            </div>

            {selectedContributionId ? (
              <ContributionFocusMap
                contributionId={selectedContributionId}
                blastRadius={blastRadius}
                onFocusContribution={(contributionId) =>
                  updateSelection({
                    contribution_id: contributionId,
                    node_id: `contribution:${contributionId}`,
                    run_id: null,
                  })
                }
              />
            ) : null}

            <form className="rounded-[20px] border border-white/8 bg-white/4 p-4" onSubmit={submitPipeline}>
              <div className="flex items-center justify-between gap-3">
                <div>
                  <div className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
                    Pipeline Action
                  </div>
                  <div className="mt-1 text-sm text-slate-300">
                    Run governed contribution-graph operations from the cockpit.
                  </div>
                </div>
                <select
                  value={pipelineForm.mode}
                  onChange={(event) =>
                    setPipelineForm((current) => ({ ...current, mode: event.target.value }))
                  }
                  className="rounded-2xl border border-white/10 bg-slate-950/70 px-3 py-2 text-sm text-white"
                >
                  {PIPELINE_MODES.map((mode) => (
                    <option key={mode} value={mode}>
                      {mode}
                    </option>
                  ))}
                </select>
              </div>
              <div className="mt-4 grid gap-3">
                <input
                  value={pipelineForm.goal}
                  onChange={(event) =>
                    setPipelineForm((current) => ({ ...current, goal: event.target.value }))
                  }
                  placeholder="Goal"
                  className="rounded-2xl border border-white/10 bg-slate-950/70 px-3 py-2 text-sm text-white outline-none placeholder:text-slate-500"
                />
                <input
                  value={pipelineForm.scenarioTemplateId}
                  onChange={(event) =>
                    setPipelineForm((current) => ({
                      ...current,
                      scenarioTemplateId: event.target.value,
                    }))
                  }
                  placeholder="Scenario template ID"
                  className="rounded-2xl border border-white/10 bg-slate-950/70 px-3 py-2 text-sm text-white outline-none placeholder:text-slate-500"
                />
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
                  <input
                    value={pipelineForm.publishVersion}
                    onChange={(event) =>
                      setPipelineForm((current) => ({
                        ...current,
                        publishVersion: event.target.value,
                      }))
                    }
                    placeholder="Publish version"
                    className="rounded-2xl border border-white/10 bg-slate-950/70 px-3 py-2 text-sm text-white outline-none placeholder:text-slate-500"
                  />
                  <input
                    value={pipelineForm.fromVersion}
                    onChange={(event) =>
                      setPipelineForm((current) => ({
                        ...current,
                        fromVersion: event.target.value,
                      }))
                    }
                    placeholder="From version"
                    className="rounded-2xl border border-white/10 bg-slate-950/70 px-3 py-2 text-sm text-white outline-none placeholder:text-slate-500"
                  />
                  <input
                    value={pipelineForm.toVersion}
                    onChange={(event) =>
                      setPipelineForm((current) => ({
                        ...current,
                        toVersion: event.target.value,
                      }))
                    }
                    placeholder="To version"
                    className="rounded-2xl border border-white/10 bg-slate-950/70 px-3 py-2 text-sm text-white outline-none placeholder:text-slate-500"
                  />
                </div>
                {approvalRequired ? (
                  <>
                    <textarea
                      value={pipelineForm.approvalRationale}
                      onChange={(event) =>
                        setPipelineForm((current) => ({
                          ...current,
                          approvalRationale: event.target.value,
                        }))
                      }
                      placeholder="Approval rationale"
                      className="min-h-24 rounded-[20px] border border-amber-300/20 bg-amber-300/5 px-3 py-2 text-sm text-white outline-none placeholder:text-amber-100/40"
                    />
                    <input
                      value={pipelineForm.approvalDecisionRef}
                      onChange={(event) =>
                        setPipelineForm((current) => ({
                          ...current,
                          approvalDecisionRef: event.target.value,
                        }))
                      }
                      placeholder="Decision reference"
                      className="rounded-2xl border border-amber-300/20 bg-amber-300/5 px-3 py-2 text-sm text-white outline-none placeholder:text-amber-100/40"
                    />
                  </>
                ) : null}
              </div>
              <div className="mt-4 flex flex-wrap gap-3">
                <button
                  type="submit"
                  className="rounded-2xl border border-cyan-300/30 bg-cyan-300/10 px-4 py-2 text-xs font-semibold uppercase tracking-[0.22em] text-cyan-100"
                >
                  Run Pipeline
                </button>
                <button
                  type="button"
                  disabled={!selectedContributionId}
                  className="rounded-2xl border border-amber-300/30 bg-amber-300/10 px-4 py-2 text-xs font-semibold uppercase tracking-[0.22em] text-amber-100"
                  onClick={() => void submitStewardPacket()}
                >
                  Export Steward Packet
                </button>
              </div>
              {!selectedContributionId ? (
                <p className="mt-3 text-xs uppercase tracking-[0.2em] text-amber-200/70">
                  Steward packet export unlocks once a contribution is in focus.
                </p>
              ) : null}
            </form>

            {pipelineResponse ? (
              <div className="rounded-[20px] border border-emerald-300/20 bg-emerald-300/5 p-4">
                <div className="text-[10px] uppercase tracking-[0.24em] text-emerald-200/70">
                  Pipeline Result
                </div>
                <div className="mt-2 text-sm text-white">
                  {pipelineResponse.runId} · {pipelineResponse.mode} · {pipelineResponse.status}
                </div>
              </div>
            ) : null}

            {packetResponse ? (
              <div className="rounded-[20px] border border-amber-300/20 bg-amber-300/5 p-4">
                <div className="text-[10px] uppercase tracking-[0.24em] text-amber-200/70">
                  Steward Packet
                </div>
                <div className="mt-2 break-all text-sm text-white">
                  {packetResponse.packetPath}
                </div>
              </div>
            ) : null}

            {pipelineError ? (
              <div className="rounded-[20px] border border-rose-300/20 bg-rose-300/5 p-4 text-sm text-rose-100">
                {pipelineError}
              </div>
            ) : null}
          </div>
        </section>

        <section
          className={panelShell("overflow-hidden")}
          role="region"
          aria-label="Contributions Live Execution"
          data-testid="contributions-live-pane"
        >
          <div className="border-b border-white/8 px-5 py-4">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <h2 className="font-mono text-sm uppercase tracking-[0.26em] text-slate-300">
                Live Execution
              </h2>
              {selectedContributionId && !liveRunId ? (
                <button
                  type="button"
                  className="rounded-2xl border border-cyan-300/30 bg-cyan-300/10 px-4 py-2 text-xs font-semibold uppercase tracking-[0.22em] text-cyan-100"
                  onClick={launchLiveExecution}
                >
                  Launch Live Run
                </button>
              ) : null}
            </div>
          </div>
          <div className="p-4">
            <A2UIRenderSpace
              spaceId={activeSpaceId}
              selectedContributionId={selectedContributionId}
              launchContributionId={launchContributionId}
              launchRequestToken={launchRequestToken}
              selectedRunId={liveRunId}
            />
          </div>
        </section>
      </div>
    </div>
  );
}
