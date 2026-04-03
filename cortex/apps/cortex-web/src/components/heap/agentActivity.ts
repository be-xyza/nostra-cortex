import { buildBenchmarkProjection } from "./benchmarkProjection.ts";

export interface AgentEvent {
  id: string;
  timestamp: string;
  agent: string;
  action: string;
  details?: string;
  status?: "started" | "running" | "completed" | "failed" | "idle";
}

export function parseAgentActivityEvent(data: Record<string, any>): AgentEvent | null {
  const payload = data?.payload || {};
  
  // Try extracting agent from payload, fallback to source if it's a known system source
  const agentId = payload.agent_id || payload.agentId || payload.agent || (data.source ? data.source : null);
  const type = String(payload.type || data.type || data.topic || "unknown");

  if (!agentId && !type.includes("workflow") && !type.includes("agent")) {
    return null;
  }

  const ts = String(data.timestamp || payload.timestamp || new Date().toISOString());
  const runId = String(payload.execution_id || payload.instance_id || payload.run_id || "sys");
  const seq = String(data.sequence || payload.step_id || payload.attempt_id || 0);
  
  const phase = payload.phase ? String(payload.phase) : null;
  const benchmark = buildBenchmarkProjection(payload.benchmark);
  const providerKind = payload.provider_kind ? String(payload.provider_kind) : null;
  const authMode = payload.auth_mode ? String(payload.auth_mode) : null;
  const actionStr = payload.action ? String(payload.action) : type;

  let evtStatus: AgentEvent["status"] = "running";
  const normalizedStatus = String(payload.status || type).toLowerCase();
  
  if (normalizedStatus.includes("start") || normalizedStatus.includes("trigger")) evtStatus = "started";
  if (normalizedStatus.includes("complete") || normalizedStatus.includes("finish") || normalizedStatus.includes("success")) evtStatus = "completed";
  if (normalizedStatus.includes("warn") || normalizedStatus.includes("fail") || normalizedStatus.includes("error")) evtStatus = "failed";
  if (normalizedStatus.includes("idle") || normalizedStatus.includes("pause")) evtStatus = "idle";

  const detailParts = [
    phase,
    benchmark?.summary ?? null,
    providerKind ? `provider:${providerKind}` : null,
    authMode && authMode !== providerKind ? `auth:${authMode}` : null,
  ].filter(Boolean);

  return {
    id: `${runId}-${seq}-${ts}`,
    timestamp: ts,
    agent: String(agentId || "system-worker"),
    action: actionStr,
    details: detailParts.length > 0 ? detailParts.join(" • ") : (payload.message ? String(payload.message) : undefined),
    status: evtStatus,
  };
}
