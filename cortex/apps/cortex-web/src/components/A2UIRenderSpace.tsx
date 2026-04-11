import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { gatewayWsBase, workbenchApi } from "../api";
import { AgentRunEventEnvelope, AgentRunRecord, SpatialExperimentRunSummary, SpatialSurfaceVariant } from "../contracts";
import { A2UIInterpreter } from "./a2ui/A2UIInterpreter";
import { A2UI_EVENT_TYPES } from "./a2ui/spatialEventContract";

const INTERACTION_EVENTS = new Set([
  "button_click",
  "approval",
  "spatial_shape_click",
  "spatial_shape_move",
  "spatial_edge_connect"
]);
const TRACKED_EVENT_TYPES = new Set<string>(A2UI_EVENT_TYPES);
const SERVER_RUN_EVENT_TYPES = new Set([
  "run_started",
  "simulation_ready",
  "surface_update",
  "approval_required",
  "run_completed",
  "run_failed"
]);

function isoNow(): string {
  return new Date().toISOString();
}

function sanitizeRunToken(value: string): string {
  return value
    .trim()
    .replace(/[^a-zA-Z0-9:_-]+/g, "_")
    .slice(0, 120);
}

function toWsUrl(streamChannel: string): string {
  if (streamChannel.startsWith("ws://") || streamChannel.startsWith("wss://")) {
    return streamChannel;
  }
  const base = gatewayWsBase();
  if (streamChannel.startsWith("/")) {
    return `${base}${streamChannel}`;
  }
  return `${base}/${streamChannel}`;
}

function payloadRecord(value: unknown): Record<string, unknown> {
  if (value && typeof value === "object" && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return {};
}

function asRunEventEnvelope(value: Record<string, unknown>): AgentRunEventEnvelope | null {
  if (
    typeof value.type !== "string" ||
    typeof value.runId !== "string" ||
    typeof value.spaceId !== "string" ||
    typeof value.timestamp !== "string"
  ) {
    return null;
  }
  return {
    type: value.type,
    runId: value.runId,
    spaceId: value.spaceId,
    timestamp: value.timestamp,
    sequence: typeof value.sequence === "number" ? value.sequence : 0,
    payload: payloadRecord(value.payload)
  };
}

type SpatialMetricsState = {
  total: number;
  approvals: number;
  buttonClicks: number;
  spatialClicks: number;
  adapterLoaded: number;
  adapterFallbacks: number;
  replayFailures: number;
  contractInvalidErrors: number;
  errorEvents: number;
  lastEvent: string;
};

/**
 * A2UIRenderSpace
 *
 * This component acts as the execution engine for A2UI schemas streamed from the runtime.
 * UI state is server-authoritative for contribution lifecycle and surface updates.
 */
export function A2UIRenderSpace({
  selectedNodeId,
  selectedContributionId,
  launchContributionId,
  launchRequestToken,
  selectedRunId,
  buildId,
  spaceId
}: {
  selectedNodeId?: string | null;
  selectedContributionId?: string | null;
  launchContributionId?: string | null;
  launchRequestToken?: number | null;
  selectedRunId?: string | null;
  buildId?: string;
  spaceId: string;
}) {
  const [a2uiPayload, setA2uiPayload] = useState<any>(null);
  const [evalMode, setEvalMode] = useState<SpatialSurfaceVariant>("compare");
  const [runStartedAt, setRunStartedAt] = useState<number | null>(null);
  const [experimentRunId, setExperimentRunId] = useState<string | null>(null);
  const [runStatus, setRunStatus] = useState<string>("idle");
  const [runTerminal, setRunTerminal] = useState(false);
  const [firstInteractionMs, setFirstInteractionMs] = useState<number | null>(null);
  const [summaryCompleted, setSummaryCompleted] = useState(false);
  const [persistError, setPersistError] = useState<string | null>(null);
  const [persistedSummary, setPersistedSummary] = useState<SpatialExperimentRunSummary | null>(null);
  const [metrics, setMetrics] = useState<SpatialMetricsState>({
    total: 0,
    approvals: 0,
    buttonClicks: 0,
    spatialClicks: 0,
    adapterLoaded: 0,
    adapterFallbacks: 0,
    replayFailures: 0,
    contractInvalidErrors: 0,
    errorEvents: 0,
    lastEvent: "none"
  });

  const runIdRef = useRef<string | null>(null);
  const socketRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<number | null>(null);
  const seenEventKeysRef = useRef<Set<string>>(new Set());
  const lastSequenceRef = useRef<number>(0);
  const lastLaunchRequestKeyRef = useRef<string | null>(null);

  const stopSocket = useCallback(() => {
    if (reconnectTimerRef.current !== null) {
      window.clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
    if (socketRef.current) {
      socketRef.current.close();
      socketRef.current = null;
    }
  }, []);

  const postExperimentEvent = useCallback(
    async (
      eventType: string,
      payload: Record<string, unknown>,
      runIdOverride?: string,
      surfaceVariantOverride?: SpatialSurfaceVariant
    ) => {
      const runId = runIdOverride ?? experimentRunId;
      if (!runId) return;

      try {
        await workbenchApi.postSpatialExperimentEvent({
          run_id: runId,
          space_id: spaceId,
          mode: "evaluation_phase5",
          surface_variant: surfaceVariantOverride ?? evalMode,
          event_type: eventType,
          timestamp: isoNow(),
          payload,
          build_id: buildId,
          host: "cortex-web"
        });
      } catch (err) {
        setPersistError(err instanceof Error ? err.message : String(err));
      }
    },
    [buildId, evalMode, experimentRunId, spaceId]
  );

  const resetRunState = useCallback(() => {
    setExperimentRunId(null);
    setRunStartedAt(null);
    setRunStatus("idle");
    setRunTerminal(false);
    seenEventKeysRef.current = new Set();
    lastSequenceRef.current = 0;
    setFirstInteractionMs(null);
    setSummaryCompleted(false);
    setPersistError(null);
    setPersistedSummary(null);
    setA2uiPayload(null);
    setMetrics({
      total: 0,
      approvals: 0,
      buttonClicks: 0,
      spatialClicks: 0,
      adapterLoaded: 0,
      adapterFallbacks: 0,
      replayFailures: 0,
      contractInvalidErrors: 0,
      errorEvents: 0,
      lastEvent: "none"
    });
  }, []);

  const restoreRunSnapshot = useCallback(
    (run: AgentRunRecord) => {
      setRunStatus(run.status ?? "unknown");
      setRunTerminal(["completed", "rejected", "failed"].includes(run.status ?? ""));
      lastSequenceRef.current = Math.max(
        0,
        ...(run.events ?? []).map((event) => event.sequence ?? 0)
      );
      seenEventKeysRef.current = new Set(
        (run.events ?? []).map((event) => `${event.runId}:${event.sequence}`)
      );
      if (run.surfaceUpdate) {
        setA2uiPayload(run.surfaceUpdate);
      }
      if (run.startedAt) {
        const started = Date.parse(run.startedAt);
        if (!Number.isNaN(started)) {
          setRunStartedAt(started);
        }
      }
    },
    []
  );

  const onRunEvent = useCallback(
    (event: AgentRunEventEnvelope) => {
      if (!SERVER_RUN_EVENT_TYPES.has(event.type)) {
        return;
      }
      const payload = payloadRecord(event.payload);
      const status = payload.status;
      if (typeof status === "string") {
        setRunStatus(status);
      }
      if (event.type === "surface_update") {
        const surfaceUpdate = payload.surfaceUpdate;
        if (surfaceUpdate && typeof surfaceUpdate === "object") {
          setA2uiPayload(surfaceUpdate);
        }
      }
      if (event.type === "run_completed") {
        const decisionStatus = typeof status === "string" ? status : "completed";
        setRunStatus(decisionStatus);
        setRunTerminal(true);
      }
      if (event.type === "run_failed") {
        setRunTerminal(true);
        const error = payload.error;
        if (typeof error === "string") {
          setPersistError(error);
        }
      }
      if (event.type === "approval_required") {
        setRunTerminal(false);
      }
    },
    []
  );

  const connectRunSocket = useCallback(
    (streamChannel: string, runId: string) => {
      stopSocket();
      let cancelled = false;

      const connect = () => {
        if (cancelled) return;
        const ws = new WebSocket(toWsUrl(streamChannel));
        socketRef.current = ws;

        ws.onmessage = (messageEvent) => {
          const raw = typeof messageEvent.data === "string" ? messageEvent.data : "";
          if (!raw) return;
          let parsed: unknown;
          try {
            parsed = JSON.parse(raw);
          } catch {
            return;
          }
          const envelope = asRunEventEnvelope(payloadRecord(parsed));
          if (!envelope) {
            return;
          }
          if (envelope.runId !== runIdRef.current) {
            return;
          }
          if (envelope.sequence <= lastSequenceRef.current) {
            return;
          }
          const eventKey = `${envelope.runId}:${envelope.sequence}`;
          if (seenEventKeysRef.current.has(eventKey)) {
            return;
          }
          lastSequenceRef.current = envelope.sequence;
          seenEventKeysRef.current.add(eventKey);
          onRunEvent(envelope);
        };

        ws.onclose = () => {
          if (cancelled) {
            return;
          }
          reconnectTimerRef.current = window.setTimeout(async () => {
            try {
              const snapshot = await workbenchApi.getAgentRun(spaceId, runId);
              restoreRunSnapshot(snapshot);
            } catch {
              // No-op: next reconnect attempt keeps trying.
            }
            connect();
          }, 1000);
        };
      };

      connect();

      return () => {
        cancelled = true;
        stopSocket();
      };
    },
    [onRunEvent, restoreRunSnapshot, spaceId, stopSocket]
  );

  useEffect(() => {
    const contributionId = launchContributionId?.trim() || null;
    const existingRunId = selectedRunId?.trim() || null;
    const hasLaunchRequest =
      contributionId && launchRequestToken !== null && launchRequestToken !== undefined;

    if (existingRunId || hasLaunchRequest) {
      return;
    }

    runIdRef.current = null;
    lastLaunchRequestKeyRef.current = null;
    stopSocket();
    resetRunState();
  }, [launchContributionId, launchRequestToken, resetRunState, selectedRunId, stopSocket]);

  useEffect(() => {
    const existingRunId = selectedRunId?.trim() || null;
    if (!existingRunId) {
      return;
    }

    lastLaunchRequestKeyRef.current = null;

    let cancelled = false;
    let disconnectSocket: (() => void) | null = null;

    const resumeExistingRun = async () => {
      try {
        resetRunState();
        const snapshot = await workbenchApi.getAgentRun(spaceId, existingRunId);
        if (cancelled) return;
        runIdRef.current = existingRunId;
        setExperimentRunId(existingRunId);
        restoreRunSnapshot(snapshot);
        if (snapshot.streamChannel) {
          disconnectSocket = connectRunSocket(snapshot.streamChannel, existingRunId);
        }
      } catch (err) {
        if (!cancelled) {
          setPersistError(err instanceof Error ? err.message : String(err));
        }
      }
    };

    void resumeExistingRun();

    return () => {
      cancelled = true;
      runIdRef.current = null;
      if (disconnectSocket) {
        disconnectSocket();
      } else {
        stopSocket();
      }
    };
  }, [connectRunSocket, resetRunState, restoreRunSnapshot, selectedRunId, spaceId, stopSocket]);

  useEffect(() => {
    const existingRunId = selectedRunId?.trim() || null;
    const contributionId = launchContributionId?.trim() || null;
    const launchRequestKey =
      contributionId && launchRequestToken !== null && launchRequestToken !== undefined
        ? `${spaceId}:${contributionId}:${launchRequestToken}`
        : null;

    if (existingRunId || !launchRequestKey || !contributionId) {
      return;
    }
    if (launchRequestKey === lastLaunchRequestKeyRef.current) {
      return;
    }

    lastLaunchRequestKeyRef.current = launchRequestKey;

    let cancelled = false;
    let disconnectSocket: (() => void) | null = null;

    const dispatchInitiative = async () => {
      try {
        resetRunState();
        const normalizedContributionId = sanitizeRunToken(contributionId);
        const response = await workbenchApi.startAgentContribution(spaceId, normalizedContributionId);
        if (cancelled) return;

        const startedAt = Date.parse(response.startedAt);
        const runStart = Number.isNaN(startedAt) ? Date.now() : startedAt;

        runIdRef.current = response.runId;
        setRunStartedAt(runStart);
        setExperimentRunId(response.runId);
        setRunStatus(response.status);
        setRunTerminal(false);

        await postExperimentEvent(
          "run_start",
          {
            startedAt: new Date(runStart).toISOString(),
            selectedNodeId,
            contributionId: normalizedContributionId,
            streamChannel: response.streamChannel,
            runtimeMode: response.runtimeMode ?? "unknown"
          },
          response.runId,
          "compare"
        );

        try {
          const snapshot = await workbenchApi.getAgentRun(spaceId, response.runId);
          if (!cancelled) {
            restoreRunSnapshot(snapshot);
          }
        } catch {
          // Snapshot is best-effort; WS stream still drives state.
        }

        disconnectSocket = connectRunSocket(response.streamChannel, response.runId);
      } catch (err) {
        lastLaunchRequestKeyRef.current = null;
        if (!cancelled) {
          setPersistError(err instanceof Error ? err.message : String(err));
        }
      }
    };

    void dispatchInitiative();

    return () => {
      cancelled = true;
      runIdRef.current = null;
      if (disconnectSocket) {
        disconnectSocket();
      } else {
        stopSocket();
      }
    };
  }, [
    connectRunSocket,
    launchContributionId,
    launchRequestToken,
    postExperimentEvent,
    resetRunState,
    restoreRunSnapshot,
    selectedNodeId,
    selectedRunId,
    spaceId,
    stopSocket,
  ]);

  useEffect(() => {
    if (!experimentRunId || summaryCompleted) return;
    void postExperimentEvent(
      "mode_switch",
      {
        surfaceVariant: evalMode
      },
      experimentRunId,
      evalMode
    );
  }, [evalMode, experimentRunId, postExperimentEvent, summaryCompleted]);

  useEffect(() => {
    const onA2uiEvent = (event: Event) => {
      const detail = (event as CustomEvent<Record<string, unknown>>).detail ?? {};
      const eventType = String(detail.eventType ?? "unknown");
      if (!TRACKED_EVENT_TYPES.has(eventType)) {
        return;
      }

      setMetrics((current) => {
        const reasonClass = String(detail.reasonClass ?? "");
        const approvalStatus = String(detail.status ?? "success");
        const isApprovalSuccess = eventType === "approval" && approvalStatus !== "error";
        const isApprovalError = eventType === "approval" && approvalStatus === "error";
        const isContractInvalid = reasonClass === "contract_invalid";
        const isError =
          eventType === "spatial_adapter_fallback" ||
          eventType === "spatial_adapter_replay_failed" ||
          isApprovalError;
        return {
          total: current.total + 1,
          approvals: current.approvals + (isApprovalSuccess ? 1 : 0),
          buttonClicks: current.buttonClicks + (eventType === "button_click" ? 1 : 0),
          spatialClicks: current.spatialClicks + (
            eventType === "spatial_shape_click" ||
            eventType === "spatial_shape_move" ||
            eventType === "spatial_edge_connect"
              ? 1
              : 0
          ),
          adapterLoaded: current.adapterLoaded + (eventType === "spatial_adapter_loaded" ? 1 : 0),
          adapterFallbacks: current.adapterFallbacks + (eventType === "spatial_adapter_fallback" ? 1 : 0),
          replayFailures: current.replayFailures + (eventType === "spatial_adapter_replay_failed" ? 1 : 0),
          contractInvalidErrors: current.contractInvalidErrors + (isContractInvalid ? 1 : 0),
          errorEvents: current.errorEvents + (isError ? 1 : 0),
          lastEvent: eventType
        };
      });

      if (INTERACTION_EVENTS.has(eventType)) {
        setFirstInteractionMs((current) => {
          if (current !== null || runStartedAt === null) return current;
          return Date.now() - runStartedAt;
        });
      }

      void postExperimentEvent(eventType, detail);
    };

    window.addEventListener("cortex:a2ui:event", onA2uiEvent as EventListener);
    return () => {
      window.removeEventListener("cortex:a2ui:event", onA2uiEvent as EventListener);
    };
  }, [postExperimentEvent, runStartedAt]);

  const linearPayload = useMemo(() => {
    if (!a2uiPayload) return null;
    return {
      ...a2uiPayload,
      children: {
        ...a2uiPayload.children,
        explicitList: (a2uiPayload.children?.explicitList ?? []).filter((child: any) => {
          const keys = Object.keys(child.componentProperties ?? {});
          return !keys.includes("SpatialPlane");
        })
      }
    };
  }, [a2uiPayload]);

  async function completeRun(): Promise<void> {
    if (!experimentRunId || runStartedAt === null || summaryCompleted || !runTerminal) {
      return;
    }

    const completedAt = Date.now();
    const taskCompletionMs = Math.max(0, completedAt - runStartedAt);
    const adapterTotal = metrics.adapterLoaded + metrics.adapterFallbacks;
    const fallbackRate = adapterTotal === 0 ? 0 : metrics.adapterFallbacks / adapterTotal;

    const improvementScoreRaw =
      metrics.spatialClicks * 1.5 +
      metrics.approvals * 2 +
      metrics.buttonClicks * 0.5 +
      (firstInteractionMs !== null ? Math.max(0, (5000 - firstInteractionMs) / 1000) : 0) -
      metrics.errorEvents * 3 -
      fallbackRate * 10;
    const improvementScore = Number(improvementScoreRaw.toFixed(2));

    const recommendation: "go" | "no_go" | "hold" =
      metrics.errorEvents > 0 || fallbackRate > 0.2
        ? "no_go"
        : improvementScore >= 3 && metrics.approvals > 0
          ? "go"
          : "hold";

    const verdictRationale =
      recommendation === "go"
        ? "Spatial interaction produced stable operator signal with acceptable overhead."
        : recommendation === "no_go"
          ? "Fallback/error profile exceeded acceptable threshold for promotion."
          : "Signal is mixed; gather another run before promotion decision.";

    const summaryPayload = {
      metrics: {
        timeToFirstInteractionMs: firstInteractionMs ?? undefined,
        taskCompletionMs,
        approvalDecisionCount: metrics.approvals,
        spatialInteractionCount: metrics.spatialClicks,
        adapterFallbackRate: fallbackRate,
        errorEventCount: metrics.errorEvents
      },
      improvementScore,
      recommendation,
      complexityDelta: {
        bundleDeltaKb: null,
        runtimeOverheadMs: taskCompletionMs,
        adapterFallbackRate: fallbackRate
      },
      verdictRationale,
      runStatus
    };

    await postExperimentEvent("run_end", summaryPayload, experimentRunId, evalMode);

    try {
      const summary = await workbenchApi.getSpatialExperimentRun(experimentRunId);
      setPersistedSummary(summary);
    } catch (err) {
      setPersistError(err instanceof Error ? err.message : String(err));
    }

    setSummaryCompleted(true);
  }

  if (!a2uiPayload || !linearPayload) {
    return <div className="a2ui-placeholder">A2UI Substrate Idle</div>;
  }

  return (
    <div className="a2ui-execution-buffer">
      <h3>Agent A2UI Execution Space</h3>
      <div className="a2ui-eval-row">
        <div className="a2ui-eval-controls">
          <button onClick={() => setEvalMode("linear")}>Linear</button>
          <button onClick={() => setEvalMode("spatial")}>Spatial</button>
          <button onClick={() => setEvalMode("compare")}>Compare</button>
          <button onClick={() => void completeRun()} disabled={summaryCompleted || !experimentRunId || !runTerminal}>
            Complete Run
          </button>
        </div>
        <div className="a2ui-eval-metrics">
          <span>run_id={experimentRunId ?? "n/a"}</span>
          <span>status={runStatus}</span>
          <span>events={metrics.total}</span>
          <span>approvals={metrics.approvals}</span>
          <span>button_clicks={metrics.buttonClicks}</span>
          <span>spatial_clicks={metrics.spatialClicks}</span>
          <span>adapter_loaded={metrics.adapterLoaded}</span>
          <span>adapter_fallbacks={metrics.adapterFallbacks}</span>
          <span>replay_failures={metrics.replayFailures}</span>
          <span>contract_invalid={metrics.contractInvalidErrors}</span>
          <span>time_to_first_ms={firstInteractionMs ?? "n/a"}</span>
          <span>last_event={metrics.lastEvent}</span>
        </div>
        {persistError && <div className="a2ui-spatial-plane__adapter-note">event persistence error: {persistError}</div>}
        {persistedSummary && (
          <div className="a2ui-eval-summary">
            <strong>verdict={persistedSummary.recommendation}</strong>
            <span>score={persistedSummary.improvement_score}</span>
            <span>completion_ms={persistedSummary.metrics.task_completion_ms ?? "n/a"}</span>
            <span>fallback_rate={persistedSummary.metrics.adapter_fallback_rate.toFixed(3)}</span>
          </div>
        )}
      </div>
      {evalMode === "compare" ? (
        <div className="a2ui-compare-grid">
          <div className="a2ui-render-surface p-4 bg-slate-50">
            <h4 className="a2ui-surface-label">Linear Surface</h4>
            <A2UIInterpreter node={linearPayload} />
          </div>
          <div className="a2ui-render-surface p-4 bg-slate-50">
            <h4 className="a2ui-surface-label">Spatial Surface</h4>
            <A2UIInterpreter node={a2uiPayload} />
          </div>
        </div>
      ) : (
        <div className="a2ui-render-surface p-4 bg-slate-50">
          <h4 className="a2ui-surface-label">
            {evalMode === "linear" ? "Linear Surface" : "Spatial Surface"}
          </h4>
          <A2UIInterpreter node={evalMode === "linear" ? linearPayload : a2uiPayload} />
        </div>
      )}
    </div>
  );
}
