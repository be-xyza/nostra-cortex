import React, { useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import {
  ArrowLeft,
  ArrowUpRight,
  CircleCheckBig,
  Clock3,
  Copy,
  Globe,
  Layout,
  Shield,
  Sparkles,
  TriangleAlert,
  Users,
} from "lucide-react";
import { workbenchApi } from "../../api";
import type { HeapBlockListItem } from "../../contracts";
import { useAvailableSpaces } from "../../store/spacesRegistry";
import { useUiStore } from "../../store/uiStore";
import {
  buildAgentExecutionRecentWorkItem,
  buildProposalReviewRecentWorkItem,
  buildPromotionReceiptRecentWorkItem,
  buildSpaceDetailModel,
} from "./spaceDetailModel";

function getSpaceIcon(spaceId: string, spaceType: "global" | "user" | "system") {
  if (spaceId === "meta") {
    return Globe;
  }
  if (spaceType === "system") {
    return Shield;
  }
  return Layout;
}

function statusTone(statusLabel: string) {
  return statusLabel === "Active"
    ? {
        badge: "border-emerald-400/20 bg-emerald-400/10 text-emerald-200",
        icon: CircleCheckBig,
      }
    : {
        badge: "border-amber-400/20 bg-amber-400/10 text-amber-100",
        icon: TriangleAlert,
      };
}

export function SpaceDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const spaces = useAvailableSpaces();
  const setActiveSpaceIds = useUiStore((state) => state.setActiveSpaceIds);
  const [promotionReceipt, setPromotionReceipt] = useState<HeapBlockListItem | null>(null);
  const [latestAgentExecution, setLatestAgentExecution] = useState<HeapBlockListItem | null>(null);
  const [latestProposal, setLatestProposal] = useState<HeapBlockListItem | null>(null);

  const space = useMemo(() => spaces.find((item) => item.id === id), [id, spaces]);
  const model = useMemo(() => (space ? buildSpaceDetailModel(space) : null), [space]);
  const receiptRecentWorkItem = useMemo(
    () => buildPromotionReceiptRecentWorkItem(promotionReceipt),
    [promotionReceipt],
  );
  const agentRecentWorkItem = useMemo(
    () => buildAgentExecutionRecentWorkItem(latestAgentExecution),
    [latestAgentExecution],
  );
  const proposalRecentWorkItem = useMemo(
    () => buildProposalReviewRecentWorkItem(latestProposal),
    [latestProposal],
  );
  const recentWorkItems = useMemo(
    () =>
      [receiptRecentWorkItem, proposalRecentWorkItem, agentRecentWorkItem, ...(model?.recentWork ?? [])].filter(
        (item): item is NonNullable<typeof item> => Boolean(item),
      ),
    [agentRecentWorkItem, model?.recentWork, proposalRecentWorkItem, receiptRecentWorkItem],
  );

  useEffect(() => {
    if (!space) {
      setPromotionReceipt(null);
      return;
    }

    let cancelled = false;
    workbenchApi
      .getHeapBlocks({
        spaceId: space.id,
        blockType: "space_promotion_receipt",
        limit: 1,
      })
      .then((response) => {
        if (!cancelled) {
          setPromotionReceipt(response.items[0] ?? null);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setPromotionReceipt(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [space]);

  useEffect(() => {
    if (!space) {
      setLatestAgentExecution(null);
      return;
    }

    let cancelled = false;
    workbenchApi
      .getHeapBlocks({
        spaceId: space.id,
        blockType: "agent_execution_record",
        limit: 1,
      })
      .then((response) => {
        if (!cancelled) {
          setLatestAgentExecution(response.items[0] ?? null);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setLatestAgentExecution(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [space]);

  useEffect(() => {
    if (!space) {
      setLatestProposal(null);
      return;
    }

    let cancelled = false;
    workbenchApi
      .getHeapBlocks({
        spaceId: space.id,
        blockType: "proposal",
        limit: 1,
      })
      .then((response) => {
        if (!cancelled) {
          setLatestProposal(response.items[0] ?? null);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setLatestProposal(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [space]);

  if (!space || !model) {
    return (
      <div className="mx-auto flex min-h-[calc(100vh-8rem)] max-w-3xl items-center justify-center px-6 py-10">
        <div className="w-full rounded-3xl border border-white/8 bg-white/[0.03] p-8 text-center shadow-[0_32px_80px_-40px_rgba(0,0,0,0.7)]">
          <p className="text-sm uppercase tracking-[0.28em] text-white/35">Space</p>
          <h1 className="mt-3 text-3xl font-semibold text-white">Space not found</h1>
          <p className="mt-3 text-sm text-white/60">
            This space is not available in the current registry view.
          </p>
          <Link
            to="/spaces"
            className="mt-6 inline-flex items-center gap-2 rounded-full border border-white/12 bg-white/[0.04] px-5 py-2.5 text-sm font-medium text-white/85 transition hover:border-white/20 hover:bg-white/[0.08]"
          >
            <ArrowLeft className="h-4 w-4" />
            Back to spaces
          </Link>
        </div>
      </div>
    );
  }

  const Icon = getSpaceIcon(space.id, space.type);
  const tone = statusTone(model.statusLabel);
  const StatusIcon = tone.icon;

  const openWorkbench = () => {
    setActiveSpaceIds([space.id]);
    navigate("/system");
  };

  const openRecentWorkItem = (href?: string) => {
    if (!href) return;
    setActiveSpaceIds([space.id]);
    navigate(href);
  };

  const copyId = async () => {
    try {
      await navigator.clipboard.writeText(space.id);
    } catch {
      // Clipboard access can fail in some browsers; keeping this silent is fine.
    }
  };

  return (
    <div className="mx-auto max-w-6xl px-6 py-8 md:px-8">
      <div className="mb-6 flex items-center gap-3 text-sm text-white/45">
        <Link
          to="/spaces"
          className="inline-flex h-11 w-11 items-center justify-center rounded-2xl border border-white/8 bg-white/[0.03] text-white/75 transition hover:border-white/18 hover:bg-white/[0.07] hover:text-white"
          aria-label="Back to spaces"
        >
          <ArrowLeft className="h-4 w-4" />
        </Link>
        <div className="flex items-center gap-2">
          <span>Spaces</span>
          <span className="text-white/20">/</span>
          <span className="text-white/80">{space.name}</span>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_320px]">
        <div className="space-y-6">
          <section className="rounded-[2rem] border border-white/8 bg-white/[0.03] p-6 shadow-[0_32px_90px_-48px_rgba(0,0,0,0.75)] md:p-8">
            <div className="flex flex-col gap-6 md:flex-row md:items-start md:justify-between">
              <div className="flex gap-5">
                <div className="flex h-24 w-24 shrink-0 items-center justify-center rounded-[1.75rem] border border-violet-400/18 bg-violet-500/8 text-violet-300">
                  <Icon className="h-10 w-10" />
                </div>
                <div className="space-y-3">
                  <div className="flex flex-wrap items-center gap-3">
                    <h1 className="text-4xl font-semibold tracking-tight text-white md:text-5xl">
                      {space.name}
                    </h1>
                    <span className={`inline-flex items-center gap-2 rounded-full border px-3 py-1 text-sm font-medium ${tone.badge}`}>
                      <StatusIcon className="h-4 w-4" />
                      {model.statusLabel}
                    </span>
                  </div>
                  <p className="max-w-2xl text-lg leading-8 text-white/62">
                    {space.description || "A shared place for work, notes, and updates."}
                  </p>
                  <p className="text-sm text-white/48">{model.statusMessage}</p>
                </div>
              </div>

              <button
                type="button"
                onClick={openWorkbench}
                className="inline-flex items-center justify-center gap-2 rounded-full bg-white px-5 py-3 text-sm font-semibold text-slate-950 transition hover:bg-white/90"
              >
                Open space
                <ArrowUpRight className="h-4 w-4" />
              </button>
            </div>
          </section>

          <section className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <div className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
              <h2 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">About</h2>
              <dl className="mt-5 space-y-4">
                {model.aboutRows.map((row) => (
                  <div key={row.label} className="rounded-2xl border border-white/6 bg-white/[0.025] px-4 py-4">
                    <dt className="text-xs font-semibold uppercase tracking-[0.22em] text-white/35">
                      {row.label}
                    </dt>
                    <dd className="mt-2 text-base leading-7 text-white/84">{row.value}</dd>
                  </div>
                ))}
              </dl>
            </div>

            <div className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
              <div className="flex items-center gap-2">
                <Users className="h-4 w-4 text-white/45" />
                <h2 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">People</h2>
              </div>
              <div className="mt-5 grid gap-3">
                {model.people.map((person) => (
                  <div
                    key={person.id}
                    className="rounded-2xl border border-white/6 bg-white/[0.025] px-4 py-4"
                  >
                    <p className="text-base font-medium text-white/88">{person.name}</p>
                    <p className="mt-1 text-sm text-white/48">{person.roleLabel}</p>
                  </div>
                ))}
              </div>
            </div>
          </section>

          <section className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
            <div className="flex items-center gap-2">
              <Sparkles className="h-4 w-4 text-white/45" />
              <h2 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Recent work</h2>
            </div>
            <div className="mt-5 grid gap-3">
              {recentWorkItems.map((item) => (
                <div
                  key={`${item.label}:${item.value}`}
                  className="rounded-2xl border border-white/6 bg-white/[0.025] px-4 py-4"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="min-w-0">
                      <p className="text-sm font-semibold uppercase tracking-[0.18em] text-white/40">
                        {item.label}
                      </p>
                      <p className="mt-2 text-base leading-7 text-white/82">{item.value}</p>
                    </div>
                    {item.href ? (
                      <button
                        type="button"
                        onClick={() => openRecentWorkItem(item.href)}
                        className="shrink-0 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1.5 text-sm font-medium text-white/78 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                      >
                        {item.actionLabel || "Open"}
                      </button>
                    ) : null}
                  </div>
                </div>
              ))}
            </div>
          </section>
        </div>

        <aside className="space-y-6">
          <section className="rounded-[1.75rem] border border-white/8 bg-white/[0.03] p-6">
            <h2 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Space details</h2>
            <div className="mt-5 space-y-5">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.22em] text-white/35">Space ID</p>
                <div className="mt-2 flex items-center gap-2 rounded-2xl border border-white/6 bg-black/20 px-3 py-3">
                  <code className="min-w-0 flex-1 truncate text-sm text-white/72">{space.id}</code>
                  <button
                    type="button"
                    onClick={copyId}
                    className="inline-flex h-9 w-9 items-center justify-center rounded-xl border border-white/8 bg-white/[0.03] text-white/65 transition hover:border-white/16 hover:bg-white/[0.08] hover:text-white"
                    aria-label="Copy space ID"
                  >
                    <Copy className="h-4 w-4" />
                  </button>
                </div>
              </div>

              <div className="rounded-2xl border border-white/6 bg-white/[0.025] px-4 py-4">
                <div className="flex items-start gap-3">
                  <Clock3 className="mt-0.5 h-4 w-4 text-white/42" />
                  <div>
                    <p className="text-xs font-semibold uppercase tracking-[0.18em] text-white/35">Created</p>
                    <p className="mt-2 text-sm leading-6 text-white/78">{model.aboutRows[2]?.value}</p>
                  </div>
                </div>
              </div>

              <div className="rounded-2xl border border-white/6 bg-white/[0.025] px-4 py-4">
                <p className="text-xs font-semibold uppercase tracking-[0.18em] text-white/35">What you can do here</p>
                <p className="mt-2 text-sm leading-6 text-white/72">
                  Open this space to read updates, review agent output, and continue work here.
                </p>
              </div>
            </div>
          </section>
        </aside>
      </div>
    </div>
  );
}
