import type { ProviderDiscoveryRecord, SystemProviderRuntimeStatusResponse } from "../../contracts.ts";
import { buildTemplateProviderId, createEmptyProviderForm, type ProviderFormState } from "./providerForm.ts";
import { resolveProviderTemplate } from "./providerTemplates.ts";

export type ProviderDiscoveryLane = "runtime_catalog" | "local_runtimes";

export function isLocalDiscoveryRecord(record: Pick<ProviderDiscoveryRecord, "providerKind" | "topology" | "endpoint">): boolean {
  const localityKind = record.topology?.localityKind?.toLowerCase();
  const endpoint = record.endpoint.toLowerCase();
  return localityKind === "local"
    || (record.providerKind ?? "").toLowerCase() === "ollama"
    || endpoint.includes("127.0.0.1:11434")
    || endpoint.includes("localhost:11434");
}

export function buildDiscoveryProviderForm(
  runtimeStatus: Pick<SystemProviderRuntimeStatusResponse, "baseUrl" | "model"> | null,
  currentModel: string,
  existingProviderIds: readonly string[] = [],
): ProviderFormState {
  const template = resolveProviderTemplate(undefined, runtimeStatus?.baseUrl ?? null);
  const nextForm = createEmptyProviderForm();
  const selectedModel = currentModel.trim() || runtimeStatus?.model.trim() || "";

  return {
    ...nextForm,
    templateId: template.id,
    providerType: template.providerType,
    providerKind: template.providerKind ?? "",
    providerId: buildTemplateProviderId(template.id, existingProviderIds),
    name: `${template.label} Provider`,
    endpoint: runtimeStatus?.baseUrl?.trim() || template.defaultEndpoint || nextForm.endpoint,
    defaultModel: selectedModel,
    apiKey: "",
    metadataJson: "{\n  \n}",
    enabled: true,
  };
}
