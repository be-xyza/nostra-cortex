import type {
  AgentContributionResponse,
  BrandPolicyResponse,
  AgentRunRecord,
  AgentRunApprovalRequest,
  CompiledNavigationPlan,
  DpubPipelineRunReport,
  DpubPipelineRunRequest,
  DpubSystemBuildResponse,
  DpubSystemReadyResponse,
  EmitHeapBlockRequest,
  ArtifactPublishRequest,
  HeapBlocksQueryParams,
  HeapChangedBlocksResponse,
  HeapStewardGateApplyResponse,
  HeapStewardGateValidateResponse,
  HeapBlockHistoryResponse,
  HeapBlocksContextResponse,
  HeapBlocksResponse,
  ContributionGraph,
  PathAssessment,
  PlatformCapabilityCatalog,
  PlatformCapabilityGraph,
  SpaceCapabilityGraph,
  SpaceCreateRequest,
  SpaceCreateResponse,
  SpatialExperimentEventRequest,
  SpatialExperimentEventResponse,
  SpatialExperimentRunSummary,
  ShellLayoutSpec
} from "./contracts.ts";

const BASE =
  ((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_CORTEX_GATEWAY_URL as string | undefined) ??
  "http://127.0.0.1:3000";
export const SPACE_ID = "nostra-governance-v0";

export function gatewayWsBase(): string {
  return BASE.replace(/^http/i, "ws");
}

export function gatewayBaseUrl(): string {
  return BASE;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {})
    }
  });
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`${response.status} ${response.statusText}: ${body}`);
  }
  return (await response.json()) as T;
}

async function requestTextOrJson(path: string, init?: RequestInit): Promise<string | Record<string, unknown>> {
  const response = await fetch(`${BASE}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {})
    }
  });
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`${response.status} ${response.statusText}: ${body}`);
  }
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    return (await response.json()) as Record<string, unknown>;
  }
  return response.text();
}

function assertValidEmitHeapBlockRequest(payload: EmitHeapBlockRequest): void {
  if (payload.schema_version !== "1.0.0" || payload.mode !== "heap") {
    throw new Error("emitHeapBlock requires schema_version=1.0.0 and mode=heap");
  }
  if (!payload.workspace_id?.trim()) {
    throw new Error("emitHeapBlock requires workspace_id");
  }
  if (!payload.source?.agent_id?.trim() || !payload.source?.emitted_at?.trim()) {
    throw new Error("emitHeapBlock requires source.agent_id and source.emitted_at");
  }
  if (!payload.block?.type?.trim() || !payload.block?.title?.trim()) {
    throw new Error("emitHeapBlock requires block.type and block.title");
  }
  const payloadType = payload.content?.payload_type;
  if (!payloadType) {
    throw new Error("emitHeapBlock requires content.payload_type");
  }
  if (payloadType === "a2ui" && !payload.content.a2ui) {
    throw new Error("emitHeapBlock requires content.a2ui when payload_type=a2ui");
  }
  if (payloadType === "rich_text" && !payload.content.rich_text) {
    throw new Error("emitHeapBlock requires content.rich_text when payload_type=rich_text");
  }
  if (payloadType === "media" && !payload.content.media) {
    throw new Error("emitHeapBlock requires content.media when payload_type=media");
  }
  if (payloadType === "structured_data" && !payload.content.structured_data) {
    throw new Error("emitHeapBlock requires content.structured_data when payload_type=structured_data");
  }
  if (payloadType === "pointer" && !payload.content.pointer) {
    throw new Error("emitHeapBlock requires content.pointer when payload_type=pointer");
  }
}

export const workbenchApi = {
  getShellLayout: () => request<ShellLayoutSpec>("/api/cortex/layout/spec"),
  getReady: () => request<DpubSystemReadyResponse>("/api/system/ready"),
  getBuild: () => request<DpubSystemBuildResponse>("/api/system/build"),
  getOverview: () => request<Record<string, unknown>>(`/api/kg/spaces/${SPACE_ID}/contribution-graph/overview`),
  getGraph: () => request<ContributionGraph>(`/api/kg/spaces/${SPACE_ID}/contribution-graph/graph?mode=d3-force`),
  getCapabilityGraph: () => request<PlatformCapabilityGraph>(`/api/system/capability-graph`),
  getCapabilityCatalog: () => request<PlatformCapabilityCatalog>(`/api/system/capability-catalog`),
  getSpaceCapabilityGraph: (spaceId: string) =>
    request<SpaceCapabilityGraph>(`/api/spaces/${encodeURIComponent(spaceId)}/capability-graph`),
  putSpaceCapabilityGraph: (
    spaceId: string,
    payload: SpaceCapabilityGraph,
    actorRole = "steward",
    actorId = "cortex-web"
  ) =>
    request<SpaceCapabilityGraph>(`/api/spaces/${encodeURIComponent(spaceId)}/capability-graph`, {
      method: "PUT",
      headers: {
        "x-cortex-role": actorRole,
        "x-cortex-actor": actorId
      },
      body: JSON.stringify(payload)
    }),
  getSpaceNavigationPlan: (
    spaceId: string,
    params: { actorRole: string; intent?: string; density?: string }
  ) => {
    const query = new URLSearchParams();
    query.set("actor_role", params.actorRole);
    if (params.intent) query.set("intent", params.intent);
    if (params.density) query.set("density", params.density);
    return request<CompiledNavigationPlan>(
      `/api/spaces/${encodeURIComponent(spaceId)}/navigation-plan?${query.toString()}`
    );
  },
  getPath: () => request<PathAssessment>(`/api/kg/spaces/${SPACE_ID}/contribution-graph/path-assessment`),
  getRuns: () => request<DpubPipelineRunReport[]>(`/api/kg/spaces/${SPACE_ID}/contribution-graph/runs?limit=10`),
  createSpace: (payload: SpaceCreateRequest) =>
    request<SpaceCreateResponse>(`/api/spaces/create`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  startAgentContribution: (spaceId: string, contributionId: string) =>
    request<AgentContributionResponse>(`/api/kg/spaces/${spaceId}/agents/contributions`, {
      method: "POST",
      body: JSON.stringify({ contributionId })
    }),
  getAgentRun: (spaceId: string, runId: string) =>
    request<AgentRunRecord>(`/api/kg/spaces/${spaceId}/agents/contributions/${runId}`),
  approveAgentRun: (spaceId: string, runId: string, payload: AgentRunApprovalRequest) =>
    request<{ accepted: boolean; runId: string; status: string }>(`/api/kg/spaces/${spaceId}/agents/contributions/${runId}/approval`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  postSpatialExperimentEvent: (payload: SpatialExperimentEventRequest) =>
    request<SpatialExperimentEventResponse>(`/api/cortex/viewspecs/experiments/spatial/events`, {
      method: "POST",
      body: JSON.stringify(payload)
    }),
  getSpatialExperimentRun: (runId: string) =>
    request<SpatialExperimentRunSummary>(`/api/cortex/viewspecs/experiments/spatial/runs/${runId}`),
  runPipeline: (payload: DpubPipelineRunRequest, actorRole: string, actorId: string) =>
    request<DpubPipelineRunReport>(`/api/kg/spaces/${SPACE_ID}/contribution-graph/pipeline/run`, {
      method: "POST",
      headers: {
        "x-cortex-role": actorRole,
        "x-cortex-actor": actorId
      },
      body: JSON.stringify(payload)
    }),
  getHeapBlocks: (params?: HeapBlocksQueryParams) => {
    const query = new URLSearchParams();
    if (params?.spaceId) query.set("spaceId", params.spaceId);
    if (params?.tag) query.set("tag", params.tag);
    if (params?.mention) query.set("mention", params.mention);
    if (params?.pageLink) query.set("pageLink", params.pageLink);
    if (params?.attribute) query.set("attribute", params.attribute);
    if (params?.blockType) query.set("blockType", params.blockType);
    if (typeof params?.hasFiles === "boolean") query.set("hasFiles", String(params.hasFiles));
    if (params?.fromTs) query.set("fromTs", params.fromTs);
    if (params?.changedSince) query.set("changedSince", params.changedSince);
    if (params?.toTs) query.set("toTs", params.toTs);
    if (typeof params?.includeDeleted === "boolean") query.set("includeDeleted", String(params.includeDeleted));
    if (typeof params?.limit === "number") query.set("limit", String(params.limit));
    if (params?.cursor) query.set("cursor", params.cursor);
    const suffix = query.toString();
    return request<HeapBlocksResponse>(`/api/cortex/studio/heap/blocks${suffix ? `?${suffix}` : ""}`);
  },
  getHeapChangedBlocks: (params?: HeapBlocksQueryParams) => {
    const query = new URLSearchParams();
    if (params?.spaceId) query.set("spaceId", params.spaceId);
    if (params?.tag) query.set("tag", params.tag);
    if (params?.mention) query.set("mention", params.mention);
    if (params?.pageLink) query.set("pageLink", params.pageLink);
    if (params?.attribute) query.set("attribute", params.attribute);
    if (params?.blockType) query.set("blockType", params.blockType);
    if (typeof params?.hasFiles === "boolean") query.set("hasFiles", String(params.hasFiles));
    if (params?.fromTs) query.set("fromTs", params.fromTs);
    if (params?.changedSince) query.set("changedSince", params.changedSince);
    if (params?.toTs) query.set("toTs", params.toTs);
    if (typeof params?.includeDeleted === "boolean") query.set("includeDeleted", String(params.includeDeleted));
    if (typeof params?.limit === "number") query.set("limit", String(params.limit));
    if (params?.cursor) query.set("cursor", params.cursor);
    const suffix = query.toString();
    return request<HeapChangedBlocksResponse>(`/api/cortex/studio/heap/changed_blocks${suffix ? `?${suffix}` : ""}`);
  },
  createHeapContextBundle: (artifactIds: string[]) =>
    request<HeapBlocksContextResponse>(`/api/cortex/studio/heap/blocks/context`, {
      method: "POST",
      headers: {
        "x-cortex-role": "operator",
        "x-cortex-actor": "cortex-web"
      },
      body: JSON.stringify({ block_ids: artifactIds })
    }),
  getHeapBlockExport: (artifactId: string, format: "markdown" | "json" = "markdown") =>
    requestTextOrJson(`/api/cortex/studio/heap/blocks/${artifactId}/export?format=${format}`),
  getHeapBlockHistory: (artifactId: string) =>
    request<HeapBlockHistoryResponse>(`/api/cortex/studio/heap/blocks/${artifactId}/history`),
  pinHeapBlock: (artifactId: string) =>
    request<{ accepted: boolean; artifactId: string; action: string; updatedAt: string }>(`/api/cortex/studio/heap/blocks/${artifactId}/pin`, {
      method: "POST",
      headers: {
        "x-cortex-role": "operator",
        "x-cortex-actor": "cortex-web"
      }
    }),
  deleteHeapBlock: (artifactId: string) =>
    request<{ accepted: boolean; artifactId: string; action: string; updatedAt: string }>(`/api/cortex/studio/heap/blocks/${artifactId}/delete`, {
      method: "POST",
      headers: {
        "x-cortex-role": "operator",
        "x-cortex-actor": "cortex-web"
      }
    }),
  emitHeapBlock: (payload: EmitHeapBlockRequest) => {
    assertValidEmitHeapBlockRequest(payload);
    return request<{ accepted: boolean; artifactId: string }>(`/api/cortex/studio/heap/emit`, {
      method: "POST",
      headers: {
        "x-cortex-role": "operator",
        "x-cortex-actor": "cortex-web"
      },
      body: JSON.stringify(payload)
    });
  },
  validateHeapStewardGate: (artifactId: string) =>
    request<HeapStewardGateValidateResponse>(
      `/api/cortex/studio/heap/blocks/${artifactId}/steward-gate/validate`,
      {
        method: "POST",
        headers: {
          "x-cortex-role": "operator",
          "x-cortex-actor": "cortex-web"
        }
      }
    ),
  applyHeapStewardEnrichment: (artifactId: string, enrichmentId: string) =>
    request<HeapStewardGateApplyResponse>(
      `/api/cortex/studio/heap/blocks/${artifactId}/steward-gate/apply`,
      {
        method: "POST",
        headers: {
          "x-cortex-role": "operator",
          "x-cortex-actor": "cortex-web"
        },
        body: JSON.stringify({ enrichmentId })
      }
    ),
  publishArtifact: (artifactId: string, payload: ArtifactPublishRequest) =>
    request<{ artifactId: string; status: string; headRevisionId: string; publishedAt?: string }>(
      `/api/cortex/studio/artifacts/${artifactId}/publish`,
      {
        method: "POST",
        headers: {
          "x-cortex-role": "steward",
          "x-cortex-actor": "cortex-web"
        },
        body: JSON.stringify(payload)
      }
    ),
  getBrandPolicy: () => request<BrandPolicyResponse>("/api/system/brand-policy")
};

/**
 * Prepares the WebSocket bridging layer for continuous A2UI surfaceUpdates.
 * Automatically handles reconnection with exponential backoff.
 */
export function connectWorkbenchStream(
  route: string,
  spaceId: string,
  onMessage: (data: unknown) => void,
  onError?: (error: unknown) => void,
  onClose?: () => void
): { close: () => void } {
  let ws: WebSocket;
  let isClosedIntentionally = false;
  let reconnectTimer: ReturnType<typeof setTimeout>;
  let backoffMs = 1000;
  const maxBackoffMs = 30000;

  const wsUrl = `${gatewayWsBase()}/ws?route=${encodeURIComponent(route)}&space_id=${encodeURIComponent(spaceId)}`;

  function connect() {
    ws = new WebSocket(wsUrl);

    ws.onmessage = (event) => {
      // Reset backoff on successful message indicating active connection
      backoffMs = 1000;
      try {
        const data = JSON.parse(event.data);
        onMessage(data);
      } catch (e) {
        if (onError) onError(e);
      }
    };

    ws.onerror = (e) => {
      if (onError) onError(e);
    };

    ws.onclose = () => {
      if (isClosedIntentionally) {
        if (onClose) onClose();
        return;
      }

      // Auto-reconnect with exponential backoff
      reconnectTimer = setTimeout(() => {
        backoffMs = Math.min(backoffMs * 1.5, maxBackoffMs);
        connect();
      }, backoffMs);
    };
  }

  connect();

  return {
    close: () => {
      isClosedIntentionally = true;
      clearTimeout(reconnectTimer);
      if (ws) {
        ws.close();
      }
    }
  };
}
