import { providerTemplateById, type ProviderTemplateId } from "./providerTemplates.ts";

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

export function buildTemplateProviderId(templateId: ProviderTemplateId, existingProviderIds: readonly string[] = []): string {
  const baseId = `${templateId}_provider`;
  const seen = new Set(existingProviderIds.map((value) => value.trim()).filter(Boolean));
  if (!seen.has(baseId)) {
    return baseId;
  }

  let suffix = 2;
  while (seen.has(`${baseId}_${suffix}`)) {
    suffix += 1;
  }
  return `${baseId}_${suffix}`;
}

export function createEmptyProviderForm(): ProviderFormState {
  return {
    templateId: "manual",
    providerId: "",
    providerType: "Llm",
    providerKind: "",
    name: "",
    endpoint: providerTemplateById("manual").defaultEndpoint,
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
  const nextProviderId = current.providerId.trim() || buildTemplateProviderId(template.id);
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
