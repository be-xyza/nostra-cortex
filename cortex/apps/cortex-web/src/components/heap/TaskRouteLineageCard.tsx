import React from "react";
import { Link2, Workflow } from "lucide-react";
import type { TaskRouteLineageSnapshot } from "./taskRouting.ts";

interface TaskRouteLineageCardProps {
  lineage: TaskRouteLineageSnapshot | null;
  onOpenArtifact?: (artifactId: string) => void;
}

function Chip({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <span className="rounded-full border border-white/5 bg-white/[0.03] px-2.5 py-1 text-[10px] font-semibold uppercase tracking-widest text-slate-300">
      {children}
    </span>
  );
}

function ValueRow({
  label,
  value,
  onClick,
}: {
  label: string;
  value?: string | null;
  onClick?: () => void;
}) {
  if (!value) {
    return null;
  }

  return (
    <div className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-white/5 bg-slate-950/40 px-3 py-2">
      <div className="text-[10px] font-black uppercase tracking-widest text-slate-500">
        {label}
      </div>
      {onClick ? (
        <button
          type="button"
          onClick={onClick}
          className="max-w-full rounded-full border border-cyan-400/20 bg-cyan-500/10 px-2.5 py-1 text-[10px] font-semibold text-cyan-100 transition hover:bg-cyan-500/20"
        >
          {value}
        </button>
      ) : (
        <div className="max-w-full truncate text-xs text-slate-200">{value}</div>
      )}
    </div>
  );
}

export function TaskRouteLineageCard({
  lineage,
  onOpenArtifact,
}: TaskRouteLineageCardProps) {
  if (!lineage) {
    return null;
  }

  return (
    <section className="rounded-2xl border border-violet-400/10 bg-violet-500/5 p-5 shadow-inner ring-1 ring-violet-500/5">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Link2 className="h-4 w-4 text-violet-300" />
            <h4 className="text-[10px] font-black uppercase tracking-[0.28em] text-violet-300/85">
              Route lineage
            </h4>
          </div>
          <div className="mt-2 text-sm font-semibold text-slate-100">
            {lineage.task_title}
          </div>
          {lineage.route_summary && (
            <div className="mt-1 text-xs leading-5 text-slate-400">
              {lineage.route_summary}
            </div>
          )}
        </div>
        {lineage.route_id ? (
          <Chip>{lineage.route_id.replaceAll("_", " ")}</Chip>
        ) : null}
      </div>

      <div className="mt-4 grid gap-2">
        <div className="grid gap-2 md:grid-cols-2">
          <ValueRow label="Decision mode" value={lineage.decision_mode} />
          <ValueRow label="Confidence" value={lineage.confidence_hint} />
        </div>
        <div className="grid gap-2 md:grid-cols-2">
          <ValueRow label="Source task" value={lineage.source_task_artifact_id} />
          <ValueRow
            label="Summary artifact"
            value={lineage.summary_artifact_title ?? lineage.summary_artifact_id}
            onClick={lineage.summary_artifact_id && onOpenArtifact
              ? () => onOpenArtifact(lineage.summary_artifact_id!)
              : undefined}
          />
        </div>
        {lineage.route_rationale && (
          <div className="rounded-xl border border-white/5 bg-black/15 px-3 py-2">
            <div className="text-[10px] font-black uppercase tracking-widest text-slate-500">
              Rationale
            </div>
            <div className="mt-1.5 text-xs leading-6 text-slate-300">
              {lineage.route_rationale}
            </div>
          </div>
        )}
        {lineage.workflow && (
          <div className="rounded-xl border border-white/5 bg-black/15 px-3 py-2">
            <div className="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-500">
              <Workflow className="h-3.5 w-3.5 text-violet-300" />
              Workflow lineage
            </div>
            <div className="mt-2 grid gap-2 md:grid-cols-2">
              <ValueRow label="Intent" value={lineage.workflow.workflow_intent_id} />
              <ValueRow label="Candidate set" value={lineage.workflow.candidate_set_id} />
              <ValueRow label="Workflow draft" value={lineage.workflow.workflow_draft_id} />
              <ValueRow label="Proposal" value={lineage.workflow.proposal_id} />
              <ValueRow label="Definition" value={lineage.workflow.definition_id} />
              <ValueRow label="Scope" value={lineage.workflow.scope_key} />
              <ValueRow label="Motif" value={lineage.workflow.motif_kind} />
              <ValueRow label="Generation mode" value={lineage.workflow.generation_mode} />
            </div>
            {lineage.workflow.proposal_digest_path && (
              <div className="mt-2 break-all text-[11px] text-slate-500">
                Proposal digest: {lineage.workflow.proposal_digest_path}
              </div>
            )}
          </div>
        )}
        {lineage.reference_paths.length > 0 && (
          <div className="rounded-xl border border-white/5 bg-black/15 px-3 py-2">
            <div className="text-[10px] font-black uppercase tracking-widest text-slate-500">
              Reference files
            </div>
            <div className="mt-1.5 flex flex-wrap gap-1.5">
              {lineage.reference_paths.map((path) => (
                <Chip key={path}>{path}</Chip>
              ))}
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
