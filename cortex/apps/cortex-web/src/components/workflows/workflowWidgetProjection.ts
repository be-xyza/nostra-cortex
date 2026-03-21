function readProps(
  componentProperties: Record<string, unknown>,
  componentType: string
): Record<string, unknown> {
  const typed = componentProperties[componentType];
  if (typed && typeof typed === "object" && !Array.isArray(typed)) {
    return typed as Record<string, unknown>;
  }
  return componentProperties;
}

type WorkflowMetric = {
  label: string;
  value: string;
  tone?: string;
  href?: string;
};

type WorkflowInstanceTimelineEntry = {
  instanceId: string;
  status: string;
  updatedAt: string;
  checkpoints: string;
  outcome: string;
  href?: string;
};

type WorkflowProjectionPreviewProjection = {
  label: string;
  kind: string;
  href?: string;
};

function readOptionalString(
  value: Record<string, unknown>,
  key: string
): string | undefined {
  const candidate = value[key];
  return typeof candidate === "string" && candidate.trim().length > 0
    ? candidate
    : undefined;
}

function asRecordArray(value: unknown): Array<Record<string, unknown>> {
  if (!Array.isArray(value)) return [];
  return value.filter((entry) => entry && typeof entry === "object") as Array<
    Record<string, unknown>
  >;
}

function readString(
  value: Record<string, unknown>,
  key: string,
  fallback = "-"
): string {
  const candidate = value[key];
  return typeof candidate === "string" && candidate.trim().length > 0
    ? candidate
    : fallback;
}

export function projectWorkflowSummaryStrip(
  componentProperties: Record<string, unknown>
): {
  eyebrow: string;
  title: string;
  description: string;
  metrics: WorkflowMetric[];
} {
  const props = readProps(componentProperties, "WorkflowSummaryStrip");
  return {
    eyebrow:
      (typeof props.eyebrow === "string" && props.eyebrow.trim()) ||
      "Workflow Summary",
    title:
      (typeof props.title === "string" && props.title.trim()) || "Workflow Cockpit Summary",
    description:
      (typeof props.description === "string" && props.description.trim()) ||
      "Live workflow governance and runtime posture.",
    metrics: asRecordArray(props.metrics).map((metric) => ({
      label: readString(metric, "label"),
      value: readString(metric, "value"),
      tone: readString(metric, "tone", "default"),
      href: readOptionalString(metric, "href"),
    })),
  };
}

export function projectWorkflowStatusBadge(
  componentProperties: Record<string, unknown>
): { label: string; status: string; emphasis: string; href?: string } {
  const props = readProps(componentProperties, "WorkflowStatusBadge");
  return {
    label:
      (typeof props.label === "string" && props.label.trim()) || "Workflow Status",
    status:
      (typeof props.status === "string" && props.status.trim()) || "unknown",
    emphasis:
      (typeof props.emphasis === "string" && props.emphasis.trim()) || "default",
    href: readOptionalString(props as Record<string, unknown>, "href"),
  };
}

export function projectWorkflowProjectionPreview(
  componentProperties: Record<string, unknown>
): {
  eyebrow: string;
  definitionId: string;
  definitionHref?: string;
  motif: string;
  digest: string;
  nodeCount: string;
  projections: WorkflowProjectionPreviewProjection[];
} {
  const props = readProps(componentProperties, "WorkflowProjectionPreview");
  return {
    eyebrow:
      (typeof props.eyebrow === "string" && props.eyebrow.trim()) ||
      "Definition Preview",
    definitionId: readString(props as Record<string, unknown>, "definitionId"),
    definitionHref: readOptionalString(
      props as Record<string, unknown>,
      "definitionHref"
    ),
    motif: readString(props as Record<string, unknown>, "motif"),
    digest: readString(props as Record<string, unknown>, "digest"),
    nodeCount: readString(props as Record<string, unknown>, "nodeCount"),
    projections: asRecordArray(props.projections).map((projection) => ({
      label: readString(projection, "label"),
      kind: readString(projection, "kind"),
      href:
        typeof projection.href === "string" && projection.href.trim().length > 0
          ? projection.href
          : undefined,
    })),
  };
}

export function projectWorkflowInstanceTimeline(
  componentProperties: Record<string, unknown>
): {
  eyebrow: string;
  title: string;
  entries: WorkflowInstanceTimelineEntry[];
} {
  const props = readProps(componentProperties, "WorkflowInstanceTimeline");
  return {
    eyebrow:
      (typeof props.eyebrow === "string" && props.eyebrow.trim()) ||
      "Runtime Timeline",
    title:
      (typeof props.title === "string" && props.title.trim()) || "Workflow Instances",
    entries: asRecordArray(props.entries).map((entry) => ({
      instanceId: readString(entry, "instanceId"),
      status: readString(entry, "status"),
      updatedAt: readString(entry, "updatedAt"),
      checkpoints: readString(entry, "checkpoints"),
      outcome: readString(entry, "outcome"),
      href: readOptionalString(entry, "href"),
    })),
  };
}
