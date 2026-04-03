import type { ProviderDiscoveryRecord, RuntimeHostRecord, SystemProviderRuntimeStatusResponse } from "../../contracts.ts";
import { buildTemplateProviderId, createEmptyProviderForm, type ProviderFormState } from "./providerForm.ts";
import { resolveProviderTemplate } from "./providerTemplates.ts";

export type ProviderDiscoveryLane = "runtime_catalog" | "local_runtimes";

function inferRuntimeHostId(endpoint: string, providerId: string): string {
  const normalized = endpoint.trim().toLowerCase();
  if (
    normalized.includes("127.0.0.1")
    || normalized.includes("localhost")
    || normalized.includes("0.0.0.0")
  ) {
    return "host.local.primary";
  }
  if (normalized.includes("204.168.175.150")) {
    return "host.vps.primary";
  }
  if (normalized.includes("ngrok") || normalized.includes("tunnel") || normalized.includes("mirror")) {
    return "host.tunnel.primary";
  }
  return `host.managed.${providerId}`;
}

export function isLocalDiscoveryRecord(record: Pick<ProviderDiscoveryRecord, "providerKind" | "topology" | "endpoint">): boolean {
  const localityKind = record.topology?.localityKind?.toLowerCase();
  const endpoint = record.endpoint.toLowerCase();
  return localityKind === "local"
    || (record.providerKind ?? "").toLowerCase() === "ollama"
    || endpoint.includes("127.0.0.1:11434")
    || endpoint.includes("localhost:11434");
}

export function describeRuntimeHostInventory(
  host: Pick<RuntimeHostRecord, "metadata" | "health">,
  providerCount: number,
  catalogModelCount: number,
): { label: string; detail: string } {
  if (providerCount > 0) {
    return {
      label: `${providerCount} provider runtime${providerCount === 1 ? "" : "s"} discovered`,
      detail: `${catalogModelCount} model${catalogModelCount === 1 ? "" : "s"} are currently visible from providers on this host.`,
    };
  }

  const detail = [
    host.metadata?.remoteDiscoveryDetail,
    typeof host.health === "object" && host.health && "detail" in host.health && typeof host.health.detail === "string"
      ? host.health.detail
      : null,
  ].find((value): value is string => typeof value === "string" && value.trim().length > 0);

  const status = [
    host.metadata?.remoteDiscoveryStatus,
    typeof host.health === "object" && host.health && "status" in host.health && typeof host.health.status === "string"
      ? host.health.status
      : null,
  ].find((value): value is string => typeof value === "string" && value.trim().length > 0);

  if (status === "no_runtime") {
    return {
      label: "No provider runtime detected yet",
      detail: detail ?? "This host is tracked, but no provider runtime has been discovered on it yet.",
    };
  }

  if (status === "probe_failed") {
    return {
      label: "Runtime probe needs attention",
      detail: detail ?? "The last runtime-host probe did not complete successfully.",
    };
  }

  return {
    label: "Host tracked for discovery",
    detail: detail ?? "This host is tracked independently from provider records until a provider runtime is discovered.",
  };
}

export function buildDiscoveryProviderForm(
  runtimeStatus: Pick<SystemProviderRuntimeStatusResponse, "baseUrl" | "model"> | null,
  currentModel: string,
  existingProviderIds: readonly string[] = [],
): ProviderFormState {
  const template = resolveProviderTemplate(undefined, runtimeStatus?.baseUrl ?? null);
  const nextForm = createEmptyProviderForm();
  const selectedModel = currentModel.trim() || runtimeStatus?.model.trim() || "";
  const providerId = buildTemplateProviderId(template.id, existingProviderIds);
  const endpoint = runtimeStatus?.baseUrl?.trim() || template.defaultEndpoint || nextForm.endpoint;

  return {
    ...nextForm,
    templateId: template.id,
    providerType: template.providerType,
    providerKind: template.providerKind ?? "",
    providerId,
    hostId: inferRuntimeHostId(endpoint, providerId),
    name: `${template.label} Provider`,
    endpoint,
    defaultModel: selectedModel,
    apiKey: "",
    metadataJson: "{\n  \n}",
    enabled: true,
  };
}
