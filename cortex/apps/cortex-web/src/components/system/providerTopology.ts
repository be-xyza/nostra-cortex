import type { ProviderLocalityKind, ProviderRecord, ProviderValidationResponse } from "../../contracts.ts";
import { humanizeProviderDiagnostic } from "./providerDiagnostics.ts";

export type ProviderReadinessState = "ready" | "neutral" | "attention" | "disabled";
export interface ProviderOperationalReadiness {
  state: ProviderReadinessState;
  label: string;
  detail: string;
  issues: string[];
}

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

export function formatProviderAccessLabel(
  provider: Pick<ProviderRecord, "authState" | "authType">,
): string {
  switch (provider.authState) {
    case "not_required":
      return "No key required";
    case "linked":
      return provider.authType === "api_key" ? "API key linked" : "Auth linked";
    case "missing":
      return "Auth missing";
    case "inherited":
    default:
      return "Inherited runtime key";
  }
}

export const formatProviderCredentialState = formatProviderAccessLabel;

export function getProviderReadiness(provider: Pick<ProviderRecord, "isActive" | "authState">): {
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

  if (provider.authState === "not_required") {
    return {
      state: "ready",
      label: "Ready",
      detail: "No auth secret is required for this provider.",
    };
  }

  if (provider.authState === "linked") {
    return {
      state: "ready",
      label: "Ready",
      detail: "A linked auth binding is available.",
    };
  }

  if (provider.authState === "missing") {
    return {
      state: "attention",
      label: "Needs auth",
      detail: "A linked auth binding is missing its secret.",
    };
  }

  return {
    state: "neutral",
    label: "Inherited",
    detail: "This provider relies on inherited runtime auth.",
  };
}

function summarizeValidationIssues(validationResult: ProviderValidationResponse | null | undefined): string[] {
  if (!validationResult || validationResult.valid) {
    return [];
  }

  return [
    validationResult.keyError ? humanizeProviderDiagnostic(validationResult.keyError) : null,
    validationResult.modelsError ? humanizeProviderDiagnostic(validationResult.modelsError) : null,
    validationResult.chatError ? humanizeProviderDiagnostic(validationResult.chatError) : null,
    validationResult.embeddingsError ? humanizeProviderDiagnostic(validationResult.embeddingsError) : null,
  ].filter((value): value is string => Boolean(value));
}

export function getProviderOperationalReadiness(
  provider: Pick<
    ProviderRecord,
    "providerType" | "providerFamily" | "isActive" | "authState" | "supportedModels" | "adapterHealthError" | "upstreamModelsError" | "endpoint" | "topology"
  >,
  validationResult?: ProviderValidationResponse | null,
): ProviderOperationalReadiness {
  const catalogSize = provider.supportedModels?.length ?? 0;
  const validationIssues = summarizeValidationIssues(validationResult);
  const runtimeIssue = humanizeProviderDiagnostic(
    catalogSize === 0 ? provider.upstreamModelsError ?? provider.adapterHealthError ?? null : null,
  );
  const issues = [...validationIssues, ...(runtimeIssue ? [runtimeIssue] : [])];
  const isAnonymousLocalRuntime =
    provider.providerType === "Llm"
    && catalogSize > 0
    && provider.authState === "not_required"
    && (
      provider.providerFamily === "Ollama"
      || formatProviderLocalityLabel(provider) === "Local"
    );

  if (!provider.isActive) {
    return {
      state: "disabled",
      label: "Disabled",
      detail: "Provider is turned off.",
      issues: [],
    };
  }

  if (issues.length > 0) {
    if (catalogSize > 0) {
      return {
        state: "attention",
        label: "Needs attention",
        detail: `Latest validation or runtime checks reported issues, even though ${catalogSize} saved model${catalogSize === 1 ? " is" : "s are"} still available.`,
        issues,
      };
    }

    return {
      state: "attention",
      label: "Needs attention",
      detail: issues[0],
      issues,
    };
  }

  if (provider.authState === "linked" && catalogSize > 0) {
    return {
      state: "ready",
      label: "Ready",
      detail: `Linked auth is available and ${catalogSize} model${catalogSize === 1 ? "" : "s"} are loaded.`,
      issues: [],
    };
  }

  if (isAnonymousLocalRuntime) {
    return {
      state: "ready",
      label: "Ready",
      detail: `Local runtime discovery surfaced ${catalogSize} model${catalogSize === 1 ? "" : "s"} without any provider-specific auth.`,
      issues: [],
    };
  }

  if (provider.authState === "linked") {
    return {
      state: "neutral",
      label: "Auth linked",
      detail: "Auth is available. Refresh the catalog to confirm the runtime and models.",
      issues: [],
    };
  }

  if (provider.authState === "missing") {
    return {
      state: "attention",
      label: "Needs auth",
      detail: "A linked auth binding is missing.",
      issues: [],
    };
  }

  return {
    state: "neutral",
    label: "Inherited",
    detail: "This provider relies on inherited runtime auth.",
    issues: [],
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

export function formatProviderBindingLabel(bindingId: string): string {
  const trimmed = bindingId.trim();
  if (!trimmed) {
    return "Unbound";
  }
  if (trimmed === "llm.default") {
    return "LLM default";
  }
  return trimmed
    .split(".")
    .map((segment) => segment.replace(/_/g, " "))
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");
}
