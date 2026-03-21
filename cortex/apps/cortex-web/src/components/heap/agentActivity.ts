export interface AgentEvent {
  id: string;
  timestamp: string;
  agent: string;
  action: string;
  details?: string;
  status?: "started" | "running" | "completed" | "failed" | "idle";
}

export function parseAgentActivityEvent(data: any): AgentEvent | null {
  const payload = data?.payload ?? {};
  const type = String(data?.type || payload?.type || "unknown");
  const agentId = payload?.agent_id || payload?.agentId || payload?.agent || data?.agentId;

  if (!agentId && !type.includes("agent")) {
    return null;
  }

  const ts = String(data?.timestamp || payload?.timestamp || new Date().toISOString());
  const runId = String(payload?.execution_id || payload?.run_id || data?.runId || "sys");
  const seq = String(data?.sequence || payload?.attempt_id || 0);
  const phase = payload?.phase ? String(payload.phase) : null;
  const grade = payload?.benchmark?.overall_grade ? String(payload.benchmark.overall_grade) : null;
  const providerKind = payload?.provider_kind ? String(payload.provider_kind) : null;
  const authMode = payload?.auth_mode ? String(payload.auth_mode) : null;
  const decision = payload?.decision ? String(payload.decision) : null;

  let evtStatus: AgentEvent["status"] = "running";
  const normalizedStatus = String(payload?.status || type).toLowerCase();
  if (normalizedStatus.includes("start")) evtStatus = "started";
  if (normalizedStatus.includes("complete") || normalizedStatus.includes("finish")) evtStatus = "completed";
  if (normalizedStatus.includes("warn") || normalizedStatus.includes("fail") || normalizedStatus.includes("error")) evtStatus = "failed";
  if (normalizedStatus.includes("idle")) evtStatus = "idle";

  const detailParts = [
    phase,
    grade,
    providerKind ? `provider:${providerKind}` : null,
    authMode && authMode !== providerKind ? `auth:${authMode}` : null,
    decision,
  ].filter(Boolean);

  return {
    id: `${runId}-${seq}-${ts}`,
    timestamp: ts,
    agent: String(agentId || "eudaemon"),
    action: type,
    details: detailParts.length > 0 ? detailParts.join(" • ") : (payload?.message ? String(payload.message) : undefined),
    status: evtStatus,
  };
}
