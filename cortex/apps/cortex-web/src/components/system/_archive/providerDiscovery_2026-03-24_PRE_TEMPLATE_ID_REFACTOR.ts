import type { SystemLlmAdapterStatusResponse } from "../../contracts.ts";
import { createEmptyProviderForm, type ProviderFormState } from "./providerForm.ts";
import { resolveProviderTemplate, slugifyProviderName } from "./providerTemplates.ts";

export function buildDiscoveryProviderForm(
  adapterStatus: Pick<SystemLlmAdapterStatusResponse, "baseUrl" | "model"> | null,
  currentModel: string,
): ProviderFormState {
  const template = resolveProviderTemplate(undefined, adapterStatus?.baseUrl ?? null);
  const nextForm = createEmptyProviderForm();
  const selectedModel = currentModel.trim() || adapterStatus?.model.trim() || "";
  const providerIdSeed = selectedModel ? `${template.label} ${selectedModel}` : `${template.label} provider`;

  return {
    ...nextForm,
    templateId: template.id,
    providerType: template.providerType,
    providerKind: template.providerKind ?? "",
    providerId: slugifyProviderName(providerIdSeed) || nextForm.providerId,
    name: `${template.label} Provider`,
    endpoint: adapterStatus?.baseUrl?.trim() || template.defaultEndpoint || nextForm.endpoint,
    defaultModel: selectedModel,
    apiKey: "",
    metadataJson: "{\n  \n}",
    enabled: true,
  };
}
