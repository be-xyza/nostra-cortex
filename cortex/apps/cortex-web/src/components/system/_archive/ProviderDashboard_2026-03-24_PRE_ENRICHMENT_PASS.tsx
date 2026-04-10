import React, { useEffect, useMemo, useState } from "react";
import { Activity, Cpu, Globe, KeyRound, RefreshCw, Server, Shield, Sparkles } from "lucide-react";
import { workbenchApi } from "../../api";
import { useProvidersRegistry } from "../../store/providersRegistry";
import {
  applyProviderTemplate,
  buildTemplateProviderId,
  createEmptyProviderForm,
  providerTypeOrder,
  type ProviderFormState,
  type ProviderType,
} from "./providerForm";
import {
  providerTemplateById,
  providerTemplates,
  resolveProviderTemplate,
  type ProviderTemplateId,
} from "./providerTemplates";
import {
  buildSelectableModelOptions,
  extractModelNames,
} from "./providerModels";
import { buildDiscoveryProviderForm } from "./providerDiscovery";
import {
  formatProviderBatchSummary,
  formatProviderHoverDetails,
  formatProviderLocalityLabel,
  formatProviderModelLabel,
  formatProviderTypeLabel,
} from "./providerTopology";
import type {
  ProviderValidationRequest,
  ProviderValidationResponse,
  SystemLlmAdapterStatusResponse,
} from "../../contracts.ts";

function parseMetadataJson(metadataJson: string): Record<string, string> | undefined {
  const trimmed = metadataJson.trim();
  if (!trimmed || trimmed === "{}") {
    return undefined;
  }

  const parsed = JSON.parse(trimmed) as Record<string, unknown>;
  return Object.fromEntries(
    Object.entries(parsed).map(([key, value]) => [key, typeof value === "string" ? value : JSON.stringify(value)]),
  );
}

function providerTypeStyle(providerType: ProviderType): { border: string; badge: string; accent: string } {
  switch (providerType) {
    case "Llm":
      return {
        border: "border-cyan-400/18",
        badge: "border-cyan-400/20 bg-cyan-400/10 text-cyan-100",
        accent: "text-cyan-200",
      };
    case "Embedding":
      return {
        border: "border-emerald-400/18",
        badge: "border-emerald-400/20 bg-emerald-400/10 text-emerald-100",
        accent: "text-emerald-200",
      };
    case "Vector":
      return {
        border: "border-violet-400/18",
        badge: "border-violet-400/20 bg-violet-400/10 text-violet-100",
        accent: "text-violet-200",
      };
    case "Batch":
      return {
        border: "border-amber-400/18",
        badge: "border-amber-400/20 bg-amber-400/10 text-amber-100",
        accent: "text-amber-200",
      };
    default:
      return {
        border: "border-white/10",
        badge: "border-white/12 bg-white/[0.05] text-white/80",
        accent: "text-white/80",
      };
  }
}

function providerTemplateTone(templateId: ProviderTemplateId): { frame: string; wash: string; glow: string } {
  switch (templateId) {
    case "openrouter":
      return {
        frame: "border-cyan-400/20 hover:border-cyan-300/35",
        wash: "bg-[radial-gradient(circle_at_top_right,rgba(34,211,238,0.16),transparent_58%),linear-gradient(180deg,rgba(255,255,255,0.015),rgba(255,255,255,0.01))]",
        glow: "shadow-[0_18px_48px_-32px_rgba(34,211,238,0.45)]",
      };
    case "openai":
      return {
        frame: "border-sky-400/20 hover:border-sky-300/35",
        wash: "bg-[radial-gradient(circle_at_top_right,rgba(56,189,248,0.16),transparent_58%),linear-gradient(180deg,rgba(255,255,255,0.015),rgba(255,255,255,0.01))]",
        glow: "shadow-[0_18px_48px_-32px_rgba(56,189,248,0.45)]",
      };
    case "ollama":
      return {
        frame: "border-emerald-400/20 hover:border-emerald-300/35",
        wash: "bg-[radial-gradient(circle_at_top_right,rgba(52,211,153,0.16),transparent_58%),linear-gradient(180deg,rgba(255,255,255,0.015),rgba(255,255,255,0.01))]",
        glow: "shadow-[0_18px_48px_-32px_rgba(52,211,153,0.45)]",
      };
    default:
      return {
        frame: "border-amber-400/20 hover:border-amber-300/35",
        wash: "bg-[radial-gradient(circle_at_top_right,rgba(251,191,36,0.14),transparent_58%),linear-gradient(180deg,rgba(255,255,255,0.015),rgba(255,255,255,0.01))]",
        glow: "shadow-[0_18px_48px_-32px_rgba(251,191,36,0.4)]",
      };
  }
}

function providerTypeIcon(providerType: ProviderType) {
  switch (providerType) {
    case "Llm":
      return <Cpu className="h-4 w-4" />;
    case "Embedding":
      return <Globe className="h-4 w-4" />;
    case "Vector":
      return <Server className="h-4 w-4" />;
    case "Batch":
      return <Sparkles className="h-4 w-4" />;
    default:
      return <Activity className="h-4 w-4" />;
  }
}

function providerStatusCopy(status: string, error: string | null): { title: string; body: string } | null {
  switch (status) {
    case "booting":
      return {
        title: "Provider registry is still booting.",
        body: "Refresh after the local shell finishes starting up.",
      };
    case "unavailable":
      return {
        title: "Provider registry is unavailable.",
        body: error ? "The registry endpoint could not be reached." : "The registry endpoint is not available yet.",
      };
    case "error":
      return {
        title: "Unable to load providers.",
        body: "Refresh to try again. If the shell is local, the provider registry may still be starting.",
      };
    default:
      return null;
  }
}

function providerCardEmptyCopy(status: string): string {
  if (status === "empty") {
    return "No providers have been configured yet.";
  }
  return "No providers surfaced yet in this environment.";
}

export const ProviderDashboard: React.FC = () => {
  const {
    providers,
    credentialBindings,
    isLoading,
    status,
    error,
    fetchProviders,
    refreshProviders,
    discoverLocalProviders,
  } = useProvidersRegistry();
  const [selectedProviderId, setSelectedProviderId] = useState<string>("");
  const [form, setForm] = useState<ProviderFormState>(() => createEmptyProviderForm());
  const [validationResult, setValidationResult] = useState<ProviderValidationResponse | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [submitMessage, setSubmitMessage] = useState<string | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [adapterStatus, setAdapterStatus] = useState<SystemLlmAdapterStatusResponse | null>(null);
  const [adapterStatusError, setAdapterStatusError] = useState<string | null>(null);
  const [isRefreshingAdapterModels, setIsRefreshingAdapterModels] = useState(false);

  useEffect(() => {
    if (status === "idle") {
      void fetchProviders();
    }
  }, [fetchProviders, status]);

  useEffect(() => {
    let cancelled = false;

    const loadAdapterStatus = async () => {
      setIsRefreshingAdapterModels(true);
      setAdapterStatusError(null);
      try {
        const nextStatus = await workbenchApi.getSystemLlmAdapterStatus();
        if (cancelled) {
          return;
        }
        setAdapterStatus(nextStatus);
        setAdapterStatusError(nextStatus.upstreamModelsError ?? nextStatus.openapiError ?? nextStatus.adapterHealthError ?? null);
      } catch (err) {
        if (cancelled) {
          return;
        }
        setAdapterStatus(null);
        setAdapterStatusError(err instanceof Error ? err.message : String(err));
      } finally {
        if (!cancelled) {
          setIsRefreshingAdapterModels(false);
        }
      }
    };

    void loadAdapterStatus();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (selectedProviderId === "" && providers.length > 0) {
      setSelectedProviderId(providers[0]?.id ?? "__new__");
    }
  }, [providers, selectedProviderId]);

  useEffect(() => {
    if (selectedProviderId && selectedProviderId !== "__new__" && !providers.some((provider) => provider.id === selectedProviderId)) {
      setSelectedProviderId("__new__");
    }
  }, [providers, selectedProviderId]);

  const selectedProvider = useMemo(
    () => (selectedProviderId && selectedProviderId !== "__new__" ? providers.find((provider) => provider.id === selectedProviderId) ?? null : null),
    [providers, selectedProviderId],
  );

  useEffect(() => {
    if (!selectedProvider) {
      setForm(createEmptyProviderForm());
      setValidationResult(null);
      return;
    }
    const template = resolveProviderTemplate(selectedProvider.llmType, selectedProvider.endpoint);

    setForm({
      templateId: template.id,
      providerId: selectedProvider.id,
      providerType: selectedProvider.providerType,
      providerKind: selectedProvider.llmType ?? template.providerKind ?? "",
      name: selectedProvider.name,
      endpoint: selectedProvider.endpoint,
      defaultModel: selectedProvider.defaultModel ?? "",
      adapterSetRef: selectedProvider.adapterSetRef ?? "",
      credentialBindingId: selectedProvider.credentialBindingId ?? "",
      apiKey: "",
      metadataJson:
        selectedProvider.metadata && Object.keys(selectedProvider.metadata).length > 0
          ? JSON.stringify(selectedProvider.metadata, null, 2)
          : "{\n  \n}",
      enabled: selectedProvider.isActive,
    });
    setValidationResult(null);
  }, [selectedProvider]);

  const providerGroups = useMemo(() => {
    const grouped = new Map<ProviderType, typeof providers>();
    providerTypeOrder.forEach((providerType) => {
      grouped.set(providerType, []);
    });

    for (const provider of providers) {
      const key = provider.providerType as ProviderType;
      const bucket = grouped.get(key) ?? [];
      bucket.push(provider);
      grouped.set(key, bucket);
    }

    return providerTypeOrder
      .map((providerType) => ({
        providerType,
        providers: (grouped.get(providerType) ?? []).slice().sort((left, right) =>
          left.name.localeCompare(right.name),
        ),
      }))
      .filter((group) => group.providers.length > 0);
  }, [providers]);

  const statusCopy = providerStatusCopy(status, error);
  const registryIsEmpty = status === "empty" && providers.length === 0;
  const providerIds = useMemo(() => providers.map((provider) => provider.id), [providers]);
  const visibleCredentialBindings = selectedProvider
    ? credentialBindings.filter((binding) => binding.providerId === selectedProvider.id)
    : credentialBindings;
  const selectedProviderCredentialBinding = selectedProvider
    ? credentialBindings.find((binding) => binding.credentialBindingId === selectedProvider.credentialBindingId) ?? null
    : null;
  const discoveredModelOptions = buildSelectableModelOptions(
    form.defaultModel,
    validationResult?.supportedModels,
    selectedProvider?.supportedModels,
    extractModelNames(adapterStatus?.upstreamModels),
    adapterStatus?.model ? [adapterStatus.model] : undefined,
    selectedProvider?.defaultModel ? [selectedProvider.defaultModel] : undefined,
  );
  const modelSelectOptions = discoveredModelOptions;
  const discoveryModelOptions = discoveredModelOptions;
  const discoverySelectedModel = form.defaultModel.trim() || adapterStatus?.model?.trim() || discoveryModelOptions[0] || "";
  const discoveryBaseUrl = adapterStatus?.baseUrl?.trim() || providerTemplateById(form.templateId).endpointHint;
  const discoveryCatalogDescription = adapterStatus
    ? adapterStatus.enabled
      ? `Configured adapter at ${discoveryBaseUrl}`
      : `Adapter discovery is disabled; current runtime reports ${discoveryBaseUrl}`
    : "Discovery is not available until the adapter status endpoint responds.";
  const modelInputId = "provider-default-model-options";

  const buildNewProviderForm = (templateId: ProviderTemplateId): ProviderFormState => {
    const baseForm = applyProviderTemplate(templateId, createEmptyProviderForm());
    return {
      ...baseForm,
      providerId: buildTemplateProviderId(templateId, providerIds),
    };
  };

  const handleProviderSelection = (nextProviderId: string) => {
    setSelectedProviderId(nextProviderId === "__new__" ? "__new__" : nextProviderId);
    if (nextProviderId === "__new__") {
      setForm(buildNewProviderForm(form.templateId));
      setValidationResult(null);
      return;
    }

    const nextProvider = providers.find((provider) => provider.id === nextProviderId);
    if (!nextProvider) {
      setSelectedProviderId("__new__");
      setForm(createEmptyProviderForm());
      setValidationResult(null);
    }
  };

  const handleDiscoverLocalProviders = async () => {
    await discoverLocalProviders();
    try {
      setIsRefreshingAdapterModels(true);
      const nextStatus = await workbenchApi.getSystemLlmAdapterStatus();
      setAdapterStatus(nextStatus);
      setAdapterStatusError(nextStatus.upstreamModelsError ?? nextStatus.openapiError ?? nextStatus.adapterHealthError ?? null);
    } catch (err) {
      setAdapterStatusError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsRefreshingAdapterModels(false);
    }
  };

  const handleStartProviderFromDiscovery = () => {
    setSelectedProviderId("__new__");
    setForm(buildDiscoveryProviderForm(adapterStatus, discoverySelectedModel, providerIds));
    setValidationResult(null);
    setSubmitError(null);
    setSubmitMessage(null);
  };

  const handleTemplateSelection = (nextTemplateId: ProviderTemplateId) => {
    setSelectedProviderId("__new__");
    setForm(buildNewProviderForm(nextTemplateId));
    setValidationResult(null);
  };

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setSubmitError(null);
    setSubmitMessage(null);

    const template = providerTemplateById(form.templateId);
    const providerId = form.providerId.trim();
    const providerName = form.name.trim();
    const providerEndpoint = form.endpoint.trim();
    const providerKind = form.providerKind.trim() || template.providerKind || selectedProvider?.llmType || undefined;
    const providerMetadata = (() => {
      try {
        return parseMetadataJson(form.metadataJson);
      } catch {
        setSubmitError("Advanced details must be valid JSON.");
        return null;
      }
    })();

    if (!providerId || !providerName || !providerEndpoint) {
      setSubmitError("Provider ID, name, and endpoint are required.");
      return;
    }
    if (providerMetadata === null) {
      return;
    }
    if (selectedProviderId === "__new__" && !form.apiKey.trim()) {
      setSubmitError("API key is required to add a new provider.");
      return;
    }

    setIsSubmitting(true);
    try {
      let probeResult: ProviderValidationResponse | null = null;
      if (form.apiKey.trim()) {
        setIsValidating(true);
        probeResult = await workbenchApi.validateSystemProvider({
          providerType: form.providerType,
          providerKind: providerKind as ProviderValidationRequest["providerKind"],
          baseUrl: providerEndpoint,
          defaultModel: form.defaultModel.trim() || undefined,
          apiKey: form.apiKey.trim(),
          validateKey: template.validateKey,
          validateChat: form.providerType === "Llm" && template.validateChat,
          validateEmbeddings: form.providerType === "Embedding" && template.validateEmbeddings,
        });
        setValidationResult(probeResult);
        if (!probeResult.valid) {
          setSubmitError("Provider validation failed. Review the endpoint, key, and discovered models.");
          return;
        }
      }

      const supportedModels = probeResult?.supportedModels ?? selectedProvider?.supportedModels ?? [];
      const nextDefaultModel =
        form.defaultModel.trim() ||
        probeResult?.selectedModel ||
        selectedProvider?.defaultModel ||
        undefined;

      await workbenchApi.putSystemProvider(providerId, {
        name: providerName,
        endpoint: providerEndpoint,
        enabled: form.enabled,
        providerType: form.providerType,
        defaultModel: nextDefaultModel,
        providerKind,
        adapterSetRef: form.adapterSetRef.trim() || undefined,
        credentialBindingId: form.credentialBindingId.trim() || undefined,
        supportedModels,
        metadata: providerMetadata,
      });

      if (form.apiKey.trim()) {
        await workbenchApi.createSystemProviderCredential({
          providerId,
          label: providerName || providerId,
          apiKey: form.apiKey.trim(),
          metadata: providerMetadata,
        });
      }

      setSelectedProviderId(providerId);
      await fetchProviders();
      setSubmitMessage(
        form.apiKey.trim()
          ? "Provider validated and saved."
          : "Provider saved.",
      );
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsValidating(false);
      setIsSubmitting(false);
    }
  };

  return (
    <div className="provider-dashboard animate-in fade-in duration-500 p-6">
      <header className="mb-8 flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div className="max-w-3xl">
          <p className="text-xs font-semibold uppercase tracking-[0.28em] text-ink-faint">Execution surface</p>
          <h1 className="mb-2 text-2xl font-bold tracking-tight text-ink">Providers</h1>
          <p className="max-w-2xl text-sm text-ink-muted">
            Configure provider runtimes, manage credentials, and see which records are local, tunneled, or cloud
            without surfacing secrets.
          </p>
        </div>
        <div className="flex flex-wrap gap-3">
          <button
            type="button"
            onClick={() => void refreshProviders()}
            disabled={isLoading}
            className="flex items-center gap-2 rounded-lg border border-border-subtle bg-surface-elevated px-4 py-2 text-sm font-semibold transition-all hover:bg-opacity-80 disabled:cursor-not-allowed disabled:opacity-60"
          >
            <RefreshCw className={`h-4 w-4 ${isLoading && status === "loading" ? "animate-spin" : ""}`} />
            Refresh
          </button>
        </div>
      </header>

      {statusCopy ? (
        <div className="error-banner mb-6 flex items-start gap-3">
          <Shield className="mt-0.5 h-5 w-5 text-bad" />
          <div className="space-y-1">
            <p className="font-medium text-bad">{statusCopy.title}</p>
            <p className="text-sm text-ink-muted">{statusCopy.body}</p>
            {error ? (
              <details className="text-xs text-ink-faint">
                <summary className="cursor-pointer">Technical details</summary>
                <p className="mt-2 font-mono">{error}</p>
              </details>
            ) : null}
          </div>
        </div>
      ) : null}

      <div className="mb-8">
        <form onSubmit={handleSubmit} className="panel">
          <div className="panel-head flex items-center justify-between gap-4">
            <div className="flex items-center gap-2">
              <KeyRound className="h-4 w-4" />
              {selectedProvider ? `Editing ${selectedProvider.name}` : "Create provider"}
            </div>
            <span className="rounded-full border border-border-subtle px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-ink-muted">
              {selectedProvider ? "Selected from cards" : "Template composer"}
            </span>
          </div>

          <div className="panel-body grid gap-5">
            <div className="grid gap-4 md:grid-cols-2">
              {providerTemplates.map((template) => {
                const isSelected = form.templateId === template.id;
                const tone = providerTemplateTone(template.id);
                return (
                  <button
                    key={template.id}
                    type="button"
                    onClick={() => handleTemplateSelection(template.id)}
                    className={[
                      "group relative overflow-hidden rounded-3xl border px-5 py-5 text-left transition-all duration-300",
                      "min-h-[224px] flex flex-col",
                      isSelected
                        ? `${tone.frame} ${tone.wash} ${tone.glow}`
                        : `border-border-subtle bg-surface-elevated/70 hover:bg-surface-elevated ${tone.frame}`,
                    ].join(" ")}
                  >
                    <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-white/20 to-transparent opacity-80" />
                    <div className="mb-4 flex items-start justify-between gap-4">
                      <div className="min-w-0">
                        <p className="text-sm font-semibold tracking-tight text-ink">{template.label}</p>
                        <p className="mt-1 text-[10px] font-semibold uppercase tracking-[0.22em] text-ink-muted">
                          {template.providerKind || "Manual"}
                        </p>
                      </div>
                      <span className="shrink-0 rounded-full border border-border-subtle/80 bg-black/10 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-ink-muted">
                        {template.providerKind || "manual"}
                      </span>
                    </div>
                    <p className="text-sm leading-6 text-ink-muted">{template.description}</p>
                    <div className="mt-5 flex flex-wrap gap-2">
                      <span className="rounded-full border border-border-subtle bg-black/10 px-2.5 py-1 text-[10px] text-ink-muted">
                        {template.validateKey ? "Key probe" : "No key probe"}
                      </span>
                      <span className="rounded-full border border-border-subtle bg-black/10 px-2.5 py-1 text-[10px] text-ink-muted">
                        {template.validateChat ? "Chat" : "No chat"}
                      </span>
                      <span className="rounded-full border border-border-subtle bg-black/10 px-2.5 py-1 text-[10px] text-ink-muted">
                        {template.validateEmbeddings ? "Embeddings" : "No embeddings"}
                      </span>
                    </div>
                    <div className="mt-auto pt-5 text-[11px] leading-5 text-ink-faint">
                      <span className="block uppercase tracking-[0.2em] text-ink-muted">Endpoint hint</span>
                      <span className="mt-1 block break-all">{template.endpointHint}</span>
                    </div>
                  </button>
                );
              })}
            </div>

            <div className="rounded-2xl border border-border-subtle bg-surface-elevated/70 px-4 py-4">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <p className="text-xs font-semibold uppercase tracking-[0.2em] text-ink-faint">Selected template</p>
                  <p className="mt-1 text-sm text-ink">{providerTemplateById(form.templateId).label}</p>
                </div>
                <p className="text-xs text-ink-muted">{providerTemplateById(form.templateId).endpointHint}</p>
              </div>
              <p className="mt-2 text-xs leading-5 text-ink-muted">
                {providerTemplateById(form.templateId).description}
              </p>
            </div>

            {validationResult ? (
              <div
                className={[
                  "rounded-2xl border px-4 py-4",
                  validationResult.valid
                    ? "border-ok/30 bg-ok/8"
                    : "border-amber-400/30 bg-amber-400/8",
                ].join(" ")}
              >
                <div className="flex items-center justify-between gap-3">
                  <div>
                    <p className="text-xs font-semibold uppercase tracking-[0.2em] text-ink-faint">
                      Validation
                    </p>
                    <p className="mt-1 text-sm text-ink">
                      {validationResult.valid ? "Provider is ready to save." : "Validation needs attention."}
                    </p>
                  </div>
                  <span className={`rounded-full px-2 py-1 text-[10px] font-semibold ${validationResult.valid ? "text-ok" : "text-bad"}`}>
                    {validationResult.valid ? "VALID" : "INVALID"}
                  </span>
                </div>
                <div className="mt-3 flex flex-wrap gap-2 text-[11px] text-ink-muted">
                  {validationResult.supportedModels.slice(0, 4).map((model) => (
                    <span key={model} className="rounded-full border border-border-subtle px-2 py-1">
                      {model}
                    </span>
                  ))}
                  {validationResult.supportedModels.length > 4 ? (
                    <span className="rounded-full border border-border-subtle px-2 py-1">
                      +{validationResult.supportedModels.length - 4} more
                    </span>
                  ) : null}
                </div>
                {(validationResult.keyError || validationResult.modelsError || validationResult.chatError || validationResult.embeddingsError) ? (
                  <details className="mt-3 text-xs text-ink-faint">
                    <summary className="cursor-pointer select-none">Validation details</summary>
                    <div className="mt-2 grid gap-1">
                      {validationResult.keyError ? <p>Key: {validationResult.keyError}</p> : null}
                      {validationResult.modelsError ? <p>Models: {validationResult.modelsError}</p> : null}
                      {validationResult.chatError ? <p>Chat: {validationResult.chatError}</p> : null}
                      {validationResult.embeddingsError ? <p>Embeddings: {validationResult.embeddingsError}</p> : null}
                    </div>
                  </details>
                ) : null}
              </div>
            ) : null}

                <div className="grid gap-4 md:grid-cols-2">
                  <label className="grid gap-2 text-sm">
                    <span className="text-ink-muted">Name</span>
                    <input
                      value={form.name}
                  onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
                  placeholder="Primary OpenRouter Provider"
                  className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                />
                  </label>
                  <label className="grid gap-2 text-sm">
                    <span className="text-ink-muted">API key</span>
                    <input
                      value={form.apiKey}
                      onChange={(event) => setForm((current) => ({ ...current, apiKey: event.target.value }))}
                      type="password"
                      placeholder="sk-..."
                  className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                />
              </label>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="grid gap-2 text-sm">
                <span className="text-ink-muted">{providerTemplateById(form.templateId).endpointLabel}</span>
                <input
                  value={form.endpoint}
                  onChange={(event) => setForm((current) => ({ ...current, endpoint: event.target.value }))}
                  placeholder={providerTemplateById(form.templateId).endpointHint}
                  className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                />
              </label>
              <label className="grid gap-2 text-sm">
                <span className="text-ink-muted">Default model</span>
                <input
                  value={form.defaultModel}
                  onChange={(event) => setForm((current) => ({ ...current, defaultModel: event.target.value }))}
                  list={modelSelectOptions.length > 0 ? modelInputId : undefined}
                  autoComplete="off"
                  spellCheck={false}
                  placeholder={
                    modelSelectOptions.length > 0
                      ? isRefreshingAdapterModels
                        ? "Refreshing models..."
                        : "Type or choose a model"
                      : providerTemplateById(form.templateId).defaultModelHint
                  }
                  className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                />
                {modelSelectOptions.length > 0 ? (
                  <datalist id={modelInputId}>
                    {modelSelectOptions.map((model) => (
                      <option key={model} value={model} />
                    ))}
                  </datalist>
                ) : null}
                <p className="text-xs leading-5 text-ink-faint">
                  {modelSelectOptions.length > 0
                    ? "Type to filter or use keyboard suggestions to pick a discovered model."
                    : adapterStatusError
                      ? `Model discovery is currently limited: ${adapterStatusError}`
                      : "Validate the provider or refresh local discovery to populate available models."}
                </p>
              </label>
            </div>

            <label className="inline-flex items-center gap-3 rounded-xl border border-border-subtle bg-surface-elevated px-4 py-3 text-sm text-ink-muted">
              <input
                type="checkbox"
                checked={form.enabled}
                onChange={(event) => setForm((current) => ({ ...current, enabled: event.target.checked }))}
                className="h-4 w-4 rounded border-border-subtle bg-surface-elevated"
              />
              Enable this provider
            </label>

            {selectedProvider ? (
              <div className="rounded-2xl border border-dashed border-border-strong px-4 py-4 text-sm text-ink-muted">
                Provider details, discovery, routing, and linked credentials now open from the selected provider card below.
              </div>
            ) : (
              <details className="rounded-2xl border border-border-subtle bg-surface-elevated px-4 py-3">
                <summary className="cursor-pointer select-none text-sm font-medium text-ink">
                  Additional settings for this draft
                </summary>
                <p className="mt-2 text-xs leading-5 text-ink-faint">
                  Use these fields when you need custom provider internals, routing links, discovery, or metadata before saving.
                </p>
                <div className="mt-4 grid gap-4">
                  <div className="grid gap-4 md:grid-cols-2">
                    <label className="grid gap-2 text-sm">
                      <span className="text-ink-muted">Internal provider ID</span>
                      <input
                        value={form.providerId}
                        onChange={(event) => setForm((current) => ({ ...current, providerId: event.target.value }))}
                        placeholder={`${form.templateId}_provider`}
                        className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                      />
                      <p className="text-xs leading-5 text-ink-faint">
                        Seeded from the template id and de-duplicated automatically when the same template is used again.
                      </p>
                    </label>
                    <label className="grid gap-2 text-sm">
                      <span className="text-ink-muted">Provider type</span>
                      <select
                        value={form.providerType}
                        onChange={(event) =>
                          setForm((current) => ({
                            ...current,
                            providerType: event.target.value as ProviderType,
                          }))
                        }
                        className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink"
                      >
                        {providerTypeOrder.map((providerType) => (
                          <option key={providerType} value={providerType}>
                            {formatProviderTypeLabel(providerType)}
                          </option>
                        ))}
                      </select>
                    </label>
                  </div>

                  <div className="grid gap-4 md:grid-cols-2">
                    <label className="grid gap-2 text-sm">
                      <span className="text-ink-muted">Adapter set</span>
                      <input
                        value={form.adapterSetRef}
                        onChange={(event) => setForm((current) => ({ ...current, adapterSetRef: event.target.value }))}
                        placeholder="adapter.default"
                        className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                      />
                    </label>
                    <label className="grid gap-2 text-sm">
                      <span className="text-ink-muted">Linked credential</span>
                      <select
                        value={form.credentialBindingId}
                        onChange={(event) =>
                          setForm((current) => ({ ...current, credentialBindingId: event.target.value }))
                        }
                        className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink"
                      >
                        <option value="">Use system default</option>
                        {visibleCredentialBindings.map((binding) => (
                          <option key={binding.credentialBindingId} value={binding.credentialBindingId}>
                            {binding.label || binding.credentialBindingId}
                          </option>
                        ))}
                      </select>
                    </label>
                  </div>

                  <details className="rounded-xl border border-border-subtle bg-black/8 px-4 py-3">
                    <summary className="cursor-pointer select-none text-sm font-medium text-ink">
                      Discovery
                    </summary>
                    <div className="mt-4 grid gap-4">
                      <div className="flex flex-wrap items-start justify-between gap-4">
                        <div className="space-y-2">
                          <p className="text-sm font-semibold text-ink">
                            Refresh the adapter catalog, then seed a provider from it.
                          </p>
                          <p className="max-w-xl text-xs leading-5 text-ink-muted">
                            Discovery is adapter-bound and updates the available model list, but it does not save anything until you explicitly start a provider.
                          </p>
                        </div>
                        <button
                          type="button"
                          onClick={() => void handleDiscoverLocalProviders()}
                          disabled={isLoading || isRefreshingAdapterModels}
                          className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-xs font-semibold transition hover:bg-opacity-80 disabled:cursor-not-allowed disabled:opacity-60"
                        >
                          {isRefreshingAdapterModels ? "Refreshing..." : "Refresh catalog"}
                        </button>
                      </div>
                      <p className="text-xs leading-5 text-ink-muted">{discoveryCatalogDescription}</p>
                      <div className="flex flex-wrap items-center gap-3">
                        <button
                          type="button"
                          onClick={handleStartProviderFromDiscovery}
                          disabled={!adapterStatus && discoveryModelOptions.length === 0}
                          className="rounded-lg bg-ink px-4 py-2 text-xs font-semibold text-surface transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
                        >
                          Start provider from discovery
                        </button>
                        <span className="text-xs text-ink-faint">
                          {discoveryModelOptions.length > 0
                            ? `${discoveryModelOptions.length} models available`
                            : "No models discovered yet"}
                        </span>
                      </div>
                    </div>
                  </details>

                  <details className="rounded-xl border border-border-subtle bg-black/8 px-4 py-3">
                    <summary className="cursor-pointer select-none text-sm font-medium text-ink">
                      Metadata
                    </summary>
                    <textarea
                      value={form.metadataJson}
                      onChange={(event) => setForm((current) => ({ ...current, metadataJson: event.target.value }))}
                      rows={6}
                      className="mt-3 w-full rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 font-mono text-xs text-ink placeholder:text-ink-faint"
                    />
                  </details>
                </div>
              </details>
            )}

            <div className="flex items-center justify-between gap-3">
              <div className="min-h-5 text-sm">
                {submitError ? <span className="text-bad">{submitError}</span> : null}
                {submitMessage ? <span className="text-ok">{submitMessage}</span> : null}
              </div>
              <div className="flex items-center gap-2">
                  <button
                  type="button"
                  onClick={() => {
                    setSelectedProviderId("__new__");
                    setForm(buildNewProviderForm(form.templateId));
                    setSubmitError(null);
                    setSubmitMessage(null);
                    setValidationResult(null);
                  }}
                  className="rounded-lg border border-border-subtle bg-surface-elevated px-4 py-2 text-sm font-semibold text-ink transition hover:bg-opacity-80"
                >
                  New custom provider
                </button>
                <button
                  type="submit"
                  disabled={isSubmitting || isValidating}
                  className="rounded-lg bg-ink px-4 py-2 text-sm font-semibold text-surface transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
                >
                  {isValidating ? "Validating..." : isSubmitting ? "Saving..." : "Validate & save provider"}
                </button>
              </div>
            </div>
          </div>
        </form>
      </div>

      <div className="space-y-6">
        {providerGroups.map(({ providerType, providers: groupProviders }) => {
          const tone = providerTypeStyle(providerType);
          return (
            <section key={providerType} className={`rounded-[1.5rem] border ${tone.border} bg-white/[0.02] shadow-[0_18px_70px_-42px_rgba(0,0,0,0.8)]`}>
              <div className="flex items-center justify-between gap-4 border-b border-white/8 px-5 py-4">
                <div className="flex items-center gap-3">
                  <div className={`flex h-10 w-10 items-center justify-center rounded-2xl border ${tone.border} bg-white/[0.02] ${tone.accent}`}>
                    {providerTypeIcon(providerType)}
                  </div>
                  <div>
                    <h2 className="text-sm font-semibold uppercase tracking-[0.24em] text-ink-faint">
                      {formatProviderTypeLabel(providerType)}
                    </h2>
                    <p className="text-xs text-ink-muted">
                      {groupProviders.length} {groupProviders.length === 1 ? "provider" : "providers"}
                    </p>
                  </div>
                </div>
                <span className={`rounded-full border ${tone.badge} px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.16em]`}>
                  {formatProviderTypeLabel(providerType)}
                </span>
              </div>

              <div className="grid grid-cols-1 gap-4 px-5 py-5 md:grid-cols-2 xl:grid-cols-3">
                {groupProviders.map((provider) => {
                  const providerModelCount = (provider.supportedModels ?? []).length;
                  return (
                  <article
                    key={provider.id}
                    role="button"
                    tabIndex={0}
                    onClick={() => handleProviderSelection(provider.id)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        handleProviderSelection(provider.id);
                      }
                    }}
                    className={[
                      "panel group cursor-pointer transition-all duration-300 hover:border-accent/30",
                      selectedProviderId === provider.id ? "border-accent/40 ring-1 ring-accent/20" : "",
                    ].join(" ")}
                    title={formatProviderHoverDetails(provider)}
                  >
                    <div className="panel-head flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        {providerTypeIcon(provider.providerType as ProviderType)}
                        <span>{provider.name}</span>
                      </div>
                      <div
                        className={`h-2 w-2 rounded-full ${provider.isActive ? "bg-ok shadow-[0_0_8px_var(--ok)]" : "bg-bad opacity-50"}`}
                      />
                    </div>
                    <div className="panel-body space-y-4">
                      <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
                        <div className="flex flex-wrap gap-2">
                          <span className={`rounded-full border ${tone.badge} px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.16em]`}>
                            {formatProviderTypeLabel(provider.providerType)}
                          </span>
                          {provider.llmType ? (
                            <span className="rounded-full border border-border-strong px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.16em] text-ink-muted">
                              {provider.llmType}
                            </span>
                          ) : null}
                        </div>
                        <span className="rounded-full border border-border-strong px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.16em] text-ink-muted">
                          {formatProviderLocalityLabel(provider)}
                        </span>
                      </div>

                      <div className="metric">
                        <span>Endpoint</span>
                        <code className="mt-1 block break-all text-xs opacity-70">{provider.endpoint}</code>
                      </div>

                      <div className="grid gap-2 text-xs">
                        <div className="flex items-center justify-between">
                          <span className="text-ink-muted">Default model</span>
                          <span className="text-ink">{formatProviderModelLabel(provider)}</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className="text-ink-muted">Credential</span>
                          <span className="text-ink">{provider.hasCredential ? "Bound" : "Unbound"}</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className="text-ink-muted">Active</span>
                          <span className="text-ink">{provider.isActive ? "Enabled" : "Disabled"}</span>
                        </div>
                      </div>

                      <div className="flex flex-wrap gap-2">
                        {providerModelCount > 0 ? (
                          <span className="rounded-full border border-border-subtle px-2 py-0.5 text-[10px] font-medium text-ink-muted">
                            {`${providerModelCount} models`}
                          </span>
                        ) : null}
                        {provider.batchPolicy ? (
                          <span className="rounded-full border border-border-subtle px-2 py-0.5 text-[10px] font-medium text-ink-muted">
                            {formatProviderBatchSummary(provider.batchPolicy)}
                          </span>
                        ) : null}
                      </div>

                      {selectedProviderId === provider.id ? (
                        <div className="space-y-3 rounded-2xl border border-accent/25 bg-accent/5 px-4 py-4">
                          <div className="flex flex-wrap items-start justify-between gap-3">
                            <div>
                              <p className="text-[10px] font-semibold uppercase tracking-[0.2em] text-ink-faint">Provider details</p>
                              <p className="mt-1 text-sm font-semibold text-ink">{provider.name}</p>
                              <p className="mt-1 text-xs text-ink-muted">
                                {selectedProviderCredentialBinding?.label || provider.credentialBindingId || "System default"}
                              </p>
                            </div>
                            <span className="rounded-full border border-border-subtle px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-ink-muted">
                              Editing in composer
                            </span>
                          </div>

                          <div className="grid gap-2 text-xs">
                            <div className="flex items-center justify-between gap-3">
                              <span className="text-ink-muted">Internal ID</span>
                              <span className="text-right text-ink">{provider.id}</span>
                            </div>
                            <div className="flex items-center justify-between gap-3">
                              <span className="text-ink-muted">Adapter set</span>
                              <span className="text-right text-ink">{provider.adapterSetRef || "Not bound"}</span>
                            </div>
                            <div className="flex items-center justify-between gap-3">
                              <span className="text-ink-muted">Locality</span>
                              <span className="text-right text-ink">{formatProviderLocalityLabel(provider)}</span>
                            </div>
                            <div className="flex items-center justify-between gap-3">
                              <span className="text-ink-muted">Credential state</span>
                              <span className="text-right text-ink">{provider.hasCredential ? "Bound" : "Uses system default"}</span>
                            </div>
                            {provider.topology?.familyId ? (
                              <div className="flex items-center justify-between gap-3">
                                <span className="text-ink-muted">Family</span>
                                <span className="text-right text-ink">{provider.topology.familyId}</span>
                              </div>
                            ) : null}
                            {provider.topology?.instanceId ? (
                              <div className="flex items-center justify-between gap-3">
                                <span className="text-ink-muted">Instance</span>
                                <span className="text-right text-ink">{provider.topology.instanceId}</span>
                              </div>
                            ) : null}
                          </div>

                          <details className="rounded-xl border border-border-subtle/70 bg-black/10 px-3 py-3">
                            <summary className="cursor-pointer select-none text-[11px] font-semibold uppercase tracking-[0.18em] text-ink-muted">
                              Usage & credentials
                            </summary>
                            <div className="mt-3 grid gap-3 text-xs">
                              <div className="flex items-center justify-between gap-3">
                                <span className="text-ink-muted">Linked credential</span>
                                <span className="text-right text-ink">
                                  {selectedProviderCredentialBinding?.label || provider.credentialBindingId || "System default"}
                                </span>
                              </div>
                              <div className="flex items-center justify-between gap-3">
                                <span className="text-ink-muted">Usage source</span>
                                <span className="text-right text-ink">
                                  {selectedProviderCredentialBinding?.source || "system"}
                                </span>
                              </div>
                              {visibleCredentialBindings.length > 0 ? (
                                <div className="grid gap-2">
                                  {visibleCredentialBindings.map((binding) => (
                                    <div key={binding.credentialBindingId} className="rounded-xl border border-border-subtle/70 bg-black/10 px-3 py-3">
                                      <div className="flex items-start justify-between gap-3">
                                        <div>
                                          <p className="text-sm font-semibold text-ink">{binding.label || binding.credentialBindingId}</p>
                                          <p className="mt-1 text-[11px] text-ink-muted">{binding.source || "system"}</p>
                                        </div>
                                        <span className={`rounded-full px-2 py-1 text-[10px] font-semibold ${binding.hasCredential ? "text-ok" : "text-ink-muted"}`}>
                                          {binding.hasCredential ? "AVAILABLE" : "Record only"}
                                        </span>
                                      </div>
                                    </div>
                                  ))}
                                </div>
                              ) : (
                                <p className="text-ink-faint">No linked credential records are registered for this provider yet.</p>
                              )}
                            </div>
                          </details>

                          <details className="rounded-xl border border-border-subtle/70 bg-black/10 px-3 py-3">
                            <summary className="cursor-pointer select-none text-[11px] font-semibold uppercase tracking-[0.18em] text-ink-muted">
                              Discovery & models
                            </summary>
                            <div className="mt-3 grid gap-3">
                              <div className="grid gap-3 sm:grid-cols-3">
                                <div className="rounded-xl border border-border-subtle/70 bg-black/10 px-3 py-3">
                                  <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-ink-faint">Base URL</p>
                                  <p className="mt-2 break-all text-xs text-ink">{discoveryBaseUrl}</p>
                                </div>
                                <div className="rounded-xl border border-border-subtle/70 bg-black/10 px-3 py-3">
                                  <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-ink-faint">Model count</p>
                                  <p className="mt-2 text-xs text-ink">
                                    {discoveryModelOptions.length > 0 ? `${discoveryModelOptions.length} available` : "No models discovered"}
                                  </p>
                                </div>
                                <div className="rounded-xl border border-border-subtle/70 bg-black/10 px-3 py-3">
                                  <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-ink-faint">Adapter status</p>
                                  <p className="mt-2 text-xs text-ink">{adapterStatus?.enabled ? "Connected" : "Idle"}</p>
                                </div>
                              </div>
                              <p className="text-xs leading-5 text-ink-muted">{discoveryCatalogDescription}</p>
                              {adapterStatusError ? (
                                <p className="text-xs text-amber-200">Discovery note: {adapterStatusError}</p>
                              ) : null}
                              {discoveryModelOptions.length > 0 ? (
                                <details className="rounded-xl border border-border-subtle/70 bg-black/10 px-3 py-3">
                                  <summary className="cursor-pointer select-none text-[11px] font-semibold uppercase tracking-[0.18em] text-ink-muted">
                                    View models
                                  </summary>
                                  <div className="mt-3 flex flex-wrap gap-2">
                                    {discoveryModelOptions.map((model) => (
                                      <button
                                        key={model}
                                        type="button"
                                        onClick={() => setForm((current) => ({ ...current, defaultModel: model }))}
                                        className={[
                                          "rounded-full border px-3 py-1 text-[11px] font-medium transition",
                                          form.defaultModel === model
                                            ? "border-accent bg-accent/10 text-ink"
                                            : "border-border-subtle bg-black/10 text-ink-muted hover:border-border-strong",
                                        ].join(" ")}
                                      >
                                        {model}
                                      </button>
                                    ))}
                                  </div>
                                </details>
                              ) : null}
                              <div className="flex flex-wrap items-center gap-3">
                                <button
                                  type="button"
                                  onClick={() => void handleDiscoverLocalProviders()}
                                  disabled={isLoading || isRefreshingAdapterModels}
                                  className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-xs font-semibold transition hover:bg-opacity-80 disabled:cursor-not-allowed disabled:opacity-60"
                                >
                                  {isRefreshingAdapterModels ? "Refreshing..." : "Refresh models"}
                                </button>
                                <button
                                  type="button"
                                  onClick={handleStartProviderFromDiscovery}
                                  disabled={!adapterStatus && discoveryModelOptions.length === 0}
                                  className="rounded-lg bg-ink px-4 py-2 text-xs font-semibold text-surface transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
                                >
                                  Start provider from discovery
                                </button>
                              </div>
                            </div>
                          </details>

                          <details className="rounded-xl border border-border-subtle/70 bg-black/10 px-3 py-3">
                            <summary className="cursor-pointer select-none text-[11px] font-semibold uppercase tracking-[0.18em] text-ink-muted">
                              Advanced configuration
                            </summary>
                            <div className="mt-3 grid gap-4 md:grid-cols-2">
                              <div className="grid gap-2 text-xs">
                                <div className="flex items-center justify-between gap-3">
                                  <span className="text-ink-muted">Provider type</span>
                                  <span className="text-right text-ink">{formatProviderTypeLabel(provider.providerType)}</span>
                                </div>
                                <div className="flex items-center justify-between gap-3">
                                  <span className="text-ink-muted">Adapter set</span>
                                  <span className="text-right text-ink">{provider.adapterSetRef || "Not bound"}</span>
                                </div>
                              </div>
                              <div className="grid gap-2 text-xs">
                                <div className="flex items-center justify-between gap-3">
                                  <span className="text-ink-muted">Composer metadata</span>
                                  <span className="text-right text-ink">
                                    {form.metadataJson.trim() && form.metadataJson.trim() !== "{}" ? "Present" : "None"}
                                  </span>
                                </div>
                                {provider.batchPolicy ? (
                                  <div className="flex items-center justify-between gap-3">
                                    <span className="text-ink-muted">Batch policy</span>
                                    <span className="text-right text-ink">{formatProviderBatchSummary(provider.batchPolicy)}</span>
                                  </div>
                                ) : null}
                              </div>
                            </div>
                            {provider.metadata && Object.keys(provider.metadata).length > 0 ? (
                              <details className="mt-3 text-xs text-ink-faint">
                                <summary className="cursor-pointer">View metadata</summary>
                                <div className="mt-2 grid gap-1">
                                  {Object.entries(provider.metadata).map(([key, value]) => (
                                    <div key={key} className="flex justify-between gap-3">
                                      <span className="uppercase tracking-[0.16em] text-ink-muted">{key}</span>
                                      <span className="break-all text-right text-ink opacity-80">{value}</span>
                                    </div>
                                  ))}
                                </div>
                              </details>
                            ) : (
                              <p className="mt-3 text-xs text-ink-faint">No advanced metadata is stored for this provider.</p>
                            )}
                          </details>
                        </div>
                      ) : (
                        <p className="pt-1 text-[10px] text-ink-faint">Click to open details</p>
                      )}
                    </div>
                  </article>
                  );
                })}
              </div>
            </section>
          );
        })}

        {providers.length === 0 && !isLoading && !statusCopy ? (
          <div className="rounded-xl border border-dashed border-border-strong py-12 text-center opacity-70">
            <Activity className="mx-auto mb-4 h-12 w-12 opacity-20" />
            <p>{providerCardEmptyCopy(status)}</p>
          </div>
        ) : null}

        {isLoading ? (
          <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
            {[1, 2, 3].map((i) => (
              <div key={i} className="panel h-64 animate-pulse opacity-50">
                <div className="panel-head h-10 bg-surface-elevated" />
                <div className="panel-body" />
              </div>
            ))}
          </div>
        ) : null}
      </div>
    </div>
  );
};
