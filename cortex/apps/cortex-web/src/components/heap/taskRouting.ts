import type { EmitHeapBlockRequest, HeapBlockListItem } from "../../contracts.ts";
import type { HeapRelationIndex } from "./heapRelations.ts";
import type { TaskRoutingContext } from "./initiativeKickoffTemplates.ts";

export type TaskRouteId =
  | "direct_agent_assignment"
  | "proposal_generation"
  | "steward_escalation";

export type TaskRouteSurfaceMode = "chat" | "generate";

export type TaskRouteConfidence = "high" | "medium" | "low";

export type WorkflowMotifKind =
  | "sequential"
  | "parallel_compare"
  | "repair_loop"
  | "fan_out_join"
  | "human_gate";

export type WorkflowGenerationMode = "deterministic_scaffold" | "motif_hybrid";

export interface TaskRouteDecision {
  route_id: TaskRouteId;
  label: string;
  confidence_hint: TaskRouteConfidence;
  surface_mode: TaskRouteSurfaceMode;
  recommended_role: string;
  recommended_capabilities: string[];
  summary: string;
  rationale: string;
  prompt: string;
}

export interface TaskRouteWorkflowArtifactRef {
  workflow_intent_id?: string;
  candidate_set_id?: string;
  workflow_draft_id?: string;
  proposal_id?: string;
  definition_id?: string;
  scope_key?: string;
  motif_kind?: WorkflowMotifKind;
  generation_mode?: WorkflowGenerationMode;
  proposal_digest_path?: string;
}

export interface TaskRouteSummaryEmitRequest {
  source_task_artifact_id: string;
  space_id: string;
  route_id: TaskRouteId;
  decision: TaskRouteDecision;
  routed_at?: string;
  workflow?: TaskRouteWorkflowArtifactRef;
}

export interface TaskRouteSourceStampEmitRequest {
  source_block: HeapBlockListItem;
  route_id: TaskRouteId;
  decision: TaskRouteDecision;
  summary_artifact_id: string;
  routed_at?: string;
  workflow?: TaskRouteWorkflowArtifactRef;
}

export interface TaskRouteExecutionResult {
  route_id: TaskRouteId;
  route_label: string;
  confidence_hint: TaskRouteConfidence;
  source_task_artifact_id: string;
  summary_artifact_id?: string;
  stamped_source_artifact_id?: string;
  workflow?: TaskRouteWorkflowArtifactRef;
  routed_at: string;
  success: boolean;
  failure_reason?: string;
}

export interface TaskRouteExecutionResultInput {
  route_id: TaskRouteId;
  route_label: string;
  confidence_hint: TaskRouteConfidence;
  source_task_artifact_id: string;
  routed_at: string;
  success: boolean;
  summary_artifact_id?: string;
  stamped_source_artifact_id?: string;
  workflow?: TaskRouteWorkflowArtifactRef;
  failure_reason?: string;
}

export interface TaskRouteLineageSnapshot {
  task_title: string;
  source_task_artifact_id: string;
  initiative_id: string;
  route_id?: TaskRouteId;
  route_label?: string;
  confidence_hint?: TaskRouteConfidence;
  decision_mode?: string;
  route_summary?: string;
  route_rationale?: string;
  summary_artifact_id?: string;
  summary_artifact_title?: string;
  summary_artifact_linked: boolean;
  workflow?: TaskRouteWorkflowArtifactRef;
  reference_paths: string[];
}

export function buildTaskRouteExecutionResult(
  input: TaskRouteExecutionResultInput,
): TaskRouteExecutionResult {
  return {
    route_id: input.route_id,
    route_label: input.route_label,
    confidence_hint: input.confidence_hint,
    source_task_artifact_id: input.source_task_artifact_id,
    summary_artifact_id: input.summary_artifact_id,
    stamped_source_artifact_id: input.stamped_source_artifact_id,
    workflow: input.workflow,
    routed_at: input.routed_at,
    success: input.success,
    failure_reason: input.failure_reason,
  };
}

const PROPOSAL_KEYWORDS = [
  "proposal",
  "workflow draft",
  "workflow-draft",
  "draft proposal",
  "candidate set",
  "ratify",
  "comparison",
  "compare",
  "stage",
  "staging",
  "branch",
];

const STEWARD_KEYWORDS = [
  "ambiguity",
  "ambiguous",
  "missing",
  "unknown",
  "conflict",
  "unsafe",
  "blocked",
  "blocker",
  "policy",
  "ownership",
  "drift",
  "risk",
  "escalate",
];

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }
  return value as Record<string, unknown>;
}

function asString(value: unknown): string | null {
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function asStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((entry) => asString(entry))
    .filter((entry): entry is string => Boolean(entry));
}

function joinSection(title: string, entries: readonly string[], emptyLabel: string): string[] {
  if (entries.length === 0) {
    return [title, emptyLabel];
  }
  return [title, ...entries.map((entry) => `- ${entry}`)];
}

function phraseHit(text: string, phrases: readonly string[]): boolean {
  const normalized = text.toLowerCase();
  return phrases.some((phrase) => normalized.includes(phrase));
}

function anySignalMatches(values: readonly string[], phrases: readonly string[]): boolean {
  return values.some((value) => phraseHit(value, phrases));
}

function joinOrFallback(values: readonly string[], emptyLabel: string): string {
  return values.length > 0 ? values.join(", ") : emptyLabel;
}

function formatTaskRouteLabel(routeId: TaskRouteId): string {
  return routeId
    .replaceAll("_", " ")
    .replace(/\b\w/g, (match) => match.toUpperCase());
}

function isTaskRouteId(value: string | undefined): value is TaskRouteId {
  return value === "direct_agent_assignment" ||
    value === "proposal_generation" ||
    value === "steward_escalation";
}

function isTaskRouteConfidence(value: string | undefined): value is TaskRouteConfidence {
  return value === "high" || value === "medium" || value === "low";
}

function readTaskRouteWorkflowRef(attributes: Record<string, string>): TaskRouteWorkflowArtifactRef | null {
  const workflow: TaskRouteWorkflowArtifactRef = {};
  if (attributes.workflow_intent_id) {
    workflow.workflow_intent_id = attributes.workflow_intent_id;
  }
  if (attributes.workflow_candidate_set_id) {
    workflow.candidate_set_id = attributes.workflow_candidate_set_id;
  }
  if (attributes.workflow_draft_id) {
    workflow.workflow_draft_id = attributes.workflow_draft_id;
  }
  if (attributes.workflow_proposal_id) {
    workflow.proposal_id = attributes.workflow_proposal_id;
  }
  if (attributes.workflow_definition_id) {
    workflow.definition_id = attributes.workflow_definition_id;
  }
  if (attributes.workflow_scope_key) {
    workflow.scope_key = attributes.workflow_scope_key;
  }
  if (attributes.workflow_motif_kind === "sequential" ||
      attributes.workflow_motif_kind === "parallel_compare" ||
      attributes.workflow_motif_kind === "repair_loop" ||
      attributes.workflow_motif_kind === "fan_out_join" ||
      attributes.workflow_motif_kind === "human_gate") {
    workflow.motif_kind = attributes.workflow_motif_kind;
  }
  if (attributes.workflow_generation_mode === "deterministic_scaffold" ||
      attributes.workflow_generation_mode === "motif_hybrid") {
    workflow.generation_mode = attributes.workflow_generation_mode;
  }
  if (attributes.workflow_proposal_digest_path) {
    workflow.proposal_digest_path = attributes.workflow_proposal_digest_path;
  }
  return Object.keys(workflow).length > 0 ? workflow : null;
}

function readTaskRouteLineageAttributes(attributes: Record<string, string>) {
  const routeId = attributes.task_route_id ?? attributes.route_id;
  const normalizedRouteId = isTaskRouteId(routeId) ? routeId : undefined;
  const confidenceHint = isTaskRouteConfidence(attributes.task_route_confidence_hint)
    ? attributes.task_route_confidence_hint
    : undefined;
  return {
    routeId: normalizedRouteId,
    routeLabel: attributes.task_route_summary || attributes.routing_summary || undefined,
    confidenceHint,
    decisionMode: attributes.task_route_decision_mode ?? attributes.routing_decision_mode,
    routeRationale: attributes.task_route_rationale ?? attributes.routing_rationale,
    routeSummary: attributes.task_route_summary ?? attributes.routing_summary,
    summaryArtifactId: attributes.task_route_summary_artifact_id,
    workflow: readTaskRouteWorkflowRef(attributes),
  };
}

function buildTaskContentFromSurface(
  surface: Record<string, unknown>,
  titleFallback: string,
  taskFallback: string,
): EmitHeapBlockRequest["content"] {
  const payloadType = asString(surface.payload_type);

  if (payloadType === "task") {
    return {
      payload_type: "task",
      task: asString(surface.task) ?? taskFallback,
    };
  }

  if (payloadType === "pointer" && asString(surface.pointer)) {
    return {
      payload_type: "pointer",
      pointer: asString(surface.pointer) ?? taskFallback,
    };
  }

  const structuredData = asRecord(surface.structured_data) ?? asRecord(surface.data);
  if (structuredData) {
    return {
      payload_type: "structured_data",
      structured_data: structuredData,
    };
  }

  const richText =
    asRecord(surface.rich_text) ??
    (typeof surface.plain_text === "string" && surface.plain_text.trim()
      ? { plain_text: surface.plain_text.trim() }
      : null) ??
    (typeof surface.text === "string" && surface.text.trim()
      ? { plain_text: surface.text.trim() }
      : null);

  if (richText) {
    const plainText =
      asString(richText.plain_text) ?? (titleFallback || taskFallback);
    return {
      payload_type: "rich_text",
      rich_text: {
        plain_text: plainText,
      },
    };
  }

  if (payloadType === "a2ui" && asRecord(surface.a2ui)) {
    return {
      payload_type: "a2ui",
      a2ui: asRecord(surface.a2ui) as EmitHeapBlockRequest["content"]["a2ui"],
    };
  }

  return {
    payload_type: "task",
    task: taskFallback || titleFallback,
  };
}

export function buildTaskRouteLineageSnapshot(
  block: HeapBlockListItem,
  relationIndex: HeapRelationIndex,
): TaskRouteLineageSnapshot | null {
  const attributes = block.projection.attributes ?? {};
  const context = parseTaskRoutingContextFromAttributes(attributes);
  const lineage = readTaskRouteLineageAttributes(attributes);
  if (
    !context &&
    !lineage.routeId &&
    !lineage.summaryArtifactId &&
    !lineage.workflow
  ) {
    return null;
  }

  const summaryArtifact = lineage.summaryArtifactId
    ? relationIndex.outboundMentions.find((item) => item.id === lineage.summaryArtifactId)
    : null;

  return {
    task_title: context?.title ?? block.projection.title,
    source_task_artifact_id: block.projection.artifactId,
    initiative_id: context?.initiative_id ?? attributes.initiative_id ?? "unknown",
    route_id: lineage.routeId,
    route_label: lineage.routeLabel ?? (lineage.routeId ? formatTaskRouteLabel(lineage.routeId) : undefined),
    confidence_hint: lineage.confidenceHint,
    decision_mode: lineage.decisionMode ?? context?.decision_mode,
    route_summary: lineage.routeSummary,
    route_rationale: lineage.routeRationale,
    summary_artifact_id: lineage.summaryArtifactId,
    summary_artifact_title: summaryArtifact?.title,
    summary_artifact_linked: Boolean(summaryArtifact),
    workflow: lineage.workflow ?? undefined,
    reference_paths: context?.reference_paths ?? [],
  };
}

function buildRouteSummaryMessage(
  context: TaskRoutingContext,
  request: TaskRouteSummaryEmitRequest,
): string {
  const workflow = request.workflow;
  const sections = [
    `${request.decision.label}`,
    "",
    `Task: ${context.title}`,
    `Objective: ${context.objective}`,
    `Route: ${request.route_id}`,
    `Confidence: ${request.decision.confidence_hint}`,
    `Recommended role: ${request.decision.recommended_role}`,
    `Recommended capabilities: ${joinOrFallback(request.decision.recommended_capabilities, "none")}`,
    "",
    ...joinSection("Required tasks:", context.required_tasks, "- No required tasks defined."),
    "",
    ...joinSection("Bottleneck signals:", context.bottleneck_signals, "- No bottleneck signals defined."),
    "",
    ...joinSection("Error signals:", context.error_signals, "- No error signals defined."),
    "",
    ...joinSection("Fallback routes:", context.fallback_routes, "- No fallback routes defined."),
    "",
    "Reference files:",
    ...context.reference_paths.map((path) => `- ${path}`),
  ];

  if (workflow) {
    sections.push(
      "",
      "Workflow draft:",
      `- intent id: ${workflow.workflow_intent_id ?? "n/a"}`,
      `- candidate set: ${workflow.candidate_set_id ?? "n/a"}`,
      `- workflow draft: ${workflow.workflow_draft_id ?? "n/a"}`,
      `- proposal: ${workflow.proposal_id ?? "n/a"}`,
      `- definition: ${workflow.definition_id ?? "n/a"}`,
      `- scope: ${workflow.scope_key ?? "n/a"}`,
      `- motif kind: ${workflow.motif_kind ?? "n/a"}`,
      `- generation mode: ${workflow.generation_mode ?? "n/a"}`,
      ...(workflow.proposal_digest_path ? [`- proposal digest: ${workflow.proposal_digest_path}`] : []),
    );
  }

  return sections.join("\n");
}

export function inferWorkflowMotifKind(context: TaskRoutingContext): WorkflowMotifKind {
  const textParts = [
    context.objective,
    ...context.required_tasks,
    ...context.fallback_routes,
  ].join(" ").toLowerCase();

  if (textParts.includes("compare") || textParts.includes("comparison") || textParts.includes("candidate")) {
    return "parallel_compare";
  }
  if (textParts.includes("fix") || textParts.includes("repair") || textParts.includes("retry") || textParts.includes("recover")) {
    return "repair_loop";
  }
  if (textParts.includes("review") || textParts.includes("approve") || textParts.includes("ratify")) {
    return "human_gate";
  }
  if (context.required_tasks.length > 3 || textParts.includes("fan out") || textParts.includes("join")) {
    return "fan_out_join";
  }
  return "sequential";
}

export function inferWorkflowGenerationMode(
  context: TaskRoutingContext,
): WorkflowGenerationMode {
  return context.routing_options.length > 1 || context.required_tasks.length > 2
    ? "motif_hybrid"
    : "deterministic_scaffold";
}

export function buildTaskRouteSummaryEmitRequest(
  context: TaskRoutingContext,
  request: TaskRouteSummaryEmitRequest,
): EmitHeapBlockRequest {
  const workflow = request.workflow;
  const routeId = request.route_id;
  const decision = request.decision;
  const message = buildRouteSummaryMessage(context, request);
  const structuredData: Record<string, unknown> = {
    type: "agent_solicitation",
    space_id: request.space_id,
    route_id: routeId,
    initiative_id: context.initiative_id,
    source_task_artifact_id: request.source_task_artifact_id,
    title: context.title,
    objective: context.objective,
    decision_mode: context.decision_mode,
    role: decision.recommended_role,
    authority_scope: routeId === "proposal_generation" ? "L2" : "L1",
    budget: {
      currency: "cycles",
      max: routeId === "proposal_generation" ? 75000 : 50000,
    },
    required_capabilities: decision.recommended_capabilities,
    summary: decision.summary,
    rationale: decision.rationale,
    message,
    routed_at: request.routed_at ?? new Date().toISOString(),
    route_confidence: decision.confidence_hint,
    route_surface_mode: decision.surface_mode,
    reference_paths: context.reference_paths,
    bottleneck_signals: context.bottleneck_signals,
    error_signals: context.error_signals,
    fallback_routes: context.fallback_routes,
    required_tasks: context.required_tasks,
  };

  if (workflow) {
    structuredData.workflow_intent_id = workflow.workflow_intent_id;
    structuredData.candidate_set_id = workflow.candidate_set_id;
    structuredData.workflow_draft_id = workflow.workflow_draft_id;
    structuredData.proposal_id = workflow.proposal_id;
    structuredData.definition_id = workflow.definition_id;
    structuredData.scope_key = workflow.scope_key;
    structuredData.motif_kind = workflow.motif_kind;
    structuredData.generation_mode = workflow.generation_mode;
    structuredData.proposal_digest_path = workflow.proposal_digest_path;
  }

  const attributes: Record<string, string> = {
    initiative_id: context.initiative_id,
    source_task_artifact_id: request.source_task_artifact_id,
    route_id: routeId,
    routing_decision_mode: context.decision_mode,
    routing_confidence_hint: decision.confidence_hint,
    routing_surface_mode: decision.surface_mode,
    routing_role: decision.recommended_role,
    routing_capabilities: joinOrFallback(decision.recommended_capabilities, "none"),
    routing_rationale: decision.rationale,
    routing_summary: decision.summary,
  };

  if (workflow?.workflow_intent_id) {
    attributes.workflow_intent_id = workflow.workflow_intent_id;
  }
  if (workflow?.candidate_set_id) {
    attributes.workflow_candidate_set_id = workflow.candidate_set_id;
  }
  if (workflow?.workflow_draft_id) {
    attributes.workflow_draft_id = workflow.workflow_draft_id;
  }
  if (workflow?.proposal_id) {
    attributes.workflow_proposal_id = workflow.proposal_id;
  }
  if (workflow?.definition_id) {
    attributes.workflow_definition_id = workflow.definition_id;
  }
  if (workflow?.scope_key) {
    attributes.workflow_scope_key = workflow.scope_key;
  }
  if (workflow?.motif_kind) {
    attributes.workflow_motif_kind = workflow.motif_kind;
  }
  if (workflow?.generation_mode) {
    attributes.workflow_generation_mode = workflow.generation_mode;
  }
  if (workflow?.proposal_digest_path) {
    attributes.workflow_proposal_digest_path = workflow.proposal_digest_path;
  }

  return {
    schema_version: "1.0.0",
    mode: "heap",
    space_id: request.space_id,
    source: {
      agent_id: "cortex-web",
      emitted_at: request.routed_at ?? new Date().toISOString(),
    },
    block: {
      type: "agent_solicitation",
      title: `${context.title} · ${decision.label}`,
      attributes,
      behaviors: ["task"],
    },
    content: {
      payload_type: "structured_data",
      structured_data: structuredData,
    },
    relations: {
      mentions: [
        {
          to_block_id: request.source_task_artifact_id,
          label: "source task",
        },
      ],
    },
  };
}

export function buildTaskRouteSourceStampEmitRequest(
  context: TaskRoutingContext,
  request: TaskRouteSourceStampEmitRequest,
): EmitHeapBlockRequest {
  const workflow = request.workflow;
  const sourceBlock = request.source_block;
  const routedAt = request.routed_at ?? new Date().toISOString();
  const existingAttributes = sourceBlock.projection.attributes ?? {};
  const surface = asRecord(sourceBlock.surfaceJson) ?? {};
  const summaryLabel = `${request.decision.label} · route summary`;
  const content = buildTaskContentFromSurface(surface, sourceBlock.projection.title, context.objective);
  const attributes: Record<string, string> = {
    ...existingAttributes,
    initiative_id: context.initiative_id,
    route_id: request.route_id,
    task_route_id: request.route_id,
    task_route_decision_mode: context.decision_mode,
    task_route_confidence_hint: request.decision.confidence_hint,
    task_route_surface_mode: request.decision.surface_mode,
    task_route_role: request.decision.recommended_role,
    task_route_capabilities: joinOrFallback(request.decision.recommended_capabilities, "none"),
    task_route_rationale: request.decision.rationale,
    task_route_summary: request.decision.summary,
    task_route_summary_artifact_id: request.summary_artifact_id,
    task_route_routed_at: routedAt,
  };

  if (workflow?.workflow_intent_id) {
    attributes.workflow_intent_id = workflow.workflow_intent_id;
  }
  if (workflow?.candidate_set_id) {
    attributes.workflow_candidate_set_id = workflow.candidate_set_id;
  }
  if (workflow?.workflow_draft_id) {
    attributes.workflow_draft_id = workflow.workflow_draft_id;
  }
  if (workflow?.proposal_id) {
    attributes.workflow_proposal_id = workflow.proposal_id;
  }
  if (workflow?.definition_id) {
    attributes.workflow_definition_id = workflow.definition_id;
  }
  if (workflow?.scope_key) {
    attributes.workflow_scope_key = workflow.scope_key;
  }
  if (workflow?.motif_kind) {
    attributes.workflow_motif_kind = workflow.motif_kind;
  }
  if (workflow?.generation_mode) {
    attributes.workflow_generation_mode = workflow.generation_mode;
  }
  if (workflow?.proposal_digest_path) {
    attributes.workflow_proposal_digest_path = workflow.proposal_digest_path;
  }

  const mentions = [
    ...(sourceBlock.projection.mentionsInline ?? []).map((artifactId) => ({
      to_block_id: artifactId,
      label: artifactId,
    })),
    {
      to_block_id: request.summary_artifact_id,
      label: summaryLabel,
    },
  ];

  return {
    schema_version: "1.0.0",
    mode: "heap",
    space_id: sourceBlock.projection.spaceId ?? request.workflow?.scope_key?.replace(/^space:/, "") ?? "",
    source: {
      agent_id: "cortex-web",
      emitted_at: routedAt,
    },
    block: {
      id: sourceBlock.projection.artifactId,
      type: sourceBlock.projection.blockType,
      title: sourceBlock.projection.title,
      attributes,
      behaviors: ["task"],
    },
    content,
    relations: {
      tags: (sourceBlock.projection.tags ?? []).map((toBlockId) => ({
        to_block_id: toBlockId,
      })),
      mentions,
      page_links: (sourceBlock.projection.pageLinks ?? []).map((toBlockId) => ({
        to_block_id: toBlockId,
      })),
    },
    crdt_projection: {
      artifact_id: sourceBlock.projection.artifactId,
    },
  };
}

export function parseTaskRoutingContextFromAttributes(
  attributes?: Record<string, string>,
): TaskRoutingContext | null {
  const rawContext = attributes?.task_context;
  if (!rawContext?.trim()) {
    return null;
  }

  try {
    const parsed = JSON.parse(rawContext) as Record<string, unknown>;
    const version = asString(parsed.version);
    const initiativeId = asString(parsed.initiative_id);
    const title = asString(parsed.title);
    const objective = asString(parsed.objective);
    const decisionMode = asString(parsed.decision_mode);
    const agentRole = asString(parsed.agent_role);
    if (
      !version ||
      !initiativeId ||
      !title ||
      !objective ||
      !decisionMode ||
      !agentRole
    ) {
      return null;
    }

    return {
      version: "1.0.0",
      initiative_id: initiativeId,
      title,
      objective,
      decision_mode: decisionMode as TaskRoutingContext["decision_mode"],
      agent_role: agentRole,
      required_capabilities: asStringArray(parsed.required_capabilities),
      required_tasks: asStringArray(parsed.required_tasks),
      success_criteria: asStringArray(parsed.success_criteria),
      bottleneck_signals: asStringArray(parsed.bottleneck_signals),
      error_signals: asStringArray(parsed.error_signals),
      fallback_routes: asStringArray(parsed.fallback_routes),
      routing_options: Array.isArray(parsed.routing_options)
        ? parsed.routing_options
            .map((option) => asRecord(option))
            .filter((option): option is Record<string, unknown> => Boolean(option))
            .map((option) => ({
              route_id: asString(option.route_id) ?? "direct_agent_assignment",
              label: asString(option.label) ?? "Route",
              description: asString(option.description) ?? "",
              when: asString(option.when) ?? "",
              outcome: asString(option.outcome) ?? "",
              confidence_hint: (asString(option.confidence_hint) ?? "low") as
                | "high"
                | "medium"
                | "low",
              requires_human_review:
                typeof option.requires_human_review === "boolean"
                  ? option.requires_human_review
                  : undefined,
            }))
        : [],
      reference_paths: asStringArray(parsed.reference_paths),
    };
  } catch {
    return null;
  }
}

function buildDirectPrompt(context: TaskRoutingContext): string {
  return [
    `Directly assign the task to the most relevant specialist agent.`,
    "",
    `Task: ${context.title}`,
    `Objective: ${context.objective}`,
    "",
    ...joinSection(
      "Required tasks:",
      context.required_tasks,
      "- No required tasks defined.",
    ),
    "",
    ...joinSection(
      "Success criteria:",
      context.success_criteria,
      "- No success criteria defined.",
    ),
    "",
    ...joinSection(
      "Bottleneck signals:",
      context.bottleneck_signals,
      "- No bottleneck signals defined.",
    ),
    "",
    ...joinSection("Error signals:", context.error_signals, "- No error signals defined."),
    "",
    "Reference files:",
    ...context.reference_paths.map((path) => `- ${path}`),
  ].join("\n");
}

function buildProposalPrompt(context: TaskRoutingContext): string {
  return [
    `Generate a governed proposal or workflow draft before execution.`,
    "",
    `Task: ${context.title}`,
    `Objective: ${context.objective}`,
    "",
    ...joinSection(
      "Required tasks:",
      context.required_tasks,
      "- No required tasks defined.",
    ),
    "",
    ...joinSection(
      "Fallback routes:",
      context.fallback_routes,
      "- No fallback routes defined.",
    ),
    "",
    "Reference files:",
    ...context.reference_paths.map((path) => `- ${path}`),
  ].join("\n");
}

function buildStewardPrompt(context: TaskRoutingContext): string {
  return [
    `Escalate this task to a steward for review before execution.`,
    "",
    `Task: ${context.title}`,
    `Objective: ${context.objective}`,
    "",
    ...joinSection(
      "Bottleneck signals:",
      context.bottleneck_signals,
      "- No bottleneck signals defined.",
    ),
    "",
    ...joinSection("Error signals:", context.error_signals, "- No error signals defined."),
    "",
    ...joinSection(
      "Fallback routes:",
      context.fallback_routes,
      "- No fallback routes defined.",
    ),
    "",
    "Reference files:",
    ...context.reference_paths.map((path) => `- ${path}`),
  ].join("\n");
}

function buildDecision(
  context: TaskRoutingContext,
  route: TaskRouteId,
): TaskRouteDecision {
  switch (route) {
    case "proposal_generation":
      return {
        route_id: route,
        label: "Generate proposal or workflow draft",
        confidence_hint: "medium",
        surface_mode: "generate",
        recommended_role: context.agent_role,
        recommended_capabilities: context.required_capabilities,
        summary: "Route the task into a governed draft before execution.",
        rationale: "The task asks for a proposal, workflow draft, or staged comparison path.",
        prompt: buildProposalPrompt(context),
      };
    case "steward_escalation":
      return {
        route_id: route,
        label: "Escalate to steward",
        confidence_hint: "low",
        surface_mode: "chat",
        recommended_role: "agent-steward",
        recommended_capabilities: ["review", "governance", "triage"],
        summary: "Route the task to a steward because the context is risky or ambiguous.",
        rationale: "Signals indicate missing context, policy ambiguity, or an unresolved blocker.",
        prompt: buildStewardPrompt(context),
      };
    case "direct_agent_assignment":
    default:
      return {
        route_id: "direct_agent_assignment",
        label: "Direct agent assignment",
        confidence_hint: "high",
        surface_mode: "chat",
        recommended_role: context.agent_role,
        recommended_capabilities: context.required_capabilities,
        summary: "Route the task directly to the most relevant specialist agent.",
        rationale: "The task is bounded enough to hand to a specialist agent without escalation.",
        prompt: buildDirectPrompt(context),
      };
  }
}

function shouldEscalate(context: TaskRoutingContext): boolean {
  if (context.routing_options.some((option) => option.requires_human_review)) {
    return true;
  }

  return (
    anySignalMatches(context.bottleneck_signals, STEWARD_KEYWORDS) ||
    anySignalMatches(context.error_signals, STEWARD_KEYWORDS) ||
    phraseHit(context.objective, ["steward review", "human review", "requires review"]) ||
    anySignalMatches(context.fallback_routes, ["steward", "human review"])
  );
}

function shouldGenerateProposal(context: TaskRoutingContext): boolean {
  const textParts = [
    context.objective,
    ...context.required_tasks,
    ...context.fallback_routes,
  ];
  return textParts.some((part) => phraseHit(part, PROPOSAL_KEYWORDS));
}

export function decideTaskRoute(context: TaskRoutingContext): TaskRouteDecision {
  if (shouldEscalate(context)) {
    return buildDecision(context, "steward_escalation");
  }

  if (shouldGenerateProposal(context)) {
    return buildDecision(context, "proposal_generation");
  }

  return buildDecision(context, "direct_agent_assignment");
}

export function selectTaskRoute(
  context: TaskRoutingContext,
  routeId: TaskRouteId,
): TaskRouteDecision {
  return buildDecision(context, routeId);
}

export function buildTaskRoutePrompt(context: TaskRoutingContext): string {
  return decideTaskRoute(context).prompt;
}
