import type { EmitHeapBlockRequest } from "../../contracts.ts";
import { GENERATED_INITIATIVE_KICKOFF_SOURCES } from "./generatedInitiativeKickoffRegistry.ts";

export interface InitiativeKickoffTemplate {
  id: string;
  initiativeId: string;
  title: string;
  blockType: "agent_solicitation";
  body: string;
  agentRole: string;
  requiredCapabilities: string[];
  referencePaths: string[];
  routingContext: TaskRoutingContext;
  approvalSummary: string;
  approvalRationale: string;
  approvalMessage: string;
}

export interface InitiativeKickoffTemplateSpec {
  id: string;
  initiativeId: string;
  title: string;
  objective?: string;
  agentRole: string;
  requiredCapabilities: string[];
  referencePaths: string[];
  requiredTasks: string[];
  successCriteria?: string[];
  bottleneckSignals?: string[];
  errorSignals?: string[];
  fallbackRoutes?: string[];
  routingOptions?: TaskRoutingOptionSpec[];
  approvalSummary?: string;
  approvalRationale?: string;
  approvalMessage?: string;
}

export interface TaskRoutingOptionSpec {
  route_id: string;
  label: string;
  description: string;
  when: string;
  outcome: string;
  confidence_hint: "high" | "medium" | "low";
  requires_human_review?: boolean;
}

export interface TaskRoutingContext {
  version: "1.0.0";
  initiative_id: string;
  title: string;
  objective: string;
  decision_mode: "auto_if_clear_else_proposal";
  agent_role: string;
  required_capabilities: string[];
  required_tasks: string[];
  success_criteria: string[];
  bottleneck_signals: string[];
  error_signals: string[];
  fallback_routes: string[];
  routing_options: TaskRoutingOptionSpec[];
  reference_paths: string[];
}

export interface InitiativeKickoffTemplateRegistryEntry {
  template: InitiativeKickoffTemplate;
  label: string;
  description: string;
}

export interface InitiativeKickoffCatalogEntrySpec extends InitiativeKickoffTemplateSpec {
  label: string;
  description: string;
}

export interface InitiativeKickoffPlanMetadata {
  id: string;
  title: string;
  status: string;
  primarySteward: string | null;
  planPath: string;
}

export interface InitiativeKickoffSourceKickoff {
  enabled?: boolean;
  initiativeId?: string | null;
  templateId?: string | null;
  label?: string | null;
  description?: string | null;
  objective?: string | null;
  agentRole?: string | null;
  requiredCapabilities?: string[];
  referencePaths?: string[];
  requiredTasks?: string[];
  successCriteria?: string[];
  bottleneckSignals?: string[];
  errorSignals?: string[];
  fallbackRoutes?: string[];
  routingOptions?: TaskRoutingOptionSpec[];
  approvalSummary?: string | null;
  approvalRationale?: string | null;
  approvalMessage?: string | null;
}

export interface InitiativeKickoffSource {
  directory: string;
  plan?: InitiativeKickoffPlanMetadata | null;
  kickoff?: InitiativeKickoffSourceKickoff | null;
}

export interface InitiativeKickoffRegistryDiagnostic {
  directory: string;
  templateId: string | null;
  code:
    | "missing_plan"
    | "missing_kickoff"
    | "kickoff_disabled"
    | "initiative_id_mismatch"
    | "inactive_plan"
    | "missing_steward"
    | "missing_title"
    | "missing_label"
    | "missing_description"
    | "missing_agent_role"
    | "missing_required_capabilities"
    | "missing_reference_paths"
    | "missing_required_tasks"
    | "missing_plan_path";
  message: string;
}

export interface ResolvedInitiativeKickoffRegistry {
  entries: InitiativeKickoffTemplateRegistryEntry[];
  diagnostics: InitiativeKickoffRegistryDiagnostic[];
}

export function buildInitiativeKickoffTemplate(
  spec: InitiativeKickoffTemplateSpec,
): InitiativeKickoffTemplate {
  const routingContext = buildTaskRoutingContext(spec);
  return {
    id: spec.id,
    initiativeId: spec.initiativeId,
    title: spec.title,
    blockType: "agent_solicitation",
    agentRole: spec.agentRole,
    requiredCapabilities: spec.requiredCapabilities,
    referencePaths: spec.referencePaths,
    body: buildGuidedTaskBody(routingContext),
    routingContext,
    approvalSummary: buildInitiativeKickoffApprovalSummary(spec),
    approvalRationale: buildInitiativeKickoffApprovalRationale(spec),
    approvalMessage: buildInitiativeKickoffApprovalMessage(spec),
  };
}

export function buildTaskRoutingContext(
  spec: InitiativeKickoffTemplateSpec,
): TaskRoutingContext {
  const objective = spec.objective?.trim() || `Complete the ${spec.title} kickoff with explicit routing context.`;
  const routingOptions = spec.routingOptions ?? buildDefaultRoutingOptions(spec);
  return {
    version: "1.0.0",
    initiative_id: spec.initiativeId,
    title: spec.title,
    objective,
    decision_mode: "auto_if_clear_else_proposal",
    agent_role: spec.agentRole,
    required_capabilities: spec.requiredCapabilities,
    required_tasks: spec.requiredTasks,
    success_criteria: spec.successCriteria ?? [],
    bottleneck_signals: spec.bottleneckSignals ?? [],
    error_signals: spec.errorSignals ?? [],
    fallback_routes: spec.fallbackRoutes ?? [],
    routing_options: routingOptions,
    reference_paths: spec.referencePaths,
  };
}

function buildDefaultRoutingOptions(
  spec: InitiativeKickoffTemplateSpec,
): TaskRoutingOptionSpec[] {
  const sharedDescription = `Route the ${spec.title} kickoff using the initiative context, bottleneck signals, and fallback routes.`;
  return [
    {
      route_id: "direct_agent_assignment",
      label: "Direct agent assignment",
      description: "Assign the task to the most relevant specialist agent when the scope is bounded and dependencies are clear.",
      when: "The work is narrow, the registry story is stable, and no governance conflict is present.",
      outcome: "Start the selected agent run with the kickoff context.",
      confidence_hint: "high",
    },
    {
      route_id: "proposal_generation",
      label: "Generate proposal or workflow draft",
      description: "Turn the kickoff into a governed proposal when the work needs staging, comparison, or a draft artifact first.",
      when: "Multiple implementation paths exist or the task crosses initiative boundaries.",
      outcome: "Emit a proposal or workflow draft before execution.",
      confidence_hint: "medium",
    },
    {
      route_id: "steward_escalation",
      label: "Escalate to steward",
      description: sharedDescription,
      when: "The kickoff reveals policy ambiguity, missing context, or a bottleneck that blocks safe routing.",
      outcome: "Route to a human steward for review before any agent run starts.",
      confidence_hint: "low",
      requires_human_review: true,
    },
  ];
}

function buildInitiativeKickoffApprovalSummary(
  spec: InitiativeKickoffTemplateSpec,
): string {
  return spec.approvalSummary?.trim() ||
    `Steward-backed kickoff approval for initiative ${spec.initiativeId} before any live kickoff task is emitted.`;
}

function buildInitiativeKickoffApprovalRationale(
  spec: InitiativeKickoffTemplateSpec,
): string {
  return spec.approvalRationale?.trim() ||
    [
      "This kickoff should not bypass stewardship.",
      "Approval confirms the initiative package, routing context, reference files, and execution guardrails are in place before work begins.",
    ].join(" ");
}

function buildInitiativeKickoffApprovalMessage(
  spec: InitiativeKickoffTemplateSpec,
): string {
  return spec.approvalMessage?.trim() ||
    `Review the ${spec.title} package and record steward approval before any live kickoff task is emitted.`;
}

function formatTaskSection(title: string, entries: readonly string[], emptyLabel: string): string[] {
  if (entries.length === 0) {
    return [title, emptyLabel];
  }
  return [title, ...entries.map((entry) => `- ${entry}`)];
}

function formatRoutingOptions(routingOptions: readonly TaskRoutingOptionSpec[]): string[] {
  if (routingOptions.length === 0) {
    return ["Routing options:", "- No routing options defined."];
  }

  return [
    "Routing options:",
    ...routingOptions.flatMap((option) => [
      `- ${option.label} (${option.route_id}, confidence: ${option.confidence_hint})`,
      `  - When: ${option.when}`,
      `  - Outcome: ${option.outcome}`,
      `  - Description: ${option.description}`,
      ...(option.requires_human_review ? ["  - Requires human review: yes"] : []),
    ]),
  ];
}

export function buildGuidedTaskBody(routingContext: TaskRoutingContext): string {
  return [
    `# ${routingContext.title}`,
    "",
    "Objective:",
    routingContext.objective,
    "",
    ...formatTaskSection("Agent required tasks:", routingContext.required_tasks.map((task) => `[ ] ${task}`), "- No required tasks defined."),
    "",
    ...formatTaskSection("Success criteria:", routingContext.success_criteria, "- No success criteria defined."),
    "",
    ...formatTaskSection("Bottleneck signals:", routingContext.bottleneck_signals, "- No bottleneck signals defined."),
    "",
    ...formatTaskSection("Error signals:", routingContext.error_signals, "- No error signals defined."),
    "",
    ...formatTaskSection("Fallback routes:", routingContext.fallback_routes, "- No fallback routes defined."),
    "",
    ...formatRoutingOptions(routingContext.routing_options),
    "",
    `Suggested agent role: ${routingContext.agent_role}`,
    `Suggested capabilities: ${routingContext.required_capabilities.join(", ")}`,
    "",
    "Reference files:",
    ...routingContext.reference_paths.map((path) => `- ${path}`),
  ].join("\n");
}

export function buildInitiativeKickoffCatalog(
  specs: InitiativeKickoffCatalogEntrySpec[],
): InitiativeKickoffTemplateRegistryEntry[] {
  return specs.map((spec) => ({
    template: buildInitiativeKickoffTemplate(spec),
    label: spec.label,
    description: spec.description,
  }));
}

function normalizedString(value: string | null | undefined): string | null {
  const trimmed = value?.trim();
  return trimmed ? trimmed : null;
}

function normalizedStringArray(values: readonly string[] | null | undefined): string[] {
  if (!Array.isArray(values)) {
    return [];
  }
  return values
    .map((value) => normalizedString(value))
    .filter((value): value is string => Boolean(value));
}

function buildKickoffDiagnostic(
  source: InitiativeKickoffSource,
  code: InitiativeKickoffRegistryDiagnostic["code"],
  message: string,
): InitiativeKickoffRegistryDiagnostic {
  return {
    directory: source.directory,
    templateId: normalizedString(source.kickoff?.templateId) ?? null,
    code,
    message,
  };
}

function isEligibleKickoffStatus(status: string | null | undefined): boolean {
  return normalizedString(status)?.toLowerCase() === "active";
}

export function buildInitiativeKickoffCatalogSpec(
  source: InitiativeKickoffSource,
): { spec: InitiativeKickoffCatalogEntrySpec | null; diagnostics: InitiativeKickoffRegistryDiagnostic[] } {
  const diagnostics: InitiativeKickoffRegistryDiagnostic[] = [];
  const plan = source.plan ?? null;
  const kickoff = source.kickoff ?? null;

  if (!plan) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_plan", "Kickoff metadata exists without a matching PLAN.md source."));
    return { spec: null, diagnostics };
  }
  if (!kickoff) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_kickoff", "Initiative has no kickoff metadata and is not launchable."));
    return { spec: null, diagnostics };
  }
  if (kickoff.enabled === false) {
    diagnostics.push(buildKickoffDiagnostic(source, "kickoff_disabled", "Kickoff metadata is present but disabled."));
    return { spec: null, diagnostics };
  }
  const kickoffInitiativeId = normalizedString(kickoff.initiativeId);
  if (kickoffInitiativeId && kickoffInitiativeId != plan.id.trim()) {
    diagnostics.push(buildKickoffDiagnostic(source, "initiative_id_mismatch", `Kickoff initiative id '${kickoffInitiativeId}' does not match PLAN.md id '${plan.id.trim()}'.`));
    return { spec: null, diagnostics };
  }
  if (!isEligibleKickoffStatus(plan.status)) {
    diagnostics.push(buildKickoffDiagnostic(source, "inactive_plan", `Initiative status '${plan.status}' is not eligible for kickoff.`));
    return { spec: null, diagnostics };
  }
  if (!normalizedString(plan.primarySteward)) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_steward", "Initiative PLAN.md is missing stewardship.primary_steward."));
  }
  if (!normalizedString(plan.title)) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_title", "Initiative PLAN.md is missing a title."));
  }
  if (!normalizedString(plan.planPath)) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_plan_path", "Initiative PLAN.md source path is missing."));
  }
  if (!normalizedString(kickoff.label)) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_label", "Kickoff metadata is missing a catalog label."));
  }
  if (!normalizedString(kickoff.description)) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_description", "Kickoff metadata is missing a catalog description."));
  }
  if (!normalizedString(kickoff.agentRole)) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_agent_role", "Kickoff metadata is missing agent_role."));
  }

  const requiredCapabilities = normalizedStringArray(kickoff.requiredCapabilities);
  if (requiredCapabilities.length === 0) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_required_capabilities", "Kickoff metadata must declare required_capabilities."));
  }

  const referencePaths = normalizedStringArray(kickoff.referencePaths);
  const planPath = normalizedString(plan.planPath);
  if (planPath && !referencePaths.includes(planPath)) {
    referencePaths.unshift(planPath);
  }
  if (referencePaths.length === 0) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_reference_paths", "Kickoff metadata must declare reference_paths."));
  }

  const requiredTasks = normalizedStringArray(kickoff.requiredTasks);
  if (requiredTasks.length === 0) {
    diagnostics.push(buildKickoffDiagnostic(source, "missing_required_tasks", "Kickoff metadata must declare required_tasks."));
  }

  if (diagnostics.length > 0) {
    return { spec: null, diagnostics };
  }

  const initiativeId = plan.id.trim();
  return {
    spec: {
      id: normalizedString(kickoff.templateId) ?? `initiative-${initiativeId}-kickoff`,
      initiativeId,
      title: plan.title.trim(),
      objective: normalizedString(kickoff.objective) ?? undefined,
      agentRole: kickoff.agentRole!.trim(),
      requiredCapabilities,
      referencePaths,
      requiredTasks,
      successCriteria: normalizedStringArray(kickoff.successCriteria),
      bottleneckSignals: normalizedStringArray(kickoff.bottleneckSignals),
      errorSignals: normalizedStringArray(kickoff.errorSignals),
      fallbackRoutes: normalizedStringArray(kickoff.fallbackRoutes),
      routingOptions: kickoff.routingOptions && kickoff.routingOptions.length > 0
        ? kickoff.routingOptions
        : undefined,
      approvalSummary: normalizedString(kickoff.approvalSummary) ?? undefined,
      approvalRationale: normalizedString(kickoff.approvalRationale) ?? undefined,
      approvalMessage: normalizedString(kickoff.approvalMessage) ?? undefined,
      label: kickoff.label!.trim(),
      description: kickoff.description!.trim(),
    },
    diagnostics,
  };
}

export function resolveInitiativeKickoffRegistry(
  sources: readonly InitiativeKickoffSource[],
): ResolvedInitiativeKickoffRegistry {
  const entries: InitiativeKickoffTemplateRegistryEntry[] = [];
  const diagnostics: InitiativeKickoffRegistryDiagnostic[] = [];

  for (const source of sources) {
    const resolved = buildInitiativeKickoffCatalogSpec(source);
    diagnostics.push(...resolved.diagnostics);
    if (resolved.spec) {
      entries.push(...buildInitiativeKickoffCatalog([resolved.spec]));
    }
  }

  return { entries, diagnostics };
}

export function filterInitiativeKickoffTemplates(
  entries: InitiativeKickoffTemplateRegistryEntry[],
  dismissedTemplateIds: readonly string[],
): InitiativeKickoffTemplateRegistryEntry[] {
  if (dismissedTemplateIds.length === 0) {
    return entries;
  }
  const dismissed = new Set(dismissedTemplateIds);
  return entries.filter((entry) => !dismissed.has(entry.template.id));
}

export function buildInitiativeKickoffEmitRequest(
  template: InitiativeKickoffTemplate,
  spaceId: string,
  emittedAt = new Date().toISOString(),
): EmitHeapBlockRequest {
  const approvalTitle = `${template.title} Approval`;
  return {
    schema_version: "1.0.0",
    mode: "heap",
    space_id: spaceId,
    source: {
      agent_id: "cortex-web",
      emitted_at: emittedAt,
    },
    block: {
      type: "agent_solicitation",
      title: approvalTitle,
      attributes: {
        initiative_id: template.initiativeId,
        initiative_kickoff_template_id: template.id,
        requested_agent_role: template.agentRole,
        required_capabilities: template.requiredCapabilities.join(", "),
        reference_paths: template.referencePaths.join(", "),
        task_context_version: template.routingContext.version,
        task_context: JSON.stringify(template.routingContext),
        routing_decision_mode: template.routingContext.decision_mode,
        approval_kind: "initiative_kickoff",
        approval_required: "steward",
        review_lane: "private_review",
      },
      behaviors: ["awaiting_approval"],
    },
    content: {
      payload_type: "structured_data",
      structured_data: {
        type: "agent_solicitation",
        solicitation_kind: "initiative_kickoff_approval",
        initiative_id: template.initiativeId,
        initiative_kickoff_template_id: template.id,
        title: template.title,
        role: template.agentRole,
        authority_scope: "steward_gate",
        required_capabilities: template.requiredCapabilities,
        summary: template.approvalSummary,
        rationale: template.approvalRationale,
        message: template.approvalMessage,
        description: template.body,
        reference_paths: template.referencePaths,
        required_tasks: template.routingContext.required_tasks,
        success_criteria: template.routingContext.success_criteria,
        bottleneck_signals: template.routingContext.bottleneck_signals,
        error_signals: template.routingContext.error_signals,
        fallback_routes: template.routingContext.fallback_routes,
        routing_options: template.routingContext.routing_options,
        decision_mode: template.routingContext.decision_mode,
        requested_agent_role: template.agentRole,
      },
    },
  };
}

const RESOLVED_INITIATIVE_KICKOFF_REGISTRY = resolveInitiativeKickoffRegistry(
  GENERATED_INITIATIVE_KICKOFF_SOURCES,
);

export const INITIATIVE_KICKOFF_TEMPLATES: InitiativeKickoffTemplateRegistryEntry[] =
  RESOLVED_INITIATIVE_KICKOFF_REGISTRY.entries;

export const INITIATIVE_KICKOFF_REGISTRY_DIAGNOSTICS: InitiativeKickoffRegistryDiagnostic[] =
  RESOLVED_INITIATIVE_KICKOFF_REGISTRY.diagnostics;

export function buildInitiative078KickoffTemplate(): InitiativeKickoffTemplate {
  const template = resolveInitiativeKickoffTemplate("initiative-078-kickoff");
  if (!template) {
    throw new Error("Initiative 078 kickoff template is not registered.");
  }
  return template;
}

export function canLaunchInitiativeKickoff(role: string | null | undefined): boolean {
  const normalizedRole = role?.trim().toLowerCase();
  return normalizedRole === "operator" || normalizedRole === "steward" || normalizedRole === "admin";
}

export function resolveInitiativeKickoffTemplate(
  templateId: string,
): InitiativeKickoffTemplate | null {
  return (
    INITIATIVE_KICKOFF_TEMPLATES.find((entry) => entry.template.id === templateId)?.template ??
    null
  );
}
