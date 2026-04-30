import { useEffect, useMemo, useState } from "react";
import type React from "react";
import { Activity, CheckCircle2, ClipboardList, ExternalLink, ShieldAlert } from "lucide-react";
import { Link } from "react-router-dom";

import { workbenchApi } from "../../api";
import type { HeapBlockListItem } from "../../contracts";
import { useActiveSpaceContext } from "../../store/spacesRegistry";
import { useUiStore } from "../../store/uiStore";

type InboxLane = "approval" | "proposal" | "execution" | "usage";

interface InboxItem {
  id: string;
  lane: InboxLane;
  title: string;
  type: string;
  timestamp: string;
  summary: string;
  href: string;
}

const LANE_META: Record<InboxLane, { label: string; tone: string }> = {
  approval: { label: "Approval Evidence", tone: "border-emerald-400/30 bg-emerald-400/10 text-emerald-100" },
  proposal: { label: "Needs Review", tone: "border-amber-400/30 bg-amber-400/10 text-amber-100" },
  execution: { label: "Execution", tone: "border-sky-400/30 bg-sky-400/10 text-sky-100" },
  usage: { label: "Usage", tone: "border-slate-400/25 bg-slate-400/10 text-slate-200" },
};

function formatDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Unknown time";
  return date.toLocaleString();
}

function summarizeBlock(block: HeapBlockListItem): string {
  const projection = block.projection;
  if (projection.blockType === "eudaemon_evidence_note") {
    const approvalRef = projection.attributes?.approval_ref;
    return approvalRef
      ? `Approved evidence packet ${approvalRef}.`
      : "Evidence note emitted by the VPS agent.";
  }
  if (projection.blockType === "self_optimization_proposal") {
    return "The VPS agent emitted a self-optimization proposal. Review for recurring recommendations before changing runtime behavior.";
  }
  if (projection.blockType === "agent_execution_record") {
    return "Execution record from the VPS agent. Use it to spot repeated failures, latency spikes, or cost drift.";
  }
  if (projection.blockType === "usage_report") {
    return "Usage report from the VPS agent. Monitor cadence and volume; no direct approval is encoded in this block.";
  }
  return "Heap activity item from the active Space.";
}

function classifyBlock(block: HeapBlockListItem): InboxItem {
  const projection = block.projection;
  const type = projection.blockType;
  const lane: InboxLane =
    type === "eudaemon_evidence_note"
      ? "approval"
      : type === "self_optimization_proposal"
        ? "proposal"
        : type === "usage_report"
          ? "usage"
          : "execution";
  return {
    id: projection.artifactId,
    lane,
    title: projection.title || type,
    type,
    timestamp: projection.updatedAt || projection.emittedAt || "",
    summary: summarizeBlock(block),
    href: `/explore?artifact_id=${encodeURIComponent(projection.artifactId)}`,
  };
}

function laneSort(item: InboxItem): number {
  if (item.lane === "proposal") return 0;
  if (item.lane === "approval") return 1;
  if (item.lane === "execution") return 2;
  return 3;
}

export function InboxPage() {
  const activeSpaceId = useActiveSpaceContext();
  const sessionUser = useUiStore((state) => state.sessionUser);
  const [items, setItems] = useState<InboxItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    workbenchApi
      .getHeapBlocks({ spaceId: activeSpaceId, limit: 100 })
      .then((response) => {
        if (cancelled) return;
        setItems(response.items.map(classifyBlock));
      })
      .catch((err) => {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [activeSpaceId]);

  const counts = useMemo(() => {
    return items.reduce(
      (acc, item) => {
        acc[item.lane] += 1;
        return acc;
      },
      { approval: 0, proposal: 0, execution: 0, usage: 0 } as Record<InboxLane, number>,
    );
  }, [items]);

  const focusItems = useMemo(
    () =>
      [...items]
        .sort((a, b) => laneSort(a) - laneSort(b) || Date.parse(b.timestamp) - Date.parse(a.timestamp))
        .slice(0, 12),
    [items],
  );
  const hasOperatorRole = sessionUser?.role === "operator";

  return (
    <div className="min-h-screen bg-[radial-gradient(circle_at_top_left,rgba(37,99,235,0.18),transparent_32rem),#020617] px-6 py-8 text-slate-100">
      <div className="mx-auto max-w-6xl space-y-6">
        <header className="flex flex-col gap-4 border-b border-white/10 pb-6 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <div className="text-[11px] font-black uppercase tracking-[0.34em] text-sky-300">Inbox & Approvals</div>
            <h1 className="mt-2 text-3xl font-black tracking-normal text-white">VPS Agent Attention Queue</h1>
            <p className="mt-3 max-w-3xl text-sm leading-6 text-slate-300">
              A focused readout of the activity currently landing in heap blocks. Proposals are review-first; approved evidence is already stewarded.
            </p>
          </div>
          <div className="rounded-2xl border border-amber-300/25 bg-amber-300/10 px-4 py-3 text-sm text-amber-100">
            {hasOperatorRole
              ? "Operator role detected. Identity verification may still gate mutations."
              : "Viewer mode active. Approval mutations and action plans require operator identity."}
          </div>
        </header>

        <section className="grid gap-3 md:grid-cols-4">
          <SummaryCard icon={<ShieldAlert size={18} />} label="Needs Review" value={counts.proposal} tone="amber" />
          <SummaryCard icon={<CheckCircle2 size={18} />} label="Approved Evidence" value={counts.approval} tone="emerald" />
          <SummaryCard icon={<Activity size={18} />} label="Execution Records" value={counts.execution} tone="sky" />
          <SummaryCard icon={<ClipboardList size={18} />} label="Usage Reports" value={counts.usage} tone="slate" />
        </section>

        <section className="rounded-2xl border border-white/10 bg-slate-950/70 shadow-[0_24px_80px_-56px_rgba(15,23,42,0.9)]">
          <div className="flex items-center justify-between border-b border-white/10 px-5 py-4">
            <div>
              <h2 className="text-sm font-black uppercase tracking-[0.22em] text-slate-200">Focus Items</h2>
              <p className="mt-1 text-xs text-slate-500">Sorted to put proposals and approval evidence ahead of routine telemetry.</p>
            </div>
            <Link className="inline-flex items-center gap-2 rounded-full border border-white/10 px-3 py-2 text-xs font-bold text-slate-200 hover:border-sky-300/40 hover:text-sky-100" to="/explore">
              Explore all
              <ExternalLink size={14} />
            </Link>
          </div>

          {loading && <div className="px-5 py-8 text-sm text-slate-400">Loading agent activity...</div>}
          {error && (
            <div className="m-5 rounded-xl border border-red-400/30 bg-red-500/10 px-4 py-3 text-sm text-red-100">
              Failed to load inbox activity: {error}
            </div>
          )}
          {!loading && !error && focusItems.length === 0 && (
            <div className="px-5 py-8 text-sm text-slate-400">No attention items are available for this Space.</div>
          )}
          {!loading && !error && focusItems.length > 0 && (
            <div className="divide-y divide-white/8">
              {focusItems.map((item) => (
                <Link key={item.id} to={item.href} className="block px-5 py-4 transition-colors hover:bg-white/[0.035]">
                  <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                    <div className="min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className={`rounded-full border px-2.5 py-1 text-[10px] font-black uppercase tracking-[0.18em] ${LANE_META[item.lane].tone}`}>
                          {LANE_META[item.lane].label}
                        </span>
                        <span className="text-xs text-slate-500">{formatDate(item.timestamp)}</span>
                      </div>
                      <h3 className="mt-2 text-base font-bold text-white">{item.title}</h3>
                      <p className="mt-1 max-w-3xl text-sm leading-6 text-slate-400">{item.summary}</p>
                    </div>
                    <div className="font-mono text-[11px] text-slate-500">{item.id.slice(0, 12)}...</div>
                  </div>
                </Link>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}

function SummaryCard({
  icon,
  label,
  value,
  tone,
}: {
  icon: React.ReactNode;
  label: string;
  value: number;
  tone: "amber" | "emerald" | "sky" | "slate";
}) {
  const toneClass = {
    amber: "border-amber-300/25 bg-amber-300/10 text-amber-100",
    emerald: "border-emerald-300/25 bg-emerald-300/10 text-emerald-100",
    sky: "border-sky-300/25 bg-sky-300/10 text-sky-100",
    slate: "border-slate-300/20 bg-slate-300/10 text-slate-100",
  }[tone];

  return (
    <div className={`rounded-2xl border p-4 ${toneClass}`}>
      <div className="flex items-center justify-between">
        <span>{icon}</span>
        <span className="text-2xl font-black">{value}</span>
      </div>
      <div className="mt-4 text-[11px] font-black uppercase tracking-[0.22em] opacity-80">{label}</div>
    </div>
  );
}
