import type { SystemProviderRuntimeStatusResponse } from "../../contracts.ts";
import { buildTemplateProviderId, createEmptyProviderForm, type ProviderFormState } from "./providerForm.ts";
import { resolveProviderTemplate } from "./providerTemplates.ts";

export function buildDiscoveryProviderForm(
  adapterStatus: Pick<SystemProviderRuntimeStatusResponse, "baseUrl" | "model"> | null,
  currentModel: string,
  existingProviderIds: readonly string[] = [],
): ProviderFormState {
  const template = resolveProviderTemplate(undefined, adapterStatus?.baseUrl ?? null);
  const nextForm = createEmptyProviderForm();
  const selectedModel = currentModel.trim() || adapterStatus?.model.trim() || "";

  return {
    ...nextForm,
    templateId: template.id,
    providerType: template.providerType,
    providerKind: template.providerKind ?? "",
    providerId: buildTemplateProviderId(template.id, existingProviderIds),
    name: `${template.label} Provider`,
    endpoint: adapterStatus?.baseUrl?.trim() || template.defaultEndpoint || nextForm.endpoint,
    defaultModel: selectedModel,
    apiKey: "",
    metadataJson: "{\n  \n}",
    enabled: true,
  };
}
