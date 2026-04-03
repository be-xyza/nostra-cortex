import React, { useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
import {
  ArrowLeft,
  ArrowUpRight,
  CircleCheckBig,
  Clock3,
  Copy,
  GitBranch,
  Globe,
  Layout,
  Layers3,
  KeyRound,
  Route,
  Server,
  Shield,
  Sparkles,
  SlidersHorizontal,
  TriangleAlert,
  Users,
  Workflow,
} from "lucide-react";
import { workbenchApi } from "../../api";
import type { HeapBlockListItem, SpaceSettingsResponse } from "../../contracts";
import { useAvailableSpaces } from "../../store/spacesRegistry";
import { useUiStore } from "../../store/uiStore";
import {
  formatProviderAccessLabel,
  formatProviderHoverDetails,
  formatProviderLocalityLabel,
  formatProviderModelLabel,
  formatProviderTopologySummary,
  formatProviderTypeLabel,
} from "../system/providerTopology";
import {
  buildAgentExecutionRecentWorkItem,
  buildProposalReviewRecentWorkItem,
  buildPromotionReceiptRecentWorkItem,
  buildSpaceDetailModel,
} from "./spaceDetailModel";
import {
  applyAuthBindingToSpaceRouting,
  parseAuthMetadataJson,
} from "./spaceProviderBinding";

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
  const [searchParams, setSearchParams] = useSearchParams();
  const spaces = useAvailableSpaces();
  const setActiveSpaceIds = useUiStore((state) => state.setActiveSpaceIds);
  const [promotionReceipt, setPromotionReceipt] = useState<HeapBlockListItem | null>(null);
  const [latestAgentExecution, setLatestAgentExecution] = useState<HeapBlockListItem | null>(null);
  const [latestProposal, setLatestProposal] = useState<HeapBlockListItem | null>(null);
  const [spaceSettings, setSpaceSettings] = useState<SpaceSettingsResponse | null>(null);
  const [settingsLoading, setSettingsLoading] = useState(false);
  const [bindingProviderId, setBindingProviderId] = useState("");
  const [bindingModel, setBindingModel] = useState("");
  const [bindingLabel, setBindingLabel] = useState("");
  const [bindingApiKey, setBindingApiKey] = useState("");
  const [bindingMetadataJson, setBindingMetadataJson] = useState("{\n  \n}");
  const [bindingApplyToSpace, setBindingApplyToSpace] = useState(true);
  const [bindingError, setBindingError] = useState<string | null>(null);
  const [bindingMessage, setBindingMessage] = useState<string | null>(null);
  const [bindingSubmitting, setBindingSubmitting] = useState(false);

  const space = useMemo(() => spaces.find((item) => item.id === id), [id, spaces]);
  const model = useMemo(() => (space ? buildSpaceDetailModel(space) : null), [space]);
  const activeTab = useMemo(() => {
    const tab = searchParams.get("tab");
    return tab === "routing" || tab === "agents" || tab === "history" || tab === "lineage" ? (tab === "lineage" ? "history" : tab) : "overview";
  }, [searchParams]);
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
  const lineagePromptCount = spaceSettings?.lineage.promptArtifacts.length ?? 0;
  const lineageFeedbackCount = spaceSettings?.lineage.feedbackArtifacts.length ?? 0;
  const lineageLegacyCount = spaceSettings?.lineage.legacyPromptGroups.length ?? 0;
  const providerOptions = useMemo(
    () =>
      (spaceSettings?.providers ?? []).filter(
        (provider) => provider.providerType === "Llm" || provider.providerType === "Embedding",
      ),
    [spaceSettings],
  );
  const selectedProvider = useMemo(
    () => providerOptions.find((provider) => provider.id === bindingProviderId) ?? null,
    [bindingProviderId, providerOptions],
  );
  const modelOptions = useMemo(() => {
    const supported = selectedProvider?.supportedModels ?? [];
    const fallback = selectedProvider?.defaultModel ? [selectedProvider.defaultModel] : [];
    const next = [...supported, ...fallback].filter((value, index, values) => Boolean(value) && values.indexOf(value) === index);
    return next.length > 0 ? next : ["Not configured"];
  }, [selectedProvider]);

  useEffect(() => {
    if (!bindingProviderId && providerOptions.length > 0) {
      const firstProvider = providerOptions[0];
      setBindingProviderId(firstProvider?.id ?? "");
      setBindingModel(firstProvider?.defaultModel || firstProvider?.supportedModels?.[0] || "");
    }
  }, [bindingProviderId, providerOptions]);

  useEffect(() => {
    if (!selectedProvider) {
      setBindingModel("");
      return;
    }

    const nextModel = selectedProvider.defaultModel || selectedProvider.supportedModels?.[0] || "";
    setBindingModel((current) => current || nextModel);
  }, [selectedProvider]);

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

  useEffect(() => {
    if (!space) {
      setSpaceSettings(null);
      return;
    }

    let cancelled = false;
    setSettingsLoading(true);
    workbenchApi
      .getSpaceSettings(space.id)
      .then((response) => {
        if (!cancelled) {
          setSpaceSettings(response);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSpaceSettings(null);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setSettingsLoading(false);
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

  const submitSpaceBinding = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!space || !spaceSettings) {
      return;
    }

    setBindingError(null);
    setBindingMessage(null);

    if (!bindingProviderId.trim() || !bindingModel.trim() || !bindingLabel.trim() || !bindingApiKey.trim()) {
      setBindingError("Provider, model, label, and API key are required.");
      return;
    }

    let metadata: Record<string, string> | undefined;
    try {
      metadata = parseAuthMetadataJson(bindingMetadataJson);
    } catch {
      setBindingError("Metadata must be valid JSON.");
      return;
    }

    setBindingSubmitting(true);
    try {
      const binding = await workbenchApi.createSystemAuthBinding({
        targetId: bindingProviderId,
        targetKind: "provider",
        authType: "api_key",
        label: bindingLabel,
        apiKey: bindingApiKey,
        metadata,
      });

      if (bindingApplyToSpace) {
        await workbenchApi.putSpaceRouting(
          space.id,
          applyAuthBindingToSpaceRouting(
            spaceSettings.routing,
            binding.authBindingId,
            bindingProviderId,
            bindingModel,
            spaceSettings.routing.adapterSetRef ?? undefined,
          ),
        );
      }

      const refreshedSettings = await workbenchApi.getSpaceSettings(space.id);
      setSpaceSettings(refreshedSettings);
      setBindingApiKey("");
      setBindingLabel("");
      setBindingMetadataJson("{\n  \n}");
      setBindingModel("");
      setBindingMessage(
        bindingApplyToSpace
          ? "Auth binding saved and applied to this space."
          : "Auth binding saved globally.",
      );
    } catch (err) {
      setBindingError(err instanceof Error ? err.message : String(err));
    } finally {
      setBindingSubmitting(false);
    }
  };

  const setActiveTab = (tab: "overview" | "routing" | "agents" | "history") => {
    const next = new URLSearchParams(searchParams);
    next.set("tab", tab);
    setSearchParams(next, { replace: true });
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

          <section className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-3">
            <div className="flex flex-wrap gap-2">
              {(
                [
                  ["overview", "Overview", Layout],
                  ["routing", "Routing", Route],
                  ["agents", "Agents", Users],
                  ["history", "History", GitBranch],
                ] as const
              ).map(([tab, label, Icon]) => {
                const active = activeTab === tab;
                return (
                  <button
                    key={tab}
                    type="button"
                    onClick={() => setActiveTab(tab)}
                    className={`inline-flex items-center gap-2 rounded-2xl border px-4 py-2.5 text-sm font-medium transition ${
                      active
                        ? "border-white/20 bg-white text-slate-950"
                        : "border-white/8 bg-white/[0.02] text-white/70 hover:border-white/16 hover:bg-white/[0.06] hover:text-white"
                    }`}
                  >
                    <Icon className="h-4 w-4" />
                    {label}
                  </button>
                );
              })}
            </div>
          </section>

          {activeTab === "overview" && (
            <>
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
            </>
          )}

          {activeTab === "routing" && (
            <section className="space-y-6">
              <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
                <div className="space-y-6">
                  <div className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
                    <div className="flex items-center justify-between gap-3">
                      <div className="flex items-center gap-2">
                        <SlidersHorizontal className="h-4 w-4 text-white/45" />
                        <h2 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Routing</h2>
                      </div>
                      <Link
                        to="/system/providers"
                        className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-white/75 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                      >
                        Providers
                        <ArrowUpRight className="h-3.5 w-3.5" />
                      </Link>
                    </div>
                    {settingsLoading && !spaceSettings ? (
                      <p className="mt-4 text-sm text-white/55">Loading space settings…</p>
                    ) : (
                      <div className="mt-5 grid gap-3">
                        <div className="rounded-2xl border border-white/6 bg-black/15 px-4 py-4">
                          <p className="text-xs uppercase tracking-[0.18em] text-white/35">Provider set</p>
                          <p className="mt-2 text-base text-white/84">{spaceSettings?.routing.adapterSetRef || "Not bound"}</p>
                        </div>
                        <div className="rounded-2xl border border-white/6 bg-black/15 px-4 py-4">
                          <p className="text-xs uppercase tracking-[0.18em] text-white/35">Model</p>
                          <p className="mt-2 text-base text-white/84">{spaceSettings?.routing.defaultModel || "Not selected"}</p>
                        </div>
                        <div className="rounded-2xl border border-white/6 bg-black/15 px-4 py-4">
                          <p className="text-xs uppercase tracking-[0.18em] text-white/35">Auth</p>
                          <p className="mt-2 text-base text-white/84">{spaceSettings?.routing.authBindingId || "Inherited runtime auth"}</p>
                        </div>
                        <div className="rounded-2xl border border-white/6 bg-black/15 px-4 py-4">
                          <p className="text-xs uppercase tracking-[0.18em] text-white/35">Agent policy</p>
                          <p className="mt-2 text-base text-white/84">{spaceSettings?.routing.agentRoutingPolicy || "Default routing policy"}</p>
                        </div>
                      </div>
                    )}
                  </div>

                  <div className="grid gap-6 xl:grid-cols-3">
                    {[
                      {
                        title: "Execution profile",
                        icon: Workflow,
                        payload: spaceSettings?.executionProfile?.payload?.executionProfile,
                        empty: "No execution profile surfaced yet.",
                      },
                      {
                        title: "Attribution domains",
                        icon: Layers3,
                        payload: spaceSettings?.attributionDomains?.payload?.domains,
                        empty: "No attribution domains surfaced yet.",
                      },
                      {
                        title: "Governance scope",
                        icon: Shield,
                        payload: spaceSettings?.governanceScope?.payload?.scope,
                        empty: "No governance scope surfaced yet.",
                      },
                    ].map(({ title, icon: Icon, payload, empty }) => (
                      <div key={title} className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
                        <div className="flex items-center gap-2">
                          <Icon className="h-4 w-4 text-white/45" />
                          <h3 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">{title}</h3>
                        </div>
                        {payload ? (
                          <pre className="mt-4 overflow-auto rounded-2xl border border-white/6 bg-black/20 p-4 text-xs leading-6 text-white/72">
                            {JSON.stringify(payload, null, 2)}
                          </pre>
                        ) : (
                          <p className="mt-4 text-sm text-white/55">{empty}</p>
                        )}
                      </div>
                    ))}
                  </div>
                </div>

                <aside className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <div className="flex items-center gap-2">
                        <Server className="h-4 w-4 text-white/45" />
                        <h3 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Providers</h3>
                      </div>
                      <p className="mt-3 text-sm leading-6 text-white/62">
                        Choose a provider, add a key, and keep the selection in this space without leaving the page.
                      </p>
                    </div>
                  </div>

                  <div className="mt-5 grid gap-3">
                    {providerOptions.length ? (
                      providerOptions.map((provider) => {
                        const isSelected = provider.id === bindingProviderId;
                        return (
                          <button
                            key={provider.id}
                            type="button"
                            onClick={() => {
                              setBindingProviderId(provider.id);
                              setBindingModel(provider.defaultModel || provider.supportedModels?.[0] || "");
                            }}
                            className={[
                              "rounded-2xl border px-4 py-4 text-left transition",
                              isSelected
                                ? "border-cyan-400/30 bg-cyan-400/10 shadow-[0_18px_46px_-24px_rgba(34,211,238,0.28)]"
                                : "border-white/6 bg-black/15 hover:border-white/12 hover:bg-black/20",
                            ].join(" ")}
                            title={formatProviderHoverDetails(provider)}
                          >
                            <div className="flex items-start justify-between gap-3">
                              <div>
                                <p className="text-sm font-semibold text-white/88">{provider.name}</p>
                                <p className="mt-1 text-xs uppercase tracking-[0.18em] text-white/38">
                                  {provider.providerType} · {provider.providerFamily || "provider"}
                                </p>
                              </div>
                              <span className={`rounded-full px-2 py-1 text-[10px] font-semibold ${provider.authState === "linked" || provider.authState === "not_required" ? "text-emerald-300" : "text-amber-200"}`}>
                                {provider.authState === "linked" || provider.authState === "not_required" ? "READY" : "NEEDS AUTH"}
                              </span>
                            </div>
                            <div className="mt-3 flex flex-wrap gap-2 text-[11px] text-white/45">
                              <span className="rounded-full border border-white/8 px-2 py-1">
                                {formatProviderModelLabel(provider)}
                              </span>
                              <span className="rounded-full border border-white/8 px-2 py-1">
                                {formatProviderLocalityLabel(provider)}
                              </span>
                              <span className="rounded-full border border-white/8 px-2 py-1">
                                {formatProviderAccessLabel(provider)}
                              </span>
                            </div>
                            <p className="mt-3 text-xs text-white/55 break-all">{provider.endpoint}</p>
                            {provider.topology ? (
                              <p className="mt-2 text-[11px] leading-5 text-white/48" title={formatProviderTopologySummary(provider)}>
                                {formatProviderTopologySummary(provider)}
                              </p>
                            ) : null}
                          </button>
                        );
                      })
                    ) : (
                      <div className="rounded-2xl border border-dashed border-white/10 px-4 py-6 text-sm text-white/55">
                        No provider records surfaced yet.
                      </div>
                    )}
                  </div>

                  <form onSubmit={submitSpaceBinding} className="mt-6 grid gap-4 border-t border-white/8 pt-6">
                    <div>
                      <div className="flex items-center gap-2">
                        <Sparkles className="h-4 w-4 text-white/45" />
                        <h4 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Quick add</h4>
                      </div>
                      <p className="mt-2 text-sm leading-6 text-white/58">
                        Create a global key and apply it to this space in one step.
                      </p>
                    </div>

                    <label className="grid gap-2 text-sm">
                      <span className="text-white/50">Provider</span>
                      <select
                        value={bindingProviderId}
                        onChange={(event) => {
                          const nextProviderId = event.target.value;
                          setBindingProviderId(nextProviderId);
                          const nextProvider = providerOptions.find((provider) => provider.id === nextProviderId) ?? null;
                          setBindingModel(nextProvider?.defaultModel || nextProvider?.supportedModels?.[0] || "");
                        }}
                        className="rounded-xl border border-white/10 bg-black/20 px-3 py-2 text-sm text-white/84"
                      >
                        <option value="">Select a provider</option>
                        {providerOptions.map((provider) => (
                          <option key={provider.id} value={provider.id}>
                            {provider.name} ({formatProviderTypeLabel(provider.providerType)})
                          </option>
                        ))}
                      </select>
                    </label>

                    <label className="grid gap-2 text-sm">
                      <span className="text-white/50">Model</span>
                      <select
                        value={bindingModel}
                        onChange={(event) => setBindingModel(event.target.value)}
                        disabled={!selectedProvider}
                        className="rounded-xl border border-white/10 bg-black/20 px-3 py-2 text-sm text-white/84 disabled:cursor-not-allowed disabled:opacity-60"
                      >
                        {modelOptions.map((model) => (
                          <option key={model} value={model}>
                            {model}
                          </option>
                        ))}
                      </select>
                    </label>

                    <label className="grid gap-2 text-sm">
                      <span className="text-white/50">Label</span>
                      <input
                        value={bindingLabel}
                        onChange={(event) => setBindingLabel(event.target.value)}
                        placeholder="Auth binding label"
                        className="rounded-xl border border-white/10 bg-black/20 px-3 py-2 text-sm text-white/84 placeholder:text-white/28"
                      />
                    </label>

                    <label className="grid gap-2 text-sm">
                      <span className="text-white/50">API key</span>
                      <input
                        value={bindingApiKey}
                        onChange={(event) => setBindingApiKey(event.target.value)}
                        type="password"
                        placeholder="sk-..."
                        className="rounded-xl border border-white/10 bg-black/20 px-3 py-2 text-sm text-white/84 placeholder:text-white/28"
                      />
                    </label>

                    <label className="grid gap-2 text-sm">
                      <span className="text-white/50">Advanced details</span>
                      <textarea
                        value={bindingMetadataJson}
                        onChange={(event) => setBindingMetadataJson(event.target.value)}
                        rows={5}
                        className="rounded-xl border border-white/10 bg-black/20 px-3 py-2 font-mono text-xs text-white/80 placeholder:text-white/28"
                      />
                    </label>

                    <label className="flex items-start gap-3 rounded-2xl border border-white/6 bg-black/15 px-4 py-3 text-sm text-white/70">
                      <input
                        type="checkbox"
                        checked={bindingApplyToSpace}
                        onChange={(event) => setBindingApplyToSpace(event.target.checked)}
                        className="mt-1 h-4 w-4 rounded border-white/20 bg-black/20 text-white"
                      />
                      <span>Use this key in the current space after saving it globally.</span>
                    </label>

                    <div className="flex items-center justify-between gap-3">
                      <div className="min-h-5 text-sm">
                        {bindingError ? <span className="text-rose-300">{bindingError}</span> : null}
                        {bindingMessage ? <span className="text-emerald-300">{bindingMessage}</span> : null}
                      </div>
                      <button
                        type="submit"
                        disabled={bindingSubmitting}
                        className="inline-flex items-center gap-2 rounded-full bg-white px-4 py-2 text-sm font-semibold text-slate-950 transition hover:bg-white/90 disabled:cursor-not-allowed disabled:opacity-60"
                      >
                        {bindingSubmitting ? "Saving..." : "Save and use"}
                      </button>
                    </div>
                  </form>
                </aside>
              </div>
            </section>
          )}

          {activeTab === "agents" && (
            <section className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
              <div className="flex items-center gap-2">
                <Server className="h-4 w-4 text-white/45" />
                <h2 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Agent runs</h2>
              </div>
              <div className="mt-5 grid gap-3">
                {(spaceSettings?.agentRuns ?? []).length === 0 ? (
                  <p className="text-sm text-white/55">No linked agent runs surfaced yet.</p>
                ) : (
                  spaceSettings!.agentRuns.map((run) => (
                    <div key={run.runId} className="rounded-2xl border border-white/6 bg-white/[0.025] px-4 py-4">
                      <div className="flex flex-wrap items-center justify-between gap-3">
                        <div>
                          <p className="text-sm font-semibold text-white/88">{run.agentId || run.workflowId}</p>
                          <p className="mt-1 text-xs uppercase tracking-[0.18em] text-white/38">
                            {run.status}
                          </p>
                        </div>
                        <div className="text-right text-xs text-white/55">
                          <p>{run.provider || "Provider pending"}</p>
                          <p>{run.model || "Not configured"}</p>
                        </div>
                      </div>
                      <div className="mt-4 grid gap-2 text-sm text-white/70 md:grid-cols-2">
                        <div>
                          <span className="text-white/38">Auth</span> {run.authMode || "unknown"}
                        </div>
                        <div>
                          <span className="text-white/38">Prompt</span>{" "}
                          {run.promptTemplateArtifactId || "unlinked"}
                        </div>
                        <div>
                          <span className="text-white/38">Execution</span>{" "}
                          {run.promptExecutionArtifactId || "unlinked"}
                        </div>
                        <div>
                          <span className="text-white/38">Parent</span> {run.parentRunId || "root"}
                        </div>
                      </div>
                      {run.childRunIds?.length ? (
                        <div className="mt-4 flex flex-wrap gap-2">
                          {run.childRunIds.map((childId) => (
                            <span
                              key={childId}
                              className="rounded-full border border-white/10 bg-white/[0.03] px-3 py-1 text-xs text-white/70"
                            >
                              {childId}
                            </span>
                          ))}
                        </div>
                      ) : null}
                    </div>
                  ))
                )}
              </div>
            </section>
          )}

          {activeTab === "history" && (
            <section className="space-y-6">
              <div className="grid gap-6 xl:grid-cols-3">
                <div className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
                  <div className="flex items-center gap-2">
                    <KeyRound className="h-4 w-4 text-white/45" />
                  <h2 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Prompt history</h2>
                  </div>
                  <p className="mt-4 text-3xl font-semibold text-white">{lineagePromptCount}</p>
                </div>
                <div className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
                  <div className="flex items-center gap-2">
                    <GitBranch className="h-4 w-4 text-white/45" />
                  <h2 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Feedback</h2>
                  </div>
                  <p className="mt-4 text-3xl font-semibold text-white">{lineageFeedbackCount}</p>
                </div>
                <div className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
                  <div className="flex items-center gap-2">
                    <Layers3 className="h-4 w-4 text-white/45" />
                  <h2 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Legacy groups</h2>
                  </div>
                  <p className="mt-4 text-3xl font-semibold text-white">{lineageLegacyCount}</p>
                </div>
              </div>

              <div className="grid gap-6 xl:grid-cols-2">
                <div className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
                  <h3 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Prompt artifacts</h3>
                  <div className="mt-5 grid gap-3">
                    {spaceSettings?.lineage.promptArtifacts.length ? (
                      spaceSettings.lineage.promptArtifacts.map((artifact) => (
                        <div key={artifact.projection.artifactId} className="rounded-2xl border border-white/6 bg-white/[0.025] px-4 py-4">
                          <p className="text-sm font-semibold text-white/88">{artifact.projection.title}</p>
                          <p className="mt-1 text-xs text-white/45">{artifact.projection.blockType}</p>
                        </div>
                      ))
                    ) : (
                      <p className="text-sm text-white/55">No prompt templates surfaced yet.</p>
                    )}
                  </div>
                </div>

                <div className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
                  <h3 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Feedback artifacts</h3>
                  <div className="mt-5 grid gap-3">
                    {spaceSettings?.lineage.feedbackArtifacts.length ? (
                      spaceSettings.lineage.feedbackArtifacts.map((artifact) => (
                        <div key={artifact.projection.artifactId} className="rounded-2xl border border-white/6 bg-white/[0.025] px-4 py-4">
                          <p className="text-sm font-semibold text-white/88">{artifact.projection.title}</p>
                          <p className="mt-1 text-xs text-white/45">{artifact.projection.blockType}</p>
                        </div>
                      ))
                    ) : (
                      <p className="text-sm text-white/55">No feedback artifacts surfaced yet.</p>
                    )}
                  </div>
                </div>
              </div>

              <div className="rounded-[1.75rem] border border-white/8 bg-white/[0.025] p-6">
                <h3 className="text-xs font-semibold uppercase tracking-[0.28em] text-white/38">Legacy prompt groups</h3>
                <div className="mt-5 grid gap-3">
                  {spaceSettings?.lineage.legacyPromptGroups.length ? (
                    spaceSettings.lineage.legacyPromptGroups.map((group) => (
                      <div key={`${group.title}:${group.contentHash ?? "unknown"}`} className="rounded-2xl border border-white/6 bg-white/[0.025] px-4 py-4">
                        <p className="text-sm font-semibold text-white/88">{group.title}</p>
                        <p className="mt-1 text-xs text-white/45">{group.artifactIds.length} linked artifacts</p>
                      </div>
                    ))
                  ) : (
                    <p className="text-sm text-white/55">No legacy groups surfaced yet.</p>
                  )}
                </div>
              </div>
            </section>
          )}
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
