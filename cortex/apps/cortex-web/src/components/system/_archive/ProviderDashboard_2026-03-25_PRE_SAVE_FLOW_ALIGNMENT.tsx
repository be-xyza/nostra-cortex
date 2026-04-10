import React, { useEffect, useMemo, useRef, useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  Activity,
  ChevronDown,
  ChevronRight,
  Cpu,
  Globe,
  KeyRound,
  Plus,
  RefreshCw,
  Search,
  Server,
  Shield,
  SlidersHorizontal,
  Sparkles,
  X,
} from "lucide-react";

import { workbenchApi } from "../../api";
import type {
  ProviderValidationRequest,
  ProviderValidationResponse,
  SystemProviderRuntimeStatusResponse,
} from "../../contracts.ts";
import { useProvidersRegistry } from "../../store/providersRegistry";
import {
  applyProviderTemplate,
  buildTemplateProviderId,
  createEmptyProviderForm,
  providerTypeOrder,
  type ProviderFormState,
  type ProviderType,
} from "./providerForm.ts";
import {
  buildDiscoveryProviderForm,
  isLocalDiscoveryRecord,
  type ProviderDiscoveryLane,
} from "./providerDiscovery.ts";
import { buildProviderCatalogState } from "./providerCatalog.ts";
import { humanizeProviderDiagnostic, resolveAdapterStatusError } from "./providerDiagnostics.ts";
import { buildDefaultModelControlState, buildProviderWorkbenchChromeState } from "./providerPresentation.ts";
import { buildProviderModelOptions, extractModelNames } from "./providerModels.ts";
import { serializeProviderFormSnapshot, shouldWarnBeforeClosingProviderPanel } from "./providerSheetState.ts";
import {
  buildProviderRegistrySections,
  readProviderRegistryPanelState,
  validateProviderDraftInput,
  writeProviderRegistryPanelState,
  type ProviderRegistryReadinessFilter,
  type ProviderRegistryTypeFilter,
} from "./providerRegistryView.ts";
import {
  providerTemplateById,
  providerTemplates,
  resolveProviderTemplate,
  type ProviderTemplateId,
} from "./providerTemplates.ts";
import {
  formatProviderBatchSummary,
  formatProviderBindingLabel,
  formatProviderCredentialState,
  formatProviderLocalityLabel,
  formatProviderModelLabel,
  formatProviderTopologySummary,
  formatProviderTypeLabel,
  getProviderOperationalReadiness,
  getProviderReadiness,
  type ProviderReadinessState,
} from "./providerTopology.ts";

type ProviderSheetTab = "overview" | "credentials" | "models" | "advanced";

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
        body: "Refresh to try again. If the local shell is starting, the provider registry may not be ready yet.",
      };
    default:
      return null;
  }
}

function providerReadinessClasses(state: ProviderReadinessState): string {
  switch (state) {
    case "ready":
      return "border-emerald-400/25 bg-emerald-400/10 text-emerald-100";
    case "attention":
      return "border-amber-400/25 bg-amber-400/10 text-amber-100";
    case "disabled":
      return "border-white/10 bg-white/[0.05] text-white/65";
    default:
      return "border-cyan-400/20 bg-cyan-400/10 text-cyan-100";
  }
}

function endpointHost(endpoint: string): string {
  try {
    return new URL(endpoint).host;
  } catch {
    return endpoint
      .replace(/^https?:\/\//, "")
      .replace(/\/+$/, "");
  }
}

export const ProviderDashboard: React.FC = () => {
  const {
    providers,
    credentialBindings,
    adapterBindings,
    discoveryRecords,
    isLoading,
    status,
    error,
    fetchProviders,
    refreshProviders,
    discoverLocalProviders,
  } = useProvidersRegistry();
  const [searchParams, setSearchParams] = useSearchParams();
  const panelState = useMemo(() => readProviderRegistryPanelState(searchParams), [searchParams]);

  const [form, setForm] = useState<ProviderFormState>(() => createEmptyProviderForm());
  const [validationResult, setValidationResult] = useState<ProviderValidationResponse | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [submitMessage, setSubmitMessage] = useState<string | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [adapterStatus, setAdapterStatus] = useState<SystemProviderRuntimeStatusResponse | null>(null);
  const [adapterStatusError, setAdapterStatusError] = useState<string | null>(null);
  const [isRefreshingAdapterModels, setIsRefreshingAdapterModels] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [providerTypeFilter, setProviderTypeFilter] = useState<ProviderRegistryTypeFilter>("all");
  const [readinessFilter, setReadinessFilter] = useState<ProviderRegistryReadinessFilter>("all");
  const [detailsOpen, setDetailsOpen] = useState(false);
  const [sheetTab, setSheetTab] = useState<ProviderSheetTab>("overview");
  const [modelSearchTerm, setModelSearchTerm] = useState("");
  const [discoverySearchTerm, setDiscoverySearchTerm] = useState("");
  const [selectedDiscoveryModel, setSelectedDiscoveryModel] = useState("");
  const [discoveryLane, setDiscoveryLane] = useState<ProviderDiscoveryLane>("runtime_catalog");
  const [isModelComboboxOpen, setIsModelComboboxOpen] = useState(false);
  const catalogRefreshAttemptedRef = useRef<Record<string, true>>({});
  const formSnapshotRef = useRef<string>("");
  const modelComboboxCloseTimerRef = useRef<number | null>(null);

  useEffect(() => {
    if (status === "idle") {
      void fetchProviders();
    }
  }, [fetchProviders, status]);

  const refreshAdapterStatus = async () => {
    setIsRefreshingAdapterModels(true);
    setAdapterStatusError(null);
    try {
      const nextStatus = await workbenchApi.getSystemProviderRuntimeStatus();
      setAdapterStatus(nextStatus);
      setAdapterStatusError(resolveAdapterStatusError(nextStatus));
      return nextStatus;
    } catch (err) {
      setAdapterStatus(null);
      setAdapterStatusError(err instanceof Error ? err.message : String(err));
      throw err;
    } finally {
      setIsRefreshingAdapterModels(false);
    }
  };

  useEffect(() => {
    let cancelled = false;

    void refreshAdapterStatus().catch((err) => {
      if (!cancelled) {
        setAdapterStatus(null);
        setAdapterStatusError(err instanceof Error ? err.message : String(err));
      }
    });

    return () => {
      cancelled = true;
    };
  }, []);

  const applyPanelState = (
    next: Parameters<typeof writeProviderRegistryPanelState>[1],
    options?: { replace?: boolean },
  ) => {
    setSearchParams(writeProviderRegistryPanelState(searchParams, next), options);
  };

  const isFormDirty = useMemo(() => {
    const isProviderPanel = panelState.kind === "provider" || panelState.kind === "create";
    if (!isProviderPanel) {
      return false;
    }
    return formSnapshotRef.current !== serializeProviderFormSnapshot(form);
  }, [form, panelState.kind]);

  const confirmPanelExit = () => {
    const shouldWarn = shouldWarnBeforeClosingProviderPanel({
      panelState,
      isDirty: isFormDirty,
      isSubmitting,
    });
    if (!shouldWarn) {
      return true;
    }
    return window.confirm("Discard unsaved provider changes?");
  };

  const openPanel = (next: Parameters<typeof writeProviderRegistryPanelState>[1]) => {
    if (!confirmPanelExit()) {
      return false;
    }
    applyPanelState(next);
    return true;
  };

  const closePanel = () => {
    if (!confirmPanelExit()) {
      return false;
    }
    applyPanelState({ kind: "none" }, { replace: true });
    return true;
  };

  const selectedProvider = useMemo(
    () => (panelState.kind === "provider" ? providers.find((provider) => provider.id === panelState.providerId) ?? null : null),
    [panelState, providers],
  );

  useEffect(() => {
    if (panelState.kind === "provider" && !selectedProvider && !isLoading) {
      applyPanelState({ kind: "none" }, { replace: true });
    }
  }, [isLoading, panelState.kind, searchParams, selectedProvider]);

  const providerIds = useMemo(() => providers.map((provider) => provider.id), [providers]);
  const defaultCreateTemplateId = useMemo<ProviderTemplateId>(
    () => discoveryRecords.some((record) => (record.providerKind ?? "").toLowerCase() === "ollama") ? "ollama" : "manual",
    [discoveryRecords],
  );
  const readyProviderCount = useMemo(
    () => providers.filter((provider) => getProviderReadiness(provider).state === "ready").length,
    [providers],
  );
  const localProviderCount = useMemo(
    () => providers.filter((provider) => formatProviderLocalityLabel(provider) === "Local").length,
    [providers],
  );
  const statusCopy = providerStatusCopy(status, error);
  const registrySections = useMemo(
    () =>
      buildProviderRegistrySections(providers, {
        searchTerm,
        providerType: providerTypeFilter,
        readiness: readinessFilter,
      }),
    [providerTypeFilter, providers, readinessFilter, searchTerm],
  );

  const buildNewProviderForm = (templateId: ProviderTemplateId): ProviderFormState => {
    const baseForm = applyProviderTemplate(templateId, createEmptyProviderForm());
    return {
      ...baseForm,
      providerId: buildTemplateProviderId(templateId, providerIds),
    };
  };

  useEffect(() => {
    setSubmitError(null);
    setSubmitMessage(null);
    setValidationResult(null);
    setModelSearchTerm("");
    setSheetTab("overview");

    if (panelState.kind === "provider" && selectedProvider) {
      const template = resolveProviderTemplate(selectedProvider.llmType, selectedProvider.endpoint);
      const nextForm = {
        templateId: template.id,
        providerId: selectedProvider.id,
        providerType: selectedProvider.providerType,
        providerKind: selectedProvider.llmType ?? template.providerKind ?? "",
        name: selectedProvider.name,
        endpoint: selectedProvider.endpoint,
        defaultModel: selectedProvider.defaultModel ?? "",
        credentialBindingId: selectedProvider.credentialBindingId ?? "",
        apiKey: "",
        metadataJson:
          selectedProvider.metadata && Object.keys(selectedProvider.metadata).length > 0
            ? JSON.stringify(selectedProvider.metadata, null, 2)
            : "{\n  \n}",
        enabled: selectedProvider.isActive,
      };
      formSnapshotRef.current = serializeProviderFormSnapshot(nextForm);
      setForm(nextForm);
      return;
    }

    if (panelState.kind === "create") {
      if (panelState.seedModel) {
        const discoveryForm = buildDiscoveryProviderForm(adapterStatus, panelState.seedModel, providerIds);
        if (panelState.templateId && panelState.templateId !== discoveryForm.templateId) {
          const nextForm = {
            ...applyProviderTemplate(panelState.templateId as ProviderTemplateId, discoveryForm),
            providerId: buildTemplateProviderId(panelState.templateId as ProviderTemplateId, providerIds),
            defaultModel: panelState.seedModel ?? discoveryForm.defaultModel,
          };
          formSnapshotRef.current = serializeProviderFormSnapshot(nextForm);
          setForm(nextForm);
          return;
        }
        formSnapshotRef.current = serializeProviderFormSnapshot(discoveryForm);
        setForm(discoveryForm);
        return;
      }

      const nextForm = buildNewProviderForm(panelState.templateId ?? defaultCreateTemplateId);
      formSnapshotRef.current = serializeProviderFormSnapshot(nextForm);
      setForm(nextForm);
      return;
    }

    formSnapshotRef.current = "";
  }, [adapterStatus, defaultCreateTemplateId, panelState, providerIds, selectedProvider]);

  const selectedProviderOperationalReadiness = selectedProvider
    ? getProviderOperationalReadiness(selectedProvider, validationResult)
    : null;
  const selectedProviderDiscoveryRecord = selectedProvider
    ? discoveryRecords.find((record) => record.providerId === selectedProvider.id) ?? null
    : null;
  const defaultLlmBinding = adapterBindings.find((binding) => binding.bindingId === "llm.default") ?? null;
  const runtimeProviderId = adapterStatus?.providerId?.trim() || defaultLlmBinding?.boundProviderId || "";
  const runtimeDiscoveryRecord = runtimeProviderId
    ? discoveryRecords.find((record) => record.providerId === runtimeProviderId) ?? null
    : null;
  const localDiscoveryRecords = useMemo(
    () =>
      discoveryRecords
        .filter((record) => isLocalDiscoveryRecord(record))
        .sort((left, right) => (left.providerId || "").localeCompare(right.providerId || "")),
    [discoveryRecords],
  );
  const selectedProviderBindingLabels = (selectedProvider?.bindingIds ?? []).map(formatProviderBindingLabel);
  const selectedProviderFamilyLabel = selectedProvider?.llmType
    || providerTemplateById(form.templateId).providerKind
    || "Custom";
  const validationMessages = validationResult
    ? [
        validationResult.keyError ? `Credential: ${humanizeProviderDiagnostic(validationResult.keyError)}` : null,
        validationResult.modelsError ? `Catalog: ${humanizeProviderDiagnostic(validationResult.modelsError)}` : null,
        validationResult.chatError ? `Chat: ${humanizeProviderDiagnostic(validationResult.chatError)}` : null,
        validationResult.embeddingsError ? `Embeddings: ${humanizeProviderDiagnostic(validationResult.embeddingsError)}` : null,
      ].filter((value): value is string => Boolean(value))
    : [];
  const validationContextMessage = validationResult && !validationResult.valid && selectedProvider && (selectedProvider.supportedModels?.length ?? 0) > 0
    ? `Saved provider state still includes ${selectedProvider.supportedModels?.length ?? 0} model${(selectedProvider.supportedModels?.length ?? 0) === 1 ? "" : "s"} from the previous successful catalog refresh.`
    : null;
  const visibleCredentialBindings = selectedProvider
    ? credentialBindings.filter((binding) => binding.providerId === selectedProvider.id)
    : credentialBindings;
  const selectedProviderCredentialBinding = selectedProvider
    ? credentialBindings.find((binding) => binding.credentialBindingId === selectedProvider.credentialBindingId) ?? null
    : null;
  const effectiveCredentialBindingId =
    form.credentialBindingId.trim() || selectedProvider?.credentialBindingId || selectedProviderCredentialBinding?.credentialBindingId || "";
  const hasStoredCatalogCredential = Boolean(
    effectiveCredentialBindingId || selectedProvider?.hasCredential || selectedProviderCredentialBinding?.hasCredential,
  );
  const discoveredModelOptions = buildProviderModelOptions({
    currentModel: form.defaultModel,
    validatedModels: validationResult?.supportedModels,
    savedProviderModels: selectedProvider?.supportedModels,
    adapterDiscoveryModels: extractModelNames(adapterStatus?.upstreamModels),
    adapterRuntimeModel: adapterStatus?.model,
    panelKind: panelState.kind,
    templateId: form.templateId,
    providerId: selectedProvider?.id,
  });
  const defaultModelControlState = buildDefaultModelControlState(
    form.templateId === "manual" ? 0 : discoveredModelOptions.length,
  );
  const filteredSheetModels = useMemo(() => {
    const normalizedSearch = modelSearchTerm.trim().toLowerCase();
    if (!normalizedSearch) {
      return discoveredModelOptions;
    }
    return discoveredModelOptions.filter((model) => model.toLowerCase().includes(normalizedSearch));
  }, [discoveredModelOptions, modelSearchTerm]);
  const runtimeDiscoveryModels = useMemo(
    () =>
      buildProviderModelOptions({
        currentModel: selectedDiscoveryModel,
        savedProviderModels: runtimeDiscoveryRecord?.supportedModels,
        adapterDiscoveryModels: extractModelNames(adapterStatus?.upstreamModels),
        adapterRuntimeModel: adapterStatus?.model,
        panelKind: "discovery",
        templateId: resolveProviderTemplate(runtimeDiscoveryRecord?.providerKind, adapterStatus?.baseUrl ?? runtimeDiscoveryRecord?.endpoint ?? null).id,
        providerId: runtimeProviderId || undefined,
      }),
    [adapterStatus?.baseUrl, adapterStatus?.model, adapterStatus?.upstreamModels, runtimeDiscoveryRecord, runtimeProviderId, selectedDiscoveryModel],
  );
  const filteredRuntimeDiscoveryModels = useMemo(() => {
    const normalizedSearch = discoverySearchTerm.trim().toLowerCase();
    if (!normalizedSearch) {
      return runtimeDiscoveryModels;
    }
    return runtimeDiscoveryModels.filter((model) => model.toLowerCase().includes(normalizedSearch));
  }, [discoverySearchTerm, runtimeDiscoveryModels]);
  const filteredLocalDiscoveryRecords = useMemo(() => {
    const normalizedSearch = discoverySearchTerm.trim().toLowerCase();
    if (!normalizedSearch) {
      return localDiscoveryRecords;
    }
    return localDiscoveryRecords.filter((record) => {
      const haystacks = [
        record.providerId,
        record.providerKind,
        record.endpoint,
        record.defaultModel,
        ...(record.supportedModels ?? []),
      ];
      return haystacks.some((value) => value?.toLowerCase().includes(normalizedSearch));
    });
  }, [discoverySearchTerm, localDiscoveryRecords]);
  const providerCatalogState = buildProviderCatalogState({
    providerId: selectedProvider?.id ?? (panelState.kind === "create" ? form.providerId : undefined),
    hasStoredCredential: hasStoredCatalogCredential,
    draftApiKey: form.apiKey,
    catalogSize: discoveredModelOptions.length,
    allowsAnonymousDiscovery:
      form.templateId === "ollama" || selectedProvider?.llmType === "Ollama" || selectedProviderDiscoveryRecord?.providerKind === "Ollama",
  });
  const modelComboboxSuggestions = useMemo(() => {
    if (defaultModelControlState.control !== "combobox") {
      return [];
    }
    const normalizedSearch = form.defaultModel.trim().toLowerCase();
    const matches = normalizedSearch
      ? discoveredModelOptions.filter((model) => model.toLowerCase().includes(normalizedSearch))
      : discoveredModelOptions;
    return matches.slice(0, 8);
  }, [defaultModelControlState.control, discoveredModelOptions, form.defaultModel]);
  const submitMessageTone = !submitError && submitMessage && /(attention|issue)/i.test(submitMessage)
    ? "text-amber-200"
    : "text-emerald-200";

  useEffect(() => {
    return () => {
      if (modelComboboxCloseTimerRef.current) {
        window.clearTimeout(modelComboboxCloseTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    if (panelState.kind !== "discovery") {
      return;
    }
    setSelectedDiscoveryModel((current) => current.trim() || panelState.seedModel || adapterStatus?.model?.trim() || runtimeDiscoveryModels[0] || "");
  }, [adapterStatus?.model, panelState, runtimeDiscoveryModels]);

  useEffect(() => {
    if (panelState.kind !== "discovery") {
      return;
    }
    setDiscoveryLane(localDiscoveryRecords.length > 0 ? "local_runtimes" : "runtime_catalog");
  }, [localDiscoveryRecords.length, panelState.kind, runtimeProviderId]);

  const discoveryBaseUrl = adapterStatus?.baseUrl?.trim()
    || runtimeDiscoveryRecord?.endpoint?.trim()
    || selectedProviderDiscoveryRecord?.endpoint?.trim()
    || providerTemplateById(form.templateId).endpointHint;
  const discoveryCatalogDescription = adapterStatus
    ? adapterStatus.enabled
      ? `Inspect the catalog for the provider currently bound to ${formatProviderBindingLabel(defaultLlmBinding?.bindingId || "llm.default")}.`
      : `Runtime status is reachable, but catalog discovery is disabled for ${discoveryBaseUrl}.`
    : "Discovery is not available until the provider runtime status endpoint responds.";
  const selectedProviderRuntimeNote = selectedProvider
    && (selectedProvider.supportedModels?.length ?? 0) === 0
    ? humanizeProviderDiagnostic(selectedProvider.upstreamModelsError ?? selectedProvider.adapterHealthError ?? null)
    : null;

  const handleDiscoverLocalProviders = async () => {
    await discoverLocalProviders();
    try {
      const nextStatus = await refreshAdapterStatus();
      setSelectedDiscoveryModel((current) => current.trim() || nextStatus.model?.trim() || extractModelNames(nextStatus.upstreamModels)[0] || "");
    } catch (err) {
      setAdapterStatusError(err instanceof Error ? err.message : String(err));
    }
  };

  const openCreatePanel = (templateId?: ProviderTemplateId, seedModel?: string) => {
    openPanel({ kind: "create", templateId, seedModel });
  };

  const bindProviderAsDefaultLlm = async (providerId: string) => {
    setSubmitError(null);
    setSubmitMessage(null);
    setIsSubmitting(true);
    try {
      await workbenchApi.putSystemProviderBinding("llm.default", {
        providerType: "Llm",
        boundProviderId: providerId,
      });
      await refreshProviders();
      await refreshAdapterStatus();
      setSubmitMessage("Default LLM binding updated.");
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleTemplateSelection = (nextTemplateId: ProviderTemplateId) => {
    if (!confirmPanelExit()) {
      return;
    }
    const nextForm = buildNewProviderForm(nextTemplateId);
    formSnapshotRef.current = serializeProviderFormSnapshot(nextForm);
    setForm(nextForm);
    applyPanelState({ kind: "create", templateId: nextTemplateId, seedModel: form.defaultModel.trim() || undefined });
  };

  const refreshProviderCatalog = async (options?: {
    providerId?: string;
    providerType?: ProviderType;
    providerKind?: string;
    baseUrl?: string;
    defaultModel?: string;
    credentialBindingId?: string;
    draftApiKey?: string;
    silent?: boolean;
  }) => {
    const template = providerTemplateById(form.templateId);
    const draftApiKey = options?.draftApiKey ?? form.apiKey;
    const resolvedProviderId = options?.providerId ?? selectedProvider?.id ?? form.providerId.trim();
    const providerId = resolvedProviderId?.trim() ? resolvedProviderId : undefined;
    const resolvedCredentialBindingId = options?.credentialBindingId ?? effectiveCredentialBindingId;
    const credentialBindingId = resolvedCredentialBindingId?.trim() ? resolvedCredentialBindingId : undefined;
    const useStoredCredential = !draftApiKey.trim() && providerCatalogState.refreshMode === "stored_credential";
    const probeProviderKind = options?.providerKind ?? form.providerKind.trim();
    const probeDefaultModel = options?.defaultModel ?? form.defaultModel.trim();

    const useAnonymousDiscovery = !draftApiKey.trim() && providerCatalogState.refreshMode === "anonymous";
    if (!draftApiKey.trim() && !useStoredCredential && !useAnonymousDiscovery) {
      if (!options?.silent) {
        setSubmitError("Add or paste a key to refresh this provider catalog.");
      }
      return null;
    }

    setIsValidating(true);
    if (!options?.silent) {
      setSubmitError(null);
      setSubmitMessage(null);
    }
    try {
      const probeResult = await workbenchApi.validateSystemProvider({
        providerType: options?.providerType ?? form.providerType,
        providerKind: (probeProviderKind || template.providerKind || undefined) as ProviderValidationRequest["providerKind"],
        providerId,
        credentialBindingId,
        useStoredCredential,
        baseUrl: options?.baseUrl ?? form.endpoint.trim(),
        defaultModel: probeDefaultModel || undefined,
        apiKey: draftApiKey.trim(),
        validateKey: template.validateKey,
        validateChat: (options?.providerType ?? form.providerType) === "Llm" && template.validateChat,
        validateEmbeddings: (options?.providerType ?? form.providerType) === "Embedding" && template.validateEmbeddings,
      });
      setValidationResult(probeResult);
      if (!options?.silent) {
        setSubmitMessage(
          probeResult.valid
            ? "Provider catalog refreshed."
            : "Catalog refresh returned issues. Review the validation notes above.",
        );
      }
      return probeResult;
    } catch (err) {
      if (!options?.silent) {
        setSubmitError(err instanceof Error ? err.message : String(err));
      }
      return null;
    } finally {
      setIsValidating(false);
    }
  };

  useEffect(() => {
    if (!selectedProvider || panelState.kind !== "provider") {
      return;
    }
    if (selectedProvider.providerType !== "Llm" || !hasStoredCatalogCredential) {
      return;
    }
    if ((selectedProvider.supportedModels ?? []).length > 0) {
      return;
    }
    if (catalogRefreshAttemptedRef.current[selectedProvider.id]) {
      return;
    }
    catalogRefreshAttemptedRef.current[selectedProvider.id] = true;

    void refreshProviderCatalog({
      providerId: selectedProvider.id,
      providerType: selectedProvider.providerType,
      providerKind: selectedProvider.llmType,
      baseUrl: selectedProvider.endpoint,
      defaultModel: selectedProvider.defaultModel,
      credentialBindingId: selectedProvider.credentialBindingId,
      draftApiKey: "",
      silent: true,
    }).then(async (probeResult) => {
      if (!probeResult || !probeResult.valid) {
        return;
      }
      await workbenchApi.putSystemProvider(selectedProvider.id, {
        name: selectedProvider.name,
        endpoint: selectedProvider.endpoint,
        enabled: selectedProvider.isActive,
        providerType: selectedProvider.providerType,
        defaultModel: probeResult.selectedModel || selectedProvider.defaultModel,
        providerKind: selectedProvider.llmType,
        credentialBindingId: selectedProvider.credentialBindingId,
        supportedModels: probeResult.supportedModels,
        metadata: selectedProvider.metadata,
      });
      await fetchProviders();
    }).catch(() => undefined);
  }, [fetchProviders, hasStoredCatalogCredential, panelState.kind, selectedProvider]);

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setSubmitError(null);
    setSubmitMessage(null);

    const validationError = validateProviderDraftInput({
      providerId: form.providerId,
      providerName: form.name,
      providerEndpoint: form.endpoint,
      metadataJson: form.metadataJson,
    });
    if (validationError) {
      setSubmitError(validationError);
      return;
    }

    const template = providerTemplateById(form.templateId);
    const currentProvider = panelState.kind === "provider" ? selectedProvider : null;
    const providerId = form.providerId.trim();
    const providerName = form.name.trim();
    const providerEndpoint = form.endpoint.trim();
    const providerKind = form.providerKind.trim() || template.providerKind || currentProvider?.llmType || undefined;
    const providerMetadata = (() => {
      try {
        return parseMetadataJson(form.metadataJson);
      } catch {
        setSubmitError("Advanced details must be valid JSON.");
        return null;
      }
    })();

    if (providerMetadata === null) {
      return;
    }

    setIsSubmitting(true);
    try {
      let nextCredentialBindingId = effectiveCredentialBindingId || undefined;

      await workbenchApi.putSystemProvider(providerId, {
        name: providerName,
        endpoint: providerEndpoint,
        enabled: form.enabled,
        providerType: form.providerType,
        defaultModel: form.defaultModel.trim() || currentProvider?.defaultModel || undefined,
        providerKind,
        credentialBindingId: nextCredentialBindingId,
        supportedModels: currentProvider?.supportedModels ?? (form.defaultModel.trim() ? [form.defaultModel.trim()] : []),
        metadata: providerMetadata,
      });

      if (form.apiKey.trim()) {
        if (nextCredentialBindingId) {
          await workbenchApi.updateSystemProviderCredential(nextCredentialBindingId, {
            providerId,
            label: providerName || providerId,
            apiKey: form.apiKey.trim(),
            metadata: providerMetadata,
          });
        } else {
          const binding = await workbenchApi.createSystemProviderCredential({
            providerId,
            label: providerName || providerId,
            apiKey: form.apiKey.trim(),
            metadata: providerMetadata,
          });
          nextCredentialBindingId = binding.credentialBindingId;
        }
      }

      let probeResult: ProviderValidationResponse | null = null;
      const shouldAutoRefreshCatalog =
        form.providerType === "Llm" &&
        (
          form.apiKey.trim().length > 0 ||
          (providerCatalogState.refreshMode === "stored_credential" &&
            (((currentProvider?.supportedModels ?? []).length === 0) ||
              providerEndpoint !== currentProvider?.endpoint ||
              (form.defaultModel.trim() || "") !== (currentProvider?.defaultModel ?? "")))
        );

      if (shouldAutoRefreshCatalog) {
        probeResult = await refreshProviderCatalog({
          providerId,
          providerType: form.providerType,
          providerKind,
          baseUrl: providerEndpoint,
          defaultModel: form.defaultModel.trim() || currentProvider?.defaultModel || undefined,
          credentialBindingId: nextCredentialBindingId,
          draftApiKey: form.apiKey,
          silent: false,
        });
        if (form.apiKey.trim() && (!probeResult || !probeResult.valid)) {
          setSubmitError("Provider validation failed. Review the endpoint, key, and discovered models.");
          return;
        }
      }

      const supportedModels =
        probeResult?.supportedModels ??
        currentProvider?.supportedModels ??
        (form.defaultModel.trim() ? [form.defaultModel.trim()] : []);
      const nextDefaultModel =
        form.defaultModel.trim() ||
        probeResult?.selectedModel ||
        currentProvider?.defaultModel ||
        undefined;

      await workbenchApi.putSystemProvider(providerId, {
        name: providerName,
        endpoint: providerEndpoint,
        enabled: form.enabled,
        providerType: form.providerType,
        defaultModel: nextDefaultModel,
        providerKind,
        credentialBindingId: nextCredentialBindingId,
        supportedModels,
        metadata: providerMetadata,
      });

      await fetchProviders();
      setValidationResult(probeResult?.valid ? probeResult : null);
      formSnapshotRef.current = serializeProviderFormSnapshot({
        ...form,
        providerId,
        name: providerName,
        endpoint: providerEndpoint,
        providerKind: providerKind ?? "",
        defaultModel: nextDefaultModel ?? "",
        credentialBindingId: nextCredentialBindingId ?? "",
      });
      applyPanelState({ kind: "provider", providerId });
      setSubmitMessage(
        probeResult?.valid
          ? "Provider saved and catalog refreshed."
          : form.apiKey.trim()
            ? "Provider saved. Catalog refresh needs attention."
            : "Provider saved.",
      );
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  const isProviderSheetOpen = panelState.kind === "provider" || panelState.kind === "create";
  const isCreateSheet = panelState.kind === "create";
  const chromeState = buildProviderWorkbenchChromeState(panelState);
  const sheetTitle = isCreateSheet
    ? "Create provider"
    : selectedProvider?.name ?? "Provider details";
  const sheetSubtitle = isCreateSheet
    ? "Create a provider without crowding the registry."
    : "Adjust provider settings, credentials, and catalog state without losing registry context.";

  return (
    <div className="provider-dashboard min-h-full bg-cortex-surface-base">
      <div className="relative isolate">
        <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(34,211,238,0.12),transparent_32%),radial-gradient(circle_at_top_right,rgba(59,130,246,0.09),transparent_28%)]" />
        <div className={["relative z-10 px-4 pb-10 pt-4 transition-[padding] duration-300 md:px-6", chromeState.contentPaddingClass].join(" ")}>
          <header
            className={[
              "sticky top-0 z-30 overflow-hidden border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.92),rgba(2,6,23,0.9))] shadow-[0_24px_80px_-40px_rgba(0,0,0,0.75)] backdrop-blur-xl",
              chromeState.compactRegistryChrome ? "mb-4 rounded-[1.5rem] px-4 py-3 md:px-4" : "mb-5 rounded-[1.75rem] px-4 py-4 md:px-5",
            ].join(" ")}
          >
            <div className="grid gap-4">
              <div className={["flex items-start gap-4", chromeState.compactRegistryChrome ? "flex-col xl:flex-row xl:items-center xl:justify-between" : "flex-col lg:flex-row lg:items-start lg:justify-between"].join(" ")}>
                <div className="min-w-0">
                  <p className="text-[10px] font-black uppercase tracking-[0.32em] text-cortex-500">Execution surface</p>
                  <div className="mt-2 flex flex-wrap items-center gap-3">
                    <h1 className={`${chromeState.titleClass} font-semibold tracking-tight text-white`}>Providers</h1>
                    <span className="rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] text-white/60">
                      {chromeState.compactRegistryChrome ? "Registry visible" : "Neutral registry"}
                    </span>
                  </div>
                  <p className={["mt-2 text-sm leading-6 text-white/62", chromeState.compactRegistryChrome ? "max-w-2xl" : "max-w-3xl"].join(" ")}>
                    {chromeState.description}
                  </p>
                </div>

                <div className="flex flex-wrap gap-2 text-[10px] uppercase tracking-[0.22em] text-white/45">
                  <span className="rounded-full border border-white/8 bg-white/[0.03] px-2.5 py-1">{providers.length} configured</span>
                  <span className="rounded-full border border-white/8 bg-white/[0.03] px-2.5 py-1">{readyProviderCount} ready</span>
                  <span className="rounded-full border border-white/8 bg-white/[0.03] px-2.5 py-1">{localProviderCount} local</span>
                </div>
              </div>

              <div className="flex flex-col gap-3 border-t border-white/8 pt-4 lg:flex-row lg:items-center lg:justify-between">
                <label className={["flex items-center gap-3 rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-2.5 text-sm text-white/70 shadow-inner shadow-black/10", chromeState.compactRegistryChrome ? "min-w-0 flex-1 lg:max-w-[30rem]" : "min-w-0 flex-1 lg:max-w-[28rem]"].join(" ")}>
                  <Search className="h-4 w-4 text-cortex-400" />
                  <input
                    value={searchTerm}
                    onChange={(event) => setSearchTerm(event.target.value)}
                    placeholder="Search providers, endpoints, or models"
                    className="w-full bg-transparent text-sm text-white outline-none placeholder:text-white/28"
                  />
                </label>

                <div className="flex flex-wrap items-center gap-2">
                  <button
                    type="button"
                    onClick={() => void refreshProviders()}
                    disabled={isLoading}
                    className="flex h-10 items-center gap-2 rounded-xl border border-white/10 bg-white/[0.03] px-3.5 text-[11px] font-semibold uppercase tracking-[0.18em] text-white/80 transition hover:bg-white/[0.08] disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    <RefreshCw className={`h-3.5 w-3.5 ${isLoading && status === "loading" ? "animate-spin" : ""}`} />
                    Refresh
                  </button>
                  <button
                    type="button"
                    onClick={() => openPanel({ kind: "discovery", seedModel: adapterStatus?.model?.trim() || undefined })}
                    className="flex h-10 items-center gap-2 rounded-xl border border-cyan-400/18 bg-cyan-500/14 px-3.5 text-[11px] font-semibold uppercase tracking-[0.18em] text-cyan-100 transition hover:bg-cyan-500/20"
                  >
                    <Sparkles className="h-3.5 w-3.5" />
                    Discovery
                  </button>
                  <button
                    type="button"
                    onClick={() => openCreatePanel(defaultCreateTemplateId)}
                    className="flex h-10 items-center gap-2 rounded-xl bg-white px-3.5 text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-950 transition hover:bg-white/90"
                  >
                    <Plus className="h-3.5 w-3.5" />
                    Add provider
                  </button>

                  <details
                    className="relative"
                    open={detailsOpen}
                    onToggle={(event) => {
                      setDetailsOpen((event.currentTarget as HTMLDetailsElement).open);
                    }}
                  >
                    <summary className="flex h-10 list-none cursor-pointer items-center gap-2 rounded-xl border border-white/10 bg-white/[0.03] px-3.5 text-[11px] font-semibold uppercase tracking-[0.18em] text-white/75 transition hover:bg-white/[0.08]">
                      <SlidersHorizontal className="h-3.5 w-3.5" />
                      Filters
                      <ChevronDown className="h-3 w-3 opacity-70" />
                    </summary>
                    {detailsOpen ? (
                      <>
                        <div className="fixed inset-0 z-40" onClick={() => setDetailsOpen(false)} />
                        <div className="absolute right-0 z-50 mt-2 w-[22rem] max-w-[calc(100vw-1rem)] overflow-hidden rounded-[1.75rem] border border-white/10 bg-[linear-gradient(180deg,rgba(15,23,42,0.98),rgba(2,6,23,0.96))] shadow-2xl">
                          <div className="p-4">
                            <div className="flex items-start justify-between gap-3">
                              <div>
                                <p className="text-[10px] font-black uppercase tracking-[0.3em] text-cortex-500">Registry filters</p>
                                <p className="mt-1 text-[11px] leading-5 text-white/58">Keep the index dense here, and open detail only when needed.</p>
                              </div>
                              <button
                                type="button"
                                onClick={() => setDetailsOpen(false)}
                                className="rounded-full border border-white/10 bg-white/[0.04] px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-white/70"
                              >
                                Close
                              </button>
                            </div>

                            <div className="mt-4 grid gap-3">
                              <label className="grid gap-2 text-sm">
                                <span className="text-[10px] font-semibold uppercase tracking-[0.2em] text-white/40">Provider type</span>
                                <select
                                  value={providerTypeFilter}
                                  onChange={(event) => setProviderTypeFilter(event.target.value as ProviderRegistryTypeFilter)}
                                  className="rounded-2xl border border-white/10 bg-white/[0.05] px-3 py-2 text-sm text-white outline-none"
                                >
                                  <option value="all">All provider types</option>
                                  {providerTypeOrder.map((providerType) => (
                                    <option key={providerType} value={providerType}>
                                      {formatProviderTypeLabel(providerType)}
                                    </option>
                                  ))}
                                </select>
                              </label>

                              <label className="grid gap-2 text-sm">
                                <span className="text-[10px] font-semibold uppercase tracking-[0.2em] text-white/40">Readiness</span>
                                <select
                                  value={readinessFilter}
                                  onChange={(event) => setReadinessFilter(event.target.value as ProviderRegistryReadinessFilter)}
                                  className="rounded-2xl border border-white/10 bg-white/[0.05] px-3 py-2 text-sm text-white outline-none"
                                >
                                  <option value="all">All readiness states</option>
                                  <option value="ready">Ready</option>
                                  <option value="neutral">Uses default</option>
                                  <option value="attention">Needs credential</option>
                                  <option value="disabled">Disabled</option>
                                </select>
                              </label>
                            </div>
                          </div>
                        </div>
                      </>
                    ) : null}
                  </details>
                </div>
              </div>
            </div>
          </header>

          {statusCopy ? (
            <div className="mb-5 flex items-start gap-3 rounded-[1.5rem] border border-red-400/18 bg-red-500/6 px-4 py-4">
              <Shield className="mt-0.5 h-5 w-5 text-red-300" />
              <div className="space-y-1">
                <p className="font-medium text-red-200">{statusCopy.title}</p>
                <p className="text-sm text-white/62">{statusCopy.body}</p>
                {error ? (
                  <details className="text-xs text-white/45">
                    <summary className="cursor-pointer">Technical details</summary>
                    <p className="mt-2 font-mono">{error}</p>
                  </details>
                ) : null}
              </div>
            </div>
          ) : null}

          <div className="space-y-5">
            {registrySections.map(({ providerType, providers: sectionProviders }) => {
              const tone = providerTypeStyle(providerType);
              return (
                <section
                  key={providerType}
                  className={[
                    `overflow-hidden border ${tone.border} bg-white/[0.025] shadow-[0_24px_80px_-44px_rgba(0,0,0,0.8)]`,
                    chromeState.compactRegistryRows ? "rounded-[1.15rem]" : "rounded-[1.35rem]",
                  ].join(" ")}
                >
                  <div className="flex items-center justify-between gap-3 border-b border-white/8 px-4 py-3">
                    <div className="flex items-center gap-3">
                      <div className={`flex h-9 w-9 items-center justify-center rounded-xl border ${tone.border} bg-white/[0.03] ${tone.accent}`}>
                        {providerTypeIcon(providerType)}
                      </div>
                      <div>
                        <h2 className="text-[11px] font-semibold uppercase tracking-[0.24em] text-white/45">
                          {formatProviderTypeLabel(providerType)}
                        </h2>
                        <p className="text-xs text-white/50">{sectionProviders.length} providers</p>
                      </div>
                    </div>
                    <span className={`rounded-full border px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.18em] ${tone.badge}`}>
                      {formatProviderTypeLabel(providerType)}
                    </span>
                  </div>

                  <div className="hidden grid-cols-[minmax(0,2.3fr)_minmax(0,1.15fr)_minmax(0,1fr)_minmax(0,0.9fr)_auto] gap-4 border-b border-white/8 bg-white/[0.02] px-4 py-2.5 text-[10px] font-semibold uppercase tracking-[0.18em] text-white/35 md:grid">
                    <div>Provider</div>
                    <div>Model</div>
                    <div>Credential</div>
                    <div>Locality</div>
                    <div className="text-right">Action</div>
                  </div>

                  <div className="divide-y divide-white/6">
                    {sectionProviders.map((provider) => {
                      const readiness = getProviderReadiness(provider);
                      return (
                        <div
                          key={provider.id}
                          className="grid gap-3 px-4 py-3 transition hover:bg-white/[0.03] md:grid-cols-[minmax(0,2.3fr)_minmax(0,1.15fr)_minmax(0,1fr)_minmax(0,0.9fr)_auto] md:items-center"
                        >
                          <div className="min-w-0">
                            <div className="flex flex-wrap items-center gap-2">
                              <p className="truncate text-sm font-semibold text-white/92">{provider.name}</p>
                              <span className={`rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.16em] ${providerReadinessClasses(readiness.state)}`}>
                                {readiness.label}
                              </span>
                              {(provider.bindingIds ?? []).includes("llm.default") ? (
                                <span className="rounded-full border border-cyan-400/20 bg-cyan-500/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.16em] text-cyan-100">
                                  Default LLM
                                </span>
                              ) : null}
                            </div>
                            <div className="mt-2 flex flex-wrap gap-2 text-[10px] uppercase tracking-[0.18em] text-white/42">
                              <span className="rounded-full border border-white/8 bg-white/[0.03] px-2 py-1">{provider.id}</span>
                              {provider.llmType ? (
                                <span className="rounded-full border border-white/8 bg-white/[0.03] px-2 py-1">{provider.llmType}</span>
                              ) : null}
                              <span className="rounded-full border border-white/8 bg-white/[0.03] px-2 py-1">{endpointHost(provider.endpoint)}</span>
                            </div>
                          </div>

                          <div className="min-w-0">
                            <p className="truncate text-sm text-white/82">{formatProviderModelLabel(provider)}</p>
                          </div>

                          <div className="min-w-0">
                            <p className="truncate text-sm text-white/82">{formatProviderCredentialState(provider)}</p>
                          </div>

                          <div className="min-w-0">
                            <p className="truncate text-sm text-white/82">{formatProviderLocalityLabel(provider)}</p>
                          </div>

                          <div className="flex items-center justify-between gap-3 md:justify-end">
                            <div className="flex flex-wrap gap-2 text-[10px] uppercase tracking-[0.16em] text-white/42 md:justify-end">
                              {(provider.supportedModels ?? []).length > 0 ? (
                                <span className="rounded-full border border-white/8 bg-white/[0.03] px-2 py-1">
                                  {(provider.supportedModels ?? []).length} models
                                </span>
                              ) : null}
                              {provider.batchPolicy ? (
                                <span className="rounded-full border border-white/8 bg-white/[0.03] px-2 py-1">
                                  {formatProviderBatchSummary(provider.batchPolicy)}
                                </span>
                              ) : null}
                            </div>
                            <button
                              type="button"
                              onClick={() => openPanel({ kind: "provider", providerId: provider.id })}
                              className="inline-flex h-9 items-center gap-1.5 rounded-xl border border-cyan-400/18 bg-cyan-500/12 px-3 text-[11px] font-semibold uppercase tracking-[0.18em] text-cyan-100 transition hover:bg-cyan-500/18 hover:text-white"
                            >
                              Open details
                              <ChevronRight className="h-3.5 w-3.5" />
                            </button>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </section>
              );
            })}

            {registrySections.length === 0 && !isLoading ? (
              <div className="rounded-[1.75rem] border border-dashed border-white/14 py-16 text-center text-white/55">
                <Activity className="mx-auto mb-4 h-12 w-12 opacity-30" />
                <p className="text-base font-medium text-white/72">No providers match the current registry view.</p>
                <p className="mt-2 text-sm text-white/45">Adjust your search or filters, or create a new provider from the header.</p>
              </div>
            ) : null}

            {isLoading ? (
              <div className="grid gap-4">
                {[1, 2, 3].map((index) => (
                  <div key={index} className="h-28 animate-pulse rounded-[1.75rem] border border-white/8 bg-white/[0.03]" />
                ))}
              </div>
            ) : null}
          </div>
        </div>
      </div>

      {isProviderSheetOpen ? (
        <>
          <button
            type="button"
            onClick={closePanel}
            aria-label="Close provider sheet"
            className="fixed inset-0 z-[110] bg-slate-950/26 backdrop-blur-[6px]"
          />
          <aside className="fixed inset-y-0 right-0 z-[120] flex w-full max-w-[46rem] flex-col border-l border-white/10 bg-[linear-gradient(180deg,rgba(15,23,42,0.985),rgba(2,6,23,0.975))] shadow-[0_32px_120px_-48px_rgba(0,0,0,0.85)]">
            <div className="border-b border-white/8 px-5 py-4">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <p className="text-[10px] font-black uppercase tracking-[0.32em] text-cortex-500">
                    {isCreateSheet ? "Create provider" : "Provider detail"}
                  </p>
                  <h2 className="mt-2 text-2xl font-semibold tracking-tight text-white">{sheetTitle}</h2>
                  <p className="mt-2 text-sm leading-6 text-white/58">{sheetSubtitle}</p>
                </div>
                <button
                  type="button"
                  onClick={closePanel}
                  className="rounded-full border border-white/10 bg-white/[0.04] p-2 text-white/70 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                >
                  <X className="h-4 w-4" />
                </button>
              </div>

              <div className="mt-4 flex flex-wrap gap-2">
                {[
                  { id: "overview", label: "Overview" },
                  { id: "credentials", label: "Credentials" },
                  { id: "models", label: "Models" },
                  { id: "advanced", label: "Advanced" },
                ].map((tab) => (
                  <button
                    key={tab.id}
                    type="button"
                    onClick={() => setSheetTab(tab.id as ProviderSheetTab)}
                    className={[
                      "rounded-full border px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.18em] transition",
                      sheetTab === tab.id
                        ? "border-cyan-400/20 bg-cyan-500/12 text-cyan-100"
                        : "border-white/10 bg-white/[0.03] text-white/60 hover:bg-white/[0.06] hover:text-white/85",
                    ].join(" ")}
                  >
                    {tab.label}
                  </button>
                ))}
              </div>
            </div>

            <form onSubmit={handleSubmit} className="flex min-h-0 flex-1 flex-col">
              <div className="flex-1 overflow-y-auto px-5 py-5">
                {validationResult && sheetTab !== "advanced" ? (
                  <div
                    className={[
                      "mb-4 rounded-[1.5rem] border px-4 py-4",
                      validationResult.valid ? "border-emerald-400/24 bg-emerald-500/8" : "border-amber-400/24 bg-amber-500/8",
                    ].join(" ")}
                  >
                    <div className="flex items-center justify-between gap-3">
                      <div>
                        <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-white/40">Validation</p>
                        <p className="mt-1 text-sm text-white/84">
                          {validationResult.valid ? "Provider is ready to save." : "Latest validation probe needs attention."}
                        </p>
                      </div>
                      <span className={`rounded-full px-2 py-1 text-[10px] font-semibold ${validationResult.valid ? "text-emerald-200" : "text-amber-100"}`}>
                        {validationResult.valid ? "VALID" : "INVALID"}
                      </span>
                    </div>
                    {validationMessages.length > 0 ? (
                      <div className="mt-3 grid gap-1 text-xs text-white/58">
                        {validationMessages.map((message) => <p key={message}>{message}</p>)}
                      </div>
                    ) : null}
                    {!validationResult.valid && validationContextMessage ? (
                      <p className="mt-3 text-xs leading-5 text-white/52">{validationContextMessage}</p>
                    ) : null}
                  </div>
                ) : null}

                {sheetTab === "overview" ? (
                  <div className="space-y-5">
                    {isCreateSheet ? (
                      <section className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                        <div className="flex items-center justify-between gap-3">
                          <div>
                            <p className="text-[10px] font-black uppercase tracking-[0.28em] text-cortex-500">Template</p>
                            <p className="mt-1 text-sm text-white/60">Choose the starting shape for this new provider.</p>
                          </div>
                          <span className="rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-white/55">
                            {providerTemplateById(form.templateId).label}
                          </span>
                        </div>
                        <div className="mt-4 flex flex-wrap gap-2">
                          {providerTemplates.map((template) => (
                            <button
                              key={template.id}
                              type="button"
                              onClick={() => handleTemplateSelection(template.id)}
                              className={[
                                "rounded-full border px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.16em] transition",
                                form.templateId === template.id
                                  ? "border-cyan-400/20 bg-cyan-500/12 text-cyan-100"
                                  : "border-white/10 bg-white/[0.03] text-white/65 hover:bg-white/[0.06] hover:text-white/85",
                              ].join(" ")}
                            >
                              {template.label}
                            </button>
                          ))}
                        </div>
                      </section>
                    ) : null}

                    <div className="grid gap-4 md:grid-cols-2">
                      <label className="grid gap-2 text-sm">
                        <span className="text-white/55">Name</span>
                        <input
                          value={form.name}
                          onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
                          placeholder="Primary OpenRouter Provider"
                          className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white outline-none placeholder:text-white/26"
                        />
                      </label>
                      <label className="grid gap-2 text-sm">
                        <span className="text-white/55">{providerTemplateById(form.templateId).endpointLabel}</span>
                        <input
                          value={form.endpoint}
                          onChange={(event) => setForm((current) => ({ ...current, endpoint: event.target.value }))}
                          placeholder={providerTemplateById(form.templateId).endpointHint}
                          className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white outline-none placeholder:text-white/26"
                        />
                      </label>
                      <label className="grid gap-2 text-sm">
                        <span className="text-white/55">Default model</span>
                        {defaultModelControlState.control === "combobox" ? (
                          <div className="grid gap-2">
                            <input
                              value={form.defaultModel}
                              onChange={(event) => setForm((current) => ({ ...current, defaultModel: event.target.value }))}
                              onFocus={() => setIsModelComboboxOpen(true)}
                              onBlur={() => {
                                if (modelComboboxCloseTimerRef.current) {
                                  window.clearTimeout(modelComboboxCloseTimerRef.current);
                                }
                                modelComboboxCloseTimerRef.current = window.setTimeout(() => {
                                  setIsModelComboboxOpen(false);
                                }, 120);
                              }}
                              placeholder={providerTemplateById(form.templateId).defaultModelHint}
                              className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white outline-none placeholder:text-white/26"
                            />
                            {isModelComboboxOpen && modelComboboxSuggestions.length > 0 ? (
                              <div className="rounded-[1.1rem] border border-white/10 bg-[linear-gradient(180deg,rgba(15,23,42,0.96),rgba(2,6,23,0.94))] p-1 shadow-[0_20px_60px_-30px_rgba(0,0,0,0.8)]">
                                {modelComboboxSuggestions.map((model) => (
                                  <button
                                    key={model}
                                    type="button"
                                    onMouseDown={(event) => {
                                      event.preventDefault();
                                      setForm((current) => ({ ...current, defaultModel: model }));
                                      setIsModelComboboxOpen(false);
                                    }}
                                    className={[
                                      "flex w-full items-center justify-between rounded-xl px-3 py-2 text-left text-sm transition",
                                      form.defaultModel === model
                                        ? "bg-cyan-500/14 text-cyan-100"
                                        : "text-white/80 hover:bg-white/[0.05]",
                                    ].join(" ")}
                                  >
                                    <span className="truncate">{model}</span>
                                    {form.defaultModel === model ? (
                                      <span className="text-[10px] font-semibold uppercase tracking-[0.16em] text-cyan-100">Selected</span>
                                    ) : null}
                                  </button>
                                ))}
                              </div>
                            ) : null}
                          </div>
                        ) : (
                          <input
                            value={form.defaultModel}
                            onChange={(event) => setForm((current) => ({ ...current, defaultModel: event.target.value }))}
                            placeholder={providerTemplateById(form.templateId).defaultModelHint}
                            className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white outline-none placeholder:text-white/26"
                          />
                        )}
                        <span className="text-xs leading-5 text-white/42">{defaultModelControlState.helperText}</span>
                      </label>
                      <div className="grid gap-2 text-sm">
                        <span className="text-white/55">Provider family</span>
                        <div className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white/86">
                          {selectedProviderFamilyLabel}
                        </div>
                        <span className="text-xs leading-5 text-white/42">Family identifies the vendor/runtime shape, like OpenRouter or Ollama.</span>
                      </div>
                    </div>

                    <label className="inline-flex items-center gap-3 rounded-[1.25rem] border border-white/10 bg-white/[0.03] px-4 py-3 text-sm text-white/62">
                      <input
                        type="checkbox"
                        checked={form.enabled}
                        onChange={(event) => setForm((current) => ({ ...current, enabled: event.target.checked }))}
                        className="h-4 w-4 rounded border-white/20 bg-white/[0.05]"
                      />
                      Enable this provider
                    </label>

                    {selectedProvider ? (
                      <div className="grid gap-3 md:grid-cols-2">
                        <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                          <div className="flex items-start justify-between gap-3">
                            <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Operational status</p>
                            {selectedProviderOperationalReadiness ? (
                              <span className={`rounded-full border px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] ${providerReadinessClasses(selectedProviderOperationalReadiness.state)}`}>
                                {selectedProviderOperationalReadiness.label}
                              </span>
                            ) : null}
                          </div>
                          <p className="mt-3 text-sm leading-6 text-white/68">{selectedProviderOperationalReadiness?.detail}</p>
                          {selectedProviderOperationalReadiness && selectedProviderOperationalReadiness.issues.length > 0 ? (
                            <div className="mt-3 grid gap-1 text-xs leading-5 text-amber-100/80">
                              {selectedProviderOperationalReadiness.issues.slice(0, 3).map((issue) => <p key={issue}>{issue}</p>)}
                            </div>
                          ) : null}
                        </div>
                        <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                          <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Provider summary</p>
                          <div className="mt-2 grid gap-2 text-sm text-white/62">
                            <p><span className="text-white/38">Capability:</span> {formatProviderTypeLabel(selectedProvider.providerType)}</p>
                            <p><span className="text-white/38">Family:</span> {selectedProviderFamilyLabel}</p>
                            <p><span className="text-white/38">Locality:</span> {formatProviderLocalityLabel(selectedProvider)}</p>
                            <p><span className="text-white/38">Catalog:</span> {discoveredModelOptions.length > 0 ? `${discoveredModelOptions.length} models loaded` : "Not loaded yet"}</p>
                          </div>
                        </div>
                      </div>
                    ) : null}
                  </div>
                ) : null}

                {sheetTab === "credentials" ? (
                  <div className="space-y-5">
                    <div className="grid gap-4 md:grid-cols-2">
                      <label className="grid gap-2 text-sm">
                        <span className="text-white/55">Linked credential</span>
                        <select
                          value={form.credentialBindingId}
                          onChange={(event) => setForm((current) => ({ ...current, credentialBindingId: event.target.value }))}
                          className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white outline-none"
                        >
                          <option value="">Use system default</option>
                          {visibleCredentialBindings.map((binding) => (
                            <option key={binding.credentialBindingId} value={binding.credentialBindingId}>
                              {binding.label || binding.credentialBindingId}
                            </option>
                          ))}
                        </select>
                      </label>
                      <label className="grid gap-2 text-sm">
                        <span className="text-white/55">New API key</span>
                        <input
                          value={form.apiKey}
                          onChange={(event) => setForm((current) => ({ ...current, apiKey: event.target.value }))}
                          type="password"
                          placeholder={hasStoredCatalogCredential ? "Paste a new key to replace or refresh this credential" : "Paste a key to create and refresh a credential"}
                          className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white outline-none placeholder:text-white/26"
                        />
                      </label>
                    </div>

                    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                      <p className="text-[10px] font-black uppercase tracking-[0.26em] text-cortex-500">Current state</p>
                      <p className="mt-2 text-sm leading-6 text-white/58">{providerCatalogState.helperText}</p>
                      {selectedProviderOperationalReadiness?.detail ? (
                        <p className="mt-2 text-sm leading-6 text-white/52">{selectedProviderOperationalReadiness.detail}</p>
                      ) : null}
                      {!validationResult?.valid && validationContextMessage ? (
                        <p className="mt-2 text-xs leading-5 text-white/48">{validationContextMessage}</p>
                      ) : null}
                      <div className="mt-3 grid gap-2 text-sm text-white/68">
                        <div className="flex items-center justify-between gap-3">
                          <span>Provider status</span>
                          <span className="text-right text-white/88">
                            {selectedProviderOperationalReadiness?.label || "Not assessed"}
                          </span>
                        </div>
                        <div className="flex items-center justify-between gap-3">
                          <span>Latest validation</span>
                          <span className="text-right text-white/88">
                            {validationResult ? (validationResult.valid ? "Passed" : "Needs attention") : "Not run in this session"}
                          </span>
                        </div>
                        <div className="flex items-center justify-between gap-3">
                          <span>Credential label</span>
                          <span className="text-right text-white/88">
                            {selectedProviderCredentialBinding?.label || form.credentialBindingId || "System default"}
                          </span>
                        </div>
                        <div className="flex items-center justify-between gap-3">
                          <span>Credential status</span>
                          <span className="text-right text-white/88">
                            {selectedProvider ? formatProviderCredentialState(selectedProvider) : "Unlinked"}
                          </span>
                        </div>
                      </div>
                      {selectedProviderOperationalReadiness && selectedProviderOperationalReadiness.issues.length > 0 ? (
                        <div className="mt-4 rounded-[1.25rem] border border-amber-400/16 bg-amber-500/8 px-3 py-3 text-xs leading-5 text-amber-100/82">
                          <p className="font-semibold uppercase tracking-[0.16em] text-amber-100/72">Latest issues</p>
                          <div className="mt-2 grid gap-1">
                            {selectedProviderOperationalReadiness.issues.map((issue) => <p key={issue}>{issue}</p>)}
                          </div>
                        </div>
                      ) : null}
                    </div>

                    <div className="grid gap-3">
                      {visibleCredentialBindings.length > 0 ? (
                        visibleCredentialBindings.map((binding) => (
                          <div key={binding.credentialBindingId} className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                            <div className="flex items-start justify-between gap-3">
                              <div>
                                <p className="text-sm font-semibold text-white">{binding.label || binding.credentialBindingId}</p>
                                <p className="mt-1 text-[11px] text-white/45">{binding.source || "system"}</p>
                              </div>
                              <span className={`rounded-full px-2 py-1 text-[10px] font-semibold ${binding.hasCredential ? "text-cyan-100" : "text-white/55"}`}>
                                {binding.hasCredential ? "KEY STORED" : "RECORD ONLY"}
                              </span>
                            </div>
                          </div>
                        ))
                      ) : (
                        <div className="rounded-[1.5rem] border border-dashed border-white/12 px-4 py-6 text-sm text-white/50">
                          No linked credential records are registered for this provider yet.
                        </div>
                      )}
                    </div>
                  </div>
                ) : null}

                {sheetTab === "models" ? (
                  <div className="space-y-5">
                    <div className="grid gap-3 md:grid-cols-3">
                      <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                        <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Base URL</p>
                        <p className="mt-2 break-all text-sm text-white/84">{discoveryBaseUrl}</p>
                      </div>
                      <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                        <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Current model</p>
                        <p className="mt-2 text-sm text-white/84">{form.defaultModel || adapterStatus?.model || "Not selected"}</p>
                      </div>
                      <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                        <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Catalog size</p>
                        <p className="mt-2 text-sm text-white/84">{discoveredModelOptions.length} visible models</p>
                      </div>
                    </div>

                    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] px-4 py-4">
                      <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                        <div>
                          <p className="text-[10px] font-black uppercase tracking-[0.24em] text-cortex-500">Catalog refresh</p>
                          <p className="mt-2 text-sm leading-6 text-white/60">{providerCatalogState.helperText}</p>
                        </div>
                        <button
                          type="button"
                          onClick={() => void refreshProviderCatalog()}
                          disabled={!providerCatalogState.canRefresh || isValidating}
                          className="rounded-full border border-white/10 bg-white/[0.03] px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-white/72 transition hover:bg-white/[0.06] hover:text-white disabled:cursor-not-allowed disabled:opacity-60"
                        >
                          {isValidating ? "Refreshing..." : "Refresh catalog"}
                        </button>
                      </div>
                    </div>

                    <label className="flex items-center gap-3 rounded-full border border-white/10 bg-white/[0.04] px-4 py-2.5 text-sm text-white/70">
                      <Search className="h-4 w-4 text-cortex-400" />
                      <input
                        value={modelSearchTerm}
                        onChange={(event) => setModelSearchTerm(event.target.value)}
                        placeholder="Filter models"
                        className="w-full bg-transparent text-sm text-white outline-none placeholder:text-white/28"
                      />
                    </label>

                    {selectedProviderRuntimeNote ? (
                      <div className="rounded-[1.5rem] border border-amber-400/18 bg-amber-500/8 px-4 py-4 text-sm text-amber-100/85">
                        Runtime discovery note: {selectedProviderRuntimeNote}
                      </div>
                    ) : null}

                    <div className="grid gap-2">
                      {filteredSheetModels.length > 0 ? (
                        filteredSheetModels.map((model) => (
                          <button
                            key={model}
                            type="button"
                            onClick={() => setForm((current) => ({ ...current, defaultModel: model }))}
                            className={[
                              "flex items-center justify-between rounded-[1.25rem] border px-4 py-3 text-left transition",
                              form.defaultModel === model
                                ? "border-cyan-400/20 bg-cyan-500/10 text-cyan-100"
                                : "border-white/10 bg-white/[0.03] text-white/72 hover:bg-white/[0.06]",
                            ].join(" ")}
                          >
                            <span className="truncate text-sm font-medium">{model}</span>
                            <span className="text-[10px] font-semibold uppercase tracking-[0.16em]">
                              {form.defaultModel === model ? "Selected" : "Use"}
                            </span>
                          </button>
                        ))
                      ) : (
                        <div className="rounded-[1.5rem] border border-dashed border-white/12 px-4 py-6 text-sm text-white/50">
                          {providerCatalogState.canRefresh
                            ? "No models are loaded yet. Refresh the catalog or adjust the search."
                            : "No models are loaded yet. Add a key in Credentials to refresh the catalog."}
                        </div>
                      )}
                    </div>
                  </div>
                ) : null}

                {sheetTab === "advanced" ? (
                  <div className="space-y-5">
                    <div className="grid gap-4 md:grid-cols-2">
                      {isCreateSheet ? (
                        <>
                          <label className="grid gap-2 text-sm">
                            <span className="text-white/55">Provider ID</span>
                            <input
                              value={form.providerId}
                              onChange={(event) => setForm((current) => ({ ...current, providerId: event.target.value }))}
                              placeholder={buildTemplateProviderId(form.templateId)}
                              className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white outline-none placeholder:text-white/26"
                            />
                            <span className="text-xs leading-5 text-white/42">Stable registry key. The default suggestion follows the selected template/runtime.</span>
                          </label>
                          <label className="grid gap-2 text-sm">
                            <span className="text-white/55">Provider type</span>
                            <select
                              value={form.providerType}
                              onChange={(event) => setForm((current) => ({ ...current, providerType: event.target.value as ProviderType }))}
                              className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white outline-none"
                            >
                              {providerTypeOrder.map((providerType) => (
                                <option key={providerType} value={providerType}>
                                  {formatProviderTypeLabel(providerType)}
                                </option>
                              ))}
                            </select>
                            <span className="text-xs leading-5 text-white/42">Type is the capability class the provider serves, like LLM or Embedding.</span>
                          </label>
                        </>
                      ) : (
                        <>
                          <div className="grid gap-2 text-sm">
                            <span className="text-white/55">Provider ID</span>
                            <div className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white/86">
                              {selectedProvider?.id}
                            </div>
                          </div>
                          <div className="grid gap-2 text-sm">
                            <span className="text-white/55">Provider type</span>
                            <div className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white/86">
                              {selectedProvider ? formatProviderTypeLabel(selectedProvider.providerType) : formatProviderTypeLabel(form.providerType)}
                            </div>
                          </div>
                        </>
                      )}
                      <div className="grid gap-2 text-sm md:col-span-2">
                        <span className="text-white/55">Active bindings</span>
                        <div className="rounded-2xl border border-white/10 bg-white/[0.04] px-3 py-3 text-sm text-white/78">
                          {selectedProviderBindingLabels.length > 0 ? (
                            <div className="flex flex-wrap gap-2">
                              {selectedProviderBindingLabels.map((bindingLabel) => (
                                <span key={bindingLabel} className="rounded-full border border-cyan-400/18 bg-cyan-500/10 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-cyan-100">
                                  {bindingLabel}
                                </span>
                              ))}
                            </div>
                          ) : (
                            "No adapter bindings yet"
                          )}
                        </div>
                      </div>
                    </div>

                    {selectedProvider ? (
                      <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                        <p className="text-[10px] font-black uppercase tracking-[0.26em] text-cortex-500">Runtime details</p>
                        <div className="mt-3 grid gap-2 text-sm text-white/68">
                          <div className="flex items-center justify-between gap-3">
                            <span>Family</span>
                            <span className="text-right text-white/88">{selectedProviderFamilyLabel}</span>
                          </div>
                          <div className="flex items-center justify-between gap-3">
                            <span>Locality</span>
                            <span className="text-right text-white/88">{formatProviderLocalityLabel(selectedProvider)}</span>
                          </div>
                          <div className="flex items-start justify-between gap-3">
                            <span>Bindings</span>
                            <div className="flex flex-wrap justify-end gap-2">
                              {selectedProviderBindingLabels.length > 0 ? selectedProviderBindingLabels.map((bindingLabel) => (
                                <span key={bindingLabel} className="rounded-full border border-white/10 bg-white/[0.03] px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-white/75">
                                  {bindingLabel}
                                </span>
                              )) : (
                                <span className="text-right text-white/88">Not bound</span>
                              )}
                            </div>
                          </div>
                          <div className="flex items-center justify-between gap-3">
                            <span>Topology</span>
                            <span className="max-w-[22rem] break-words text-right text-white/88">{formatProviderTopologySummary(selectedProvider)}</span>
                          </div>
                        </div>
                      </div>
                    ) : null}

                    <label className="grid gap-2 text-sm">
                      <span className="text-white/55">Metadata JSON</span>
                      <textarea
                        value={form.metadataJson}
                        onChange={(event) => setForm((current) => ({ ...current, metadataJson: event.target.value }))}
                        rows={10}
                        className="rounded-[1.5rem] border border-white/10 bg-white/[0.04] px-3 py-3 font-mono text-xs text-white outline-none"
                      />
                    </label>
                  </div>
                ) : null}
              </div>

              <div className="border-t border-white/8 px-5 py-4">
                <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                  <div className="min-h-5 text-sm">
                    {submitError ? <span className="text-red-300">{submitError}</span> : null}
                    {!submitError && submitMessage ? <span className={submitMessageTone}>{submitMessage}</span> : null}
                  </div>
                  <div className="flex flex-wrap items-center gap-2">
                    {isCreateSheet ? (
                      <button
                        type="button"
                        onClick={() => openPanel({ kind: "discovery", seedModel: form.defaultModel.trim() || undefined })}
                        className="rounded-full border border-white/10 bg-white/[0.03] px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-white/70 transition hover:bg-white/[0.06] hover:text-white"
                      >
                        Open discovery
                      </button>
                    ) : null}
                    {!isCreateSheet && selectedProvider?.providerType === "Llm" && !(selectedProvider.bindingIds ?? []).includes("llm.default") ? (
                      <button
                        type="button"
                        onClick={() => void bindProviderAsDefaultLlm(selectedProvider.id)}
                        disabled={isSubmitting || isValidating}
                        className="rounded-full border border-cyan-400/18 bg-cyan-500/10 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-cyan-100 transition hover:bg-cyan-500/16 disabled:cursor-not-allowed disabled:opacity-60"
                      >
                        Bind to LLM default
                      </button>
                    ) : null}
                    <button
                      type="submit"
                      disabled={isSubmitting || isValidating}
                      className="rounded-full bg-white px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-950 transition hover:bg-white/90 disabled:cursor-not-allowed disabled:opacity-60"
                    >
                      {isValidating ? "Validating..." : isSubmitting ? "Saving..." : isCreateSheet ? "Create provider" : "Save provider"}
                    </button>
                  </div>
                </div>
              </div>
            </form>
          </aside>
        </>
      ) : null}

      {panelState.kind === "discovery" ? (
        <>
          <button
            type="button"
            onClick={closePanel}
            aria-label="Close provider discovery"
            className="fixed inset-0 z-[110] bg-slate-950/26 backdrop-blur-[6px]"
          />
          <aside className="fixed inset-y-0 right-0 z-[120] flex w-full max-w-[40rem] flex-col border-l border-white/10 bg-[linear-gradient(180deg,rgba(15,23,42,0.985),rgba(2,6,23,0.975))] shadow-[0_32px_120px_-48px_rgba(0,0,0,0.85)]">
            <div className="border-b border-white/8 px-5 py-4">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <p className="text-[10px] font-black uppercase tracking-[0.32em] text-cortex-500">Provider discovery</p>
                  <h2 className="mt-2 text-2xl font-semibold tracking-tight text-white">Discovery lanes</h2>
                  <p className="mt-2 text-sm leading-6 text-white/58">
                    Separate active runtime catalog discovery from machine-local runtime discovery so cloud bindings do not hide local providers.
                  </p>
                </div>
                <button
                  type="button"
                  onClick={closePanel}
                  className="rounded-full border border-white/10 bg-white/[0.04] p-2 text-white/70 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                >
                  <X className="h-4 w-4" />
                </button>
              </div>
              <div className="mt-4 inline-flex rounded-full border border-white/10 bg-white/[0.04] p-1">
                {[
                  { id: "runtime_catalog", label: "Bound runtime catalog" },
                  { id: "local_runtimes", label: "Local runtimes" },
                ].map((lane) => (
                  <button
                    key={lane.id}
                    type="button"
                    onClick={() => setDiscoveryLane(lane.id as ProviderDiscoveryLane)}
                    className={[
                      "rounded-full px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] transition",
                      discoveryLane === lane.id
                        ? "bg-white text-slate-950"
                        : "text-white/68 hover:bg-white/[0.06] hover:text-white",
                    ].join(" ")}
                  >
                    {lane.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex-1 overflow-y-auto px-5 py-5">
              {discoveryLane === "runtime_catalog" ? (
                <>
                  <div className="grid gap-3 md:grid-cols-3">
                    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                      <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Bound slot</p>
                      <p className="mt-2 text-sm font-semibold text-white">{formatProviderBindingLabel(defaultLlmBinding?.bindingId || "llm.default")}</p>
                      <p className="mt-2 text-xs leading-5 text-white/50">{adapterStatus?.enabled ? "Current execution provider is reachable" : "Execution runtime is idle or unavailable"}</p>
                    </div>
                    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                      <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Endpoint</p>
                      <p className="mt-2 break-all text-sm text-white/84">{discoveryBaseUrl}</p>
                    </div>
                    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                      <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Active model</p>
                      <p className="mt-2 text-sm text-white/84">{adapterStatus?.model || selectedDiscoveryModel || "Unavailable"}</p>
                    </div>
                  </div>

                  <div className="mt-4 rounded-[1.5rem] border border-white/10 bg-white/[0.03] px-4 py-4 text-sm text-white/60">
                    {discoveryCatalogDescription}
                  </div>

                  {adapterStatusError || statusCopy ? (
                    <div className="mt-4 rounded-[1.5rem] border border-amber-400/18 bg-amber-500/8 px-4 py-4 text-sm text-amber-100/85">
                      {adapterStatusError || statusCopy?.body}
                    </div>
                  ) : null}

                  <label className="mt-5 flex items-center gap-3 rounded-full border border-white/10 bg-white/[0.04] px-4 py-2.5 text-sm text-white/70">
                    <Search className="h-4 w-4 text-cortex-400" />
                    <input
                      value={discoverySearchTerm}
                      onChange={(event) => setDiscoverySearchTerm(event.target.value)}
                      placeholder="Filter runtime models"
                      className="w-full bg-transparent text-sm text-white outline-none placeholder:text-white/28"
                    />
                  </label>

                  <div className="mt-5 grid gap-2">
                    {filteredRuntimeDiscoveryModels.length > 0 ? (
                      filteredRuntimeDiscoveryModels.map((model) => (
                        <button
                          key={model}
                          type="button"
                          onClick={() => setSelectedDiscoveryModel(model)}
                          className={[
                            "flex items-center justify-between rounded-[1.25rem] border px-4 py-3 text-left transition",
                            selectedDiscoveryModel === model
                              ? "border-cyan-400/20 bg-cyan-500/10 text-cyan-100"
                              : "border-white/10 bg-white/[0.03] text-white/72 hover:bg-white/[0.06]",
                          ].join(" ")}
                        >
                          <span className="truncate text-sm font-medium">{model}</span>
                          <span className="text-[10px] font-semibold uppercase tracking-[0.16em]">
                            {selectedDiscoveryModel === model ? "Selected" : "Pick"}
                          </span>
                        </button>
                      ))
                    ) : (
                      <div className="rounded-[1.5rem] border border-dashed border-white/12 px-4 py-6 text-sm text-white/50">
                        No bound-runtime models match the current search.
                      </div>
                    )}
                  </div>
                </>
              ) : (
                <>
                  <div className="grid gap-3 md:grid-cols-3">
                    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                      <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Local runtimes</p>
                      <p className="mt-2 text-sm font-semibold text-white">{localDiscoveryRecords.length} detected</p>
                      <p className="mt-2 text-xs leading-5 text-white/50">Machine-local providers stay visible even when the default binding points to cloud.</p>
                    </div>
                    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                      <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Autodetect target</p>
                      <p className="mt-2 break-all text-sm text-white/84">http://127.0.0.1:11434</p>
                    </div>
                    <div className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                      <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-white/38">Purpose</p>
                      <p className="mt-2 text-sm text-white/84">Detect installed local models and surface them as ordinary provider records.</p>
                    </div>
                  </div>

                  <div className="mt-4 rounded-[1.5rem] border border-white/10 bg-white/[0.03] px-4 py-4 text-sm text-white/60">
                    Local discovery checks for machine-local runtimes like Ollama, loads their catalogs, and keeps them available even when `llm.default` is bound elsewhere.
                  </div>

                  {localDiscoveryRecords.length === 0 ? (
                    <div className="mt-4 rounded-[1.5rem] border border-amber-400/18 bg-amber-500/8 px-4 py-4 text-sm text-amber-100/85">
                      No local runtimes are currently registered. Refresh discovery to probe Ollama and other local endpoints on this system.
                    </div>
                  ) : null}

                  <label className="mt-5 flex items-center gap-3 rounded-full border border-white/10 bg-white/[0.04] px-4 py-2.5 text-sm text-white/70">
                    <Search className="h-4 w-4 text-cortex-400" />
                    <input
                      value={discoverySearchTerm}
                      onChange={(event) => setDiscoverySearchTerm(event.target.value)}
                      placeholder="Filter local runtimes or models"
                      className="w-full bg-transparent text-sm text-white outline-none placeholder:text-white/28"
                    />
                  </label>

                  <div className="mt-5 grid gap-3">
                    {filteredLocalDiscoveryRecords.length > 0 ? (
                      filteredLocalDiscoveryRecords.map((record) => {
                        const linkedProvider = providers.find((provider) => provider.id === record.providerId) ?? null;
                        const isBoundToDefault = linkedProvider ? (linkedProvider.bindingIds ?? []).includes("llm.default") : false;
                        return (
                          <div key={record.providerId} className="rounded-[1.5rem] border border-white/10 bg-white/[0.03] p-4">
                            <div className="flex items-start justify-between gap-3">
                              <div>
                                <p className="text-sm font-semibold text-white">{linkedProvider?.name || record.providerKind || record.providerId}</p>
                                <p className="mt-1 break-all text-xs text-white/45">{record.endpoint}</p>
                              </div>
                              <span className="rounded-full border border-white/10 bg-white/[0.04] px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-white/60">
                                {(record.topology?.localityKind || "Local").toUpperCase()}
                              </span>
                            </div>
                            <div className="mt-4 grid gap-3 md:grid-cols-3">
                              <div>
                                <p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-white/38">Default model</p>
                                <p className="mt-1 text-sm text-white/80">{record.defaultModel || record.supportedModels?.[0] || "Not detected"}</p>
                              </div>
                              <div>
                                <p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-white/38">Catalog size</p>
                                <p className="mt-1 text-sm text-white/80">{record.supportedModels?.length ?? 0} models</p>
                              </div>
                              <div>
                                <p className="text-[10px] font-semibold uppercase tracking-[0.16em] text-white/38">Bindings</p>
                                <p className="mt-1 text-sm text-white/80">
                                  {linkedProvider?.bindingIds?.length ? linkedProvider.bindingIds.map(formatProviderBindingLabel).join(", ") : "Not bound"}
                                </p>
                              </div>
                            </div>
                            <div className="mt-4 flex flex-wrap gap-2">
                              <button
                                type="button"
                                onClick={() => openPanel({ kind: "provider", providerId: linkedProvider?.id || record.providerId })}
                                className="rounded-full border border-white/10 bg-white/[0.03] px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-white/72 transition hover:bg-white/[0.06] hover:text-white"
                              >
                                Open provider
                              </button>
                              {linkedProvider && !isBoundToDefault && linkedProvider.providerType === "Llm" ? (
                                <button
                                  type="button"
                                  onClick={() => void bindProviderAsDefaultLlm(linkedProvider.id)}
                                  className="rounded-full bg-cyan-500/18 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-cyan-100 transition hover:bg-cyan-500/24"
                                >
                                  Bind to LLM default
                                </button>
                              ) : null}
                            </div>
                          </div>
                        );
                      })
                    ) : (
                      <div className="rounded-[1.5rem] border border-dashed border-white/12 px-4 py-6 text-sm text-white/50">
                        No local runtimes match the current discovery search.
                      </div>
                    )}
                  </div>
                </>
              )}
            </div>

            <div className="border-t border-white/8 px-5 py-4">
              <div className="flex flex-wrap items-center gap-2">
                <button
                  type="button"
                  onClick={() => void handleDiscoverLocalProviders()}
                  disabled={isLoading || isRefreshingAdapterModels}
                  className="rounded-full border border-white/10 bg-white/[0.03] px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-white/70 transition hover:bg-white/[0.06] hover:text-white disabled:cursor-not-allowed disabled:opacity-60"
                >
                  {isRefreshingAdapterModels ? "Refreshing..." : discoveryLane === "runtime_catalog" ? "Refresh discovery" : "Refresh local discovery"}
                </button>
                {discoveryLane === "runtime_catalog" ? (
                  <>
                    <button
                      type="button"
                      onClick={() => openCreatePanel(resolveProviderTemplate(undefined, adapterStatus?.baseUrl ?? null).id, selectedDiscoveryModel || adapterStatus?.model || undefined)}
                      disabled={!selectedDiscoveryModel && !adapterStatus?.model}
                      className="rounded-full bg-cyan-500/18 px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-cyan-100 transition hover:bg-cyan-500/24 disabled:cursor-not-allowed disabled:opacity-60"
                    >
                      Create provider from runtime model
                    </button>
                    <button
                      type="button"
                      onClick={() => {
                        if (defaultLlmBinding?.boundProviderId) {
                          openPanel({ kind: "provider", providerId: defaultLlmBinding.boundProviderId });
                        }
                      }}
                      disabled={!defaultLlmBinding?.boundProviderId}
                      className="rounded-full border border-white/10 bg-white/[0.03] px-4 py-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-white/70 transition hover:bg-white/[0.06] hover:text-white"
                    >
                      Open bound provider
                    </button>
                  </>
                ) : null}
              </div>
            </div>
          </aside>
        </>
      ) : null}
    </div>
  );
};
