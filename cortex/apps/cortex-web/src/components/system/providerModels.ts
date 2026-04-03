import type { ProviderTemplateId } from "./providerTemplates.ts";

export function normalizeModelOptions(...sources: Array<readonly string[] | undefined>): string[] {
  const seen = new Set<string>();
  const options: string[] = [];

  for (const source of sources) {
    if (!source) {
      continue;
    }

    for (const value of source) {
      const model = value.trim();
      if (!model || seen.has(model)) {
        continue;
      }

      seen.add(model);
      options.push(model);
    }
  }

  return options;
}

export function buildSelectableModelOptions(
  currentModel: string,
  ...sources: Array<readonly string[] | undefined>
): string[] {
  const options = normalizeModelOptions(...sources);
  const trimmedCurrent = currentModel.trim();
  if (trimmedCurrent && !options.includes(trimmedCurrent)) {
    options.unshift(trimmedCurrent);
  }
  return options;
}

export function buildProviderModelOptions(input: {
  currentModel: string;
  validatedModels?: readonly string[];
  savedProviderModels?: readonly string[];
  adapterDiscoveryModels?: readonly string[];
  adapterRuntimeModel?: string;
  panelKind: "none" | "provider" | "create" | "discovery";
  templateId?: ProviderTemplateId;
  providerId?: string;
}): string[] {
  const includeAdapterCatalog =
    input.panelKind === "discovery" ||
    (input.panelKind === "create" && input.templateId === "ollama");

  return buildSelectableModelOptions(
    input.currentModel,
    input.validatedModels,
    input.savedProviderModels,
    includeAdapterCatalog ? input.adapterDiscoveryModels : undefined,
    includeAdapterCatalog && input.adapterRuntimeModel ? [input.adapterRuntimeModel] : undefined,
  );
}

export function extractModelNames(payload: unknown): string[] {
  const collected: string[] = [];
  const pushCandidate = (value: unknown) => {
    if (typeof value !== "string") {
      return;
    }
    const model = value.trim();
    if (model) {
      collected.push(model);
    }
  };
  const extractRow = (entry: unknown) => {
    if (typeof entry === "string") {
      pushCandidate(entry);
      return;
    }

    if (!entry || typeof entry !== "object") {
      return;
    }

    const record = entry as Record<string, unknown>;
    pushCandidate(record.id);
    pushCandidate(record.name);
    pushCandidate(record.model);
  };

  if (Array.isArray(payload)) {
    for (const entry of payload) {
      extractRow(entry);
    }
    return normalizeModelOptions(collected);
  }

  if (payload && typeof payload === "object") {
    const record = payload as Record<string, unknown>;
    const data = record.data;
    if (Array.isArray(data)) {
      for (const entry of data) {
        extractRow(entry);
      }
      return normalizeModelOptions(collected);
    }

    const models = record.models;
    if (Array.isArray(models)) {
      for (const entry of models) {
        extractRow(entry);
      }
      return normalizeModelOptions(collected);
    }

    pushCandidate(record.id);
    pushCandidate(record.name);
    pushCandidate(record.model);
  }

  return normalizeModelOptions(collected);
}
