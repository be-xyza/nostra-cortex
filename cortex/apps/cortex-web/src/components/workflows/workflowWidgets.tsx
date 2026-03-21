import React from "react";
import { useNavigate } from "react-router-dom";

import { openGatewayApiArtifact } from "../../api";
import { classifyWorkbenchHref, normalizeWorkflowHref } from "./artifactRouting.ts";
import { useWorkflowArtifactInspector } from "./WorkflowArtifactInspectorContext.tsx";
import {
  projectWorkflowInstanceTimeline,
  projectWorkflowProjectionPreview,
  projectWorkflowStatusBadge,
  projectWorkflowSummaryStrip,
} from "./workflowWidgetProjection.ts";

function toneClass(tone: string): string {
  switch (tone) {
    case "warning":
      return "border-cortex-warn/50 text-cortex-warn";
    case "error":
      return "border-cortex-bad/50 text-cortex-bad";
    case "success":
      return "border-cortex-ok/50 text-cortex-ok";
    default:
      return "border-cortex-line text-cortex-ink";
  }
}

function useWorkflowWidgetLinkAction() {
  const navigate = useNavigate();
  const inspector = useWorkflowArtifactInspector();

  return React.useCallback(
    async (href?: string) => {
      if (!href) return;
      const kind = classifyWorkbenchHref(href);
      if (kind === "gateway_api") {
        if (inspector) {
          await inspector.openArtifact(href);
          return;
        }
        await openGatewayApiArtifact(href, "new_tab");
        return;
      }
      if (kind === "internal_workbench") {
        navigate(normalizeWorkflowHref(href));
        return;
      }
      if (typeof window !== "undefined") {
        window.location.assign(href);
      }
    },
    [inspector, navigate]
  );
}

export function WorkflowSummaryStrip({
  componentProperties,
}: {
  componentProperties: Record<string, unknown>;
}) {
  const projection = projectWorkflowSummaryStrip(componentProperties);
  const openHref = useWorkflowWidgetLinkAction();
  return (
    <section className="rounded-[22px] border border-cortex-line bg-[linear-gradient(135deg,rgba(11,29,54,0.94),rgba(6,18,36,0.96))] p-5 shadow-[0_18px_48px_rgba(2,8,20,0.32)]">
      <div className="flex flex-col gap-1">
        <div className="text-[11px] uppercase tracking-[0.24em] text-cortex-ink-faint">
          {projection.eyebrow}
        </div>
        <h3 className="text-xl font-semibold text-cortex-ink">{projection.title}</h3>
        <p className="text-sm text-cortex-ink-muted">{projection.description}</p>
      </div>
      <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        {projection.metrics.map((metric) => (
          <button
            key={metric.label}
            type="button"
            disabled={!metric.href}
            onClick={() => void openHref(metric.href)}
            className={`rounded-[18px] border bg-cortex-bg/80 px-4 py-3 ${toneClass(metric.tone || "default")}`}
          >
            <div className="text-[11px] uppercase tracking-[0.18em] text-cortex-ink-faint">
              {metric.label}
            </div>
            <div className="mt-2 text-2xl font-semibold text-cortex-ink">
              {metric.value}
            </div>
          </button>
        ))}
      </div>
    </section>
  );
}

export function WorkflowStatusBadge({
  componentProperties,
}: {
  componentProperties: Record<string, unknown>;
}) {
  const badge = projectWorkflowStatusBadge(componentProperties);
  const openHref = useWorkflowWidgetLinkAction();
  const content = (
    <>
      <span className="text-cortex-ink-faint">{badge.label}</span>
      <span className="font-semibold text-cortex-ink">{badge.status}</span>
    </>
  );

  if (badge.href) {
    return (
      <button
        type="button"
        onClick={() => void openHref(badge.href)}
        className={`inline-flex items-center gap-2 rounded-full border bg-cortex-bg px-3 py-2 text-xs uppercase tracking-[0.18em] ${toneClass(
          badge.emphasis
        )}`}
      >
        {content}
      </button>
    );
  }

  return (
    <div
      className={`inline-flex items-center gap-2 rounded-full border bg-cortex-bg px-3 py-2 text-xs uppercase tracking-[0.18em] ${toneClass(
        badge.emphasis
      )}`}
    >
      {content}
    </div>
  );
}

export function WorkflowProjectionPreview({
  componentProperties,
}: {
  componentProperties: Record<string, unknown>;
}) {
  const projection = projectWorkflowProjectionPreview(componentProperties);
  const openHref = useWorkflowWidgetLinkAction();
  return (
    <section className="rounded-[22px] border border-cortex-line bg-cortex-bg-panel p-4 shadow-[0_14px_36px_rgba(2,8,20,0.24)]">
      <div className="flex flex-col gap-1">
        <div className="text-[11px] uppercase tracking-[0.2em] text-cortex-ink-faint">
          {projection.eyebrow}
        </div>
        {projection.definitionHref ? (
          <button
            type="button"
            onClick={() => void openHref(projection.definitionHref)}
            className="w-fit text-left text-base font-semibold text-cortex-ink hover:text-cortex-accent"
          >
            {projection.definitionId}
          </button>
        ) : (
          <div className="text-base font-semibold text-cortex-ink">
            {projection.definitionId}
          </div>
        )}
        <div className="text-sm text-cortex-ink-muted">
          motif={projection.motif} · nodes={projection.nodeCount}
        </div>
        <div className="text-xs text-cortex-ink-faint">digest={projection.digest}</div>
      </div>
      <div className="mt-4 flex flex-wrap gap-2">
        {projection.projections.map((item) => (
          <button
            key={`${item.kind}-${item.label}`}
            type="button"
            disabled={!item.href}
            onClick={() => void openHref(item.href)}
            className="rounded-full border border-cortex-line bg-cortex-bg px-3 py-2 text-xs uppercase tracking-[0.18em] text-cortex-ink-muted disabled:cursor-default disabled:opacity-100"
          >
            {item.label}
          </button>
        ))}
      </div>
    </section>
  );
}

export function WorkflowInstanceTimeline({
  componentProperties,
}: {
  componentProperties: Record<string, unknown>;
}) {
  const projection = projectWorkflowInstanceTimeline(componentProperties);
  const openHref = useWorkflowWidgetLinkAction();
  return (
    <section className="rounded-[22px] border border-cortex-line bg-cortex-bg-panel p-4 shadow-[0_14px_36px_rgba(2,8,20,0.24)]">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-[11px] uppercase tracking-[0.2em] text-cortex-ink-faint">
            {projection.eyebrow}
          </div>
          <h3 className="text-base font-semibold text-cortex-ink">{projection.title}</h3>
        </div>
        <div className="text-xs text-cortex-ink-muted">
          {projection.entries.length} tracked
        </div>
      </div>
      <div className="mt-4 flex flex-col gap-3">
        {projection.entries.map((entry) => (
          <button
            key={entry.instanceId}
            type="button"
            disabled={!entry.href}
            onClick={() => void openHref(entry.href)}
            className="rounded-[18px] border border-cortex-line bg-cortex-bg px-4 py-3 text-left disabled:cursor-default disabled:opacity-100"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="text-sm font-medium text-cortex-ink">{entry.instanceId}</div>
              <div className="text-[11px] uppercase tracking-[0.18em] text-cortex-ink-faint">
                {entry.updatedAt}
              </div>
            </div>
            <div className="mt-2 flex flex-wrap gap-2 text-xs uppercase tracking-[0.16em] text-cortex-ink-muted">
              <span>{entry.status}</span>
              <span>checkpoints={entry.checkpoints}</span>
              <span>outcome={entry.outcome}</span>
            </div>
          </button>
        ))}
      </div>
    </section>
  );
}
