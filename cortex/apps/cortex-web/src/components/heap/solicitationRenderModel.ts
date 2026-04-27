export interface SolicitationRenderModel {
  roleLabel: string;
  requestedRoleLabel: string | null;
  authorityScopeLabel: string;
  budgetLabel: string | null;
  capabilityLabels: string[];
  summary: string;
  requestedActionLabel: string | null;
  feedbackHint: string;
  reviewOutcomeMode: string;
  reviewOutcomeLabel: string;
  kindLabel: string;
  sourceRefLabels: string[];
  uncertaintyLabels: string[];
  recommendationLabels: string[];
}

export interface StewardFeedbackRenderModel {
  artifactId: string | null;
  parentArtifactId: string | null;
  decisionValue: string;
  decisionLabel: string;
  decisionTone: "approved" | "rejected" | "neutral";
  summary: string;
  submittedBy: string;
  submittedAt: string | null;
}

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
  const normalized = value.trim();
  return normalized.length ? normalized : null;
}

function asStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .map((entry) => asString(entry))
    .filter((entry): entry is string => Boolean(entry));
}

function formatBudgetLabel(value: unknown): string | null {
  const budget = asRecord(value);
  if (!budget) {
    return null;
  }

  const currency = asString(budget.currency);
  const max = typeof budget.max === "number" && Number.isFinite(budget.max)
    ? budget.max
    : typeof budget.max === "string" && budget.max.trim()
      ? Number.parseFloat(budget.max)
      : null;

  if (!currency || max === null || Number.isNaN(max)) {
    return null;
  }

  const normalizedAmount = Number.isInteger(max) ? String(max) : max.toFixed(2);
  return `${currency} ${normalizedAmount}`;
}

function humanizeToken(value: string): string {
  return value
    .split(/[_\-.]+/g)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatSolicitationKind(value: unknown): string {
  const kind = asString(value);
  if (kind === "hermes_advisory_review") {
    return "Hermes Advisory Review";
  }
  if (!kind) {
    return "Review Request";
  }
  return humanizeToken(kind);
}

function resolveReviewOutcomeMode(data: Record<string, unknown>): string {
  const explicit = asString(data.review_outcome_mode);
  if (explicit) {
    return explicit;
  }
  if (
    asString(data.solicitation_kind) === "initiative_kickoff_approval"
    || asString(data.approval_kind) === "initiative_kickoff"
  ) {
    return "emit_task";
  }
  return "record_only";
}

function reviewOutcomeLabel(mode: string): string {
  if (mode === "emit_task") {
    return "Approval creates the next task";
  }
  if (mode === "emit_proposal") {
    return "Approval creates a proposal";
  }
  if (mode === "signal_run") {
    return "Approval signals a live run";
  }
  return "Approval stores steward feedback";
}

function reviewFeedbackHint(data: Record<string, unknown>, mode: string): string {
  if (mode === "emit_task") {
    return "Approving creates the kickoff task. Rejecting stores steward feedback only.";
  }
  if (mode === "signal_run") {
    return "Approving signals the waiting run. Rejecting records the decision and keeps execution blocked.";
  }
  if (asString(data.solicitation_kind) === "hermes_advisory_review") {
    return "Approving or rejecting stores steward feedback on this artifact. It does not grant Hermes execution authority.";
  }
  return "Approving or rejecting stores steward feedback on this artifact. Any separate execution step remains explicit.";
}

function summarizeSolicitation(data: Record<string, unknown>): string {
  return (
    asString(data.summary) ??
    asString(data.description) ??
    asString(data.rationale) ??
    asString(data.message) ??
    "Review the proposal details and record steward feedback before any separate execution step."
  );
}

function normalizeDecision(value: unknown): {
  decisionValue: string;
  decisionLabel: string;
  decisionTone: StewardFeedbackRenderModel["decisionTone"];
} {
  const normalized = (asString(value) ?? "recorded").toLowerCase();

  if (normalized === "approved" || normalized === "approve") {
    return {
      decisionValue: "approved",
      decisionLabel: "Approved",
      decisionTone: "approved",
    };
  }

  if (normalized === "rejected" || normalized === "reject") {
    return {
      decisionValue: "rejected",
      decisionLabel: "Rejected",
      decisionTone: "rejected",
    };
  }

  return {
    decisionValue: normalized,
    decisionLabel: normalized.charAt(0).toUpperCase() + normalized.slice(1),
    decisionTone: "neutral",
  };
}

export function buildSolicitationRenderModel(
  value: unknown,
): SolicitationRenderModel | null {
  const data = asRecord(value);
  if (!data) {
    return null;
  }

  if (asString(data.type) !== "agent_solicitation") {
    return null;
  }

  const outcomeMode = resolveReviewOutcomeMode(data);

  return {
    roleLabel: asString(data.role) ?? "unspecified",
    requestedRoleLabel: asString(data.requested_agent_role),
    authorityScopeLabel: asString(data.authority_scope) ?? "unspecified",
    budgetLabel: formatBudgetLabel(data.budget),
    capabilityLabels: asStringArray(data.required_capabilities),
    summary: summarizeSolicitation(data),
    requestedActionLabel:
      asString(data.requested_action) ??
      asString(data.recommendation) ??
      asString(data.message),
    reviewOutcomeMode: outcomeMode,
    reviewOutcomeLabel: reviewOutcomeLabel(outcomeMode),
    kindLabel: formatSolicitationKind(data.solicitation_kind),
    sourceRefLabels: asStringArray(data.source_refs),
    uncertaintyLabels: asStringArray(data.uncertainties),
    recommendationLabels: [
      ...asStringArray(data.recommendations),
      ...asStringArray(data.next_steps),
    ],
    feedbackHint: reviewFeedbackHint(data, outcomeMode),
  };
}

export function buildStewardFeedbackRenderModel(
  value: unknown,
): StewardFeedbackRenderModel | null {
  const data = asRecord(value);
  if (!data) {
    return null;
  }

  if (asString(data.type) !== "steward_feedback") {
    return null;
  }

  const decision = normalizeDecision(data.decision);
  return {
    artifactId: asString(data.artifact_id),
    parentArtifactId: asString(data.parent_artifact_id),
    decisionValue: decision.decisionValue,
    decisionLabel: decision.decisionLabel,
    decisionTone: decision.decisionTone,
    summary:
      asString(data.feedback) ??
      asString(data.message) ??
      asString(data.notes) ??
      "Steward feedback was recorded for this block.",
    submittedBy: asString(data.submitted_by) ?? "unknown",
    submittedAt: asString(data.submitted_at),
  };
}
