import { providerTemplateById, slugifyProviderName, type ProviderTemplateId } from "./providerTemplates.ts";

export const providerTypeOrder = ["Llm", "Embedding", "Vector", "Batch"] as const;
export type ProviderType = (typeof providerTypeOrder)[number];

export interface ProviderFormState {
  templateId: ProviderTemplateId;
  providerId: string;
  providerType: ProviderType;
  providerKind: string;
  name: string;
  endpoint: string;
  defaultModel: string;
  adapterSetRef: string;
  credentialBindingId: string;
  apiKey: string;
  metadataJson: string;
  enabled: boolean;
}

export function createEmptyProviderForm(): ProviderFormState {
  return {
    templateId: "openrouter",
    providerId: "",
    providerType: "Llm",
    providerKind: "OpenRouter",
    name: "",
    endpoint: providerTemplateById("openrouter").defaultEndpoint,
    defaultModel: "",
    adapterSetRef: "",
    credentialBindingId: "",
    apiKey: "",
    metadataJson: "{\n  \n}",
    enabled: true,
  };
}

export function applyProviderTemplate(templateId: ProviderTemplateId, current: ProviderFormState): ProviderFormState {
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
    defaultModel: template.id === current.templateId ? current.defaultModel.trim() : "",
    adapterSetRef: template.id === current.templateId ? current.adapterSetRef.trim() : "",
    credentialBindingId: template.id === current.templateId ? current.credentialBindingId.trim() : "",
  };
}
