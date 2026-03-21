import type { DpubBlastRadiusResponse } from "../../contracts";
import {
  buildContributionFocusGraphModel,
  type ContributionFocusGraphGroup,
} from "./contributionsRouteState";

function relationTone(key: ContributionFocusGraphGroup["key"]): string {
  switch (key) {
    case "dependsOn":
    case "references":
      return "border-cyan-300/20 bg-cyan-300/10 text-cyan-100";
    case "dependedBy":
    case "referencedBy":
      return "border-emerald-300/20 bg-emerald-300/10 text-emerald-100";
    case "invalidates":
    case "invalidatedBy":
      return "border-amber-300/20 bg-amber-300/10 text-amber-100";
    case "supersedes":
    case "supersededBy":
      return "border-fuchsia-300/20 bg-fuchsia-300/10 text-fuchsia-100";
    default:
      return "border-white/10 bg-white/5 text-slate-100";
  }
}

export function ContributionFocusMap({
  contributionId,
  blastRadius,
  onFocusContribution,
}: {
  contributionId: string;
  blastRadius: DpubBlastRadiusResponse | null;
  onFocusContribution: (contributionId: string) => void;
}) {
  const graph = buildContributionFocusGraphModel(contributionId, blastRadius);

  return (
    <section
      className="rounded-[20px] border border-white/8 bg-white/4 p-4"
      role="region"
      aria-label="Contribution Focus Map"
      data-testid="contribution-focus-map"
    >
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="text-[10px] uppercase tracking-[0.24em] text-slate-500">
            Contribution Focus Map
          </div>
          <p className="mt-1 max-w-xl text-sm leading-6 text-slate-300">
            A steward-first one-hop graph. Nostra defines the relation set, and Cortex renders a
            compact interactive map instead of forcing a full canvas when the operator only needs
            the current blast radius.
          </p>
        </div>
        <div className="rounded-2xl border border-white/10 bg-slate-950/60 px-3 py-2 text-right">
          <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500">Focus</div>
          <div className="mt-1 font-mono text-sm text-white">{contributionId}</div>
        </div>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-3">
        <div className="rounded-[24px] border border-cyan-300/20 bg-[radial-gradient(circle_at_top,rgba(34,211,238,0.16),rgba(4,12,24,0.9))] p-4 sm:col-span-1">
          <div className="text-[10px] uppercase tracking-[0.22em] text-cyan-100/70">
            Center Node
          </div>
          <button
            type="button"
            className="mt-3 w-full rounded-[20px] border border-cyan-200/25 bg-cyan-300/12 px-4 py-4 text-left"
            onClick={() => onFocusContribution(contributionId)}
            aria-label={`Focus relation ${contributionId}`}
          >
            <div className="text-[11px] uppercase tracking-[0.2em] text-cyan-100/70">
              Contribution
            </div>
            <div className="mt-2 font-mono text-base text-white">{contributionId}</div>
            <div className="mt-3 text-xs text-cyan-50/80">
              {graph.edges.length} relation{graph.edges.length === 1 ? "" : "s"} across{" "}
              {graph.groups.length} active lane{graph.groups.length === 1 ? "" : "s"}.
            </div>
          </button>
        </div>

        <div className="grid gap-3 sm:col-span-2 lg:grid-cols-2">
          {graph.groups.length > 0 ? (
            graph.groups.map((group) => (
              <div
                key={group.key}
                className={`rounded-[20px] border p-4 ${relationTone(group.key)}`}
              >
                <div className="flex items-center justify-between gap-3">
                  <div className="text-[10px] uppercase tracking-[0.22em] opacity-80">
                    {group.label}
                  </div>
                  <div className="rounded-full border border-current/20 px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.2em]">
                    {group.items.length}
                  </div>
                </div>
                <div className="mt-3 flex flex-wrap gap-2">
                  {group.items.map((item) => (
                    <button
                      key={`${group.key}:${item}`}
                      type="button"
                      className="rounded-full border border-current/25 bg-black/20 px-3 py-1.5 text-xs font-semibold"
                      onClick={() => onFocusContribution(item)}
                      aria-label={`Focus relation ${item}`}
                    >
                      {item}
                    </button>
                  ))}
                </div>
              </div>
            ))
          ) : (
            <div className="rounded-[20px] border border-white/10 bg-slate-950/60 p-4 text-sm leading-6 text-slate-300 lg:col-span-2">
              No relation edges are available yet for this contribution. Once the graph artifact is
              present, this map will surface a focused neighborhood instead of a blank full-graph
              canvas.
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
