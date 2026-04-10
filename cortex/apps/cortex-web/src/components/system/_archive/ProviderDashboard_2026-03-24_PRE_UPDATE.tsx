import React, { useEffect, useMemo, useState } from "react";
import { Activity, Cpu, Globe, KeyRound, RefreshCw, Server, Shield, Sparkles } from "lucide-react";
import { workbenchApi } from "../../api";
import { useProvidersRegistry } from "../../store/providersRegistry";
import {
  providerTemplateById,
  providerTemplates,
  resolveProviderTemplate,
  slugifyProviderName,
  type ProviderTemplateId,
} from "./providerTemplates";
import {
  formatProviderBatchSummary,
  formatProviderHoverDetails,
  formatProviderLocalityLabel,
  formatProviderModelLabel,
  formatProviderTopologySummary,
  formatProviderTypeLabel,
} from "./providerTopology";
import type { ProviderValidationRequest, ProviderValidationResponse } from "../../contracts.ts";

const providerTypeOrder = ["Llm", "Embedding", "Vector", "Batch"] as const;
type ProviderType = (typeof providerTypeOrder)[number];

interface ProviderFormState {
  templateId: ProviderTemplateId;
  providerId: string;
  providerType: ProviderType;
  providerKind: string;
  name: string;
  endpoint: string;
  defaultModel: string;
  adapterSetRef: string;
  credentialLabel: string;
  apiKey: string;
  metadataJson: string;
  enabled: boolean;
}

function createEmptyProviderForm(): ProviderFormState {
  return {
    templateId: "openrouter",
    providerId: "",
    providerType: "Llm",
    providerKind: "OpenRouter",
    name: "",
    endpoint: providerTemplateById("openrouter").defaultEndpoint,
    defaultModel: "",
    adapterSetRef: "",
    credentialLabel: providerTemplateById("openrouter").credentialLabel,
    apiKey: "",
    metadataJson: "{\n  \n}",
    enabled: true,
  };
}

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

function applyProviderTemplate(templateId: ProviderTemplateId, current: ProviderFormState): ProviderFormState {
  const template = providerTemplateById(templateId);
  const nextProviderId = current.providerId.trim() || slugifyProviderName(template.label);
  const nextName = current.name.trim() || `${template.label} Provider`;
  return {
    ...current,
    templateId: template.id,
    providerType: template.providerType,
    providerKind: template.providerKind ?? "",
    providerId: nextProviderId,
    name: nextName,
    endpoint: template.defaultEndpoint || (template.id === current.templateId ? current.endpoint : ""),
    defaultModel: current.defaultModel,
    credentialLabel: current.credentialLabel.trim() || template.credentialLabel,
  };
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

  useEffect(() => {
    void fetchProviders();
  }, [fetchProviders]);

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

    const selectedBinding = credentialBindings.find(
      (binding) => binding.credentialBindingId === selectedProvider.credentialBindingId,
    );
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
      credentialLabel: selectedBinding?.label ?? selectedProvider.name,
      apiKey: "",
      metadataJson:
        selectedProvider.metadata && Object.keys(selectedProvider.metadata).length > 0
          ? JSON.stringify(selectedProvider.metadata, null, 2)
          : "{\n  \n}",
      enabled: selectedProvider.isActive,
    });
    setValidationResult(null);
  }, [credentialBindings, selectedProvider]);

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
  const providerOptions = providers.slice().sort((left, right) => left.name.localeCompare(right.name));

  const handleProviderSelection = (nextProviderId: string) => {
    setSelectedProviderId(nextProviderId === "__new__" ? "__new__" : nextProviderId);
    if (nextProviderId === "__new__") {
      setForm((current) => applyProviderTemplate(current.templateId, createEmptyProviderForm()));
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

  const handleTemplateSelection = (nextTemplateId: ProviderTemplateId) => {
    setForm((current) => applyProviderTemplate(nextTemplateId, current));
    setValidationResult(null);
    if (selectedProviderId === "") {
      setSelectedProviderId("__new__");
    }
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
        supportedModels[0] ||
        undefined;

      await workbenchApi.putSystemProvider(providerId, {
        name: providerName,
        endpoint: providerEndpoint,
        enabled: form.enabled,
        providerType: form.providerType,
        defaultModel: nextDefaultModel,
        adapterSetRef: form.adapterSetRef.trim() || undefined,
        providerKind,
        supportedModels,
        metadata: providerMetadata,
      });

      if (form.apiKey.trim()) {
        await workbenchApi.createSystemProviderCredential({
          providerId,
          label: form.credentialLabel.trim() || providerName || providerId,
          apiKey: form.apiKey.trim(),
          metadata: providerMetadata,
        });
      }

      setSelectedProviderId(providerId);
      await fetchProviders();
      setSubmitMessage(
        form.apiKey.trim()
          ? "Provider validated, saved, and credential bound."
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
            Configure provider runtimes, bind credentials, and see which records are local, tunneled, or cloud
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
          <button
            type="button"
            onClick={() => void discoverLocalProviders()}
            disabled={isLoading}
            className="flex items-center gap-2 rounded-lg border border-border-subtle bg-surface-elevated px-4 py-2 text-sm font-semibold transition-all hover:bg-opacity-80 disabled:cursor-not-allowed disabled:opacity-60"
          >
            <Sparkles className={`h-4 w-4 ${status === "discovering" ? "animate-pulse" : ""}`} />
            Discover local providers
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

      <div className="mb-8 grid gap-6 xl:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)]">
        <form onSubmit={handleSubmit} className="panel">
          <div className="panel-head flex items-center justify-between gap-4">
            <div className="flex items-center gap-2">
              <KeyRound className="h-4 w-4" />
              Provider details
            </div>
            <label className="min-w-[220px]">
              <span className="sr-only">Load existing provider</span>
              <select
                value={selectedProviderId || "__new__"}
                onChange={(event) => handleProviderSelection(event.target.value)}
                className="w-full rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink"
              >
                <option value="__new__">Create new provider</option>
                {providerOptions.map((provider) => (
                  <option key={provider.id} value={provider.id}>
                    {provider.name} ({formatProviderTypeLabel(provider.providerType)})
                  </option>
                ))}
              </select>
            </label>
          </div>

          <div className="panel-body grid gap-5">
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
              {providerTemplates.map((template) => {
                const isSelected = form.templateId === template.id;
                return (
                  <button
                    key={template.id}
                    type="button"
                    onClick={() => handleTemplateSelection(template.id)}
                    className={[
                      "rounded-2xl border px-4 py-4 text-left transition-all",
                      isSelected
                        ? "border-cyan-400/35 bg-cyan-400/10 shadow-[0_18px_48px_-28px_rgba(34,211,238,0.4)]"
                        : "border-border-subtle bg-surface-elevated hover:border-border-strong hover:bg-opacity-90",
                    ].join(" ")}
                  >
                    <div className="flex items-center justify-between gap-3">
                      <p className="text-sm font-semibold text-ink">{template.label}</p>
                      <span className="rounded-full border border-border-subtle px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-ink-muted">
                        {template.providerKind || "manual"}
                      </span>
                    </div>
                    <p className="mt-2 text-xs leading-5 text-ink-muted">{template.description}</p>
                    <div className="mt-3 flex flex-wrap gap-2">
                      <span className="rounded-full border border-border-subtle px-2 py-0.5 text-[10px] text-ink-muted">
                        {template.validateKey ? "key" : "no key probe"}
                      </span>
                      <span className="rounded-full border border-border-subtle px-2 py-0.5 text-[10px] text-ink-muted">
                        {template.validateChat ? "chat" : "no chat"}
                      </span>
                      <span className="rounded-full border border-border-subtle px-2 py-0.5 text-[10px] text-ink-muted">
                        {template.validateEmbeddings ? "embeddings" : "no embeddings"}
                      </span>
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
                <span className="text-ink-muted">{providerTemplateById(form.templateId).credentialLabel}</span>
                <input
                  value={form.apiKey}
                  onChange={(event) => setForm((current) => ({ ...current, apiKey: event.target.value }))}
                  type="password"
                  placeholder="sk-..."
                  className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                />
              </label>
            </div>

            <details className="rounded-2xl border border-border-subtle bg-surface-elevated px-4 py-3">
              <summary className="cursor-pointer select-none text-sm font-medium text-ink">
                Advanced configuration
              </summary>
              <div className="mt-4 grid gap-4">
                <div className="grid gap-4 md:grid-cols-2">
                  <label className="grid gap-2 text-sm">
                    <span className="text-ink-muted">Provider ID</span>
                    <input
                      value={form.providerId}
                      onChange={(event) => setForm((current) => ({ ...current, providerId: event.target.value }))}
                      placeholder="openrouter_primary"
                      className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                    />
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
                      placeholder={providerTemplateById(form.templateId).defaultModelHint}
                      className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                    />
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
                    <span className="text-ink-muted">Credential label</span>
                    <input
                      value={form.credentialLabel}
                      onChange={(event) => setForm((current) => ({ ...current, credentialLabel: event.target.value }))}
                      placeholder="Primary key binding"
                      className="rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
                    />
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

                <details className="rounded-xl border border-border-subtle bg-surface-elevated px-4 py-3">
                  <summary className="cursor-pointer select-none text-sm font-medium text-ink">
                    Advanced metadata
                  </summary>
                  <p className="mt-2 text-xs leading-5 text-ink-faint">
                    Optional JSON metadata for provider and credential records.
                  </p>
                  <textarea
                    value={form.metadataJson}
                    onChange={(event) => setForm((current) => ({ ...current, metadataJson: event.target.value }))}
                    rows={6}
                    className="mt-3 w-full rounded-lg border border-border-subtle bg-surface-elevated px-3 py-2 font-mono text-xs text-ink placeholder:text-ink-faint"
                  />
                </details>
              </div>
            </details>

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
                    setForm((current) => applyProviderTemplate(current.templateId, createEmptyProviderForm()));
                    setSubmitError(null);
                    setSubmitMessage(null);
                    setValidationResult(null);
                  }}
                  className="rounded-lg border border-border-subtle bg-surface-elevated px-4 py-2 text-sm font-semibold text-ink transition hover:bg-opacity-80"
                >
                  New provider
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

        <section className="panel">
          <div className="panel-head flex items-center gap-2">
            <Sparkles className="h-4 w-4" />
            Credential bindings
          </div>
          <div className="panel-body grid gap-3">
            {credentialBindings.length === 0 ? (
              <div className="rounded-xl border border-dashed border-border-strong px-4 py-8 text-center text-sm text-ink-muted">
                No credential bindings are registered yet.
              </div>
            ) : (
              credentialBindings.map((binding) => (
                <div key={binding.credentialBindingId} className="rounded-xl border border-border-subtle bg-surface-elevated px-4 py-4">
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <p className="text-sm font-semibold text-ink">{binding.label || binding.credentialBindingId}</p>
                      <p className="mt-1 text-xs text-ink-muted">
                        {binding.providerId} · {binding.source || "system"}
                      </p>
                    </div>
                    <span className={`rounded-full px-2 py-1 text-[10px] font-semibold ${binding.hasCredential ? "text-ok" : "text-bad"}`}>
                      {binding.hasCredential ? "ENABLED" : "MISSING"}
                    </span>
                  </div>
                  {binding.metadata && Object.keys(binding.metadata).length > 0 ? (
                    <details className="mt-3 text-xs text-ink-faint">
                      <summary className="cursor-pointer">Metadata</summary>
                      <div className="mt-2 grid grid-cols-1 gap-1">
                        {Object.entries(binding.metadata).map(([key, value]) => (
                          <div key={key} className="flex justify-between gap-3">
                            <span className="uppercase tracking-[0.16em] text-ink-muted">{key}</span>
                            <span className="break-all text-right text-ink opacity-80">{value}</span>
                          </div>
                        ))}
                      </div>
                    </details>
                  ) : null}
                  <p className="mt-3 text-[11px] text-ink-faint">Updated {binding.updatedAt}</p>
                </div>
              ))
            )}
          </div>
        </section>
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
                {groupProviders.map((provider) => (
                  <article
                    key={provider.id}
                    className="panel group transition-all duration-300 hover:border-accent/30"
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
                    <div className="panel-body">
                      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
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

                      <div className="space-y-4">
                        <div className="metric">
                          <span>Endpoint</span>
                          <code className="mt-1 block break-all text-xs opacity-70">{provider.endpoint}</code>
                        </div>

                        <div className="grid gap-2 text-xs">
                          <div className="flex items-center justify-between">
                            <span className="text-ink-muted">Model</span>
                            <span className="text-ink">{formatProviderModelLabel(provider)}</span>
                          </div>
                          <div className="flex items-center justify-between">
                            <span className="text-ink-muted">Credential</span>
                            <span className="text-ink">{provider.hasCredential ? "Bound" : "Unbound"}</span>
                          </div>
                          <div className="flex items-center justify-between">
                            <span className="text-ink-muted">Key</span>
                            <span className="text-ink">{provider.credentialBindingId || "none"}</span>
                          </div>
                          {provider.adapterSetRef ? (
                            <div className="flex items-center justify-between">
                              <span className="text-ink-muted">Adapter set</span>
                              <span className="text-ink">{provider.adapterSetRef}</span>
                            </div>
                          ) : null}
                          {provider.batchPolicy ? (
                            <div className="flex items-center justify-between">
                              <span className="text-ink-muted">Batch</span>
                              <span className="text-ink">{formatProviderBatchSummary(provider.batchPolicy)}</span>
                            </div>
                          ) : null}
                        </div>

                        {(provider.supportedModels ?? []).length > 0 ? (
                          <div className="flex flex-wrap gap-2 pt-1">
                            {(provider.supportedModels ?? []).slice(0, 3).map((model) => (
                              <span
                                key={model}
                                className="rounded-full border border-border-subtle px-2 py-0.5 text-[10px] font-medium text-ink-muted"
                              >
                                {model}
                              </span>
                            ))}
                            {(provider.supportedModels ?? []).length > 3 ? (
                              <span className="rounded-full border border-border-subtle px-2 py-0.5 text-[10px] font-medium text-ink-muted">
                                +{(provider.supportedModels ?? []).length - 3} more
                              </span>
                            ) : null}
                          </div>
                        ) : null}

                        {provider.topology ? (
                          <details className="metric">
                            <summary className="cursor-pointer select-none text-sm">Runtime details</summary>
                            <div className="mt-2 grid grid-cols-1 gap-1">
                              <div className="flex justify-between text-[11px]">
                                <span className="text-ink-muted lowercase">family</span>
                                <span className="text-ink opacity-80">{provider.topology.familyId}</span>
                              </div>
                              {provider.topology.profileId ? (
                                <div className="flex justify-between text-[11px]">
                                  <span className="text-ink-muted lowercase">profile</span>
                                  <span className="text-ink opacity-80">{provider.topology.profileId}</span>
                                </div>
                              ) : null}
                              <div className="flex justify-between text-[11px]">
                                <span className="text-ink-muted lowercase">instance</span>
                                <span className="text-ink opacity-80">{provider.topology.instanceId}</span>
                              </div>
                              {provider.topology.deviceId ? (
                                <div className="flex justify-between text-[11px]">
                                  <span className="text-ink-muted lowercase">device</span>
                                  <span className="text-ink opacity-80">{provider.topology.deviceId}</span>
                                </div>
                              ) : null}
                              {provider.topology.environmentId ? (
                                <div className="flex justify-between text-[11px]">
                                  <span className="text-ink-muted lowercase">environment</span>
                                  <span className="text-ink opacity-80">{provider.topology.environmentId}</span>
                                </div>
                              ) : null}
                              {provider.topology.lastSeenAt ? (
                                <div className="flex justify-between text-[11px]">
                                  <span className="text-ink-muted lowercase">last seen</span>
                                  <span className="text-ink opacity-80">{provider.topology.lastSeenAt}</span>
                                </div>
                              ) : null}
                              {provider.topology.discoverySource ? (
                                <div className="flex justify-between text-[11px]">
                                  <span className="text-ink-muted lowercase">source</span>
                                  <span className="text-ink opacity-80">{provider.topology.discoverySource}</span>
                                </div>
                              ) : null}
                            </div>
                          </details>
                        ) : null}

                        {provider.metadata && Object.keys(provider.metadata).length > 0 ? (
                          <details className="metric">
                            <summary className="cursor-pointer select-none text-sm">Advanced metadata</summary>
                            <div className="mt-2 grid grid-cols-1 gap-1">
                              {Object.entries(provider.metadata).map(([key, val]) => (
                                <div key={key} className="flex justify-between text-[11px]">
                                  <span className="text-ink-muted lowercase">{key}</span>
                                  <span className="text-ink opacity-80">{val}</span>
                                </div>
                              ))}
                            </div>
                          </details>
                        ) : null}

                        <div className="flex items-center justify-between border-t border-border-subtle pt-2">
                          <span className="text-[10px] font-bold uppercase tracking-wider text-ink-faint">Status</span>
                          <span className={`text-xs font-bold ${provider.isActive ? "text-ok" : "text-bad"}`}>
                            {provider.isActive ? "OPERATIONAL" : "OFFLINE"}
                          </span>
                        </div>
                      </div>
                    </div>
                  </article>
                ))}
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
