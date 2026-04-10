import React from "react";
import { ArrowRight, Bot, Workflow, ShieldAlert } from "lucide-react";
import type { TaskRoutingContext } from "./initiativeKickoffTemplates.ts";
import {
  decideTaskRoute,
  parseTaskRoutingContextFromAttributes,
  selectTaskRoute,
  type TaskRouteDecision,
  type TaskRouteId,
} from "./taskRouting.ts";

interface TaskRoutingDecisionCardProps {
  attributes?: Record<string, string>;
  onRouteSelected?: (
    routeId: TaskRouteId,
    context: TaskRoutingContext,
    decision: TaskRouteDecision,
  ) => void;
}

function SignalList({
  title,
  entries,
}: {
  title: string;
  entries: string[];
}) {
  if (entries.length === 0) {
    return null;
  }

  return (
    <div className="rounded-xl border border-white/5 bg-black/15 px-3 py-2">
      <div className="text-[10px] font-black uppercase tracking-widest text-slate-500">
        {title}
      </div>
      <div className="mt-1.5 flex flex-wrap gap-1.5">
        {entries.map((entry) => (
          <span
            key={entry}
            className="rounded-full border border-white/5 bg-white/[0.03] px-2 py-0.5 text-[10px] text-slate-300"
          >
            {entry}
          </span>
        ))}
      </div>
    </div>
  );
}

export function TaskRoutingDecisionCard({
  attributes,
  onRouteSelected,
}: TaskRoutingDecisionCardProps) {
  const context = parseTaskRoutingContextFromAttributes(attributes);
  if (!context) {
    return null;
  }

  const recommendedDecision = decideTaskRoute(context);
  const routes: TaskRouteId[] = [
    "direct_agent_assignment",
    "proposal_generation",
    "steward_escalation",
  ];

  return (
    <section className="rounded-2xl border border-cyan-400/10 bg-cyan-500/5 p-5 shadow-inner ring-1 ring-cyan-500/5">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Workflow className="h-4 w-4 text-cyan-300" />
            <h4 className="text-[10px] font-black uppercase tracking-[0.28em] text-cyan-300/85">
              Task router
            </h4>
          </div>
          <div className="mt-2 text-sm font-semibold text-slate-100">
            {context.title}
          </div>
          <div className="mt-1 text-xs leading-5 text-slate-400">
            {recommendedDecision.summary}
          </div>
        </div>
        <span
          className={`shrink-0 rounded-full px-2.5 py-1 text-[10px] font-black uppercase tracking-widest ${
            recommendedDecision.confidence_hint === "high"
              ? "bg-emerald-500/15 text-emerald-300"
              : recommendedDecision.confidence_hint === "medium"
                ? "bg-amber-500/15 text-amber-300"
                : "bg-rose-500/15 text-rose-300"
          }`}
        >
          {recommendedDecision.route_id.replaceAll("_", " ")}
        </span>
      </div>

      <div className="mt-4 grid gap-3">
        <div className="grid gap-3 md:grid-cols-2">
          <SignalList title="Objective" entries={[context.objective]} />
          <SignalList title="Capabilities" entries={context.required_capabilities} />
        </div>

        <SignalList title="Required tasks" entries={context.required_tasks} />
        <div className="grid gap-3 md:grid-cols-2">
          <SignalList title="Bottleneck signals" entries={context.bottleneck_signals} />
          <SignalList title="Error signals" entries={context.error_signals} />
        </div>
        <div className="grid gap-3 md:grid-cols-2">
          <SignalList title="Fallback routes" entries={context.fallback_routes} />
          <SignalList title="Reference files" entries={context.reference_paths} />
        </div>
      </div>

      <div className="mt-4 rounded-xl border border-white/5 bg-slate-950/30 p-3">
        <div className="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-500">
          {recommendedDecision.route_id === "direct_agent_assignment" ? (
            <Bot className="h-3.5 w-3.5 text-cyan-300" />
          ) : recommendedDecision.route_id === "proposal_generation" ? (
            <Workflow className="h-3.5 w-3.5 text-cyan-300" />
          ) : (
            <ShieldAlert className="h-3.5 w-3.5 text-cyan-300" />
          )}
          Recommended route
        </div>
        <div className="mt-2 text-sm font-semibold text-slate-100">
          {recommendedDecision.label}
        </div>
        <div className="mt-1 text-xs leading-5 text-slate-400">
          {recommendedDecision.rationale}
        </div>
      </div>

      <div className="mt-4 flex flex-wrap gap-2">
        {routes.map((routeId) => {
          const decision = routeId === recommendedDecision.route_id
            ? recommendedDecision
            : selectTaskRoute(context, routeId);
          const isRecommended = routeId === recommendedDecision.route_id;

          return (
            <button
              key={routeId}
              type="button"
              onClick={() => onRouteSelected?.(routeId, context, decision)}
              className={`inline-flex items-center gap-2 rounded-full border px-3 py-2 text-[11px] font-bold uppercase tracking-widest transition ${
                isRecommended
                  ? "border-cyan-400/30 bg-cyan-500/15 text-cyan-200"
                  : "border-white/5 bg-white/[0.03] text-slate-300 hover:border-cyan-400/20 hover:bg-cyan-500/10 hover:text-cyan-100"
              }`}
            >
              {decision.label}
              <ArrowRight className="h-3.5 w-3.5" />
            </button>
          );
        })}
      </div>
    </section>
  );
}
