import type { ProviderRecord } from "../../contracts.ts";
import { providerTypeOrder, type ProviderType } from "./providerForm.ts";
import { getProviderReadiness, type ProviderReadinessState } from "./providerTopology.ts";
import type { ProviderTemplateId } from "./providerTemplates.ts";

export type ProviderRegistryReadinessFilter =
  | "all"
  | ProviderReadinessState;

export type ProviderRegistryTypeFilter = "all" | ProviderType;

export type ProviderRegistryPanelState =
  | { kind: "none" }
  | { kind: "provider"; providerId: string }
  | { kind: "create"; templateId?: ProviderTemplateId; seedModel?: string }
  | { kind: "discovery"; providerId?: string; seedModel?: string };

export interface ProviderRegistrySection {
  providerType: ProviderType;
  providers: ProviderRecord[];
}

const PANEL_PARAM = "panel";
const PROVIDER_ID_PARAM = "providerId";
const TEMPLATE_ID_PARAM = "templateId";
const SEED_MODEL_PARAM = "seedModel";

const VALID_TEMPLATE_IDS = new Set<ProviderTemplateId>(["openrouter", "openai", "ollama", "manual"]);

function normalizePanelValue(value: string | null): string {
  return value?.trim().toLowerCase() ?? "";
}

function trimOrUndefined(value: string | null): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

export function readProviderRegistryPanelState(searchParams: URLSearchParams): ProviderRegistryPanelState {
  const panel = normalizePanelValue(searchParams.get(PANEL_PARAM));
  const providerId = trimOrUndefined(searchParams.get(PROVIDER_ID_PARAM));
  const templateId = trimOrUndefined(searchParams.get(TEMPLATE_ID_PARAM));
  const seedModel = trimOrUndefined(searchParams.get(SEED_MODEL_PARAM));

  if (panel === "provider" && providerId) {
    return { kind: "provider", providerId };
  }

  if (panel === "create") {
    return {
      kind: "create",
      templateId: templateId && VALID_TEMPLATE_IDS.has(templateId as ProviderTemplateId)
        ? (templateId as ProviderTemplateId)
        : undefined,
      seedModel,
    };
  }

  if (panel === "discovery") {
    return {
      kind: "discovery",
      providerId,
      seedModel,
    };
  }

  return { kind: "none" };
}

export function writeProviderRegistryPanelState(
  current: URLSearchParams,
  next: ProviderRegistryPanelState,
): URLSearchParams {
  const params = new URLSearchParams(current);
  params.delete(PANEL_PARAM);
  params.delete(PROVIDER_ID_PARAM);
  params.delete(TEMPLATE_ID_PARAM);
  params.delete(SEED_MODEL_PARAM);

  switch (next.kind) {
    case "provider":
      params.set(PANEL_PARAM, "provider");
      params.set(PROVIDER_ID_PARAM, next.providerId);
      break;
    case "create":
      params.set(PANEL_PARAM, "create");
      if (next.templateId) {
        params.set(TEMPLATE_ID_PARAM, next.templateId);
      }
      if (next.seedModel) {
        params.set(SEED_MODEL_PARAM, next.seedModel);
      }
      break;
    case "discovery":
      params.set(PANEL_PARAM, "discovery");
      if (next.providerId) {
        params.set(PROVIDER_ID_PARAM, next.providerId);
      }
      if (next.seedModel) {
        params.set(SEED_MODEL_PARAM, next.seedModel);
      }
      break;
    case "none":
    default:
      break;
  }

  return params;
}

function matchesSearch(provider: ProviderRecord, searchTerm: string): boolean {
  const normalizedSearch = searchTerm.trim().toLowerCase();
  if (!normalizedSearch) {
    return true;
  }

  return [
    provider.name,
    provider.providerFamily,
    provider.hostId,
    provider.endpoint,
    provider.defaultModel,
  ]
    .filter((value): value is string => typeof value === "string")
    .some((value) => value.toLowerCase().includes(normalizedSearch));
}

export function buildProviderRegistrySections(
  providers: ProviderRecord[],
  options: {
    searchTerm: string;
    providerType: ProviderRegistryTypeFilter;
    readiness: ProviderRegistryReadinessFilter;
  },
): ProviderRegistrySection[] {
  const grouped = new Map<ProviderType, ProviderRecord[]>();
  providerTypeOrder.forEach((type) => grouped.set(type, []));

  for (const provider of providers) {
    const readiness = getProviderReadiness(provider).state;
    const nextType = provider.providerType as ProviderType;
    if (options.providerType !== "all" && nextType !== options.providerType) {
      continue;
    }
    if (options.readiness !== "all" && readiness !== options.readiness) {
      continue;
    }
    if (!matchesSearch(provider, options.searchTerm)) {
      continue;
    }

    const bucket = grouped.get(nextType);
    if (bucket) {
      bucket.push(provider);
    }
  }

  return providerTypeOrder
    .map((providerType) => ({
      providerType,
      providers: (grouped.get(providerType) ?? []).slice().sort((left, right) =>
        left.name.localeCompare(right.name) || left.id.localeCompare(right.id),
      ),
    }))
    .filter((section) => section.providers.length > 0);
}

export function validateProviderDraftInput(input: {
  providerId: string;
  providerName: string;
  providerEndpoint: string;
  metadataJson: string;
}): string | null {
  if (!input.providerId.trim() || !input.providerName.trim() || !input.providerEndpoint.trim()) {
    return "Provider ID, name, and endpoint are required.";
  }

  try {
    const trimmed = input.metadataJson.trim();
    if (trimmed && trimmed !== "{}") {
      JSON.parse(trimmed);
    }
  } catch {
    return "Advanced details must be valid JSON.";
  }

  return null;
}
