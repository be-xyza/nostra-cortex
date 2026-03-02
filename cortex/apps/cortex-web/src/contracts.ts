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
  dfxPortHealthy: boolean;
  gatewayPort: number;
  notes: string[];
}

export interface DpubSystemBuildResponse {
  buildId: string;
  buildTimeUtc: string;
  gatewayDispatchMode: string;
  gatewayPort: number;
  workspaceRoot: string;
}

export interface InitiativeNode {
  id: string;
  title: string;
  status: string;
  layer: string;
  portfolio_role?: string;
}

export interface InitiativeEdge {
  from: string;
  to: string;
  edge_kind?: string;
  confidence?: number;
  is_explicit?: boolean;
}

export interface InitiativeGraph {
  graph_root_hash: string;
  nodes: InitiativeNode[];
  edges: InitiativeEdge[];
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
}

export interface CapabilityEdge {
  from: string;
  to: string;
  relationship: string;
}

export interface PlatformCapabilityGraph {
  schema_version: string;
  generated_at: string;
  source_of_truth: string;
  nodes: CapabilityNode[];
  edges: CapabilityEdge[];
}

export interface PathAssessment {
  goal?: string;
  recommended_path?: {
    name?: string;
    risk_score?: number;
    node_ids?: string[];
  };
}

export interface AgentInitiativeResponse {
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
  reference_uri?: string;
  template_id?: string;
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
  initiativeId: string;
  status: string;
  startedAt: string;
  updatedAt: string;
  streamChannel?: string;
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

export type SpatialSurfaceVariant = "linear" | "spatial" | "compare";

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
  workspaceId?: string;
  blockId?: string;
  title: string;
  blockType: string;
  updatedAt: string;
  emittedAt?: string;
  tags: string[];
  mentionsInline: string[];
  mentionsQuery?: string[];
  fileKeys?: string[];
  hasFiles?: boolean;
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
}

export interface NavigationGraphSpec {
  entries: NavigationEntrySpec[];
}

export interface ShellLayoutSpec {
  layoutId: string;
  navigationGraph: NavigationGraphSpec;
}
