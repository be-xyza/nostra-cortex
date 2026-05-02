export type Json = Record<string, unknown>;

export interface DpubApprovalEnvelope {
  approvedBy: string;
  rationale: string;
  approvedAt: string;
  decisionRef: string;
}

export interface DpubPipelineRunRequest {
  mode: string;
  goal?: string;
  scenarioTemplateId?: string;
  publishVersion?: string;
  fromVersion?: string;
  toVersion?: string;
  approval?: DpubApprovalEnvelope;
}

export interface DpubPipelineRunReport {
  runId: string;
  mode: string;
  status: string;
  startedAt: string;
  finishedAt?: string;
  graphRootHashAfter?: string;
  error?: string;
}

export interface DpubSystemReadyResponse {
  ready: boolean;
  icpNetworkHealthy: boolean;
  dfxPortHealthy: boolean;
  gatewayPort: number;
  notes: string[];
}

export interface DpubBlastRadiusResponse {
  contributionId: string;
  dependsOn: string[];
  dependedBy: string[];
  invalidates: string[];
  invalidatedBy: string[];
  supersedes: string[];
  supersededBy: string[];
  references: string[];
  referencedBy: string[];
}

export interface DpubStewardPacketExportRequest {
  goal?: string;
  fromVersion?: string;
  toVersion?: string;
  approval?: DpubApprovalEnvelope;
}

export interface DpubStewardPacketExportResponse {
  packetPath: string;
  goal: string;
  fromVersion: string;
  toVersion: string;
}

export interface DpubSystemBuildResponse {
  buildId: string;
  buildTimeUtc: string;
  gatewayDispatchMode: string;
  spaceRoot: string;
  spaceId: string;
}

export type SpaceSourceMode = "registered" | "observed";
export type SpaceReadinessStatus = "pass" | "fail" | "in_progress";

export interface SpaceRoleGrant {
  spaceId: string;
  roles: string[];
  claims: string[];
}

export interface AuthSession {
  schemaVersion: string;
  generatedAt: string;
  principal?: string;
  sessionId: string;
  identityVerified: boolean;
  identitySource: string;
  authMode: "dev_override" | "principal_binding" | "session_claims" | "read_fallback";
  grantedRoles: string[];
  activeRole: string;
  globalClaims: string[];
  spaceGrants: SpaceRoleGrant[];
  allowRoleSwitch: boolean;
  allowUnverifiedRoleHeader: boolean;
  expiresAt?: string;
}

export interface InternetIdentityDelegationProof {
  principal: string;
  identityProvider: string;
  delegationChain: {
    publicKey: string;
    delegations: Array<{
      delegation: {
        pubkey: string;
        expiration: string;
        targets?: string[];
      };
      signature: string;
    }>;
  };
  signedAt: string;
}

export interface WhoAmIResponse {
  schemaVersion: string;
  generatedAt: string;
  principal?: string;
  requestedRole: string;
  effectiveRole: string;
  claims: string[];
  identityVerified: boolean;
  identitySource: string;
  authzDevMode: boolean;
  allowUnverifiedRoleHeader: boolean;
  authzDecisionVersion: string;
}

export type ContributionKind = "initiative" | "pr" | "bounty" | "decision" | "task";

export interface ContributionNode {
  id: string;
  resource_ref?: string;
  title: string;
  kind?: ContributionKind;
  status: string;
  layer: string;
  portfolio_role?: string;
  space_id?: string;
}

export interface ContributionEdge {
  from: string;
  to: string;
  edge_kind?: string;
  confidence?: number;
  is_explicit?: boolean;
}

export interface ContributionGraph {
  graph_root_hash: string;
  nodes: ContributionNode[];
  edges: ContributionEdge[];
}

export interface GraphPhysicsConfig {
  repulsionStrength: number;
  linkDistance: number;
  centerGravity: number;
}

export interface CapabilityNode {
  id: string;
  title: string;
  description: string;
  intent_type: string;
  route_id?: string;
  required_role?: string;
  pattern_id?: string;
  promotion_status?: string;
  invariant_violations?: Array<{
    policy_id?: string;
    severity?: string;
    message?: string;
    context?: Record<string, unknown>;
  }>;
  cluster_key?: string;
  domain?: string;
  locked_reason?: string;
  visibility_state?: string;
  health?: string;
  priority?: string;
  variance?: string;
  surfacing_heuristic?: SurfacingHeuristic;
  operational_frequency?: OperationalFrequency;
  inspector?: {
    route_id?: string;
    category?: string;
    pattern_label?: string;
    required_role?: string;
    required_role_rank?: number;
    operator_critical?: boolean;
    approval_required?: boolean;
    promotion_status?: string;
    surfacing_heuristic?: SurfacingHeuristic;
    operational_frequency?: OperationalFrequency;
    placement_constraint?: PlacementConstraint;
  };
}

export interface CapabilityEdge {
  from: string;
  to: string;
  relationship: string;
  relationship_label?: string;
  confidence?: number;
  policy_ref?: string;
  rationale?: string;
  directionality?: string;
}

export interface CapabilityLayoutGroup {
  key: string;
  label: string;
  order: number;
  color: string;
}

export interface CapabilityLayoutHints {
  engine: string;
  seed: string;
  cluster_by: string;
  groups: CapabilityLayoutGroup[];
}

export interface CapabilityLegend {
  intent_type_colors: Record<string, string>;
  relationship_styles: Record<string, string>;
  lock_semantics: string;
}

export interface PlatformCapabilityGraph {
  schema_version: string;
  generated_at: string;
  source_of_truth: string;
  graph_hash?: string;
  layout_hints?: CapabilityLayoutHints;
  legend?: CapabilityLegend;
  capabilities_version?: string;
  nodes: CapabilityNode[];
  edges: CapabilityEdge[];
}

export type SurfacingHeuristic =
  | "PrimaryCore"
  | "Secondary"
  | "ContextualDeep"
  | "Hidden";
export type OperationalFrequency = "Continuous" | "Daily" | "AdHoc" | "Rare";

export interface DomainEntityRef {
  entityType: string;
  entityId?: string;
  label?: string;
}

export interface PlacementConstraint {
  preferredNavBand?: string;
  preferredCategory?: string;
  allowContextualOnly?: boolean;
  maxNavDepth?: number;
}

export interface PlatformCapabilityCatalogNode {
  id: { 0: string } | string;
  resourceRef?: string;
  name: string;
  description: string;
  intentType: string;
  routeId?: string;
  category?: string;
  requiredRole?: string;
  icon?: string;
  surfacingHeuristic?: SurfacingHeuristic;
  operationalFrequency?: OperationalFrequency;
  domainEntities?: DomainEntityRef[];
  placementConstraint?: PlacementConstraint;
  rootPath?: string;
}

export interface PlatformCapabilityCatalogEdge {
  source: { 0: string } | string;
  target: { 0: string } | string;
  relationship: string;
}

export interface PlatformCapabilityCatalog {
  schemaVersion: string;
  catalogVersion: string;
  catalogHash?: string;
  nodes: PlatformCapabilityCatalogNode[];
  edges: PlatformCapabilityCatalogEdge[];
}

export interface SpaceCapabilityNodeOverride {
  capabilityId: { 0: string } | string;
  localAlias?: string;
  isActive: boolean;
  localRequiredRole?: string;
  surfacingHeuristic?: SurfacingHeuristic;
  operationalFrequency?: OperationalFrequency;
  placementConstraint?: PlacementConstraint;
}

export interface SpaceCapabilityGraph {
  schemaVersion: string;
  spaceId: string;
  baseCatalogVersion: string;
  baseCatalogHash: string;
  nodes: SpaceCapabilityNodeOverride[];
  edges: PlatformCapabilityCatalogEdge[];
  updatedAt: string;
  updatedBy: string;
  lineageRef?: string;
}

export interface SpaceCapabilityGraphUpsertResponse {
  accepted: boolean;
  spaceId: string;
  capabilityGraphHash: string;
  capabilityGraphVersion: string;
  storedAt: string;
}

export interface SpaceRegistryRecord {
  spaceId: string;
  creationMode: string;
  referenceUri?: string | null;
  templateId?: string | null;
  draftId?: string | null;
  draftSourceMode?: string | null;
  lineageNote?: string | null;
  governanceScope?: "personal" | "private" | "public" | null;
  visibilityState?: "owner_only" | "members_only" | "discoverable" | null;
  capabilityGraphUri?: string | null;
  capabilityGraphVersion?: string | null;
  capabilityGraphHash?: string | null;
  status: string;
  createdAt: string;
  owner: string;
  members: string[];
  archetype?: string | null;
  sourceMode?: SpaceSourceMode;
  readinessSummary?: SpaceReadinessStatus;
  readiness?: {
    registry: SpaceReadinessStatus;
    navigationPlan: SpaceReadinessStatus;
    agentRuns: SpaceReadinessStatus;
    contributionGraphArtifact: SpaceReadinessStatus;
    contributionGraphRuns: SpaceReadinessStatus;
    capabilityGraph: SpaceReadinessStatus;
    summary: SpaceReadinessStatus;
  };
}

export interface SpaceRoutingOverrideRecord {
  agentId?: string | null;
  adapterSetRef?: string | null;
  providerId?: string | null;
  defaultModel?: string | null;
  authBindingId?: string | null;
  authMode?: string | null;
}

export interface SpaceRoutingRecord {
  adapterSetRef?: string | null;
  providerId?: string | null;
  defaultModel?: string | null;
  authBindingId?: string | null;
  agentRoutingPolicy?: string | null;
  agentOverrides: Record<string, SpaceRoutingOverrideRecord>;
  updatedAt?: string | null;
  updatedBy?: string | null;
}

export interface DecisionSurfaceEnvelope<TPayload = Json> {
  decisionId: string;
  workflowId: string;
  status: string;
  sourceOfTruth?: string | null;
  degradedReason?: string | null;
  payload?: TPayload;
}

export interface SpaceExecutionProfilePayload {
  spaceId: string;
  executionProfile: {
    executionTopology?: string;
    consensusMode?: string;
    trustBoundary?: string;
    updatedBy?: string;
    updatedAt?: number | string;
  };
}

export interface SpaceAttributionDomainsPayload {
  spaceId: string;
  domains: Array<{
    id: string;
    attributionMode?: string;
    reattachmentPolicy?: string;
    governanceVisibility?: string;
    auditabilityLevel?: string;
    weightPolicyRef?: string;
    updatedBy?: string;
    updatedAt?: number | string;
  }>;
}

export interface SpaceGovernanceScopePayload {
  spaceId: string;
  scope: {
    allowed?: boolean;
    reason?: string;
    effectiveWeight?: number;
    requiresReview?: boolean;
    gateDecision?: string;
    requiredActions?: string[];
    policyRef?: string;
    policyVersion?: string | number;
  };
}

export interface SpaceLineageRecord {
  promptArtifacts: HeapBlockListItem[];
  feedbackArtifacts: HeapBlockListItem[];
  legacyPromptGroups: Array<{
    title: string;
    contentHash?: string | null;
    artifactIds: string[];
  }>;
}

export interface SpaceSettingsResponse {
  schemaVersion: string;
  generatedAt: string;
  spaceId: string;
  routing: SpaceRoutingRecord;
  executionProfile?: DecisionSurfaceEnvelope<SpaceExecutionProfilePayload>;
  attributionDomains?: DecisionSurfaceEnvelope<SpaceAttributionDomainsPayload>;
  governanceScope?: DecisionSurfaceEnvelope<SpaceGovernanceScopePayload>;
  providers: ProviderRecord[];
  agentRuns: AgentRunSummary[];
  lineage: SpaceLineageRecord;
}

export interface SpacesListResponse {
  schemaVersion: string;
  generatedAt: string;
  count: number;
  items: SpaceRegistryRecord[];
}

export interface CompilationContext {
  spaceId: string;
  actorRole: string;
  intent?: string;
  density?: string;
}

export interface CompiledNavigationEntry {
  capabilityId: string;
  routeId: string;
  label: string;
  icon: string;
  category: string;
  requiredRole: string;
  navSlot: string;
  navBand: string;
  surfacingHeuristic: string;
  operationalFrequency: string;
  rank: number;
}

export interface CompiledSurfacingPlan {
  primaryCore: string[];
  secondary: Record<string, string[]>;
  contextualDeep: string[];
  hidden: string[];
}

export interface CompiledNavigationPlan {
  schemaVersion: string;
  generatedAt: string;
  spaceId: string;
  actorRole: string;
  intent?: string;
  density?: string;
  planHash: string;
  entries: CompiledNavigationEntry[];
  surfacing: CompiledSurfacingPlan;
}

export type SurfaceZone =
  | "heap_page_bar"
  | "heap_selection_bar"
  | "heap_detail_footer"
  | "heap_detail_header"
  | "heap_card_menu";

export type PageType =
  | "heap_board"
  | "heap_detail";

export interface ActionSelectionContext {
  selectedArtifactIds: string[];
  activeArtifactId?: string;
  selectedCount: number;
  selectedBlockTypes?: string[];
}

export interface CompiledActionPlanRequest {
  schemaVersion: "1.0.0" | string;
  spaceId: string;
  actorRole: string;
  routeId: string;
  pageType: PageType;
  intent?: string;
  density?: string;
  zones: SurfaceZone[];
  selection: ActionSelectionContext;
  activeFilters?: {
    viewMode?: string;
    filterMode?: string;
    selectedTags?: string[];
    selectedPageLinks?: string[];
  };
  featureFlags?: {
    heapCreateFlowEnabled?: boolean;
    heapParityEnabled?: boolean;
  };
}

export type ToolbarActionKind =
  | "command"
  | "mutation"
  | "navigation"
  | "panel_toggle"
  | "download"
  | "destructive";

export interface ToolbarActionDescriptor {
  id: string;
  capabilityId: string;
  zone: SurfaceZone;
  label: string;
  shortLabel?: string;
  icon?: string;
  kind: ToolbarActionKind;
  action: string;
  priority: number;
  group: "primary" | "secondary" | "danger" | "selection";
  emphasis?: "default" | "primary" | "accent" | "danger";
  visible: boolean;
  enabled: boolean;
  disabledReason?: string;
  selectionConstraints?: {
    minSelected?: number;
    maxSelected?: number;
    requireSingleSelection?: boolean;
  };
  confirmation?: {
    required: boolean;
    style?: "danger" | "default";
    title?: string;
    message?: string;
  };
  stewardGate?: {
    required: boolean;
  };
}

export interface ActionZonePlan {
  zone: SurfaceZone;
  layoutHint: "row" | "pillbar";
  actions: ToolbarActionDescriptor[];
}

export interface CompiledActionPlan {
  schemaVersion: "1.0.0" | string;
  generatedAt: string;
  planHash: string;
  spaceId: string;
  routeId: string;
  pageType: PageType;
  actorRole: string;
  zones: ActionZonePlan[];
}

export type WorkflowProjectionKind =
  | "flow_graph_v1"
  | "a2ui_surface_v1"
  | "serverless_workflow_v0_8"
  | "normalized_graph_v1"
  | "execution_topology_v1";

export type WorkflowMotifKind =
  | "sequential"
  | "parallel_compare"
  | "repair_loop"
  | "fan_out_join"
  | "human_gate";

export type WorkflowGenerationMode = "deterministic_scaffold" | "motif_hybrid";

export interface WorkflowScope {
  spaceId?: string;
  routeId?: string;
  role?: string;
}

export interface WorkflowConstraintRule {
  constraintId: string;
  label: string;
  expression: string;
  hard?: boolean;
  locked?: boolean;
}

export type WorkflowNodeKind =
  | "capability_call"
  | "human_checkpoint"
  | "evaluation_gate"
  | "parallel"
  | "switch"
  | "loop"
  | "subflow_ref"
  | "terminal";

export interface WorkflowCheckpointPolicy {
  resumeAllowed: boolean;
  cancelAllowed: boolean;
  pauseAllowed?: boolean;
  timeoutSeconds?: number;
}

export interface WorkflowLoopPolicy {
  maxIterations?: number;
  terminationExpression?: string;
}

export interface WorkflowNode {
  nodeId: string;
  label: string;
  kind: WorkflowNodeKind;
  reads: string[];
  writes: string[];
  evidenceOutputs: string[];
  authorityRequirements: string[];
  checkpointPolicy?: WorkflowCheckpointPolicy;
  loopPolicy?: WorkflowLoopPolicy;
  subflowRef?: string;
  config: Json;
}

export interface WorkflowEdge {
  edgeId: string;
  from: string;
  to: string;
  relation: string;
}

export interface WorkflowGraph {
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
}

export interface WorkflowContextContract {
  allowedSections: string[];
}

export interface WorkflowConfidence {
  score: number;
  rationale: string;
}

export interface WorkflowLineage {
  parentWorkflowDraftId?: string;
  parentDefinitionId?: string;
  forkReason?: string;
  mergeRefs: string[];
}

export interface WorkflowDraftPolicy {
  recommendationOnly: boolean;
  requireReview: boolean;
  allowShadowExecution: boolean;
}

export interface WorkflowProvenance {
  createdBy: string;
  createdAt: string;
  sourceMode: string;
}

export interface WorkflowGovernanceRef {
  gateLevel: string;
  gateStatus: string;
  decisionGateId: string;
  replayContractRef: string;
  sourceOfTruth: string;
  lineageId: string;
  degradedReason?: string;
  definitionDigest: string;
  bindingDigest: string;
}

export interface WorkflowDefinition {
  schemaVersion: string;
  definitionId: string;
  scope: WorkflowScope;
  intentRef?: string;
  intent: string;
  motifKind: WorkflowMotifKind;
  constraints: WorkflowConstraintRule[];
  graph: WorkflowGraph;
  contextContract: WorkflowContextContract;
  confidence: WorkflowConfidence;
  lineage: WorkflowLineage;
  policy: WorkflowDraftPolicy;
  provenance: WorkflowProvenance;
  governanceRef?: WorkflowGovernanceRef;
  digest?: string;
}

export interface WorkflowDraftEnvelope {
  [key: string]: unknown;
}

export interface WorkflowProposalEnvelope {
  [key: string]: unknown;
}

export interface WorkflowIntentEnvelope {
  [key: string]: unknown;
}

export interface WorkflowCandidateValidationEnvelope {
  valid: boolean;
  errors?: unknown[];
  warnings?: unknown[];
}

export interface WorkflowCandidateCompileEnvelope {
  valid: boolean;
  digest?: string;
  warnings?: unknown[];
  [key: string]: unknown;
}

export interface WorkflowCandidateEnvelope {
  candidateId: string;
  workflowDraft: WorkflowDraftEnvelope;
  validation: WorkflowCandidateValidationEnvelope;
  compileResult?: WorkflowCandidateCompileEnvelope | null;
  generationTrace: Json;
  inputHash: string;
}

export interface WorkflowCandidateSet {
  candidateSetId: string;
  scopeKey: string;
  intent: string;
  motifKind: WorkflowMotifKind;
  constraints?: WorkflowConstraintRule[];
  mode: WorkflowGenerationMode;
  createdBy: string;
  createdAt: string;
  candidates: WorkflowCandidateEnvelope[];
}

export interface WorkflowIntentRequest {
  intent: string;
  motifKind: WorkflowMotifKind;
  scope?: WorkflowScope;
  constraints?: WorkflowConstraintRule[];
  authorityCeiling?: string;
  createdBy?: string;
  sourceMode?: string;
}

export interface WorkflowIntentResponse {
  accepted: boolean;
  workflowIntent: WorkflowIntentEnvelope;
}

export interface WorkflowCandidateRequest {
  intent: string;
  motifKind: WorkflowMotifKind;
  scope?: WorkflowScope;
  generationMode?: WorkflowGenerationMode;
  candidateSetId?: string;
  constraints?: WorkflowConstraintRule[];
  count?: number;
  createdBy?: string;
  sourceMode?: string;
}

export interface WorkflowCandidatesResponse {
  schemaVersion: string;
  generatedAt: string;
  candidateSetId: string;
  candidates: WorkflowCandidateEnvelope[];
  blockedCount: number;
}

export interface WorkflowCandidateStageRequest {
  candidateId: string;
  stagedBy: string;
  rationale: string;
  expectedInputHash: string;
}

export interface WorkflowCandidateStageResponse {
  accepted: boolean;
  workflowDraftId: string;
  scopeKey: string;
  storedAt: string;
}

export interface WorkflowProposeRequest {
  proposedBy: string;
  rationale: string;
}

export interface WorkflowDefinitionResponse {
  schema_version: string;
  generated_at: string;
  definition: WorkflowDefinition;
}

export interface WorkflowProjectionResponse {
  schema_version: string;
  generated_at: string;
  projection_kind: WorkflowProjectionKind;
  projection: Json | unknown[];
  available_projections?: Array<{
    kind: WorkflowProjectionKind;
    label: string;
  }>;
}

export interface WorkflowInstanceResponse {
  schema_version: string;
  generated_at: string;
  instance: Json;
}

export interface WorkflowTraceResponse {
  schema_version: string;
  generated_at: string;
  trace: Json | Json[] | unknown[];
}

export interface WorkflowCheckpointResponse {
  schema_version: string;
  generated_at: string;
  checkpoints: unknown[];
}

export interface WorkflowOutcomeResponse {
  schema_version: string;
  generated_at: string;
  outcome: Json | null;
}

export interface WorkflowReplayResponse {
  schema_version: string;
  generated_at: string;
  replay: Json;
}

export interface WorkflowDigestResponse {
  schema_version: string;
  generated_at: string;
  digest: Json;
}

export interface WorkflowActiveScopeResponse {
  schema_version: string;
  generated_at: string;
  active: Json;
}

export interface WorkflowTopologyNode {
  id: string;
  type: "state" | "decision" | "action" | "gate" | "start" | "end";
  label: string;
  status: "pending" | "active" | "completed" | "failed" | "skipped";
  metadata?: Json;
}

export interface WorkflowTopologyEdge {
  id: string;
  from: string;
  to: string;
  label?: string;
  status: "idle" | "traversed" | "blocked";
}

export interface WorkflowTopology {
  nodes: WorkflowTopologyNode[];
  edges: WorkflowTopologyEdge[];
}

export interface WorkflowTopologyResponse {
  schema_version: string;
  generated_at: string;
  topology: WorkflowTopology;
}

export interface PathAssessment {
  goal?: string;
  recommended_path?: {
    name?: string;
    risk_score?: number;
    node_ids?: string[];
  };
}

export interface AgentContributionResponse {
  accepted: boolean;
  runId: string;
  workflowId: string;
  status: string;
  startedAt: string;
  streamChannel: string;
  runtimeMode?: string;
  temporalWorkflowId?: string;
  temporalRunId?: string;
  projectionMode?: string;
}

export interface SpaceCreateRequest {
  space_id: string;
  creation_mode?: "blank" | "import" | "template";
  owner?: string;
  archetype?: string;
  governance_scope?: "personal" | "private" | "public";
  reference_uri?: string;
  template_id?: string;
  draft_id?: string;
  draft_source_mode?: "blank" | "template" | "reference";
  lineage_note?: string;
}

export interface SpaceCreateResponse {
  space_id: string;
  status: string;
  message: string;
}

export interface AgentRunEventEnvelope {
  type: string;
  runId: string;
  spaceId: string;
  timestamp: string;
  sequence: number;
  payload?: Record<string, unknown>;
}

export interface AgentRunApprovalRequest {
  decision: "approved" | "rejected" | "needs_modification";
  rationale?: string;
  actor: string;
  decisionRef?: string;
}

export interface AgentRunRecord {
  runId: string;
  workflowId: string;
  spaceId: string;
  contributionId: string;
  agentId?: string;
  status: string;
  startedAt: string;
  updatedAt: string;
  streamChannel?: string;
  provider?: string;
  model?: string;
  authMode?: string;
  responseId?: string;
  promptTemplateArtifactId?: string;
  promptTemplateRevisionId?: string;
  promptExecutionArtifactId?: string;
  parentRunId?: string;
  childRunIds?: string[];
  providerTraceSummary?: Json | null;
  simulation?: Record<string, unknown>;
  surfaceUpdate?: Record<string, unknown>;
  authorityOutcome?: Record<string, unknown>;
  temporalBinding?: {
    workflowId: string;
    temporalRunId?: string;
    taskQueue?: string;
    namespace?: string;
    projectionMode?: string;
    status?: string;
    lastProjectedSequence?: number;
  };
  shadowSummary?: {
    comparedAt: string;
    status: string;
    criticalCount: number;
    warningCount: number;
    infoCount: number;
    divergences: Array<{
      severity: "critical" | "warning" | "info";
      code: string;
      message: string;
      expected?: unknown;
      actual?: unknown;
    }>;
  };
  approvalTimeoutSeconds?: number;
  events: AgentRunEventEnvelope[];
}

export interface AgentRunSummary {
  runId: string;
  workflowId: string;
  spaceId: string;
  contributionId: string;
  agentId?: string;
  provider?: string;
  model?: string;
  authMode?: string;
  promptTemplateArtifactId?: string;
  promptExecutionArtifactId?: string;
  parentRunId?: string;
  childRunIds?: string[];
  status: string;
  startedAt: string;
  updatedAt: string;
  authorityLevel?: string;
  requiresReview: boolean;
}

export interface WorkRouterStatusResponse {
  service: string;
  mode: string;
  maxDispatchLevel: string;
  mutationAllowed: boolean;
  liveTransportEnabled: boolean;
  health: string;
  pendingCount: number;
  exportedCount: number;
  outboxEnvelopeCount: number;
  unknownResponseCount: number;
  lastObservedAt?: string;
  lastEvidenceId?: string;
  lastEvidenceStatus?: string;
  lastEvidenceFinishedAt?: string;
  authority: {
    sourceMutationAllowed: boolean;
    runtimeMutationAllowed: boolean;
    forbiddenActionsConfirmed: string[];
  };
}

export interface WorkRouterPendingDispatchSummary {
  runId: string;
  status: string;
  taskRef?: string;
  route?: string;
  riskLevel?: string;
  maxLevel?: string;
  transportKind?: string;
  requestId?: string;
  channelRef?: string;
  createdAt?: string;
  startedAt?: string;
  messagePreview?: string;
}

export interface WorkRouterUnknownResponseSummary {
  unknownId: string;
  rawText: string;
  normalizedText: string;
  status: string;
  createdAt?: string;
  proposedClassification?: string;
  proposedMapping?: Record<string, unknown>;
}

export interface WorkRouterDispatchQueueResponse {
  pending: WorkRouterPendingDispatchSummary[];
  unknowns: WorkRouterUnknownResponseSummary[];
}

export type SpatialSurfaceVariant = "linear" | "spatial" | "compare";

export interface SpatialPlaneLayoutPoint {
  x: number;
  y: number;
}

export interface SpatialPlaneLayoutBody {
  shape_positions: Record<string, SpatialPlaneLayoutPoint>;
  collapsed_groups: Record<string, boolean>;
  view_state?: {
    zoom?: number;
    pan_x?: number;
    pan_y?: number;
  };
  selected_shape_ids?: string[];
}

export interface SpatialPlaneLayoutLineage {
  view_spec_id?: string;
  workflow_id?: string;
  graph_hash?: string;
  space_id?: string;
  updated_by: string;
  updated_at: string;
}

export interface SpatialPlaneLayoutV1 {
  schema_version: string;
  plane_id: string;
  view_spec_id: string;
  space_id: string;
  revision: number;
  layout: SpatialPlaneLayoutBody;
  lineage: SpatialPlaneLayoutLineage;
}

export interface SpatialPlaneLayoutResponse {
  accepted: boolean;
  layout: SpatialPlaneLayoutV1;
}

export interface SpatialExperimentEventRequest {
  run_id: string;
  space_id: string;
  mode: string;
  surface_variant: SpatialSurfaceVariant;
  event_type: string;
  timestamp: string;
  payload: Json;
  build_id?: string;
  host: string;
}

export interface SpatialExperimentEventResponse {
  accepted: boolean;
  stored_key: string;
  event_id: string;
}

export interface SpatialExperimentMetrics {
  time_to_first_interaction_ms?: number;
  task_completion_ms?: number;
  approval_decision_count: number;
  spatial_interaction_count: number;
  adapter_fallback_rate: number;
  error_event_count: number;
}

export interface SpatialExperimentComplexityDelta {
  bundle_delta_kb?: number;
  runtime_overhead_ms?: number;
  adapter_fallback_rate: number;
}

export interface SpatialExperimentRunSummary {
  schema_version: string;
  generated_at: string;
  run_id: string;
  space_id: string;
  mode: string;
  surface_variant: string;
  host: string;
  build_id?: string;
  metrics: SpatialExperimentMetrics;
  improvement_score: number;
  recommendation: "go" | "no_go" | "hold";
  complexity_delta: SpatialExperimentComplexityDelta;
  verdict_rationale?: string;
  event_count: number;
  event_key: string;
}

export interface HeapBlockProjection {
  artifactId: string;
  spaceId?: string;
  blockId?: string;
  title: string;
  blockType: string;
  updatedAt: string;
  emittedAt?: string;
  tags: string[];
  mentionsInline: string[];
  mentionsQuery?: string[];
  pageLinks?: string[];
  fileKeys?: string[];
  hasFiles?: boolean;
  status?: string;
  attributes?: Record<string, string>;
}

export interface HeapBlockListItem {
  projection: HeapBlockProjection;
  surfaceJson: Json;
  warnings?: string[];
  pinnedAt?: string;
  deletedAt?: string;
}

export interface HeapBlocksResponse {
  schemaVersion: string;
  generatedAt: string;
  count: number;
  hasMore: boolean;
  nextCursor?: string;
  items: HeapBlockListItem[];
}

export interface HeapDeletedListItem {
  artifactId: string;
  deletedAt: string;
}

export interface HeapChangedBlocksResponse {
  schemaVersion: "1.0.0";
  generatedAt: string;
  count: number;
  hasMore: boolean;
  nextCursor?: string;
  changed: HeapBlockListItem[];
  deleted: HeapDeletedListItem[];
}

export interface HeapBlocksQueryParams {
  spaceId?: string;
  tag?: string;
  mention?: string;
  pageLink?: string;
  attribute?: string;
  blockType?: string;
  hasFiles?: boolean;
  fromTs?: string;
  changedSince?: string;
  toTs?: string;
  includeDeleted?: boolean;
  limit?: number;
  cursor?: string;
}

export interface HeapContextBundleBlock {
  artifact_id: string;
  title: string;
  block_type: string;
  tags: string[];
  mentions: string[];
  surface_json: Json;
  updated_at: string;
}

export interface HeapBlocksContextResponse {
  context_bundle: {
    blocks: HeapContextBundleBlock[];
    block_count: number;
    prepared_at: string;
  };
}

export interface ChatAgentIdentity {
  id: string;
  label: string;
  route: string;
  mode: string;
}

export interface ChatConversationAnchorContract {
  kind: "page" | "view" | "block" | "component";
  label: string;
  href: string;
  routeId?: string;
  artifactId?: string;
  viewId?: string;
  blockId?: string;
  componentId?: string;
}

export interface ChatMessageTextPart {
  type: "text";
  text: string;
}

export interface ChatMessageA2uiPart {
  type: "a2ui";
  surfaceId: string;
  title?: string;
  tree: Json;
}

export interface ChatMessagePointerPart {
  type: "pointer";
  href: string;
  label: string;
  artifactId?: string;
  description?: string;
}

export type ChatMessagePart =
  | ChatMessageTextPart
  | ChatMessageA2uiPart
  | ChatMessagePointerPart;

export interface ChatConversationMessage {
  id: string;
  role: "user" | "agent";
  timestamp: string;
  text: string;
  content: ChatMessagePart[];
  artifactIds: string[];
  agent?: ChatAgentIdentity;
}

export interface ChatConversationSummary {
  threadId: string;
  title: string;
  anchor: ChatConversationAnchorContract | null;
  messageCount: number;
  lastMessagePreview: string;
  createdAt: string;
  updatedAt: string;
  recentTurns: Array<{
    role: "user" | "agent";
    text: string;
    timestamp: string;
  }>;
}

export interface ChatConversationSummaryResponse {
  generatedAt: string;
  count: number;
  items: ChatConversationSummary[];
}

export interface ChatConversationDetail extends ChatConversationSummary {
  messages: ChatConversationMessage[];
}

export interface HeapBlockHistoryVersion {
  version: number;
  timestamp: string;
  mutation_type: string;
  actor: string;
}

export interface HeapArtifactRevisionRecord {
  revisionId: string;
  revisionNumber: number;
  createdAt: string;
  createdBy: string;
  parentRevisionId?: string | null;
  published?: boolean;
  contentHash?: string | null;
}

export interface HeapLineageRelationRecord {
  relation: string;
  artifactId: string;
  title?: string;
  blockType?: string;
  subtitle?: string;
}

export interface HeapLegacyDuplicateRecord {
  title: string;
  contentHash?: string | null;
  artifactIds: string[];
}

export interface HeapBlockHistoryResponse {
  artifact_id: string;
  versions: HeapBlockHistoryVersion[];
  revisions?: HeapArtifactRevisionRecord[];
  lineage?: HeapLineageRelationRecord[];
  uploadExtractionRuns?: HeapUploadExtractionRunRecord[];
  legacyDuplicates?: HeapLegacyDuplicateRecord[];
}
export interface HeapUploadArtifactThumbnail {
  type: string;
  size: string;
  path?: string;
  width?: number;
  height?: number;
}

export interface HeapUploadArtifactResponse {
  upload_id: string;
  resource_ref: string;
  hash: string;
  name: string;
  mime_type: string;
  file_size: number;
  is_uploaded: boolean;
  thumbnails: HeapUploadArtifactThumbnail[];
  extraction_supported: boolean;
}

export type HeapUploadExtractionStatus = "submitted" | "running" | "completed" | "needs_review" | "failed";

export interface HeapUploadExtractionTriggerResponse {
  job_id: string;
  status: "submitted";
  upload_id: string;
  requested_parser_profile?: string;
}

export interface HeapUploadParserProfileRecord {
  parser_profile: "auto" | "docling" | "liteparse" | "markitdown" | string;
  configured: boolean;
  supports_mime: string[];
  role: "primary" | "fallback" | "normalizer" | string;
  parser_hint: string;
}

export interface HeapUploadParserProfilesResponse {
  generated_at: string;
  items: HeapUploadParserProfileRecord[];
}

export interface HeapUploadExtractionStatusResponse {
  job_id: string;
  upload_id: string;
  status: HeapUploadExtractionStatus;
  created_at?: string;
  requested_parser_profile?: string;
  parser_backend?: string;
  confidence?: number;
  flags?: string[];
  result_ref?: string;
  summary?: string;
  page_count?: number;
  block_count?: number;
  last_updated_at?: string;
}

export interface HeapUploadExtractionRunRecord extends HeapUploadExtractionStatusResponse {}

export interface HeapUploadExtractionRunsResponse {
  generated_at: string;
  upload_id: string;
  items: HeapUploadExtractionRunRecord[];
}

export interface HeapUploadExtractionRunDetail {
  job_id: string;
  upload_id: string;
  status: HeapUploadExtractionStatus;
  created_at?: string;
  requested_parser_profile?: string;
  parser_backend?: string;
  confidence?: number;
  flags?: string[];
  result_ref?: string;
  summary?: string;
  page_count?: number;
  block_count?: number;
  last_updated_at?: string;
  attempted_backends?: string[];
  model_id?: string;
  first_page_preview?: string[];
  first_page_block_count?: number;
}


export interface CommonsIntegrityViolation {
  rule_id: string;
  affected_nodes: string[];
  severity: string;
  explanation: string;
}

export type ProviderType = "Llm" | "Embedding" | "Vector" | "Batch";
export type LlmProviderType = "OpenAI" | "Anthropic" | "Ollama" | "Ignition" | "OpenRouter" | "DoubleWord" | "Mock";
export type ProviderLocalityKind = "Local" | "Tunneled" | "Cloud";
export type ProviderBatchCadenceKind = "Immediate" | "Interval" | "TimeWindow" | "Scoped" | "Manual";
export type ProviderBatchScopeKind = "ProviderFamily" | "ProviderProfile" | "Space" | "Agent" | "Session" | "RequestGroup";
export type ProviderBatchFlushPolicy = "OnInterval" | "OnWindowClose" | "OnThreshold" | "OnIdle" | "Manual";

export interface ProviderBatchWindow {
  intervalSeconds?: number;
  maxItems?: number;
  maxAgeSeconds?: number;
  timezone?: string;
}

export interface ProviderBatchPolicy {
  providerFamilyId: string;
  providerProfileId?: string;
  cadenceKind: ProviderBatchCadenceKind;
  scopeKind: ProviderBatchScopeKind;
  flushPolicy: ProviderBatchFlushPolicy;
  orderingKey?: string;
  dedupeKey?: string;
  batchWindow?: ProviderBatchWindow;
}

export interface ProviderTopology {
  familyId: string;
  profileId?: string;
  instanceId: string;
  deviceId?: string;
  environmentId?: string;
  localityKind: ProviderLocalityKind;
  lastSeenAt?: string;
  discoverySource?: string;
}

export interface ProviderRecord {
  id: string;
  name: string;
  providerType: ProviderType;
  providerFamily?: LlmProviderType | string;
  hostId: string;
  endpoint: string;
  isActive: boolean;
  priority: number;
  defaultModel?: string;
  supportedModels?: string[];
  failMode?: "fallback" | "fail_closed";
  adapterHealth?: Json;
  adapterHealthError?: string;
  openapiPaths?: string[];
  upstreamModelsError?: string;
  authMode?: string;
  authState?: "not_required" | "linked" | "inherited" | "missing";
  authSource?: string;
  authBindingId?: string;
  authType?: "none" | "api_key" | "bearer_token" | "pat" | "ssh_key" | "ssh_password";
  bindingIds?: string[];
  configJson?: string;
  batchPolicy?: ProviderBatchPolicy;
  metadata?: Record<string, string>;
  topology?: ProviderTopology;
}

export interface RuntimeHostRecord {
  hostId: string;
  name: string;
  hostKind: "local" | "vps" | "tunnel" | "managed";
  endpoint: string;
  localityKind: ProviderLocalityKind | string;
  deviceId?: string;
  environmentId?: string;
  health?: Json;
  capabilities?: string[];
  remoteDiscoveryEnabled: boolean;
  executionRoutable: boolean;
  updatedAt?: string;
  metadata?: Record<string, string>;
}

export interface AuthBindingRecord {
  authBindingId: string;
  targetKind: "provider" | "host";
  targetId: string;
  providerId?: string;
  authType: "none" | "api_key" | "bearer_token" | "pat" | "ssh_key" | "ssh_password";
  label?: string;
  hasSecret: boolean;
  source?: string;
  updatedAt: string;
  metadata?: Record<string, string>;
}

export interface ExecutionBindingRecord {
  bindingId: string;
  providerType: ProviderType;
  boundProviderId: string;
  updatedAt?: string;
  metadata?: Record<string, string>;
}

export interface ProviderDiscoveryRecord {
  providerId: string;
  providerType: ProviderType;
  providerKind?: LlmProviderType;
  endpoint: string;
  defaultModel?: string;
  supportedModels?: string[];
  adapterHealth?: Json;
  adapterHealthError?: string;
  openapiPaths?: string[];
  upstreamModelsError?: string;
  failMode?: "fallback" | "fail_closed";
  topology?: ProviderTopology;
  updatedAt?: string;
  metadata?: Record<string, string>;
}

export interface OperatorProviderInventoryResponse {
  schemaVersion: string;
  generatedAt: string;
  providers: ProviderRecord[];
}

export interface RuntimeHostInventoryResponse {
  schemaVersion: string;
  generatedAt: string;
  runtimeHosts: RuntimeHostRecord[];
}

export interface AuthBindingInventoryResponse {
  schemaVersion: string;
  generatedAt: string;
  authBindings: AuthBindingRecord[];
}

export interface ExecutionBindingStatusResponse {
  schemaVersion: string;
  generatedAt: string;
  executionBindings: ExecutionBindingRecord[];
}

export interface ProviderDiscoveryInventoryResponse {
  schemaVersion: string;
  generatedAt: string;
  discoveryRecords: ProviderDiscoveryRecord[];
}

export interface SystemProvidersResponse {
  providers: ProviderRecord[];
  runtimeHosts?: RuntimeHostRecord[];
  authBindings?: AuthBindingRecord[];
  executionBindings?: ExecutionBindingRecord[];
  discoveryRecords?: ProviderDiscoveryRecord[];
}

export interface SystemProviderRuntimeStatusResponse {
  enabled: boolean;
  baseUrl: string;
  failMode: "fallback" | "fail_closed";
  model: string;
  providerId?: string;
  bindingId?: string;
  adapterHealth?: Json;
  adapterHealthError?: string;
  openapiPaths?: string[];
  openapiError?: string;
  upstreamModels?: Json;
  upstreamModelsError?: string;
}

export interface ProviderValidationRequest {
  providerType: ProviderType;
  providerKind?: LlmProviderType;
  providerId?: string;
  authBindingId?: string;
  useStoredAuth?: boolean;
  baseUrl: string;
  defaultModel?: string;
  apiKey: string;
  validateKey?: boolean;
  validateChat?: boolean;
  validateEmbeddings?: boolean;
}

export interface ProviderValidationResponse {
  providerType: ProviderType;
  providerKind?: LlmProviderType;
  endpoint: string;
  canonicalBaseUrl: string;
  validateKey: boolean;
  validateChat: boolean;
  validateEmbeddings: boolean;
  keyValid: boolean;
  keyError?: string;
  keyInfo?: Json;
  modelsValid: boolean;
  modelsError?: string;
  chatValid: boolean;
  chatError?: string;
  embeddingsValid: boolean;
  embeddingsError?: string;
  supportedModels: string[];
  selectedModel?: string;
  valid: boolean;
  discoveredAt: string;
}

export interface SuggestedEnrichment {
  enrichmentId: string;
  kind: "mention" | "tag" | "duration" | "pull_request";
  displayLabel: string;
  matchedText: string;
  start: number;
  end: number;
  metadata: Json;
}

export interface CommonsEnforcementOutcome {
  mode: "shadow" | "warn_or_block";
  shouldBlock: boolean;
  shouldWarn: boolean;
  violations: CommonsIntegrityViolation[];
  suggestedEnrichments: SuggestedEnrichment[];
}

export interface HeapStewardGateValidateResponse {
  schemaVersion: string;
  artifactId: string;
  status: "pass" | "action_required";
  outcome: CommonsEnforcementOutcome;
  surface?: Json;
  stewardGateToken?: string;
}

export interface HeapStewardGateApplyResponse {
  schemaVersion: string;
  accepted: boolean;
  artifactId: string;
  enrichmentId: string;
  childArtifactId: string;
  childBlockId: string;
  validation: HeapStewardGateValidateResponse;
}

export interface ArtifactDecisionProof {
  decisionId: string;
  signature: string;
  signer: string;
  algorithm?: string;
  nonce?: string;
  expiresAt?: string;
}

export interface ArtifactGovernanceEnvelope {
  approvedBy: string;
  rationale: string;
  approvedAt: string;
  actorId: string;
  decisionProof: ArtifactDecisionProof;
  nonce?: string;
  expiresAt?: string;
}

export interface ArtifactPublishRequest {
  leaseId?: string;
  expectedRevisionId?: string;
  notes?: string;
  governance?: ArtifactGovernanceEnvelope;
  stewardGateToken?: string;
}

export type HeapPayloadType = "a2ui" | "rich_text" | "media" | "structured_data" | "pointer" | "task";

export interface EmitHeapBlockSource {
  agent_id: string;
  session_id?: string;
  request_id?: string;
  emitted_at: string;
}

export interface EmitHeapBlockContent {
  payload_type: HeapPayloadType;
  a2ui?: {
    surface_id: string;
    protocol_version: string;
    renderer?: "dioxus" | "lit" | "web" | "native" | "other";
    view_type?: string;
    tree: Json;
    data_model?: Json;
  };
  rich_text?: {
    plain_text: string;
    title_doc?: Json;
    text_doc?: Json;
  };
  task?: string;
  media?: {
    hash: string;
    mime_type: string;
  };
  structured_data?: Json;
  pointer?: string;
}

type EmitHeapBlockSpaceSelector =
  | { space_id: string; workspace_id?: never }
  | { space_id?: never; workspace_id: string }
  | { space_id: string; workspace_id: string };

export type EmitHeapBlockRequest = EmitHeapBlockSpaceSelector & {
  schema_version: "1.0.0";
  mode: "heap";
  source: EmitHeapBlockSource;
  block: {
    id?: string;
    type: string;
    title: string;
    icon?: string;
    icon_type?: "emoji" | "image" | "icon" | "none";
    color?: string;
    main_tag?: string;
    attributes?: Record<string, string>;
    behaviors?: string[];
  };
  content: EmitHeapBlockContent;
  relations?: {
    tags?: Array<{ to_block_id: string; meta?: Json }>;
    mentions?: Array<{ to_block_id: string; label?: string; source_path?: string }>;
    page_links?: Array<{ to_block_id: string; source_path?: string }>;
  };
  files?: Array<{
    hash: string;
    file_size: number;
    name: string;
    mime_type?: string;
    path?: string;
    is_uploaded?: boolean;
    thumbnails?: Array<{
      type: string;
      size: string;
      path?: string;
      width?: number;
      height?: number;
    }>;
  }>;
  apps?: Array<{
    id: string;
    name?: string;
    app_type?: string;
    filter?: Json;
    sort?: Json;
    mapping?: Json;
  }>;
  meta?: {
    schema_version?: string;
    canonical_hash?: string;
    request_path?: string;
    reply_to_block_id?: string;
  };
  projection_hints?: Json;
  crdt_projection?: Json;
};

export interface EmitHeapBlockResponse {
  accepted: boolean;
  artifactId: string;
  blockId?: string;
  opId?: string;
  idempotent?: boolean;
  sourceOfTruth?: string;
  fallbackActive?: boolean;
}

export type GateSummaryKind = "siq" | "testing";

export interface EmitGateSummaryHeapBlockRequest {
  schemaVersion: "1.0.0";
  spaceId: string;
  kind: GateSummaryKind;
  artifactId?: string;
}

export interface A2UISubmitFeedbackRequest {
  artifactId: string;
  feedbackData: Record<string, unknown>;
}

export interface A2UISubmitFeedbackResponse {
  accepted: boolean;
  artifactId: string;
  feedbackArtifactId: string;
  storedAt: string;
  reviewOutcomeMode?: string;
  followUpBlockType?: string;
  followUpArtifactId?: string;
}

export interface EmitGateSummaryHeapBlockResponse {
  schemaVersion: string;
  accepted: boolean;
  kind: GateSummaryKind;
  workspaceId: string;
  heapWorkspaceId: string;
  artifactId: string;
  blockId: string;
  emittedAt: string;
}

export interface BrandPalette {
  outer_base: string;
  outer_gradient_to: string;
  inner_base: string;
  inner_gradient_to: string;
}

export interface LabsBounds {
  gap_min_degrees: number;
  gap_max_degrees: number;
  stroke_min_px: number;
  stroke_max_px: number;
}

export interface TemporalVariantPolicy {
  force_gradient: boolean;
  stroke_cap: string;
  palette: BrandPalette;
}

export interface PhilosophicalModeBaselinePolicy {
  gap_degrees: number;
  stroke_width_delta_px: number;
  stroke_cap: string;
  force_gradient: boolean;
}

export interface TechnicalModeBaselinePolicy {
  stroke_cap: string;
  force_gradient: boolean;
}

export interface BrandModeBaselinesPolicy {
  philosophical: PhilosophicalModeBaselinePolicy;
  technical: TechnicalModeBaselinePolicy;
}

export interface BrandHostDefaultsPolicy {
  default_temporal_state: string;
  default_authority: "official" | "labs" | string;
  theme_mode_map: Record<string, "philosophical" | "technical" | "custom" | string>;
}

export interface BrandPolicyDocument {
  schema_version: string;
  policy_id: string;
  policy_version: number;
  kernel: {
    mark_composition: string;
    technical_canonical_gap_degrees: number;
    ring_radius_px: number;
    dot_radius_px: number;
    base_stroke_width_px: number;
    steward_gated: boolean;
  };
  style: {
    allow_labs_customizations: boolean;
    labs_bounds: LabsBounds;
    official_palette: BrandPalette;
    mode_baselines?: BrandModeBaselinesPolicy;
    host_defaults?: BrandHostDefaultsPolicy;
    temporal_variants: Record<string, TemporalVariantPolicy>;
    motion: {
      philosophical: {
        container_transition_sec: number;
        ring_transition_sec: number;
        stroke_transition_sec: number;
        ring_animation_duration_sec: number;
        ring_rotation_delta_deg: number;
        ring_stroke_delta_px: number;
        dot_transition_sec: number;
        dot_animation_duration_sec: number;
        dot_pulse_radius_delta_px: number;
        dot_pulse_opacity_min: number;
      };
      technical: {
        container_transition_ms: number;
        ring_transition_ms: number;
        stroke_transition_ms: number;
        dot_transition_ms: number;
        step_count: number;
        ring_step_count: number;
      };
    };
  };
  temporal_windows: Array<{
    state: string;
    recurrence: string;
    start_month_day: string;
    end_month_day: string;
    start_time_utc: string;
    end_time_utc: string;
  }>;
  updated_at_ns: number;
  source_ref?: string;
}

export interface BrandPolicyResponse {
  policy: BrandPolicyDocument;
  policyVersion: number;
  policyDigest: string;
  activeTemporalState: string;
  serverTimeUtc: string;
  sourceOfTruth: "canister" | "cache" | "fallback";
  degradedReason?: string;
  policyNormalization?: "none" | "legacy_defaults_applied";
}

export interface NavigationEntrySpec {
  routeId: string;
  label: string;
  icon: string;
  category: string;
  requiredRole: string;
  navSlot?: string;
  navMeta?: {
    badgeCount?: number;
    badgeTone?: "default" | "info" | "warn" | "critical";
    attention?: boolean;
    attentionLabel?: string;
    collapsibleHint?: "expanded" | "rail" | "hidden";
  };
}

export interface NavigationGraphSpec {
  entries: NavigationEntrySpec[];
}

export interface ShellLayoutSpec {
  layoutId: string;
  navigationGraph: NavigationGraphSpec;
}
