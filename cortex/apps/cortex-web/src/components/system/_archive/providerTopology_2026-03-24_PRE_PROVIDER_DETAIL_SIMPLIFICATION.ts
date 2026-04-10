import type { ProviderLocalityKind, ProviderRecord } from "../../contracts.ts";

export type ProviderReadinessState = "ready" | "neutral" | "attention" | "disabled";

export function formatProviderTypeLabel(providerType: ProviderRecord["providerType"]): string {
  switch (providerType) {
    case "Llm":
      return "LLM";
    case "Embedding":
      return "Embedding";
    case "Vector":
      return "Vector";
    case "Batch":
      return "Batch";
    default:
      return providerType;
  }
}

export function inferProviderLocalityKind(provider: Pick<ProviderRecord, "endpoint" | "topology">): ProviderLocalityKind {
  const explicitKind = provider.topology?.localityKind;
  if (explicitKind) {
    return explicitKind;
  }

  const endpoint = provider.endpoint.trim().toLowerCase();
  if (
    endpoint.includes("127.0.0.1") ||
    endpoint.includes("localhost") ||
    endpoint.includes("0.0.0.0")
  ) {
    return "Local";
  }

  if (
    endpoint.includes("vercel.app") ||
    endpoint.includes("ngrok") ||
    endpoint.includes("tunnel") ||
    endpoint.includes("mirror")
  ) {
    return "Tunneled";
  }

  return "Cloud";
}

export function formatProviderLocalityLabel(provider: Pick<ProviderRecord, "endpoint" | "topology">): string {
  return inferProviderLocalityKind(provider);
}

export function formatProviderModelLabel(provider: Pick<ProviderRecord, "defaultModel">): string {
  return provider.defaultModel?.trim() || "Not configured";
}

export function formatProviderCredentialState(
  provider: Pick<ProviderRecord, "hasCredential" | "credentialBindingId">,
): string {
  if (provider.hasCredential) {
    return "Connected";
  }

  if (provider.credentialBindingId?.trim()) {
    return "Credential missing";
  }

  return "System default";
}

export function getProviderReadiness(provider: Pick<ProviderRecord, "isActive" | "hasCredential" | "credentialBindingId">): {
  state: ProviderReadinessState;
  label: string;
  detail: string;
} {
  if (!provider.isActive) {
    return {
      state: "disabled",
      label: "Disabled",
      detail: "Provider is turned off.",
    };
  }

  if (provider.hasCredential) {
    return {
      state: "ready",
      label: "Ready",
      detail: "Credential is available.",
    };
  }

  if (provider.credentialBindingId?.trim()) {
    return {
      state: "attention",
      label: "Needs credential",
      detail: "A linked credential record is missing.",
    };
  }

  return {
    state: "neutral",
    label: "Uses default",
    detail: "No provider-specific credential is linked.",
  };
}

export function formatProviderTopologySummary(provider: ProviderRecord): string {
  const topology = provider.topology;
  const batchPolicy = provider.batchPolicy;
  if (!topology) {
    return batchPolicy ? formatProviderBatchSummary(batchPolicy) : "Runtime details not surfaced yet.";
  }

  const parts = [
    `family ${topology.familyId}`,
    topology.profileId ? `profile ${topology.profileId}` : null,
    `instance ${topology.instanceId}`,
    topology.deviceId ? `device ${topology.deviceId}` : null,
    topology.environmentId ? `environment ${topology.environmentId}` : null,
    topology.lastSeenAt ? `last seen ${topology.lastSeenAt}` : null,
    topology.discoverySource ? `source ${topology.discoverySource}` : null,
    batchPolicy ? formatProviderBatchSummary(batchPolicy) : null,
  ].filter((value): value is string => Boolean(value));

  return parts.join(" · ");
}

export function formatProviderHoverDetails(provider: ProviderRecord): string {
  const topology = provider.topology;
  const batchPolicy = provider.batchPolicy;
  const parts = [
    `Provider: ${provider.name}`,
    `Model: ${formatProviderModelLabel(provider)}`,
    `Locality: ${formatProviderLocalityLabel(provider)}`,
    topology?.familyId ? `Family: ${topology.familyId}` : null,
    topology?.profileId ? `Profile: ${topology.profileId}` : null,
    topology?.instanceId ? `Instance: ${topology.instanceId}` : null,
    topology?.deviceId ? `Device: ${topology.deviceId}` : null,
    topology?.environmentId ? `Environment: ${topology.environmentId}` : null,
    batchPolicy ? `Batch: ${formatProviderBatchSummary(batchPolicy)}` : null,
  ].filter((value): value is string => Boolean(value));

  return parts.join("\n");
}

export function formatProviderBatchSummary(batchPolicy: NonNullable<ProviderRecord["batchPolicy"]>): string {
  const cadence = batchPolicy.cadenceKind.replace(/([a-z])([A-Z])/g, "$1 $2");
  const scope = batchPolicy.scopeKind.replace(/([a-z])([A-Z])/g, "$1 $2");
  const flush = batchPolicy.flushPolicy.replace(/([a-z])([A-Z])/g, "$1 $2");
  const parts = [cadence, scope, flush];
  if (batchPolicy.batchWindow?.intervalSeconds) {
    parts.push(`${batchPolicy.batchWindow.intervalSeconds}s`);
  }
  return parts.join(" · ");
}
