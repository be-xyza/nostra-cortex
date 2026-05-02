import type {
  AgentContributionResponse,
  BrandPolicyResponse,
  AgentRunRecord,
  AgentRunSummary,
  AgentRunApprovalRequest,
  CompiledActionPlan,
  CompiledActionPlanRequest,
  CompiledNavigationPlan,
  DpubBlastRadiusResponse,
  DpubPipelineRunReport,
  DpubPipelineRunRequest,
  DpubStewardPacketExportRequest,
  DpubStewardPacketExportResponse,
  DpubSystemBuildResponse,
  DpubSystemReadyResponse,
  EmitHeapBlockRequest,
  EmitHeapBlockResponse,
  EmitGateSummaryHeapBlockRequest,
  EmitGateSummaryHeapBlockResponse,
  ArtifactPublishRequest,
  AuthSession,
  InternetIdentityDelegationProof,
  HeapBlocksQueryParams,
  HeapChangedBlocksResponse,
  HeapStewardGateApplyResponse,
  HeapStewardGateValidateResponse,
  HeapBlockHistoryResponse,
  HeapBlocksContextResponse,
  HeapBlocksResponse,
  ChatConversationDetail,
  ChatConversationSummaryResponse,
  HeapUploadArtifactResponse,
  HeapUploadExtractionRunDetail,
  HeapUploadExtractionRunsResponse,
  HeapUploadParserProfilesResponse,
  HeapUploadExtractionStatusResponse,
  HeapUploadExtractionTriggerResponse,
  ContributionGraph,
  PathAssessment,
  PlatformCapabilityCatalog,
  PlatformCapabilityGraph,
  SpaceCapabilityGraph,
  SpaceCapabilityGraphUpsertResponse,
  SpaceCreateRequest,
  SpaceCreateResponse,
  SpaceRoutingOverrideRecord,
  SpaceRoutingRecord,
  SpaceReadinessStatus,
  SpatialPlaneLayoutResponse,
  SpatialPlaneLayoutV1,
  SpaceSettingsResponse,
  SpacesListResponse,
  SpatialExperimentEventRequest,
  SpatialExperimentEventResponse,
  SpatialExperimentRunSummary,
  ShellLayoutSpec,
  WhoAmIResponse,
  A2UISubmitFeedbackRequest,
  A2UISubmitFeedbackResponse,
  WorkflowActiveScopeResponse,
  WorkflowCandidateRequest,
  WorkflowCandidateSet,
  WorkflowCandidateStageRequest,
  WorkflowCandidateStageResponse,
  WorkflowCandidatesResponse,
  WorkflowCheckpointResponse,
  WorkflowDefinitionResponse,
  WorkflowDigestResponse,
  WorkflowIntentRequest,
  WorkflowIntentResponse,
  WorkflowInstanceResponse,
  WorkflowOutcomeResponse,
  WorkflowProposeRequest,
  WorkflowProposalEnvelope,
  WorkflowProjectionKind,
  WorkflowProjectionResponse,
  WorkflowReplayResponse,
  WorkflowTraceResponse,
  WorkRouterDispatchQueueResponse,
  WorkRouterStatusResponse,
  SystemProvidersResponse,
  OperatorProviderInventoryResponse,
  RuntimeHostInventoryResponse,
  AuthBindingRecord,
  AuthBindingInventoryResponse,
  ExecutionBindingStatusResponse,
  ProviderDiscoveryInventoryResponse,
  ProviderValidationRequest,
  ProviderValidationResponse,
  SystemProviderRuntimeStatusResponse,
} from "./contracts.ts";
import { filterPreviewDeletedBlocks, filterPreviewHeapBlocks } from "./store/previewFixtureCatalog.ts";
import type { SpaceDesignProfilePreviewFixture } from "./store/spaceDesignProfilePreviewContract.ts";
import { readPreviewFixturesEnabledFromStorage } from "./shared/previewFixtures.ts";
import {
  getLocalDevBootstrapHeapBlocks,
  isLocalDevBootstrapEnabled,
  shouldUseLocalDevSpaceBootstrap,
  submitLocalDevBootstrapFeedback,
} from "./localDevBootstrap.ts";
import { buildFallbackAuthSession, buildFallbackWhoami } from "./components/commons/shellBootstrapFallback.ts";
import { readWindowRequestedSpaceId } from "./serviceWorker/requestScope.ts";
import {
  isGatewayApiPath as isWorkflowGatewayApiPath,
  normalizeWorkflowHref,
  parseWorkflowArtifactPath,
} from "./components/workflows/artifactRouting.ts";

const BASE =
  ((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_CORTEX_GATEWAY_URL as string | undefined) ??
  "";
export const SPACE_ID = "01ARZ3NDEKTSV4RRFFQ69G5FAV";

function isLocalDevelopmentHost(hostname: string): boolean {
  return hostname === "localhost" || hostname === "127.0.0.1";
}

export function resolveGatewayBaseUrl(): string {
  if (typeof window !== "undefined" && isLocalDevelopmentHost(window.location.hostname)) {
    return window.location.origin;
  }
  return BASE;
}

function defaultSpaceId(): string {
  const requested = readWindowRequestedSpaceId();
  if (requested) {
    return requested;
  }
  if (typeof window !== "undefined") {
    try {
      const stored = window.localStorage.getItem("cortex.shell.space.id");
      if (stored?.trim()) {
        return stored.split(",")[0]?.trim() || SPACE_ID;
      }
    } catch {
      // ignore storage failures
    }
  }

  const env = ((import.meta as unknown as { env?: Record<string, string | undefined> }).env ?? {});
  const envSpace = (env.VITE_SPACE_ID as string | undefined)?.trim();
  if (envSpace) {
    return envSpace;
  }
  return "meta";
}

export function resolveWorkbenchSpaceId(spaceId?: string): string {
  const normalized = spaceId?.trim();
  if (normalized) return normalized;
  return defaultSpaceId();
}

export function gatewayWsBase(): string {
  if (typeof window !== "undefined" && isLocalDevelopmentHost(window.location.hostname)) {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${protocol}//${window.location.host}`;
  }
  if (!BASE) {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${protocol}//${window.location.host}`;
  }
  return BASE.replace(/^http/i, "ws");
}

export function gatewayBaseUrl(): string {
  return resolveGatewayBaseUrl();
}

export function isGatewayApiPath(path: string): boolean {
  return isWorkflowGatewayApiPath(path);
}

export function resolveRequestCredentialsForBase(baseUrl: string): RequestCredentials {
  if (!baseUrl || typeof window === "undefined") {
    return "include";
  }
  try {
    const gatewayOrigin = new URL(baseUrl, window.location.origin).origin;
    return gatewayOrigin === window.location.origin ? "include" : "omit";
  } catch {
    return "include";
  }
}

export function resolveRequestCredentials(): RequestCredentials {
  return resolveRequestCredentialsForBase(resolveGatewayBaseUrl());
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${resolveGatewayBaseUrl()}${path}`, {
    ...init,
    credentials: init?.credentials ?? resolveRequestCredentials(),
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
  const response = await fetch(`${resolveGatewayBaseUrl()}${path}`, {
    ...init,
    credentials: init?.credentials ?? resolveRequestCredentials(),
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

async function requestMultipart<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${resolveGatewayBaseUrl()}${path}`, {
    ...init,
    credentials: init?.credentials ?? resolveRequestCredentials(),
    headers: {
      ...(init?.headers ?? {})
    }
  });
  if (!response.ok) {
    const body = await response.text();
    throw new Error(`${response.status} ${response.statusText}: ${body}`);
  }
  return (await response.json()) as T;
}

function shouldExposePreviewFixtures(): boolean {
  return readPreviewFixturesEnabledFromStorage();
}

function shouldUseLocalhostDevFallback(spaceId?: string): boolean {
  return shouldUseLocalDevSpaceBootstrap(spaceId);
}

function shouldPromoteLocalDevSession(session: AuthSession): boolean {
  return (
    isLocalDevBootstrapEnabled()
    && session.authMode === "read_fallback"
    && session.activeRole === "viewer"
    && !session.identityVerified
  );
}

function shouldPromoteLocalDevWhoami(whoami: WhoAmIResponse): boolean {
  return (
    isLocalDevBootstrapEnabled()
    && !whoami.identityVerified
    && whoami.effectiveRole === "viewer"
    && !whoami.authzDevMode
  );
}

function resolveEmitHeapBlockSpaceId(payload: EmitHeapBlockRequest): string {
  const candidate =
    ("space_id" in payload ? payload.space_id : undefined)
    ?? ("workspace_id" in payload ? payload.workspace_id : undefined);
  return candidate?.trim() ?? "";
}

function normalizeEmitHeapBlockRequest(payload: EmitHeapBlockRequest): EmitHeapBlockRequest {
  const spaceId = resolveEmitHeapBlockSpaceId(payload);
  const { workspace_id: _legacyWorkspaceId, ...rest } = payload;
  return {
    ...rest,
    space_id: spaceId,
  };
}

function assertValidEmitHeapBlockRequest(payload: EmitHeapBlockRequest): void {
  if (payload.schema_version !== "1.0.0" || payload.mode !== "heap") {
    throw new Error("emitHeapBlock requires schema_version=1.0.0 and mode=heap");
  }
  if (!resolveEmitHeapBlockSpaceId(payload)) {
    throw new Error("emitHeapBlock requires space_id");
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
  if (payloadType === "task" && !payload.content.task) {
    throw new Error("emitHeapBlock requires content.task when payload_type=task");
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
  getSession: (actorRole: string, actorId: string) =>
    request<AuthSession>("/api/system/session", {
      headers: {
        "x-cortex-role": actorRole,
        "x-cortex-actor": actorId,
      }
    }).then((session) => (
      shouldPromoteLocalDevSession(session)
        ? buildFallbackAuthSession(actorId, actorRole)
        : session
    )),
  setActiveRole: (role: string, spaceId?: string, actorId?: string) =>
    request<AuthSession>("/api/system/session/active-role", {
      method: "POST",
      headers: actorId
        ? {
            "x-cortex-actor": actorId,
          }
        : undefined,
      body: JSON.stringify({
        role,
        ...(spaceId ? { spaceId } : {}),
      }),
    }).then((session) => (
      shouldPromoteLocalDevSession(session)
        ? buildFallbackAuthSession(actorId, role)
        : session
    )).catch((error) => {
      if (!isLocalDevBootstrapEnabled()) {
        throw error;
      }
      return buildFallbackAuthSession(actorId, role);
    }),
  createInternetIdentitySession: (proof: InternetIdentityDelegationProof, actorId?: string) =>
    request<AuthSession>("/api/system/session/internet-identity", {
      method: "POST",
      headers: actorId
        ? {
            "x-cortex-actor": actorId,
          }
        : undefined,
      body: JSON.stringify(proof),
    }),
  getWhoami: (actorRole: string, actorId: string) =>
    request<WhoAmIResponse>("/api/system/whoami", {
      headers: {
        "x-cortex-role": actorRole,
        "x-cortex-actor": actorId
      }
    }).then((whoami) => (
      shouldPromoteLocalDevWhoami(whoami)
        ? buildFallbackWhoami(actorId, actorRole)
        : whoami
    )).catch((error) => {
      if (!isLocalDevBootstrapEnabled()) {
        throw error;
      }
      return buildFallbackWhoami(actorId, actorRole);
    }),
  getReady: () => request<DpubSystemReadyResponse>("/api/system/ready"),
  getBuild: () => request<DpubSystemBuildResponse>("/api/system/build"),
  getOverview: (spaceId?: string) => request<Record<string, unknown>>(`/api/kg/spaces/${encodeURIComponent(resolveWorkbenchSpaceId(spaceId))}/contribution-graph/overview`),
  getGraph: (spaceId: string) => request<ContributionGraph>(`/api/kg/spaces/${encodeURIComponent(spaceId)}/contribution-graph/graph?mode=d3-force`),
  getCapabilityGraph: () => request<PlatformCapabilityGraph>(`/api/system/capability-graph`),
  getCapabilityCatalog: () => request<PlatformCapabilityCatalog>(`/api/system/capability-catalog`),
  getSpaceDesignProfilePreview: () =>
    request<SpaceDesignProfilePreviewFixture>("/api/system/ux/space-design-profiles"),
  getSpaces: () => request<SpacesListResponse>(`/api/spaces`),
  getSpaceReadiness: (spaceId: string) =>
    request<{
      schemaVersion: string;
      generatedAt: string;
      spaceId: string;
      sourceMode: "registered" | "observed";
      readinessSummary: SpaceReadinessStatus;
      readiness: {
        registry: SpaceReadinessStatus;
        navigationPlan: SpaceReadinessStatus;
        agentRuns: SpaceReadinessStatus;
        contributionGraphArtifact: SpaceReadinessStatus;
        contributionGraphRuns: SpaceReadinessStatus;
        capabilityGraph: SpaceReadinessStatus;
        summary: SpaceReadinessStatus;
      };
    }>(`/api/spaces/${encodeURIComponent(spaceId)}/readiness`),
  getSpaceSettings: (spaceId: string) =>
    request<SpaceSettingsResponse>(`/api/spaces/${encodeURIComponent(spaceId)}/settings`),
  putSpaceRouting: (spaceId: string, payload: SpaceRoutingRecord) =>
    request<SpaceRoutingRecord>(`/api/spaces/${encodeURIComponent(spaceId)}/routing`, {
      method: "PUT",
      body: JSON.stringify(payload),
    }),
  putSpaceAgentRouting: (
    spaceId: string,
    agentId: string,
    payload: SpaceRoutingOverrideRecord,
  ) =>
    request<SpaceRoutingRecord>(
      `/api/spaces/${encodeURIComponent(spaceId)}/agents/${encodeURIComponent(agentId)}/routing`,
      {
        method: "PUT",
        body: JSON.stringify(payload),
      },
    ),
  getSpaceCapabilityGraph: (spaceId: string) =>
    request<SpaceCapabilityGraph>(`/api/spaces/${encodeURIComponent(spaceId)}/capability-graph`),
  putSpaceCapabilityGraph: (
    spaceId: string,
    payload: SpaceCapabilityGraph,
    actorRole = "steward",
    actorId = "cortex-web"
  ) =>
    request<SpaceCapabilityGraphUpsertResponse>(`/api/spaces/${encodeURIComponent(spaceId)}/capability-graph`, {
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
  getSpaceActionPlan: (
    spaceId: string,
    payload: CompiledActionPlanRequest,
    actorRole = "operator",
    actorId = "cortex-web"
  ) => {
    return request<CompiledActionPlan>(
      `/api/spaces/${encodeURIComponent(spaceId)}/action-plan`,
      {
        method: "POST",
        headers: {
          "x-cortex-role": actorRole,
          "x-cortex-actor": actorId
        },
        body: JSON.stringify(payload)
      }
    );
  },
  getPath: (spaceId?: string) => request<PathAssessment>(`/api/kg/spaces/${encodeURIComponent(resolveWorkbenchSpaceId(spaceId))}/contribution-graph/path-assessment`),
  getRuns: (spaceId?: string) => request<DpubPipelineRunReport[]>(`/api/kg/spaces/${encodeURIComponent(resolveWorkbenchSpaceId(spaceId))}/contribution-graph/runs?limit=10`),
  getSystemAgentRuns: (spaceId?: string) =>
    request<AgentRunSummary[]>(`/api/system/agents/runs?space_id=${encodeURIComponent(resolveWorkbenchSpaceId(spaceId))}`),
  getSystemAgentRun: (spaceId: string, runId: string) =>
    request<AgentRunRecord>(`/api/system/agents/runs/${encodeURIComponent(spaceId)}/${encodeURIComponent(runId)}`),
  getSystemWorkRouterStatus: () =>
    request<WorkRouterStatusResponse>("/api/system/work-router/status"),
  getSystemWorkRouterDispatches: () =>
    request<WorkRouterDispatchQueueResponse>("/api/system/work-router/dispatches"),
  getContributionBlastRadius: (contributionId: string, spaceId?: string) =>
    request<DpubBlastRadiusResponse>(
      `/api/kg/spaces/${encodeURIComponent(resolveWorkbenchSpaceId(spaceId))}/contribution-graph/blast-radius?contributionId=${encodeURIComponent(contributionId)}`
    ),
  createSpace: (
    payload: SpaceCreateRequest,
    actorRole = "steward",
    actorId = "cortex-web"
  ) =>
    request<SpaceCreateResponse>(`/api/spaces/create`, {
      method: "POST",
      headers: {
        "x-cortex-role": actorRole,
        "x-cortex-actor": actorId
      },
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
  getSpatialPlaneLayout: (spaceId: string, viewSpecId: string) =>
    request<SpatialPlaneLayoutResponse>(
      `/api/cortex/viewspecs/spatial/layouts/${encodeURIComponent(spaceId)}/${encodeURIComponent(viewSpecId)}`
    ),
  saveSpatialPlaneLayout: (spaceId: string, viewSpecId: string, payload: SpatialPlaneLayoutV1) =>
    request<SpatialPlaneLayoutResponse>(
      `/api/cortex/viewspecs/spatial/layouts/${encodeURIComponent(spaceId)}/${encodeURIComponent(viewSpecId)}`,
      {
        method: "POST",
        body: JSON.stringify(payload)
      }
    ),
  getSpatialExperimentRun: (runId: string) =>
    request<SpatialExperimentRunSummary>(`/api/cortex/viewspecs/experiments/spatial/runs/${runId}`),
  exportStewardPacket: (
    payload: DpubStewardPacketExportRequest,
    actorRole: string,
    actorId: string,
    spaceId?: string
  ) =>
    request<DpubStewardPacketExportResponse>(
      `/api/kg/spaces/${encodeURIComponent(resolveWorkbenchSpaceId(spaceId))}/contribution-graph/steward-packet/export`,
      {
        method: "POST",
        headers: {
          "x-cortex-role": actorRole,
          "x-cortex-actor": actorId
        },
        body: JSON.stringify(payload)
      }
    ),
  runPipeline: (payload: DpubPipelineRunRequest, actorRole: string, actorId: string, spaceId?: string) =>
    request<DpubPipelineRunReport>(`/api/kg/spaces/${encodeURIComponent(resolveWorkbenchSpaceId(spaceId))}/contribution-graph/pipeline/run`, {
      method: "POST",
      headers: {
        "x-cortex-role": actorRole,
        "x-cortex-actor": actorId
      },
      body: JSON.stringify(payload)
    }),
  getHeapBlocks: (params?: HeapBlocksQueryParams) => {
    const requestedSpaceId = params?.spaceId ?? resolveWorkbenchSpaceId();
    const query = new URLSearchParams();
    if (params?.spaceId) query.set("space_id", params.spaceId);
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
    return request<HeapBlocksResponse>(`/api/cortex/studio/heap/blocks${suffix ? `?${suffix}` : ""}`)
      .then(async (response) => {
        const sanitizedResponse = shouldExposePreviewFixtures()
          ? response
          : {
              ...response,
              count: filterPreviewHeapBlocks(response.items).length,
              items: filterPreviewHeapBlocks(response.items),
            };

        if (
          shouldUseLocalhostDevFallback(requestedSpaceId)
          && sanitizedResponse.items.length === 0
        ) {
          const items = await getLocalDevBootstrapHeapBlocks(requestedSpaceId);
          return {
            ...sanitizedResponse,
            count: items.length,
            items,
          };
        }

        return sanitizedResponse;
      })
      .catch(async (error) => {
        if (!shouldUseLocalhostDevFallback(requestedSpaceId)) {
          throw error;
        }
        const items = await getLocalDevBootstrapHeapBlocks(requestedSpaceId);
        return {
          schemaVersion: "1.0.0",
          generatedAt: new Date().toISOString(),
          count: items.length,
          hasMore: false,
          items,
        };
      });
  },
  getHeapChangedBlocks: (params?: HeapBlocksQueryParams) => {
    const query = new URLSearchParams();
    if (params?.spaceId) query.set("space_id", params.spaceId);
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
    return request<HeapChangedBlocksResponse>(`/api/cortex/studio/heap/changed_blocks${suffix ? `?${suffix}` : ""}`)
      .then((response) => shouldExposePreviewFixtures()
        ? response
        : {
            ...response,
            count: filterPreviewHeapBlocks(response.changed).length,
            changed: filterPreviewHeapBlocks(response.changed),
            deleted: filterPreviewDeletedBlocks(response.deleted),
          });
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
  listChatConversations: () =>
    request<ChatConversationSummaryResponse>(`/api/cortex/chat/conversations`),
  getChatConversation: (threadId: string) =>
    request<ChatConversationDetail>(`/api/cortex/chat/conversations/${encodeURIComponent(threadId)}`),
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
  uploadHeapFile: (
    payload: { file: File; spaceId: string; title?: string; sourceAgentId?: string },
    actorRole = "operator",
    actorId = "cortex-web"
  ) => {
    const formData = new FormData();
    formData.append("file", payload.file);
    formData.append("space_id", payload.spaceId);
    if (payload.title?.trim()) {
      formData.append("title", payload.title.trim());
    }
    formData.append("source_agent_id", payload.sourceAgentId?.trim() || actorId);
    return requestMultipart<HeapUploadArtifactResponse>(`/api/cortex/studio/uploads`, {
      method: "POST",
      headers: {
        "x-cortex-role": actorRole,
        "x-cortex-actor": actorId
      },
      body: formData
    });
  },
  triggerHeapUploadExtraction: (
    uploadId: string,
    parserProfile?: string,
    actorRole = "operator",
    actorId = "cortex-web"
  ) => request<HeapUploadExtractionTriggerResponse>(`/api/cortex/studio/uploads/${encodeURIComponent(uploadId)}/extract`, {
    method: "POST",
    headers: {
      "x-cortex-role": actorRole,
      "x-cortex-actor": actorId
    },
    body: JSON.stringify(parserProfile ? { parser_profile: parserProfile } : {})
  }),
  getHeapUploadParserProfiles: (
    actorRole = "operator",
    actorId = "cortex-web"
  ) => request<HeapUploadParserProfilesResponse>(`/api/cortex/studio/uploads/parser-profiles`, {
    headers: {
      "x-cortex-role": actorRole,
      "x-cortex-actor": actorId
    }
  }),
  getHeapUploadExtractionStatus: (
    uploadId: string,
    actorRole = "operator",
    actorId = "cortex-web"
  ) => request<HeapUploadExtractionStatusResponse>(`/api/cortex/studio/uploads/${encodeURIComponent(uploadId)}/extraction`, {
    headers: {
      "x-cortex-role": actorRole,
      "x-cortex-actor": actorId
    }
  }),
  getHeapUploadExtractionRuns: (
    uploadId: string,
    actorRole = "operator",
    actorId = "cortex-web"
  ) => request<HeapUploadExtractionRunsResponse>(`/api/cortex/studio/uploads/${encodeURIComponent(uploadId)}/extractions`, {
    headers: {
      "x-cortex-role": actorRole,
      "x-cortex-actor": actorId
    }
  }),
  getHeapUploadExtractionRun: (
    uploadId: string,
    jobId: string,
    actorRole = "operator",
    actorId = "cortex-web"
  ) => request<HeapUploadExtractionRunDetail>(`/api/cortex/studio/uploads/${encodeURIComponent(uploadId)}/extractions/${encodeURIComponent(jobId)}`, {
    headers: {
      "x-cortex-role": actorRole,
      "x-cortex-actor": actorId
    }
  }),
  emitHeapBlock: (
    payload: EmitHeapBlockRequest,
    actorRole = "operator",
    actorId = "cortex-web"
  ) => {
    assertValidEmitHeapBlockRequest(payload);
    const normalizedPayload = normalizeEmitHeapBlockRequest(payload);
    return request<EmitHeapBlockResponse>(`/api/cortex/studio/heap/emit`, {
      method: "POST",
      headers: {
        "x-cortex-role": actorRole,
        "x-cortex-actor": actorId
      },
      body: JSON.stringify(normalizedPayload)
    });
  },
  emitGateSummaryHeapBlock: (payload: EmitGateSummaryHeapBlockRequest) =>
    request<EmitGateSummaryHeapBlockResponse>(`/api/system/gates/emit-heap-block`, {
      method: "POST",
      headers: {
        "x-cortex-role": "operator",
        "x-cortex-actor": "cortex-web"
      },
      body: JSON.stringify(payload)
    }),
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
  submitA2UIFeedback: (
    artifactId: string,
    payload: A2UISubmitFeedbackRequest["feedbackData"],
    actorRole = "operator",
    actorId = "cortex-web",
  ) =>
    request<A2UISubmitFeedbackResponse>(`/api/cortex/studio/heap/blocks/${artifactId}/a2ui/feedback`, {
      method: "POST",
      headers: {
        "x-cortex-role": actorRole,
        "x-cortex-actor": actorId
      },
      body: JSON.stringify(payload)
    }).catch((error) => {
      if (!isLocalDevBootstrapEnabled()) {
        throw error;
      }
      return submitLocalDevBootstrapFeedback(artifactId, payload, actorRole, actorId);
    }),
  getWorkflowDraftProposalReplay: (proposalId: string) =>
    request<WorkflowReplayResponse>(
      `/api/cortex/workflow-drafts/proposals/${encodeURIComponent(proposalId)}/replay`
    ),
  getWorkflowDraftProposalDigest: (proposalId: string) =>
    request<WorkflowDigestResponse>(
      `/api/cortex/workflow-drafts/proposals/${encodeURIComponent(proposalId)}/digest`
    ),
  getWorkflowDefinition: (definitionId: string) =>
    request<WorkflowDefinitionResponse>(
      `/api/cortex/workflow-definitions/${encodeURIComponent(definitionId)}`
    ),
  getWorkflowDefinitionProjection: (
    definitionId: string,
    projectionKind: WorkflowProjectionKind
  ) =>
    request<WorkflowProjectionResponse>(
      `/api/cortex/workflow-definitions/${encodeURIComponent(
        definitionId
      )}/projections/${encodeURIComponent(projectionKind)}`
    ),
  getWorkflowActiveDefinition: (scopeKey: string) =>
    request<WorkflowActiveScopeResponse>(
      `/api/cortex/workflow-definitions/active/${encodeURIComponent(scopeKey)}`
    ),
  postWorkflowIntent: (payload: WorkflowIntentRequest) =>
    request<WorkflowIntentResponse>("/api/cortex/workflow-intents", {
      method: "POST",
      body: JSON.stringify(payload),
    }),
  postWorkflowCandidates: (payload: WorkflowCandidateRequest) =>
    request<WorkflowCandidatesResponse>("/api/cortex/workflow-drafts/candidates", {
      method: "POST",
      body: JSON.stringify(payload),
    }),
  getWorkflowCandidateSet: (candidateSetId: string) =>
    request<{
      schemaVersion: string;
      generatedAt: string;
      candidateSet: WorkflowCandidateSet;
    }>(`/api/cortex/workflow-drafts/candidates/${encodeURIComponent(candidateSetId)}`),
  stageWorkflowCandidate: (
    candidateSetId: string,
    payload: WorkflowCandidateStageRequest,
  ) =>
    request<WorkflowCandidateStageResponse>(
      `/api/cortex/workflow-drafts/candidates/${encodeURIComponent(candidateSetId)}/stage`,
      {
        method: "POST",
        body: JSON.stringify(payload),
      },
    ),
  proposeWorkflowDraft: (
    workflowDraftId: string,
    payload: WorkflowProposeRequest,
  ) =>
    request<{
      accepted: boolean;
      proposal: WorkflowProposalEnvelope;
    }>(`/api/cortex/workflow-drafts/${encodeURIComponent(workflowDraftId)}/propose`, {
      method: "POST",
      body: JSON.stringify(payload),
    }),
  getWorkflowInstance: (instanceId: string) =>
    request<WorkflowInstanceResponse>(
      `/api/cortex/workflow-instances/${encodeURIComponent(instanceId)}`
    ),
  getWorkflowInstanceTrace: (instanceId: string) =>
    request<WorkflowTraceResponse>(
      `/api/cortex/workflow-instances/${encodeURIComponent(instanceId)}/trace`
    ),
  getWorkflowInstanceCheckpoints: (instanceId: string) =>
    request<WorkflowCheckpointResponse>(
      `/api/cortex/workflow-instances/${encodeURIComponent(instanceId)}/checkpoints`
    ),
  getWorkflowInstanceOutcome: (instanceId: string) =>
    request<WorkflowOutcomeResponse>(
      `/api/cortex/workflow-instances/${encodeURIComponent(instanceId)}/outcome`
    ),
  getBrandPolicy: () => request<BrandPolicyResponse>("/api/system/brand-policy"),
  getSystemProviders: () => request<SystemProvidersResponse>("/api/system/providers"),
  getSystemProviderInventory: () =>
    request<OperatorProviderInventoryResponse>("/api/system/provider-inventory"),
  getSystemRuntimeHosts: () =>
    request<RuntimeHostInventoryResponse>("/api/system/runtime-hosts"),
  getSystemAuthBindings: () =>
    request<AuthBindingInventoryResponse>("/api/system/auth-bindings"),
  getSystemExecutionBindings: () =>
    request<ExecutionBindingStatusResponse>("/api/system/execution-bindings"),
  getSystemProviderDiscovery: () =>
    request<ProviderDiscoveryInventoryResponse>("/api/system/provider-discovery"),
  getSystemProviderRuntimeStatus: () => request<SystemProviderRuntimeStatusResponse>("/api/system/provider-runtime/status"),
  discoverSystemProviders: () =>
    request<SystemProvidersResponse>("/api/system/providers/discover", {
      method: "POST",
    }),
  validateSystemProvider: (payload: ProviderValidationRequest) =>
    request<ProviderValidationResponse>("/api/system/providers/validate", {
      method: "POST",
      body: JSON.stringify(payload),
    }),
  putSystemProviderBinding: (
    bindingId: string,
    payload: {
      providerType?: string;
      boundProviderId: string;
      metadata?: Record<string, string>;
    },
  ) =>
    request<SystemProvidersResponse>(`/api/system/provider-bindings/${encodeURIComponent(bindingId)}`, {
      method: "PUT",
      body: JSON.stringify(payload),
    }),
  putSystemProvider: (providerId: string, payload: Record<string, unknown>) =>
    request<SystemProvidersResponse>(`/api/system/providers/${encodeURIComponent(providerId)}`, {
      method: "PUT",
      body: JSON.stringify(payload),
    }),
  createSystemAuthBinding: (payload: {
    targetId?: string;
    targetKind?: "provider" | "host";
    authType?: "none" | "api_key" | "bearer_token" | "pat" | "ssh_key" | "ssh_password";
    label: string;
    apiKey: string;
    metadata?: Record<string, string>;
  }) =>
    request<AuthBindingRecord>("/api/system/auth-bindings", {
      method: "POST",
      body: JSON.stringify(payload),
    }),
  updateSystemAuthBinding: (
    authBindingId: string,
    payload: {
      label?: string;
      apiKey?: string;
      source?: string;
      targetId?: string;
      targetKind?: "provider" | "host";
      authType?: "none" | "api_key" | "bearer_token" | "pat" | "ssh_key" | "ssh_password";
      metadata?: Record<string, string>;
    },
  ) =>
    request<AuthBindingRecord>(
      `/api/system/auth-bindings/${encodeURIComponent(authBindingId)}`,
      {
        method: "PUT",
        body: JSON.stringify(payload),
      },
    ),
};

export async function openGatewayApiArtifact(
  path: string,
  mode: "inline" | "new_tab"
): Promise<unknown> {
  const normalized = normalizeWorkflowHref(path);
  if (mode === "new_tab") {
    if (typeof window !== "undefined") {
      window.open(`${gatewayBaseUrl()}${normalized}`, "_blank", "noopener,noreferrer");
    }
    return null;
  }

  const descriptor = parseWorkflowArtifactPath(normalized);
  if (!descriptor) {
    return requestTextOrJson(normalized);
  }

  switch (descriptor.kind) {
    case "proposal_replay":
      return workbenchApi.getWorkflowDraftProposalReplay(descriptor.proposalId);
    case "proposal_digest":
      return workbenchApi.getWorkflowDraftProposalDigest(descriptor.proposalId);
    case "definition":
      return workbenchApi.getWorkflowDefinition(descriptor.definitionId);
    case "definition_projection":
      return workbenchApi.getWorkflowDefinitionProjection(
        descriptor.definitionId,
        descriptor.projectionKind
      );
    case "active_definition":
      return workbenchApi.getWorkflowActiveDefinition(descriptor.scopeKey);
    case "instance":
      return workbenchApi.getWorkflowInstance(descriptor.instanceId);
    case "instance_trace":
      return workbenchApi.getWorkflowInstanceTrace(descriptor.instanceId);
    case "instance_checkpoints":
      return workbenchApi.getWorkflowInstanceCheckpoints(descriptor.instanceId);
    case "instance_outcome":
      return workbenchApi.getWorkflowInstanceOutcome(descriptor.instanceId);
  }
}

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
